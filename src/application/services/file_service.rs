use std::sync::Arc;
use thiserror::Error;
use async_trait::async_trait;

use crate::domain::repositories::file_repository::FileRepositoryError;
use crate::application::dtos::file_dto::FileDto;
use crate::application::ports::inbound::FileUseCase;
use crate::application::ports::outbound::FileStoragePort;
use crate::common::errors::DomainError;
use futures::Stream;
use bytes::Bytes;

/**
 * File service-specific error types.
 * 
 * This enum represents the application-level errors that can occur during file operations,
 * providing a translation layer between domain/infrastructure errors and application errors.
 */
#[derive(Debug, Error)]
pub enum FileServiceError {
    /// Returned when a requested file cannot be found
    #[error("File not found: {0}")]
    NotFound(String),
    
    /// Returned when a file operation conflicts with existing files
    #[error("File already exists: {0}")]
    Conflict(String),
    
    /// Returned when file access fails due to permissions or I/O issues
    #[error("File access error: {0}")]
    AccessError(String),
    
    /// Returned when a file path is invalid
    #[error("Invalid file path: {0}")]
    InvalidPath(String),
    
    /// Generic internal error for unexpected failures
    #[error("Internal error: {0}")]
    InternalError(String),
}

/**
 * Converts repository errors to service errors.
 * 
 * This implementation maps low-level repository errors to more 
 * application-appropriate error types, abstracting away the implementation details.
 */
impl From<FileRepositoryError> for FileServiceError {
    fn from(err: FileRepositoryError) -> Self {
        match err {
            FileRepositoryError::NotFound(id) => FileServiceError::NotFound(id),
            FileRepositoryError::AlreadyExists(path) => FileServiceError::Conflict(path),
            FileRepositoryError::InvalidPath(path) => FileServiceError::InvalidPath(path),
            FileRepositoryError::IoError(e) => FileServiceError::AccessError(e.to_string()),
            FileRepositoryError::Timeout(msg) => FileServiceError::AccessError(format!("Operation timed out: {}", msg)),
            _ => FileServiceError::InternalError(err.to_string()),
        }
    }
}

/**
 * Converts domain errors to service errors.
 * 
 * This implementation ensures that general domain errors are properly translated
 * to file service-specific errors while preserving their semantic meaning.
 */
impl From<DomainError> for FileServiceError {
    fn from(err: DomainError) -> Self {
        match err.kind {
            crate::common::errors::ErrorKind::NotFound => FileServiceError::NotFound(err.to_string()),
            crate::common::errors::ErrorKind::AlreadyExists => FileServiceError::Conflict(err.to_string()),
            crate::common::errors::ErrorKind::InvalidInput => FileServiceError::InvalidPath(err.to_string()),
            crate::common::errors::ErrorKind::AccessDenied => FileServiceError::AccessError(err.to_string()),
            _ => FileServiceError::InternalError(err.to_string()),
        }
    }
}

/**
 * Converts service errors to domain errors.
 * 
 * This implementation allows service errors to be propagated up the call stack as
 * domain errors when crossing architectural boundaries.
 */
impl From<FileServiceError> for DomainError {
    fn from(err: FileServiceError) -> Self {
        match err {
            FileServiceError::NotFound(id) => DomainError::not_found("File", id),
            FileServiceError::Conflict(path) => DomainError::already_exists("File", path),
            FileServiceError::InvalidPath(path) => DomainError::validation_error("File", format!("Invalid path: {}", path)),
            FileServiceError::AccessError(msg) => DomainError::access_denied("File", msg),
            FileServiceError::InternalError(msg) => DomainError::internal_error("File", msg),
        }
    }
}

/**
 * Type alias for results of file service operations.
 * 
 * Provides a convenient way to return either a successful value or a FileServiceError.
 */
pub type FileServiceResult<T> = Result<T, FileServiceError>;

/**
 * Service component for file operations in the application layer.
 * 
 * The FileService implements the application use cases related to files by orchestrating
 * domain logic and infrastructure components. It acts as an adapter between the inbound
 * ports (interfaces) and outbound ports (repositories), translating between DTOs and
 * domain entities.
 */
pub struct FileService {
    /// Repository responsible for file storage operations
    file_repository: Arc<dyn FileStoragePort>,
}

impl FileService {
    /// Creates a new file service
    pub fn new(file_repository: Arc<dyn FileStoragePort>) -> Self {
        Self { file_repository }
    }
    
