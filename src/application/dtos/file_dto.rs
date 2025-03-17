use serde::Serialize;
use crate::domain::entities::file::File;

/// DTO for file responses
#[derive(Debug, Serialize)]
pub struct FileDto {
    /// File ID
    pub id: String,
    
    /// File name
    pub name: String,
    
    /// Path to the file (relative)
    pub path: String,
    
    /// Size in bytes
    pub size: u64,
    
    /// MIME type
    pub mime_type: String,
    
    /// Parent folder ID
    pub folder_id: Option<String>,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Last modification timestamp
    pub modified_at: u64,
}

impl From<File> for FileDto {
    fn from(file: File) -> Self {
        Self {
            id: file.id,
            name: file.name,
            path: file.path.to_string_lossy().to_string(),
            size: file.size,
            mime_type: file.mime_type,
            folder_id: file.folder_id,
            created_at: file.created_at,
            modified_at: file.modified_at,
        }
    }
}