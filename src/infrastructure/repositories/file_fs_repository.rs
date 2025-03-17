use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use async_trait::async_trait;
use tokio::{fs, io::AsyncWriteExt};
use uuid::Uuid;
use mime_guess::from_path;
use serde::{Serialize, Deserialize};

use crate::domain::entities::file::File;
use crate::domain::repositories::file_repository::{
    FileRepository, FileRepositoryError, FileRepositoryResult
};
use crate::domain::repositories::folder_repository::FolderRepository;

/// Structure to store file IDs mapped to their paths
#[derive(Serialize, Deserialize, Debug, Default)]
struct FileIdMap {
    path_to_id: HashMap<String, String>,
}

/// Filesystem implementation of the FileRepository interface
pub struct FileFsRepository {
    root_path: PathBuf,
    folder_repository: Arc<dyn FolderRepository>,
    id_map: Mutex<FileIdMap>,
}

impl FileFsRepository {
    /// Creates a new filesystem-based file repository
    pub fn new(root_path: PathBuf, folder_repository: Arc<dyn FolderRepository>) -> Self {
        let id_map = Mutex::new(Self::load_id_map(&root_path));
        Self { root_path, folder_repository, id_map }
    }
    
    /// Loads the file ID map from disk
    fn load_id_map(root_path: &PathBuf) -> FileIdMap {
        let map_path = root_path.join("file_ids.json");
        
        if map_path.exists() {
            match std::fs::read_to_string(&map_path) {
                Ok(content) => {
                    match serde_json::from_str::<FileIdMap>(&content) {
                        Ok(map) => {
                            tracing::info!("Loaded file ID map with {} entries", map.path_to_id.len());
                            return map;
                        },
                        Err(e) => {
                            tracing::error!("Error parsing file ID map: {}", e);
                        }
                    }
                },
                Err(e) => {
                    tracing::error!("Error reading file ID map: {}", e);
                }
            }
        }
        
        // Return empty map if file doesn't exist or there was an error
        FileIdMap::default()
    }
    
    /// Saves the file ID map to disk
    fn save_id_map(&self) {
        let map_path = self.root_path.join("file_ids.json");
        
        let map = self.id_map.lock().unwrap();
        match serde_json::to_string_pretty(&*map) {
            Ok(json) => {
                match std::fs::write(&map_path, json) {
                    Ok(_) => {
                        tracing::info!("Saved file ID map with {} entries", map.path_to_id.len());
                    },
                    Err(e) => {
                        tracing::error!("Error writing file ID map: {}", e);
                    }
                }
            },
            Err(e) => {
                tracing::error!("Error serializing file ID map: {}", e);
            }
        }
    }
    
    /// Generates a unique ID for a file
    fn generate_id(&self) -> String {
        Uuid::new_v4().to_string()
    }
    
    /// Gets ID for a file path or generates a new one
    fn get_or_create_id(&self, path: &Path) -> String {
        let path_str = path.to_string_lossy().to_string();
        
        let mut map = self.id_map.lock().unwrap();
        
        // Check if we already have an ID for this path
        if let Some(id) = map.path_to_id.get(&path_str) {
            return id.clone();
        }
        
        // Generate a new ID
        let id = self.generate_id();
        map.path_to_id.insert(path_str, id.clone());
        
        // No need to save immediately - we'll save when file operations complete
        
        id
    }
    
    /// Resolves a relative path to an absolute path
    fn resolve_path(&self, relative_path: &Path) -> PathBuf {
        self.root_path.join(relative_path)
    }
}

