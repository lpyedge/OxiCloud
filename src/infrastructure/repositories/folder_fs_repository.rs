use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use tokio::fs;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use crate::domain::entities::folder::Folder;
use crate::domain::repositories::folder_repository::{
    FolderRepository, FolderRepositoryError, FolderRepositoryResult
};

/// Estructura para almacenar la relación entre paths y IDs
#[derive(Debug, Serialize, Deserialize, Default)]
struct FolderIdMap {
    path_to_id: HashMap<String, String>,
}

/// Filesystem implementation of the FolderRepository interface
pub struct FolderFsRepository {
    root_path: PathBuf,
    id_map: Arc<Mutex<FolderIdMap>>,
}

impl FolderFsRepository {
    /// Creates a new filesystem-based folder repository
    pub fn new(root_path: PathBuf) -> Self {
        // Crear o cargar el mapeo de IDs
        let id_map = Arc::new(Mutex::new(FolderIdMap::default()));
        
        // Intentar cargar el mapeo existente si existe
        let map_path = root_path.join("folder_ids.json");
        if let Ok(contents) = std::fs::read_to_string(map_path) {
            if let Ok(loaded_map) = serde_json::from_str::<FolderIdMap>(&contents) {
                let mut map = id_map.lock().unwrap();
                *map = loaded_map;
                tracing::info!("Loaded folder ID map with {} entries", map.path_to_id.len());
            }
        }
        
        Self { root_path, id_map }
    }
    
    /// Guarda el mapeo de IDs a disco
    fn save_id_map(&self) {
        let map_path = self.root_path.join("folder_ids.json");
        let map = self.id_map.lock().unwrap();
        if let Ok(json) = serde_json::to_string_pretty(&*map) {
            std::fs::write(map_path, json).ok();
        }
    }
    
    /// Obtiene o genera un ID para un path
    fn get_or_create_id(&self, path: &Path) -> String {
        let path_str = path.to_string_lossy().to_string();
        let mut map = self.id_map.lock().unwrap();
        
        if let Some(id) = map.path_to_id.get(&path_str) {
            return id.clone();
        }
        
        // Si no existe, genera un nuevo ID
        let id = Uuid::new_v4().to_string();
        map.path_to_id.insert(path_str, id.clone());
        
        // Guardar el mapa actualizado
        drop(map); // Liberar el mutex antes de guardar
        self.save_id_map();
        
        id
    }
    
    /// Generates a unique ID for a folder
    #[allow(dead_code)]
    fn generate_id(&self) -> String {
        Uuid::new_v4().to_string()
    }
    
    /// Resolves a relative path to an absolute path
    fn resolve_path(&self, relative_path: &Path) -> PathBuf {
        self.root_path.join(relative_path)
    }
    
    /// Creates the physical directory on the filesystem
    async fn create_directory(&self, path: &Path) -> Result<(), std::io::Error> {
        fs::create_dir_all(path).await
    }
}

#[async_trait]
impl FolderRepository for FolderFsRepository {
    async fn create_folder(&self, name: String, parent_path: Option<PathBuf>) -> FolderRepositoryResult<Folder> {
        // Calculate the new folder path
        let path = match &parent_path {
            Some(parent) => parent.join(&name),
            None => PathBuf::from(&name),
        };
        
        // Check if folder already exists
        if self.folder_exists(&path).await? {
            return Err(FolderRepositoryError::AlreadyExists(path.to_string_lossy().to_string()));
        }
        
        // Create the physical directory
        let abs_path = self.resolve_path(&path);
        self.create_directory(&abs_path).await
            .map_err(FolderRepositoryError::IoError)?;
        
        // Determine parent ID if any
        let parent_id = if let Some(parent) = &parent_path {
            if !parent.as_os_str().is_empty() {
                let parent_folder = self.get_folder_by_path(parent).await?;
                Some(parent_folder.id)
            } else {
                None
            }
        } else {
            None
        };
        
        // Create and return the folder entity with a persisted ID
        let id = self.get_or_create_id(&path);
        let folder = Folder::new(id, name, path, parent_id);
        
        tracing::debug!("Created folder with ID: {}", folder.id);
        Ok(folder)
    }
    
