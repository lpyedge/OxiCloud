use std::sync::Arc;
use axum::{
    extract::{Path, State, Multipart},
    http::{StatusCode, header},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::application::services::file_service::FileService;
use crate::domain::repositories::file_repository::FileRepositoryError;

type AppState = Arc<FileService>;

/// Handler for file-related API endpoints
pub struct FileHandler;

impl FileHandler {
    /// Uploads a file
    pub async fn upload_file(
        State(service): State<AppState>,
        mut multipart: Multipart,
    ) -> impl IntoResponse {
        // Extract file from multipart request
        let mut file_part = None;
        let mut folder_id = None;
        
        while let Some(field) = multipart.next_field().await.unwrap_or(None) {
            let name = field.name().unwrap_or("").to_string();
            
            if name == "file" {
                file_part = Some((
                    field.file_name().unwrap_or("unnamed").to_string(),
                    field.content_type().unwrap_or("application/octet-stream").to_string(),
                    field.bytes().await.unwrap_or_default(),
                ));
            } else if name == "folder_id" {
                let folder_id_value = field.text().await.unwrap_or_default();
                if !folder_id_value.is_empty() {
                    folder_id = Some(folder_id_value);
                }
            }
        }
        
        // Check if file was provided
        if let Some((filename, content_type, data)) = file_part {
            // Upload file from bytes
            match service.upload_file_from_bytes(filename, folder_id, content_type, data.to_vec()).await {
                Ok(file) => (StatusCode::CREATED, Json(file)).into_response(),
                Err(err) => {
                    let status = match &err {
                        FileRepositoryError::AlreadyExists(_) => StatusCode::CONFLICT,
                        FileRepositoryError::NotFound(_) => StatusCode::NOT_FOUND,
                        _ => StatusCode::INTERNAL_SERVER_ERROR,
                    };
                    
                    (status, Json(serde_json::json!({
                        "error": err.to_string()
                    }))).into_response()
                }
            }
        } else {
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "No file provided"
            }))).into_response()
        }
    }
    
    /// Downloads a file
    pub async fn download_file(
        State(service): State<AppState>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        // Get file info and content
        let file_result = service.get_file(&id).await;
        let content_result = service.get_file_content(&id).await;
        
        match (file_result, content_result) {
            (Ok(file), Ok(content)) => {
                // Create response with proper headers
                let headers = [
                    (header::CONTENT_TYPE, file.mime_type),
                    (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", file.name)),
                ];
                
                (StatusCode::OK, headers, content).into_response()
            },
            (Err(err), _) | (_, Err(err)) => {
                let status = match &err {
                    FileRepositoryError::NotFound(_) => StatusCode::NOT_FOUND,
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
        match service.list_files(folder_id).await {
            Ok(files) => {
                // Always return an array even if empty
                (StatusCode::OK, Json(files)).into_response()
            },
            Err(err) => {
                let status = match &err {
                    FileRepositoryError::NotFound(_) => StatusCode::NOT_FOUND,
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
        match service.delete_file(&id).await {
            Ok(_) => StatusCode::NO_CONTENT.into_response(),
            Err(err) => {
                let status = match &err {
                    FileRepositoryError::NotFound(_) => StatusCode::NOT_FOUND,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };
                
                (status, Json(serde_json::json!({
                    "error": err.to_string()
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
        tracing::info!("API request: Mover archivo con ID: {} a carpeta: {:?}", id, payload.folder_id);
        
        // Primero verificar si el archivo existe
        match service.get_file(&id).await {
            Ok(file) => {
                tracing::info!("Archivo encontrado: {} (ID: {}), procediendo con la operación de mover", file.name, id);
                
                // Para carpetas de destino, simplemente confiamos en que la 
                // operación de mover verificará su existencia
                if let Some(folder_id) = &payload.folder_id {
                    tracing::info!("Se intentará mover a carpeta: {}", folder_id);
                }
                
                // Proceder con la operación de mover
                match service.move_file(&id, payload.folder_id).await {
                    Ok(file) => {
                        tracing::info!("Archivo movido exitosamente: {} (ID: {})", file.name, file.id);
                        (StatusCode::OK, Json(file)).into_response()
                    },
                    Err(err) => {
                        let status = match &err {
                            FileRepositoryError::NotFound(_) => {
                                tracing::error!("Error al mover archivo - no encontrado: {}", err);
                                StatusCode::NOT_FOUND
                            },
                            FileRepositoryError::AlreadyExists(_) => {
                                tracing::error!("Error al mover archivo - ya existe: {}", err);
                                StatusCode::CONFLICT
                            },
                            _ => {
                                tracing::error!("Error al mover archivo: {}", err);
                                StatusCode::INTERNAL_SERVER_ERROR
                            }
                        };
                        
                        (status, Json(serde_json::json!({
                            "error": format!("Error al mover el archivo: {}", err.to_string()),
                            "code": status.as_u16(),
                            "details": format!("Error al mover archivo con ID: {} - {}", id, err)
                        }))).into_response()
                    }
                }
            },
            Err(err) => {
                tracing::error!("Error al encontrar archivo para mover - no existe: {} (ID: {})", err, id);
                (StatusCode::NOT_FOUND, Json(serde_json::json!({
                    "error": format!("El archivo con ID: {} no existe", id),
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