use serde::{Serialize, Deserialize};
use crate::domain::entities::folder::Folder;

/// DTO for folder creation requests
#[derive(Debug, Deserialize)]
pub struct CreateFolderDto {
    /// Name of the folder to create
    pub name: String,
    
    /// Parent folder ID (None for root level)
    pub parent_id: Option<String>,
}

/// DTO for folder rename requests
#[derive(Debug, Deserialize)]
pub struct RenameFolderDto {
    /// New name for the folder
    pub name: String,
}

/// DTO for folder move requests
#[derive(Debug, Deserialize)]
pub struct MoveFolderDto {
    /// New parent folder ID (None for root level)
    pub parent_id: Option<String>,
}

/// DTO for folder responses
#[derive(Debug, Serialize)]
pub struct FolderDto {
    /// Folder ID
    pub id: String,
    
    /// Folder name
    pub name: String,
    
    /// Path to the folder (relative)
    pub path: String,
    
    /// Parent folder ID
    pub parent_id: Option<String>,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Last modification timestamp
    pub modified_at: u64,
    
    /// Whether this is a root folder
    pub is_root: bool,
}

impl From<Folder> for FolderDto {
    fn from(folder: Folder) -> Self {
        let is_root = folder.parent_id.is_none();
        Self {
            id: folder.id,
            name: folder.name,
            path: folder.path.to_string_lossy().to_string(),
            parent_id: folder.parent_id,
            created_at: folder.created_at,
            modified_at: folder.modified_at,
            is_root,
        }
    }
}