    async fn get_folder_by_id(&self, id: &str) -> FolderRepositoryResult<Folder> {
        tracing::debug!("Buscando carpeta con ID: {}", id);
        
        // First try to find the path associated with this ID
        let path_opt = {
            let map = self.id_map.lock().unwrap();
            // Invertir el mapeo para buscar por ID
            map.path_to_id.iter()
                .find_map(|(path, folder_id)| if folder_id == id { Some(path.clone()) } else { None })
        };
        
        if let Some(path_str) = path_opt {
            let path = PathBuf::from(path_str);
            tracing::debug!("Encontrado path para ID {}: {:?}", id, path);
            return self.get_folder_by_path(&path).await;
        }
        
        // Fallback: buscar a través de todas las carpetas
        tracing::debug!("ID {} no encontrado en el mapa, buscando a través de todas las carpetas", id);
        let all_folders = self.list_folders(None).await?;
        
        // Imprimir IDs disponibles para depuración
        for folder in &all_folders {
            tracing::debug!("Carpeta disponible - ID: {}, Nombre: {}", folder.id, folder.name);
        }
        
        // Find the folder with the matching ID
        all_folders.into_iter()
            .find(|folder| folder.id == id)
            .ok_or_else(|| FolderRepositoryError::NotFound(id.to_string()))
    }
    
