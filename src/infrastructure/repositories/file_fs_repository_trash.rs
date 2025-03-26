use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, error, instrument};

use crate::domain::repositories::file_repository::FileRepositoryResult;
use crate::infrastructure::repositories::file_fs_repository::FileFsRepository;

// Este archivo contiene la implementación de los métodos relacionados con la papelera
// para el repositorio de archivos FileFsRepository

// Implementación de métodos de papelera para el repositorio de archivos
impl FileFsRepository {
    // Obtiene la ruta completa a la papelera
    fn get_trash_dir(&self) -> PathBuf {
        let trash_dir = self.get_root_path().join(".trash").join("files");
        debug!("Base trash directory: {}", trash_dir.display());
        trash_dir
    }
    
    // Obtiene la ruta de la papelera para un usuario específico (si se proporciona)
    fn get_user_trash_dir(&self, user_id: Option<&str>) -> PathBuf {
        let base_trash_dir = self.get_trash_dir();
        
        if let Some(uid) = user_id {
            let user_trash_dir = base_trash_dir.join(uid);
            debug!("User-specific trash directory: {}", user_trash_dir.display());
            user_trash_dir
        } else {
            // Use a default user directory if not specified
            let default_dir = base_trash_dir.join("00000000-0000-0000-0000-000000000000");
            debug!("Default user trash directory: {}", default_dir.display());
            default_dir
        }
    }
    
    // Crea una ruta única en la papelera para el archivo
    async fn create_trash_file_path(&self, file_id: &str) -> FileRepositoryResult<PathBuf> {
        debug!("Creating trash file path for file ID: {}", file_id);
        
        // Get the trash directory for the default user
        let user_trash_dir = self.get_user_trash_dir(Some("00000000-0000-0000-0000-000000000000"));
        
        // Ensure the user's trash directory exists
        debug!("Ensuring user trash directory exists: {}", user_trash_dir.display());
        if !user_trash_dir.exists() {
            debug!("Creating user trash directory: {}", user_trash_dir.display());
            fs::create_dir_all(&user_trash_dir).await
                .map_err(|e| {
                    error!("Failed to create user trash directory: {}", e);
                    FileRepositoryError::IoError(e)
                })?;
            debug!("User trash directory created successfully");
        } else {
            debug!("User trash directory already exists");
        }
        
        // Create a unique path for the file in the trash
        let trash_file_path = user_trash_dir.join(file_id);
        debug!("Trash file path: {}", trash_file_path.display());
        
        Ok(trash_file_path)
    }
}

// Implementación de los métodos públicos del trait FileRepository relacionados con la papelera
// Note: The FileRepository trait implementation has been moved to file_fs_repository.rs
// to avoid duplicate implementations

// Implementation of internal methods for trash functionality
impl FileFsRepository {
    /// Helper method that will be used for trash functionality 
    pub(crate) async fn _trash_move_to_trash(&self, file_id: &str) -> FileRepositoryResult<()> {
        debug!("Moviendo archivo a la papelera: {}", file_id);
        
        // Obtener la ruta física del archivo
        // Creamos un método independiente para acceder al servicio de mapeo de IDs
        debug!("Obteniendo ruta del archivo con ID: {}", file_id);
        let file_path = match self.id_mapping_service().get_file_path(file_id).await {
            Ok(path) => {
                debug!("Ruta del archivo obtenida: {}", path.display());
                path
            },
            Err(e) => {
                error!("Error obteniendo ruta del archivo {}: {:?}", file_id, e);
                return Err(FileRepositoryError::IdMappingError(format!("Failed to get file path: {}", e)));
            }
        };
        
        // Verificamos que el archivo existe
        debug!("Verificando que el archivo existe: {}", file_path.display());
        if !self.file_exists(&file_path).await? {
            error!("Archivo no encontrado en la ruta especificada: {}", file_path.display());
            return Err(FileRepositoryError::NotFound(format!("File not found: {}", file_id)));
        }
        debug!("Archivo encontrado, continuando con la operación");
        
        // Crear directorio en la papelera si no existe
        debug!("Creando path para archivo en papelera");
        let trash_file_path = self.create_trash_file_path(file_id).await?;
        debug!("Path en papelera: {}", trash_file_path.display());
        
        // Mover el archivo físicamente a la papelera (no actualiza mappings)
        debug!("Moviendo archivo físicamente a papelera: {} -> {}", file_path.display(), trash_file_path.display());
        match fs::rename(&file_path, &trash_file_path).await {
            Ok(_) => {
                debug!("Archivo movido a papelera exitosamente: {} -> {}", file_path.display(), trash_file_path.display());
                
                // Invalidar la caché del archivo original
                debug!("Invalidando caché para: {}", file_path.display());
                self.metadata_cache().invalidate(&file_path).await;
                
                // Actualizar el mapeo al nuevo path en la papelera
                debug!("Actualizando mapeo de ID a nuevo path en papelera");
                if let Err(e) = self.id_mapping_service().update_file_path(file_id, &trash_file_path).await {
                    error!("Error actualizando mapeo de archivo en papelera: {}", e);
                    return Err(FileRepositoryError::MappingError(format!("Failed to update mapping: {}", e)));
                }
                debug!("Mapeo actualizado exitosamente");
                
                debug!("Operación de mover a papelera completada con éxito para el archivo: {}", file_id);
                Ok(())
            },
            Err(e) => {
                error!("Error moviendo archivo a papelera: {} -> {}: {}", 
                       file_path.display(), trash_file_path.display(), e);
                Err(FileRepositoryError::IoError(e))
            }
        }
    }
    
