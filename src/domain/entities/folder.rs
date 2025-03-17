use std::path::PathBuf;
use serde::{Serialize, Deserialize};

/// Represents a folder entity in the domain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Folder {
    /// Unique identifier for the folder
    pub id: String,
    
    /// Name of the folder
    pub name: String,
    
    /// Path to the folder (relative to user's root)
    pub path: PathBuf,
    
    /// Parent folder ID (None if it's a root folder)
    pub parent_id: Option<String>,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Last modification timestamp
    pub modified_at: u64,
}

impl Folder {
    /// Creates a new folder
    pub fn new(id: String, name: String, path: PathBuf, parent_id: Option<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        Self {
            id,
            name,
            path,
            parent_id,
            created_at: now,
            modified_at: now,
        }
    }
    
    /// Returns the absolute path of the folder
    #[allow(dead_code)]
    pub fn get_absolute_path(&self, root_path: &PathBuf) -> PathBuf {
        root_path.join(&self.path)
    }
    
    /// Updates folder modification time
    pub fn touch(&mut self) {
        self.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}