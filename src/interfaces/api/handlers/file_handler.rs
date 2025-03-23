use std::sync::Arc;
use axum::{
    extract::{Path, State, Multipart, Query},
    http::{StatusCode, header, HeaderName, HeaderValue, Response},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::collections::HashMap;
use futures::Stream;
use futures::StreamExt;
use std::task::{Context, Poll};
use std::pin::Pin;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use std::path::PathBuf;

use crate::application::services::file_service::{FileService, FileServiceError};
use crate::infrastructure::services::compression_service::{
    CompressionService, GzipCompressionService, CompressionLevel
};

type AppState = Arc<FileService>;

/// Handler for file-related API endpoints
pub struct FileHandler;

// Simpler approach to make streams Unpin - use Pin<Box<dyn Stream>> directly
struct BoxedStream<T> {
    inner: Pin<Box<dyn Stream<Item = T> + Send + 'static>>,
}

impl<T> Stream for BoxedStream<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Accessing the field directly is safe because BoxedStream is not a structural pinning type
        unsafe { self.get_unchecked_mut().inner.as_mut().poll_next(cx) }
    }
}

// This is safe because BoxedStream's inner field is already Pin<Box<dyn Stream>>
impl<T> Unpin for BoxedStream<T> {}

impl<T> BoxedStream<T> {
    #[allow(dead_code)]
    fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = T> + Send + 'static,
    {
        BoxedStream {
            inner: Box::pin(stream),
        }
    }
}

