use std::path::PathBuf;
use serde::{Serialize, Deserialize};

/// Represents a file entity in the domain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct File {
    /// Unique identifier for the file
    pub id: String,
    
    /// Name of the file
    pub name: String,
    
    /// Path to the file (relative to user's root)
    pub path: PathBuf,
    
    /// Size of the file in bytes
    pub size: u64,
    
    /// MIME type of the file
    pub mime_type: String,
    
    /// Parent folder ID
    pub folder_id: Option<String>,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Last modification timestamp
    pub modified_at: u64,
}

impl File {
    /// Creates a new file
    pub fn new(
        id: String,
        name: String,
        path: PathBuf,
        size: u64,
        mime_type: String,
        folder_id: Option<String>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        Self {
            id,
            name,
            path,
            size,
            mime_type,
            folder_id,
            created_at: now,
            modified_at: now,
        }
    }
    
    /// Updates file modification time
    #[allow(dead_code)]
    pub fn touch(&mut self) {
        self.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}