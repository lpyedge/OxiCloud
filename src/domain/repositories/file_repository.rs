use std::path::PathBuf;
use async_trait::async_trait;
use crate::domain::entities::file::File;

/// Error types for file repository operations
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum FileRepositoryError {
    #[error("File not found: {0}")]
    NotFound(String),
    
    #[error("File already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Invalid file path: {0}")]
    InvalidPath(String),
    
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for file repository operations
pub type FileRepositoryResult<T> = Result<T, FileRepositoryError>;

/// Repository interface for file operations (primary port)
#[async_trait]
pub trait FileRepository: Send + Sync + 'static {
    /// Gets a folder by its ID - helper method for file repository to work with folders
    #[allow(dead_code)]
    async fn get_folder_by_id(&self, id: &str) -> FileRepositoryResult<crate::domain::entities::folder::Folder>;
    /// Saves a file from bytes
    async fn save_file_from_bytes(
        &self,
        name: String,
        folder_id: Option<String>,
        content_type: String,
        content: Vec<u8>,
    ) -> FileRepositoryResult<File>;
    
    /// Saves a file with a specific ID
    #[allow(dead_code)]
    async fn save_file_with_id(
        &self,
        id: String,
        name: String,
        folder_id: Option<String>,
        content_type: String,
        content: Vec<u8>,
    ) -> FileRepositoryResult<File>;
    
    /// Gets a file by its ID
    async fn get_file_by_id(&self, id: &str) -> FileRepositoryResult<File>;
    
    /// Lists files in a folder
    async fn list_files(&self, folder_id: Option<&str>) -> FileRepositoryResult<Vec<File>>;
    
    /// Deletes a file
    async fn delete_file(&self, id: &str) -> FileRepositoryResult<()>;
    
    /// Deletes a file and its entry from the map
    #[allow(dead_code)]
    async fn delete_file_entry(&self, id: &str) -> FileRepositoryResult<()>;
    
    /// Gets file content as bytes
    async fn get_file_content(&self, id: &str) -> FileRepositoryResult<Vec<u8>>;
    
    /// Checks if a file exists at the given path
    async fn file_exists(&self, path: &PathBuf) -> FileRepositoryResult<bool>;
}