#[async_trait]
impl FileRepository for FileFsRepository {
    async fn get_folder_by_id(&self, id: &str) -> FileRepositoryResult<crate::domain::entities::folder::Folder> {
        match self.folder_repository.get_folder_by_id(id).await {
            Ok(folder) => Ok(folder),
            Err(e) => Err(FileRepositoryError::Other(format!("Folder not found: {}", e))),
        }
    }
    async fn save_file_from_bytes(
        &self,
        name: String,
        folder_id: Option<String>,
        content_type: String,
        content: Vec<u8>,
    ) -> FileRepositoryResult<File>
    {
        // Get the folder path
        let folder_path = match &folder_id {
            Some(id) => {
                match self.folder_repository.get_folder_by_id(id).await {
                    Ok(folder) => {
                        tracing::info!("Using folder path: {:?} for folder_id: {:?}", folder.path, id);
                        folder.path
                    },
                    Err(e) => {
                        tracing::error!("Error getting folder: {}", e);
                        PathBuf::new()
                    },
                }
            },
            None => PathBuf::new(),
        };
        
        // Create the file path
        let file_path = if folder_path.as_os_str().is_empty() {
            PathBuf::from(&name)
        } else {
            folder_path.join(&name)
        };
        tracing::info!("Created file path: {:?}", file_path);
        
        // Check if file already exists
        let exists = self.file_exists(&file_path).await?;
        tracing::info!("File exists check: {} for path: {:?}", exists, file_path);
        
        if exists {
            tracing::warn!("File already exists at path: {:?}", file_path);
            return Err(FileRepositoryError::AlreadyExists(file_path.to_string_lossy().to_string()));
        }
        
        // Create parent directories if they don't exist
        let abs_path = self.resolve_path(&file_path);
        if let Some(parent) = abs_path.parent() {
            fs::create_dir_all(parent).await
                .map_err(FileRepositoryError::IoError)?;
        }
        
        // Write the file
        let mut file = fs::File::create(&abs_path).await
            .map_err(FileRepositoryError::IoError)?;
        
        file.write_all(&content).await
            .map_err(FileRepositoryError::IoError)?;
        
        // Get file metadata
        let metadata = fs::metadata(&abs_path).await
            .map_err(FileRepositoryError::IoError)?;
            
        // Determine the MIME type
        let mime_type = if content_type.is_empty() {
            from_path(&file_path)
                .first_or_octet_stream()
                .to_string()
        } else {
            content_type
        };
        
        // Create and return the file entity with a persistent ID
        let id = self.get_or_create_id(&file_path);
        let file = File::new(
            id,
            name,
            file_path.clone(),
            metadata.len(),
            mime_type,
            folder_id,
        );
        
        // Save the ID map
        self.save_id_map();
        
        tracing::info!("Saved file: {} with ID: {}", file_path.display(), file.id);
        Ok(file)
    }
    
    async fn save_file_with_id(
        &self,
        id: String,
        name: String,
        folder_id: Option<String>,
        content_type: String,
        content: Vec<u8>,
    ) -> FileRepositoryResult<File>
    {
        // Get the folder path
        let folder_path = match &folder_id {
            Some(fid) => {
                match self.folder_repository.get_folder_by_id(fid).await {
                    Ok(folder) => {
                        tracing::info!("Using folder path: {:?} for folder_id: {:?}", folder.path, fid);
                        folder.path
                    },
                    Err(e) => {
                        tracing::error!("Error getting folder: {}", e);
                        PathBuf::new()
                    },
                }
            },
            None => PathBuf::new(),
        };
        
        // Create the file path
        let file_path = if folder_path.as_os_str().is_empty() {
            PathBuf::from(&name)
        } else {
            folder_path.join(&name)
        };
        tracing::info!("Created file path with ID: {:?} for file: {}", file_path, id);
        
        // Check if file already exists (different from the one we're moving)
        let exists = self.file_exists(&file_path).await?;
        tracing::info!("File exists check: {} for path: {:?}", exists, file_path);
        
        // For save_file_with_id, we'll force overwrite if needed
        if exists {
            tracing::warn!("File already exists at path: {:?} - will overwrite", file_path);
            // Delete the existing file
            let abs_path = self.resolve_path(&file_path);
            if let Err(e) = fs::remove_file(&abs_path).await {
                tracing::error!("Failed to delete existing file: {} - {}", abs_path.display(), e);
                return Err(FileRepositoryError::IoError(e));
            }
        }
        
        // Create parent directories if they don't exist
        let abs_path = self.resolve_path(&file_path);
        if let Some(parent) = abs_path.parent() {
            fs::create_dir_all(parent).await
                .map_err(FileRepositoryError::IoError)?;
        }
        
        // Write the file
        let mut file = fs::File::create(&abs_path).await
            .map_err(FileRepositoryError::IoError)?;
        
        file.write_all(&content).await
            .map_err(FileRepositoryError::IoError)?;
        
        // Get file metadata
        let metadata = fs::metadata(&abs_path).await
            .map_err(FileRepositoryError::IoError)?;
            
        // Determine the MIME type
        let mime_type = if content_type.is_empty() {
            from_path(&file_path)
                .first_or_octet_stream()
                .to_string()
        } else {
            content_type
        };
        
        // Create the file entity with the provided ID
        let file_entity = File::new(
            id.clone(),
            name,
            file_path.clone(),
            metadata.len(),
            mime_type,
            folder_id,
        );
        
        // Update the ID map
        {
            let mut map = self.id_map.lock().unwrap();
            
            // First, remove any existing entries for this ID
            let entries_to_remove: Vec<String> = map.path_to_id.iter()
                .filter(|(_, v)| **v == id)
                .map(|(k, _)| k.clone())
                .collect();
                
            for key in entries_to_remove {
                tracing::info!("Removing old mapping: {} -> {}", key, id);
                map.path_to_id.remove(&key);
            }
            
            // Then add the new entry
            let path_str = file_path.to_string_lossy().to_string();
            tracing::info!("Adding new mapping: {} -> {}", path_str, id);
            map.path_to_id.insert(path_str, id);
        }
        
        // Save the ID map
        self.save_id_map();
        
        tracing::info!("Saved file with specific ID: {} at path: {}", file_entity.id, file_path.display());
        Ok(file_entity)
    }
    