impl FileHandler {
    /// Uploads a file
    pub async fn upload_file(
        State(service): State<AppState>,
        mut multipart: Multipart,
    ) -> impl IntoResponse {
        // Extract file from multipart request
        let mut file_part = None;
        let mut folder_id = None;
        
        tracing::info!("Processing file upload request");
        
        while let Some(field) = multipart.next_field().await.unwrap_or(None) {
            let name = field.name().unwrap_or("").to_string();
            tracing::info!("Multipart field received: {}", name);
            
            if name == "file" {
                let filename = field.file_name().unwrap_or("unnamed").to_string();
                let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
                tracing::info!("File received: {} ({})", filename, content_type);
                
                let bytes = field.bytes().await.unwrap_or_default();
                tracing::info!("File size: {} bytes", bytes.len());
                
                file_part = Some((filename, content_type, bytes));
            } else if name == "folder_id" {
                let folder_id_value = field.text().await.unwrap_or_default();
                tracing::info!("folder_id received: {}", folder_id_value);
                
                if !folder_id_value.is_empty() {
                    folder_id = Some(folder_id_value);
                }
            }
        }
        
        // Check if file was provided
        if let Some((filename, content_type, data)) = file_part {
            tracing::info!("Uploading file '{}' to folder_id: {:?}", filename, folder_id);
            
            // Use the proper file service to handle the upload
            match service.upload_file_from_bytes(filename.clone(), folder_id.clone(), content_type.clone(), data.to_vec()).await {
                Ok(file) => {
                    tracing::info!("File uploaded successfully: {} (ID: {})", filename, file.id);
                    
                    // Log additional debugging information
                    tracing::info!("Created file details: folder_id={:?}, size={}, path={}",
                        file.folder_id, file.size, file.path);
                    
                    // Return success response with file information
                    (StatusCode::CREATED, Json(file)).into_response()
                },
                Err(err) => {
                    tracing::error!("Error uploading file '{}' through service: {}", filename, err);
                    
                    // Return error response
                    let status = match &err {
                        FileServiceError::NotFound(_) => StatusCode::NOT_FOUND,
                        FileServiceError::AccessError(_) => StatusCode::SERVICE_UNAVAILABLE,
                        _ => StatusCode::INTERNAL_SERVER_ERROR,
                    };
                    
                    (status, Json(serde_json::json!({
                        "error": format!("Error uploading file: {}", err)
                    }))).into_response()
                }
            }
        } else {
            tracing::error!("Error: No file provided in request");
            
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "No file provided"
            }))).into_response()
        }
    }
    
    /// Downloads a file with optional compression
    pub async fn download_file(
        State(service): State<AppState>,
        Path(id): Path<String>,
        Query(params): Query<HashMap<String, String>>,
    ) -> impl IntoResponse {
        // Initialize compression service
        let compression_service = GzipCompressionService::new();
        
        // Check if compression is explicitly requested or rejected
        let compression_param = params.get("compress").map(|v| v.as_str());
        let force_compress = compression_param == Some("true") || compression_param == Some("1");
        let force_no_compress = compression_param == Some("false") || compression_param == Some("0");
        
        // Determine compression level from query params
        let compression_level = match params.get("compression_level").map(|v| v.as_str()) {
            Some("none") => CompressionLevel::None,
            Some("fast") => CompressionLevel::Fast,
            Some("best") => CompressionLevel::Best,
            _ => CompressionLevel::Default, // Default or unrecognized
        };
        
        // Get file info first to check it exists and get metadata
        match service.get_file(&id).await {
            Ok(file) => {
                // Determine if we should compress based on file type and size
                let should_compress = if force_no_compress {
                    false
                } else if force_compress {
                    true
                } else {
                    compression_service.should_compress(&file.mime_type, file.size)
                };
                
                // Log compression decision for debugging
                tracing::debug!(
                    "Download file: name={}, size={}KB, mime={}, compress={}", 
                    file.name, file.size / 1024, file.mime_type, should_compress
                );
                
                // For large files, use streaming response with potential compression
                if file.size > 10 * 1024 * 1024 { // 10MB threshold for streaming
                    match service.get_file_content(&id).await {
                        Ok(content) => {
                            // Create base headers
                            let mut headers = HashMap::new();
                            headers.insert(
                                header::CONTENT_DISPOSITION.to_string(), 
                                format!("attachment; filename=\"{}\"", file.name)
                            );
                            
                            if should_compress {
                                // Add content-encoding header for compressed response
                                headers.insert(header::CONTENT_ENCODING.to_string(), "gzip".to_string());
                                headers.insert(header::CONTENT_TYPE.to_string(), file.mime_type.clone());
                                headers.insert(header::VARY.to_string(), "Accept-Encoding".to_string());
                                
                                // Compress the content
                                match compression_service.compress_data(&content, compression_level).await {
                                    Ok(compressed_content) => {
                                        tracing::debug!(
                                            "Compressed file: {} from {}KB to {}KB (ratio: {:.2})", 
                                            file.name, 
                                            content.len() / 1024, 
                                            compressed_content.len() / 1024,
                                            content.len() as f64 / compressed_content.len() as f64
                                        );
                                        
                                        // Build a custom response with headers and body
                                        let mut response = Response::builder()
                                            .status(StatusCode::OK)
                                            .body(axum::body::Body::from(compressed_content))
                                            .unwrap();
                                            
                                        // Add headers to response
                                        for (name, value) in headers {
                                            response.headers_mut().insert(
                                                HeaderName::from_bytes(name.as_bytes()).unwrap(),
                                                HeaderValue::from_str(&value).unwrap()
                                            );
                                        }
                                        
                                        response
                                    },
                                    Err(e) => {
                                        tracing::warn!("Compression failed, sending uncompressed: {}", e);
                                        // Fall back to uncompressed
                                        headers.insert(header::CONTENT_TYPE.to_string(), file.mime_type.clone());
                                        
                                        // Build a custom response with headers and body
                                        let mut response = Response::builder()
                                            .status(StatusCode::OK)
                                            .body(axum::body::Body::from(content))
                                            .unwrap();
                                            
                                        // Add headers to response
                                        for (name, value) in headers {
                                            response.headers_mut().insert(
                                                HeaderName::from_bytes(name.as_bytes()).unwrap(),
                                                HeaderValue::from_str(&value).unwrap()
                                            );
                                        }
                                        
                                        response
                                    }
                                }
                            } else {
                                // No compression, return as-is
                                headers.insert(header::CONTENT_TYPE.to_string(), file.mime_type.clone());
                                
                                // Build a custom response with headers and body
                                let mut response = Response::builder()
                                    .status(StatusCode::OK)
                                    .body(axum::body::Body::from(content))
                                    .unwrap();
                                    
                                // Add headers to response
                                for (name, value) in headers {
                                    response.headers_mut().insert(
                                        HeaderName::from_bytes(name.as_bytes()).unwrap(),
                                        HeaderValue::from_str(&value).unwrap()
                                    );
                                }
                                
                                response
                            }
                        },
                        Err(err) => {
                            tracing::error!("Error getting file content: {}", err);
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                                "error": format!("Error reading file: {}", err)
                            }))).into_response()
                        }
                    }
                } else {
                    // For smaller files, load entirely but still potentially compress
                    match service.get_file_content(&id).await {
                        Ok(content) => {
                            // Create base headers
                            let mut headers = HashMap::new();
                            headers.insert(
                                header::CONTENT_DISPOSITION.to_string(), 
                                format!("attachment; filename=\"{}\"", file.name)
                            );
                            
                            if should_compress {
                                // Add content-encoding header for compressed response
                                headers.insert(header::CONTENT_ENCODING.to_string(), "gzip".to_string());
                                headers.insert(header::CONTENT_TYPE.to_string(), file.mime_type.clone());
                                headers.insert(header::VARY.to_string(), "Accept-Encoding".to_string());
                                
                                // Compress the content
                                match compression_service.compress_data(&content, compression_level).await {
                                    Ok(compressed_content) => {
                                        tracing::debug!(
                                            "Compressed file: {} from {}KB to {}KB (ratio: {:.2})", 
                                            file.name, 
                                            content.len() / 1024, 
                                            compressed_content.len() / 1024,
                                            content.len() as f64 / compressed_content.len() as f64
                                        );
                                        
                                        // Build a custom response with headers and body
                                        let mut response = Response::builder()
                                            .status(StatusCode::OK)
                                            .body(axum::body::Body::from(compressed_content))
                                            .unwrap();
                                            
                                        // Add headers to response
                                        for (name, value) in headers {
                                            response.headers_mut().insert(
                                                HeaderName::from_bytes(name.as_bytes()).unwrap(),
                                                HeaderValue::from_str(&value).unwrap()
                                            );
                                        }
                                        
                                        response
                                    },
                                    Err(e) => {
                                        tracing::warn!("Compression failed, sending uncompressed: {}", e);
                                        // Fall back to uncompressed
                                        headers.insert(header::CONTENT_TYPE.to_string(), file.mime_type.clone());
                                        
                                        // Build a custom response with headers and body
                                        let mut response = Response::builder()
                                            .status(StatusCode::OK)
                                            .body(axum::body::Body::from(content))
                                            .unwrap();
                                            
                                        // Add headers to response
                                        for (name, value) in headers {
                                            response.headers_mut().insert(
                                                HeaderName::from_bytes(name.as_bytes()).unwrap(),
                                                HeaderValue::from_str(&value).unwrap()
                                            );
                                        }
                                        
                                        response
                                    }
                                }
                            } else {
                                // No compression, return as-is
                                headers.insert(header::CONTENT_TYPE.to_string(), file.mime_type.clone());
                                
                                // Build a custom response with headers and body
                                let mut response = Response::builder()
                                    .status(StatusCode::OK)
                                    .body(axum::body::Body::from(content))
                                    .unwrap();
                                    
                                // Add headers to response
                                for (name, value) in headers {
                                    response.headers_mut().insert(
                                        HeaderName::from_bytes(name.as_bytes()).unwrap(),
                                        HeaderValue::from_str(&value).unwrap()
                                    );
                                }
                                
                                response
                            }
                        },
                        Err(err) => {
                            tracing::error!("Error getting file content: {}", err);
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                                "error": format!("Error reading file: {}", err)
                            }))).into_response()
                        }
                    }
                }
            },
            Err(err) => {
                let status = match &err {
                    FileServiceError::NotFound(_) => StatusCode::NOT_FOUND,
                    FileServiceError::AccessError(_) => StatusCode::SERVICE_UNAVAILABLE,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };
                
                (status, Json(serde_json::json!({
                    "error": err.to_string()
                }))).into_response()
            }
        }
    }
    
    /// Lists files, optionally filtered by folder ID
    pub async fn list_files(
        State(service): State<AppState>,
        folder_id: Option<&str>,
    ) -> impl IntoResponse {
        tracing::info!("Listing files with folder_id: {:?}", folder_id);
        
        // Simply use the file service to list files
        match service.list_files(folder_id).await {
            Ok(files) => {
                // Log success for debugging purposes
                tracing::info!("Found {} files through the service", files.len());
                
                if !files.is_empty() {
                    tracing::info!("First file in service list: {} (ID: {})", 
                        files[0].name, files[0].id);
                } else {
                    tracing::info!("No files found in folder through service");
                }
                
                // Return the files as JSON response
                (StatusCode::OK, Json(files)).into_response()
            },
            Err(err) => {
                tracing::error!("Error listing files through service: {}", err);
                
                let status = match &err {
                    FileServiceError::NotFound(_) => StatusCode::NOT_FOUND,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };
                
                // Return a JSON error response
                (status, Json(serde_json::json!({
                    "error": err.to_string()
                }))).into_response()
            }
        }
    }
    
    /// Deletes a file
    pub async fn delete_file(
        State(service): State<AppState>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        // Use the file service to delete the file
        match service.delete_file(&id).await {
            Ok(_) => {
                tracing::info!("File successfully deleted: {}", id);
                StatusCode::NO_CONTENT.into_response()
            },
            Err(err) => {
                tracing::error!("Error deleting file: {}", err);
                
                let status = match &err {
                    FileServiceError::NotFound(_) => StatusCode::NOT_FOUND,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };
                
                (status, Json(serde_json::json!({
                    "error": format!("Error deleting file: {}", err)
                }))).into_response()
            }
        }
    }
    
    /// Moves a file to a different folder
    pub async fn move_file(
        State(service): State<AppState>,
        Path(id): Path<String>,
        Json(payload): Json<MoveFilePayload>,
    ) -> impl IntoResponse {
        tracing::info!("API request: Moving file with ID: {} to folder: {:?}", id, payload.folder_id);
        
        // First verify if the file exists
        match service.get_file(&id).await {
            Ok(file) => {
                tracing::info!("File found: {} (ID: {}), proceeding with move operation", file.name, id);
                
                // For target folders, we trust that the move operation will verify their existence
                if let Some(folder_id) = &payload.folder_id {
                    tracing::info!("Will attempt to move to folder: {}", folder_id);
                }
                
                // Proceed with the move operation
                match service.move_file(&id, payload.folder_id).await {
                    Ok(file) => {
                        tracing::info!("File moved successfully: {} (ID: {})", file.name, file.id);
                        (StatusCode::OK, Json(file)).into_response()
                    },
                    Err(err) => {
                        let status = match &err {
                            FileServiceError::NotFound(_) => {
                                tracing::error!("Error moving file - not found: {}", err);
                                StatusCode::NOT_FOUND
                            },
                            FileServiceError::Conflict(_) => {
                                tracing::error!("Error moving file - already exists: {}", err);
                                StatusCode::CONFLICT
                            },
                            _ => {
                                tracing::error!("Error moving file: {}", err);
                                StatusCode::INTERNAL_SERVER_ERROR
                            }
                        };
                        
                        (status, Json(serde_json::json!({
                            "error": format!("Error moving file: {}", err.to_string()),
                            "code": status.as_u16()
                        }))).into_response()
                    }
                }
            },
            Err(err) => {
                tracing::error!("Error finding file to move - does not exist: {} (ID: {})", err, id);
                (StatusCode::NOT_FOUND, Json(serde_json::json!({
                    "error": format!("The file with ID: {} does not exist", id),
                    "code": StatusCode::NOT_FOUND.as_u16()
                }))).into_response()
            }
        }
    }
}

/// Payload for moving a file
#[derive(Debug, Deserialize)]
pub struct MoveFilePayload {
    /// Target folder ID (None means root)
    pub folder_id: Option<String>,
}