    async fn get_folder_by_path(&self, path: &PathBuf) -> FolderRepositoryResult<Folder> {
        // Check if the physical directory exists
        let abs_path = self.resolve_path(path);
        if !abs_path.exists() || !abs_path.is_dir() {
            return Err(FolderRepositoryError::NotFound(path.to_string_lossy().to_string()));
        }
        
        // Extract folder name and parent path
        let name = path.file_name()
            .ok_or_else(|| FolderRepositoryError::InvalidPath(path.to_string_lossy().to_string()))?
            .to_string_lossy()
            .to_string();
            
        let parent_path = path.parent().map(|p| p.to_path_buf());
        
        // Determine parent ID if any
        let parent_id = if let Some(parent) = &parent_path {
            if !parent.as_os_str().is_empty() {
                match self.get_folder_by_path(parent).await {
                    Ok(parent_folder) => Some(parent_folder.id),
                    Err(_) => None,
                }
            } else {
                None
            }
        } else {
            None
        };
        
        // Get a consistent ID for this path
        let id = self.get_or_create_id(path);
        tracing::debug!("Found folder with path: {:?}, assigned ID: {}", path, id);
        
        // Get folder metadata for timestamps
        let metadata = fs::metadata(&abs_path).await
            .map_err(FolderRepositoryError::IoError)?;
            
        let created_at = metadata.created()
            .map(|time| time.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
            .unwrap_or_else(|_| 0);
            
        let modified_at = metadata.modified()
            .map(|time| time.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
            .unwrap_or_else(|_| 0);
        
        // Create and return the folder entity
        let mut folder = Folder::new(id, name, path.clone(), parent_id);
        folder.created_at = created_at;
        folder.modified_at = modified_at;
        
        Ok(folder)
    }
    
    async fn list_folders(&self, parent_id: Option<&str>) -> FolderRepositoryResult<Vec<Folder>> {
        let parent_path = match parent_id {
            Some(id) => {
                let parent = self.get_folder_by_id(id).await?;
                parent.path
            },
            None => PathBuf::from(""),
        };
        
        let abs_parent_path = self.resolve_path(&parent_path);
        let mut folders = Vec::new();
        
        // Read directory entries
        let mut entries = fs::read_dir(abs_parent_path).await
            .map_err(FolderRepositoryError::IoError)?;
            
        while let Some(entry) = entries.next_entry().await
            .map_err(FolderRepositoryError::IoError)? {
            
            let metadata = entry.metadata().await
                .map_err(FolderRepositoryError::IoError)?;
                
            // Only include directories
            if metadata.is_dir() {
                let path = if parent_path.as_os_str().is_empty() {
                    PathBuf::from(entry.file_name())
                } else {
                    parent_path.join(entry.file_name())
                };
                
                match self.get_folder_by_path(&path).await {
                    Ok(folder) => folders.push(folder),
                    Err(_) => continue,
                }
            }
        }
        
        Ok(folders)
    }
    
    async fn rename_folder(&self, id: &str, new_name: String) -> FolderRepositoryResult<Folder> {
        let folder = self.get_folder_by_id(id).await?;
        tracing::debug!("Renombrando carpeta con ID: {}, Nombre: {}", id, folder.name);
        
        // Calculate new path
        let parent_path = folder.path.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from(""));
            
        let new_path = if parent_path.as_os_str().is_empty() {
            PathBuf::from(&new_name)
        } else {
            parent_path.join(&new_name)
        };
        
        // Check if target already exists
        if self.folder_exists(&new_path).await? {
            return Err(FolderRepositoryError::AlreadyExists(new_path.to_string_lossy().to_string()));
        }
        
        // Rename the physical directory
        let abs_old_path = self.resolve_path(&folder.path);
        let abs_new_path = self.resolve_path(&new_path);
        
        fs::rename(&abs_old_path, &abs_new_path).await
            .map_err(FolderRepositoryError::IoError)?;
            
        // Actualizar el mapa de IDs - eliminar la entrada antigua y añadir la nueva
        let path_str = new_path.to_string_lossy().to_string();
        {
            let mut map = self.id_map.lock().unwrap();
            let old_path_str = folder.path.to_string_lossy().to_string();
            map.path_to_id.remove(&old_path_str);
            map.path_to_id.insert(path_str.clone(), id.to_string());
        }
        
        // Guardar el mapa actualizado
        self.save_id_map();
        
        // Create and return updated folder entity
        let mut updated_folder = Folder::new(
            folder.id.clone(),
            new_name,
            new_path.clone(),
            folder.parent_id.clone(),
        );
        updated_folder.created_at = folder.created_at;
        updated_folder.touch();
        
        tracing::debug!("Carpeta renombrada exitosamente: ID={}, Nuevo nombre={}", id, updated_folder.name);
        Ok(updated_folder)
    }
    
    async fn move_folder(&self, id: &str, new_parent_id: Option<&str>) -> FolderRepositoryResult<Folder> {
        let folder = self.get_folder_by_id(id).await?;
        tracing::debug!("Moviendo carpeta con ID: {}, Nombre: {}", id, folder.name);
        
        // Get new parent path
        let new_parent_path = match new_parent_id {
            Some(parent_id) => {
                let parent = self.get_folder_by_id(parent_id).await?;
                parent.path
            },
            None => PathBuf::from(""),
        };
        
        // Calculate new path
        let new_path = if new_parent_path.as_os_str().is_empty() {
            PathBuf::from(&folder.name)
        } else {
            new_parent_path.join(&folder.name)
        };
        
        // Check if target already exists
        if self.folder_exists(&new_path).await? {
            return Err(FolderRepositoryError::AlreadyExists(new_path.to_string_lossy().to_string()));
        }
        
        // Move the physical directory
        let abs_old_path = self.resolve_path(&folder.path);
        let abs_new_path = self.resolve_path(&new_path);
        
        fs::rename(&abs_old_path, &abs_new_path).await
            .map_err(FolderRepositoryError::IoError)?;
            
        // Actualizar el mapa de IDs - eliminar la entrada antigua y añadir la nueva
        let path_str = new_path.to_string_lossy().to_string();
        {
            let mut map = self.id_map.lock().unwrap();
            let old_path_str = folder.path.to_string_lossy().to_string();
            map.path_to_id.remove(&old_path_str);
            map.path_to_id.insert(path_str.clone(), id.to_string());
        }
        
        // Guardar el mapa actualizado
        self.save_id_map();
        
        // Create and return updated folder entity
        let new_parent_id = if let Some(parent_id) = new_parent_id {
            Some(parent_id.to_string())
        } else {
            None
        };
        
        let mut updated_folder = Folder::new(
            folder.id.clone(),
            folder.name.clone(),
            new_path.clone(),
            new_parent_id,
        );
        updated_folder.created_at = folder.created_at;
        updated_folder.touch();
        
        tracing::debug!("Carpeta movida exitosamente: ID={}, Nueva ruta={:?}", id, new_path);
        Ok(updated_folder)
    }
    
    async fn delete_folder(&self, id: &str) -> FolderRepositoryResult<()> {
        let folder = self.get_folder_by_id(id).await?;
        tracing::debug!("Eliminando carpeta con ID: {}, Nombre: {}", id, folder.name);
        
        // Delete the physical directory
        let abs_path = self.resolve_path(&folder.path);
        fs::remove_dir_all(abs_path).await
            .map_err(FolderRepositoryError::IoError)?;
            
        // Actualizar el mapa de IDs - eliminar la entrada
        {
            let mut map = self.id_map.lock().unwrap();
            let path_str = folder.path.to_string_lossy().to_string();
            map.path_to_id.remove(&path_str);
        }
        
        // Guardar el mapa actualizado
        self.save_id_map();
        
        tracing::debug!("Carpeta eliminada exitosamente: ID={}", id);
        Ok(())
    }
    
    async fn folder_exists(&self, path: &PathBuf) -> FolderRepositoryResult<bool> {
        let abs_path = self.resolve_path(path);
        Ok(abs_path.exists() && abs_path.is_dir())
    }
}