    async fn get_file_by_id(&self, id: &str) -> FileRepositoryResult<File> {
        // Find path by ID in the map
        let path_str = {
            let map = self.id_map.lock().unwrap();
            match map.path_to_id.iter().find(|(_, v)| v == &id) {
                Some((path, _)) => path.clone(),
                None => {
                    tracing::error!("No file found with ID: {}", id);
                    return Err(FileRepositoryError::NotFound(id.to_string()));
                }
            }
        };
        
        // Convert path string to PathBuf
        let file_path = PathBuf::from(path_str);
        
        // Check if file exists
        let abs_path = self.resolve_path(&file_path);
        if !abs_path.exists() || !abs_path.is_file() {
            tracing::error!("File not found at path: {}", file_path.display());
            return Err(FileRepositoryError::NotFound(format!("File {} not found at {}", id, file_path.display())));
        }
        
        // Get file metadata
        let metadata = fs::metadata(&abs_path).await
            .map_err(|e| {
                tracing::error!("Error getting metadata: {}", e);
                FileRepositoryError::IoError(e)
            })?;
        
        // Get file name
        let name = match file_path.file_name() {
            Some(os_str) => os_str.to_string_lossy().to_string(),
            None => {
                tracing::error!("Invalid file path: {}", file_path.display());
                return Err(FileRepositoryError::InvalidPath(file_path.to_string_lossy().to_string()));
            }
        };
        
        // Determine parent folder ID
        let parent_dir = file_path.parent().unwrap_or(Path::new(""));
        let folder_id = if parent_dir.as_os_str().is_empty() {
            None
        } else {
            let parent_path_buf = PathBuf::from(parent_dir);
            match self.folder_repository.get_folder_by_path(&parent_path_buf).await {
                Ok(folder) => Some(folder.id),
                Err(_) => None,
            }
        };
        
        // Determine MIME type
        let mime_type = from_path(&file_path)
            .first_or_octet_stream()
            .to_string();
        
        // Create file entity
        let mut file = File::new(
            id.to_string(),
            name,
            file_path,
            metadata.len(),
            mime_type,
            folder_id,
        );
        
        // Set timestamps if available
        if let Ok(created) = metadata.created() {
            if let Ok(since_epoch) = created.duration_since(std::time::UNIX_EPOCH) {
                file.created_at = since_epoch.as_secs();
            }
        }
        
        if let Ok(modified) = metadata.modified() {
            if let Ok(since_epoch) = modified.duration_since(std::time::UNIX_EPOCH) {
                file.modified_at = since_epoch.as_secs();
            }
        }
        
        Ok(file)
    }
    
