use std::path::PathBuf;
use async_trait::async_trait;
use crate::domain::entities::folder::Folder;

/// Error types for folder repository operations
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum FolderRepositoryError {
    #[error("Folder not found: {0}")]
    NotFound(String),
    
    #[error("Folder already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Invalid folder path: {0}")]
    InvalidPath(String),
    
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for folder repository operations
pub type FolderRepositoryResult<T> = Result<T, FolderRepositoryError>;

/// Repository interface for folder operations (primary port)
#[async_trait]
pub trait FolderRepository: Send + Sync + 'static {
    /// Creates a new folder
    async fn create_folder(&self, name: String, parent_path: Option<PathBuf>) -> FolderRepositoryResult<Folder>;
    
    /// Gets a folder by its ID
    async fn get_folder_by_id(&self, id: &str) -> FolderRepositoryResult<Folder>;
    
    /// Gets a folder by its path
    async fn get_folder_by_path(&self, path: &PathBuf) -> FolderRepositoryResult<Folder>;
    
    /// Lists folders in a parent folder
    async fn list_folders(&self, parent_id: Option<&str>) -> FolderRepositoryResult<Vec<Folder>>;
    
    /// Renames a folder
    async fn rename_folder(&self, id: &str, new_name: String) -> FolderRepositoryResult<Folder>;
    
    /// Moves a folder to a new parent
    async fn move_folder(&self, id: &str, new_parent_id: Option<&str>) -> FolderRepositoryResult<Folder>;
    
    /// Deletes a folder
    async fn delete_folder(&self, id: &str) -> FolderRepositoryResult<()>;
    
    /// Checks if a folder exists at the given path
    async fn folder_exists(&self, path: &PathBuf) -> FolderRepositoryResult<bool>;
}