    /// Restaura un archivo desde la papelera a su ubicación original
    #[instrument(skip(self))]
    pub(crate) async fn _trash_restore_from_trash(&self, file_id: &str, original_path: &str) -> FileRepositoryResult<()> {
        debug!("Restaurando archivo {} a {}", file_id, original_path);
        
        // Try to get the current path from the ID mapping service
        let current_path_result = self.id_mapping_service().get_file_path(file_id).await;
        
        match current_path_result {
            Ok(current_path) => {
                debug!("Ruta actual en papelera: {}", current_path.display());
                
                // Check if the file exists in the trash
                let file_exists = match fs::metadata(&current_path).await {
                    Ok(_) => {
                        debug!("Archivo existe en papelera");
                        true
                    },
                    Err(e) => {
                        debug!("Archivo no existe en papelera: {} - {}", current_path.display(), e);
                        false
                    }
                };
                
                if !file_exists {
                    error!("El archivo no existe físicamente en la papelera: {}", current_path.display());
                    return Err(FileRepositoryError::NotFound(format!("File not found in trash: {}", file_id)));
                }
                
                // Parse the original path to a PathBuf
                let original_path_buf = PathBuf::from(original_path);
                debug!("Ruta original para restauración: {}", original_path_buf.display());
                
                // Check if a file already exists at the destination
                let target_exists = fs::metadata(&original_path_buf).await.is_ok();
                if target_exists {
                    debug!("Ya existe un archivo en la ruta de destino, generando ruta alternativa");
                    
                    // Generate a unique path by adding a suffix
                    // Extract filename and extension
                    let file_name = original_path_buf.file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "restored_file".to_string());
                        
                    let parent_dir = original_path_buf.parent()
                        .unwrap_or_else(|| std::path::Path::new(""));
                        
                    let (stem, ext) = if let Some(dot_pos) = file_name.rfind('.') {
                        (file_name[..dot_pos].to_string(), file_name[dot_pos..].to_string())
                    } else {
                        (file_name, "".to_string())
                    };
                    
                    // Create a new name with a timestamp
                    let timestamp = chrono::Utc::now().timestamp();
                    let new_name = format!("{}_{}{}", stem, timestamp, ext);
                    
                    // Create the alternative path
                    let alternative_path = parent_dir.join(new_name);
                    debug!("Ruta alternativa para restauración: {}", alternative_path.display());
                    
                    // Ensure the parent directory exists
                    if let Some(parent) = alternative_path.parent() {
                        if !parent.exists() {
                            debug!("Creando directorio padre para restauración: {}", parent.display());
                            match fs::create_dir_all(parent).await {
                                Ok(_) => debug!("Directorio padre creado exitosamente"),
                                Err(e) => {
                                    error!("Error creando directorio padre: {} - {}", parent.display(), e);
                                    return Err(FileRepositoryError::IoError(e));
                                }
                            }
                        }
                    }
                    
                    // Move the file from trash to the alternative location
                    debug!("Moviendo archivo de papelera a ubicación alternativa: {} -> {}", 
                           current_path.display(), alternative_path.display());
                    match fs::rename(&current_path, &alternative_path).await {
                        Ok(_) => {
                            debug!("Archivo restaurado exitosamente a ubicación alternativa");
                            
                            // Invalidate cache entries
                            debug!("Invalidando caché para archivo en papelera");
                            self.metadata_cache().invalidate(&current_path).await;
                            
                            // Update the ID mapping
                            debug!("Actualizando mapeo de ID a nueva ubicación");
                            if let Err(e) = self.id_mapping_service().update_file_path(file_id, &alternative_path).await {
                                error!("Error actualizando mapeo de archivo restaurado: {}", e);
                                return Err(FileRepositoryError::MappingError(
                                    format!("Failed to update mapping: {}", e)
                                ));
                            }
                            
                            debug!("Restauración a ubicación alternativa completada con éxito");
                            Ok(())
                        },
                        Err(e) => {
                            error!("Error restaurando archivo a ubicación alternativa: {}", e);
                            Err(FileRepositoryError::IoError(e))
                        }
                    }
                } else {
                    // Ensure the parent directory exists
                    if let Some(parent) = original_path_buf.parent() {
                        if !parent.exists() {
                            debug!("Creando directorio padre para restauración: {}", parent.display());
                            match fs::create_dir_all(parent).await {
                                Ok(_) => debug!("Directorio padre creado exitosamente"),
                                Err(e) => {
                                    error!("Error creando directorio padre: {} - {}", parent.display(), e);
                                    return Err(FileRepositoryError::IoError(e));
                                }
                            }
                        }
                    }
                    
                    // Move the file from trash to its original location
                    debug!("Moviendo archivo de papelera a ubicación original: {} -> {}", 
                           current_path.display(), original_path_buf.display());
                    match fs::rename(&current_path, &original_path_buf).await {
                        Ok(_) => {
                            debug!("Archivo restaurado exitosamente a ubicación original");
                            
                            // Invalidate cache entries
                            debug!("Invalidando caché para archivo en papelera");
                            self.metadata_cache().invalidate(&current_path).await;
                            
                            // Update the ID mapping
                            debug!("Actualizando mapeo de ID a ubicación original");
                            if let Err(e) = self.id_mapping_service().update_file_path(file_id, &original_path_buf).await {
                                error!("Error actualizando mapeo de archivo restaurado: {}", e);
                                return Err(FileRepositoryError::MappingError(
                                    format!("Failed to update mapping: {}", e)
                                ));
                            }
                            
                            debug!("Restauración a ubicación original completada con éxito");
                            Ok(())
                        },
                        Err(e) => {
                            error!("Error restaurando archivo a ubicación original: {}", e);
                            Err(FileRepositoryError::IoError(e))
                        }
                    }
                }
            },
            Err(e) => {
                error!("Error obteniendo ruta actual del archivo {}: {:?}", file_id, e);
                
                // Check if the error is because the ID was not found
                if format!("{}", e).contains("not found") {
                    debug!("ID no encontrado en mapeo, archivo ya no existe en papelera");
                    return Err(FileRepositoryError::NotFound(format!("File not found in trash: {}", file_id)));
                }
                
                return Err(FileRepositoryError::IdMappingError(
                    format!("Failed to get file path: {}", e)
                ));
            }
        }
    }
    
    /// Elimina un archivo permanentemente (usado por la papelera)
    #[instrument(skip(self))]
    pub(crate) async fn _trash_delete_file_permanently(&self, file_id: &str) -> FileRepositoryResult<()> {
        debug!("Eliminando archivo permanentemente: {}", file_id);
        
        // Get the file path using the ID mapping service
        let file_path_result = self.id_mapping_service().get_file_path(file_id).await;
        
        match file_path_result {
            Ok(file_path) => {
                debug!("Encontrada ruta para archivo: {} -> {}", file_id, file_path.display());
                
                // Check if the file physically exists before attempting to delete
                let file_exists = fs::metadata(&file_path).await.is_ok();
                
                if file_exists {
                    debug!("Archivo existe físicamente, eliminando: {}", file_path.display());
                    
                    // Delete the file physically
                    if let Err(e) = fs::remove_file(&file_path).await {
                        error!("Error eliminando archivo permanentemente: {} - {}", file_path.display(), e);
                        // Don't report error if the file already doesn't exist
                        if e.kind() != std::io::ErrorKind::NotFound {
                            return Err(FileRepositoryError::IoError(e));
                        }
                    } else {
                        debug!("Archivo eliminado físicamente con éxito");
                    }
                    
                    // Invalidate cache for this file
                    debug!("Invalidando caché para el archivo: {}", file_path.display());
                    self.metadata_cache().invalidate(&file_path).await;
                } else {
                    debug!("Archivo no existe físicamente, solo limpiando mapeos: {}", file_path.display());
                }
                
                // Always remove the ID mapping regardless of whether the file exists
                debug!("Eliminando mapeo de ID: {}", file_id);
                match self.id_mapping_service().remove_id(file_id).await {
                    Ok(_) => debug!("Mapeo de ID eliminado con éxito"),
                    Err(e) => {
                        error!("Error eliminando mapeo del archivo: {}", e);
                        // Only return error for critical mapping errors, otherwise continue
                        if format!("{}", e).contains("not found") {
                            debug!("ID mapping not found, ignoring this error for deletion");
                        } else {
                            return Err(FileRepositoryError::MappingError(format!("Failed to remove mapping: {}", e)));
                        }
                    }
                };
                
                debug!("Archivo eliminado permanentemente con éxito: {}", file_id);
                Ok(())
            },
            Err(e) => {
                // This could happen if the file is already deleted or wasn't properly indexed
                error!("Error obteniendo ruta del archivo {}: {:?}", file_id, e);
                
                // Check if the error is because the ID was not found
                if format!("{}", e).contains("not found") {
                    debug!("ID no encontrado en mapeo, considerando borrado exitoso: {}", file_id);
                    // In this case, we consider the file already deleted
                    return Ok(());
                }
                
                return Err(FileRepositoryError::IdMappingError(format!("Failed to get file path: {}", e)));
            }
        }
    }
}

// Re-exportaciones necesarias para el compilador
use crate::domain::repositories::file_repository::FileRepositoryError;