    async fn list_files(&self, folder_id: Option<&str>) -> FileRepositoryResult<Vec<File>> {
        let mut files = Vec::new();
        
        tracing::info!("Listing files in folder_id: {:?}", folder_id);
        
        // Get the folder path
        let folder_path = match folder_id {
            Some(id) => {
                match self.folder_repository.get_folder_by_id(id).await {
                    Ok(folder) => {
                        tracing::info!("Found folder with path: {:?}", folder.path);
                        folder.path
                    },
                    Err(e) => {
                        tracing::error!("Error getting folder by ID: {}: {}", id, e);
                        return Ok(Vec::new());
                    },
                }
            },
            None => PathBuf::new(),
        };
        
        // Get the absolute folder path
        let abs_folder_path = self.resolve_path(&folder_path);
        tracing::info!("Absolute folder path: {:?}", abs_folder_path);
        
        // Ensure the directory exists
        if !abs_folder_path.exists() || !abs_folder_path.is_dir() {
            tracing::error!("Directory does not exist or is not a directory: {:?}", abs_folder_path);
            return Ok(Vec::new());
        }
        
        tracing::info!("Directory exists, reading contents");
        
        // Alternative approach - check files in the map that belong to this folder_id
        let file_candidates: Vec<(String, String)> = {
            let map = self.id_map.lock().unwrap();
            tracing::info!("Checking map with {} entries for files in folder_id: {:?}", map.path_to_id.len(), folder_id);
            
            // Filter by folder path prefix and collect filtered entries
            let candidates = map.path_to_id.iter()
                .filter(|(path_str, _)| {
                    let path = PathBuf::from(path_str);
                    
                    // Check if this file belongs to the requested folder
                    match &folder_id {
                        Some(_) => {
                            // For specific folder, check if path starts with folder path
                            let parent_path = path.parent().unwrap_or_else(|| Path::new(""));
                            parent_path == folder_path
                        },
                        None => {
                            // For root folder, check if file is directly in root (no parent or parent is empty)
                            let parent = path.parent().unwrap_or_else(|| Path::new(""));
                            parent.as_os_str().is_empty() || parent == Path::new(".")
                        }
                    }
                })
                .map(|(path, id)| (path.clone(), id.clone()))
                .collect();
                
            candidates
        };
        
        // Process candidates after releasing the mutex lock
        for (path_str, file_id) in file_candidates {
            let path = PathBuf::from(&path_str);
            tracing::info!("Found file in target folder: {} with ID: {}", path_str, file_id);
            
            // Verify file exists physically
            let abs_path = self.resolve_path(&path);
            if !abs_path.exists() || !abs_path.is_file() {
                tracing::warn!("File in map doesn't exist physically: {} (ID: {})", path_str, file_id);
                continue;
            }
            
            // Get file info
            match fs::metadata(&abs_path).await {
                Ok(metadata) => {
                    let file_name = path.file_name()
                        .map(|os_str| os_str.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unnamed".to_string());
                        
                    // Determine MIME type
                    let mime_type = from_path(&path)
                        .first_or_octet_stream()
                        .to_string();
                    
                    // Get timestamps
                    let created_at = metadata.created()
                        .map(|time| time.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
                        .unwrap_or_else(|_| 0);
                        
                    let modified_at = metadata.modified()
                        .map(|time| time.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
                        .unwrap_or_else(|_| 0);
                    
                    let mut file = File::new(
                        file_id.clone(),
                        file_name.clone(),
                        path.clone(),
                        metadata.len(),
                        mime_type,
                        folder_id.map(String::from),
                    );
                    
                    file.created_at = created_at;
                    file.modified_at = modified_at;
                    
                    tracing::info!("Adding file to result list: {} (path: {:?})", file.name, path);
                    files.push(file);
                },
                Err(e) => {
                    tracing::warn!("Failed to get metadata for file: {} - {}", path_str, e);
                }
            }
        }
            
        if !files.is_empty() {
            tracing::info!("Found {} files in folder {:?} from map", files.len(), folder_id);
            return Ok(files);
        }
        
        // If we didn't find files in the map or the list is empty, fall back to directory scan
        tracing::info!("Scanning directory for files: {:?}", abs_folder_path);
        
        // Read directory entries
        let mut entries = fs::read_dir(abs_folder_path).await
            .map_err(FileRepositoryError::IoError)?;
            
        while let Some(entry) = entries.next_entry().await
            .map_err(FileRepositoryError::IoError)? {
            
            let path = entry.path();
            tracing::info!("Found entry: {:?}", path);
            
            let metadata = entry.metadata().await
                .map_err(FileRepositoryError::IoError)?;
                
            // Only include files, not directories
            if metadata.is_file() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                tracing::info!("Found file: {}", file_name);
                
                let file_path = if folder_path.as_os_str().is_empty() {
                    PathBuf::from(&file_name)
                } else {
                    folder_path.join(&file_name)
                };
                
                tracing::info!("File path (relative to root): {:?}", file_path);
                
                // Determine MIME type
                let mime_type = from_path(&file_path)
                    .first_or_octet_stream()
                    .to_string();
                
                // Get timestamps
                let created_at = metadata.created()
                    .map(|time| time.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
                    .unwrap_or_else(|_| 0);
                    
                let modified_at = metadata.modified()
                    .map(|time| time.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
                    .unwrap_or_else(|_| 0);
                
                // Check if this file is already in our list (could happen with case-insensitive filesystems)
                let duplicate = files.iter().any(|f: &File| f.name.to_lowercase() == file_name.to_lowercase());
                if duplicate {
                    tracing::warn!("Skipping duplicate file with name: {} (case-insensitive match)", file_name);
                    continue;
                }
                
                // Create file entity with persistent ID
                let id = self.get_or_create_id(&file_path);
                tracing::info!("Using ID for file: {} for path: {:?}", id, file_path);
                
                // Check if file is a PDF - just for debugging
                if file_name.to_lowercase().ends_with(".pdf") {
                    tracing::info!("PDF file detected: {} with ID: {}", file_name, id);
                }
                
                let mut file = File::new(
                    id,
                    file_name,
                    file_path.clone(),
                    metadata.len(),
                    mime_type,
                    folder_id.map(String::from),
                );
                
                file.created_at = created_at;
                file.modified_at = modified_at;
                
                tracing::info!("Adding file to result list: {} (path: {:?})", file.name, file_path);
                files.push(file);
            } else {
                tracing::info!("Skipping directory: {:?}", path);
            }
        }
        
        tracing::info!("Found {} files in folder {:?}", files.len(), folder_id);
        
        // Let's see what's in the map
        {
            let map = self.id_map.lock().unwrap();
            tracing::info!("ID map has {} entries", map.path_to_id.len());
            for (path, id) in &map.path_to_id {
                tracing::info!("Map entry: {} -> {}", path, id);
            }
        }
        
        Ok(files)
    }
    
    async fn delete_file(&self, id: &str) -> FileRepositoryResult<()> {
        let file = self.get_file_by_id(id).await?;
        
        // Delete the physical file
        let abs_path = self.resolve_path(&file.path);
        tracing::info!("Deleting physical file: {}", abs_path.display());
        
        fs::remove_file(abs_path).await
            .map_err(FileRepositoryError::IoError)?;
        
        tracing::info!("Physical file deleted successfully: {}", file.path.display());    
        Ok(())
    }
    
    async fn delete_file_entry(&self, id: &str) -> FileRepositoryResult<()> {
        let file = self.get_file_by_id(id).await?;
        
        // Delete the physical file
        let abs_path = self.resolve_path(&file.path);
        tracing::info!("Deleting physical file and entry for ID: {}", id);
        
        // Try to delete the file, but continue even if it fails
        let delete_result = fs::remove_file(&abs_path).await;
        match &delete_result {
            Ok(_) => tracing::info!("Physical file deleted successfully: {}", file.path.display()),
            Err(e) => tracing::warn!("Failed to delete physical file: {} - {}", file.path.display(), e),
        };
        
        // Remove all entries for this ID from the map
        {
            let mut map = self.id_map.lock().unwrap();
            // We don't need the path string anymore since we're finding all entries for this ID
            
            // Find all paths that map to this ID
            let paths_to_remove: Vec<String> = map.path_to_id.iter()
                .filter(|(_, v)| **v == id)
                .map(|(k, _)| k.clone())
                .collect();
                
            // Remove each path
            for path in &paths_to_remove {
                tracing::info!("Removing map entry: {} -> {}", path, id);
                map.path_to_id.remove(path);
            }
            
            tracing::info!("Removed {} map entries for ID: {}", paths_to_remove.len(), id);
        }
        
        // Save the updated map
        self.save_id_map();
        
        // Return success if we deleted the file, otherwise propagate the error
        if delete_result.is_ok() {
            Ok(())
        } else {
            // Still return Ok - we've removed the entry from the map,
            // and we want the operation to continue even if the file deletion failed
            Ok(())
        }
    }
    
    async fn get_file_content(&self, id: &str) -> FileRepositoryResult<Vec<u8>> {
        let file = self.get_file_by_id(id).await?;
        
        // Read the file content
        let abs_path = self.resolve_path(&file.path);
        let content = fs::read(abs_path).await
            .map_err(FileRepositoryError::IoError)?;
            
        Ok(content)
    }
    
    async fn file_exists(&self, path: &PathBuf) -> FileRepositoryResult<bool> {
        let abs_path = self.resolve_path(path);
        
        // Check if file exists and is a file (not a directory)
        let exists = abs_path.exists() && abs_path.is_file();
        
        tracing::info!("Checking if file exists: {} - path: {}", exists, abs_path.display());
        
        // If it exists, try to get metadata to verify it's accessible
        if exists {
            match fs::metadata(&abs_path).await {
                Ok(metadata) => {
                    if metadata.is_file() {
                        tracing::info!("File exists and is accessible: {}", abs_path.display());
                        return Ok(true);
                    } else {
                        tracing::warn!("Path exists but is not a file: {}", abs_path.display());
                        return Ok(false);
                    }
                },
                Err(e) => {
                    tracing::warn!("File exists but metadata check failed: {} - {}", abs_path.display(), e);
                    return Ok(false);
                }
            }
        }
        
        Ok(false)
    }
}