    /// Creates a stub implementation for testing and middleware
    pub fn new_stub() -> impl FileUseCase {
        struct FileServiceStub;
        
        #[async_trait]
        impl FileUseCase for FileServiceStub {
            async fn upload_file(
                &self,
                _name: String,
                _folder_id: Option<String>,
                _content_type: String,
                _content: Vec<u8>,
            ) -> Result<FileDto, DomainError> {
                Ok(FileDto::empty())
            }
            
            async fn get_file(&self, _id: &str) -> Result<FileDto, DomainError> {
                Ok(FileDto::empty())
            }
            
            async fn list_files(&self, _folder_id: Option<&str>) -> Result<Vec<FileDto>, DomainError> {
                Ok(vec![])
            }
            
            async fn delete_file(&self, _id: &str) -> Result<(), DomainError> {
                Ok(())
            }
            
            async fn get_file_content(&self, _id: &str) -> Result<Vec<u8>, DomainError> {
                Ok(vec![])
            }
            
            async fn get_file_stream(&self, _id: &str) -> Result<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>, DomainError> {
                let empty_stream = futures::stream::empty();
                Ok(Box::new(empty_stream))
            }
            
            async fn move_file(&self, _file_id: &str, _folder_id: Option<String>) -> Result<FileDto, DomainError> {
                Ok(FileDto::empty())
            }
        }
        
        FileServiceStub
    }
    
    /// Uploads a new file from bytes
    pub async fn upload_file_from_bytes(
        &self,
        name: String,
        folder_id: Option<String>,
        content_type: String,
        content: Vec<u8>,
    ) -> FileServiceResult<FileDto>
    {
        let file = self.file_repository.save_file(name, folder_id, content_type, content).await
            .map_err(FileServiceError::from)?;
        Ok(FileDto::from(file))
    }
    
    /// Gets a file by ID
    pub async fn get_file(&self, id: &str) -> FileServiceResult<FileDto> {
        let file = self.file_repository.get_file(id).await
            .map_err(FileServiceError::from)?;
        Ok(FileDto::from(file))
    }
    
    /// Lists files in a folder
    pub async fn list_files(&self, folder_id: Option<&str>) -> FileServiceResult<Vec<FileDto>> {
        let files = self.file_repository.list_files(folder_id).await
            .map_err(FileServiceError::from)?;
        Ok(files.into_iter().map(FileDto::from).collect())
    }
    
    /// Deletes a file
    pub async fn delete_file(&self, id: &str) -> FileServiceResult<()> {
        self.file_repository.delete_file(id).await
            .map_err(FileServiceError::from)
    }
    
    /// Gets file content as bytes - use for small files only
    pub async fn get_file_content(&self, id: &str) -> FileServiceResult<Vec<u8>> {
        self.file_repository.get_file_content(id).await
            .map_err(FileServiceError::from)
    }
    
    /// Gets file content as stream - better for large files
    pub async fn get_file_stream(&self, id: &str) -> FileServiceResult<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>> {
        self.file_repository.get_file_stream(id).await
            .map_err(FileServiceError::from)
    }
    
    /// Moves a file to a new folder using filesystem operations directly
    pub async fn move_file(&self, file_id: &str, folder_id: Option<String>) -> FileServiceResult<FileDto> {
        tracing::info!("Moving file with ID: {} to folder: {:?}", file_id, folder_id);
        
        // Use the efficient repository implementation that uses rename
        let moved_file = self.file_repository.move_file(file_id, folder_id).await
            .map_err(|e| {
                tracing::error!("Error moving file (ID: {}): {}", file_id, e);
                FileServiceError::from(e)
            })?;
        
        tracing::info!("File moved successfully: {} (ID: {}) to folder: {:?}", 
                       moved_file.name(), moved_file.id(), moved_file.folder_id());
        
        Ok(FileDto::from(moved_file))
    }
}

#[async_trait]
impl FileUseCase for FileService {
    async fn upload_file(
        &self,
        name: String,
        folder_id: Option<String>,
        content_type: String,
        content: Vec<u8>,
    ) -> Result<FileDto, DomainError> {
        FileService::upload_file_from_bytes(self, name, folder_id, content_type, content).await
            .map_err(DomainError::from)
    }
    
    async fn get_file(&self, id: &str) -> Result<FileDto, DomainError> {
        FileService::get_file(self, id).await
            .map_err(DomainError::from)
    }
    
    async fn list_files(&self, folder_id: Option<&str>) -> Result<Vec<FileDto>, DomainError> {
        FileService::list_files(self, folder_id).await
            .map_err(DomainError::from)
    }
    
    async fn delete_file(&self, id: &str) -> Result<(), DomainError> {
        FileService::delete_file(self, id).await
            .map_err(DomainError::from)
    }
    
    async fn get_file_content(&self, id: &str) -> Result<Vec<u8>, DomainError> {
        FileService::get_file_content(self, id).await
            .map_err(DomainError::from)
    }
    
    async fn get_file_stream(&self, id: &str) -> Result<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>, DomainError> {
        FileService::get_file_stream(self, id).await
            .map_err(DomainError::from)
    }
    
    async fn move_file(&self, file_id: &str, folder_id: Option<String>) -> Result<FileDto, DomainError> {
        FileService::move_file(self, file_id, folder_id).await
            .map_err(DomainError::from)
    }
}