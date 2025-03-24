use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use async_trait::async_trait;
use tracing::{debug, error, instrument};

use crate::domain::repositories::file_repository::{FileRepository, FileRepositoryResult};
use crate::common::errors::ErrorKind;
use crate::infrastructure::repositories::file_fs_repository::FileFsRepository;

// Este archivo contiene la implementación de los métodos relacionados con la papelera
// para el repositorio de archivos FileFsRepository

// Implementación de métodos de papelera para el repositorio de archivos
impl FileFsRepository {
    // Obtiene la ruta completa a la papelera
    fn get_trash_dir(&self) -> PathBuf {
        self.get_root_path().join(".trash").join("files")
    }
    
    // Crea una ruta única en la papelera para el archivo
    async fn create_trash_file_path(&self, file_id: &str) -> FileRepositoryResult<PathBuf> {
        let trash_dir = self.get_trash_dir();
        
        // Asegurarse que el directorio de la papelera existe
        if !trash_dir.exists() {
            fs::create_dir_all(&trash_dir).await
                .map_err(|e| FileRepositoryError::IoError(e))?;
        }
        
        // Crear una ruta única para el archivo en la papelera
        Ok(trash_dir.join(file_id))
    }
}

// Implementación de los métodos públicos del trait FileRepository relacionados con la papelera
// Implementation of internal methods for trash functionality
// These will be enabled when the trash feature is re-enabled
impl FileFsRepository {
    /// Helper method that will be used for trash functionality 
    #[allow(dead_code)]
    pub(crate) async fn _trash_move_to_trash(&self, file_id: &str) -> FileRepositoryResult<()> {
        debug!("Moviendo archivo a la papelera: {}", file_id);
        
        // Obtener la ruta física del archivo
        // Creamos un método independiente para acceder al servicio de mapeo de IDs
        let file_path = match self.id_mapping_service().get_file_path(file_id).await {
            Ok(path) => path,
            Err(e) => {
                error!("Error obteniendo ruta del archivo {}: {:?}", file_id, e);
                return Err(FileRepositoryError::IdMappingError(format!("Failed to get file path: {}", e)));
            }
        };
        
        // Verificamos que el archivo existe
        if !self.file_exists(&file_path).await? {
            return Err(FileRepositoryError::NotFound(format!("File not found: {}", file_id)));
        }
        
        // Crear directorio en la papelera si no existe
        let trash_file_path = self.create_trash_file_path(file_id).await?;
        
        // Mover el archivo físicamente a la papelera (no actualiza mappings)
        match fs::rename(&file_path, &trash_file_path).await {
            Ok(_) => {
                debug!("Archivo movido a papelera: {} -> {}", file_path.display(), trash_file_path.display());
                
                // Invalidar la caché del archivo original
                self.metadata_cache().invalidate(&file_path).await;
                
                // Actualizar el mapeo al nuevo path en la papelera
                if let Err(e) = self.id_mapping_service().update_file_path(file_id, &trash_file_path).await {
                    error!("Error actualizando mapeo de archivo en papelera: {}", e);
                    return Err(FileRepositoryError::MappingError(format!("Failed to update mapping: {}", e)));
                }
                
                Ok(())
            },
            Err(e) => {
                error!("Error moviendo archivo a papelera: {}", e);
                Err(FileRepositoryError::IoError(e))
            }
        }
    }
    
    /// Restaura un archivo desde la papelera a su ubicación original
    #[allow(dead_code)]
    pub(crate) async fn _trash_restore_from_trash(&self, file_id: &str, original_path: &str) -> FileRepositoryResult<()> {
        debug!("Restaurando archivo {} a {}", file_id, original_path);
        
        // Obtener la ruta actual en la papelera
        let current_path = match self.id_mapping_service().get_file_path(file_id).await {
            Ok(path) => path,
            Err(e) => {
                error!("Error obteniendo ruta actual del archivo {}: {:?}", file_id, e);
                return Err(FileRepositoryError::IdMappingError(format!("Failed to get file path: {}", e)));
            }
        };
        
        // Convertir la ruta original a PathBuf
        let original_path_buf = PathBuf::from(original_path);
        
        // Asegurar que el directorio de destino existe
        if let Some(parent) = original_path_buf.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await
                    .map_err(|e| {
                        error!("Error creando directorio padre para restauración: {}", e);
                        FileRepositoryError::IoError(e)
                    })?;
            }
        }
        
        // Mover el archivo de la papelera a su ubicación original
        match fs::rename(&current_path, &original_path_buf).await {
            Ok(_) => {
                debug!("Archivo restaurado: {} -> {}", current_path.display(), original_path_buf.display());
                
                // Invalidar la caché del archivo en la papelera
                self.metadata_cache().invalidate(&current_path).await;
                
                // Actualizar el mapeo a la ruta original
                if let Err(e) = self.id_mapping_service().update_file_path(file_id, &original_path_buf).await {
                    error!("Error actualizando mapeo de archivo restaurado: {}", e);
                    return Err(FileRepositoryError::MappingError(format!("Failed to update mapping: {}", e)));
                }
                
                Ok(())
            },
            Err(e) => {
                error!("Error restaurando archivo: {}", e);
                Err(FileRepositoryError::IoError(e))
            }
        }
    }
    
    /// Elimina un archivo permanentemente (usado por la papelera)
    #[instrument(skip(self))]
    #[allow(dead_code)]
    pub(crate) async fn _trash_delete_file_permanently(&self, file_id: &str) -> FileRepositoryResult<()> {
        debug!("Eliminando archivo permanentemente: {}", file_id);
        
        // Este es similar al delete_file pero no verifica permisos ni hace validaciones adicionales
        let file_path = match self.id_mapping_service().get_file_path(file_id).await {
            Ok(path) => path,
            Err(e) => {
                error!("Error obteniendo ruta del archivo {}: {:?}", file_id, e);
                return Err(FileRepositoryError::IdMappingError(format!("Failed to get file path: {}", e)));
            }
        };
        
        // Eliminar el archivo físicamente
        if let Err(e) = fs::remove_file(&file_path).await {
            error!("Error eliminando archivo permanentemente: {}", e);
            // No reporte error si el archivo ya no existe
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(FileRepositoryError::IoError(e));
            }
        }
        
        // Invalidar caché
        self.metadata_cache().invalidate(&file_path).await;
        
        // Eliminar el mapeo
        if let Err(e) = self.id_mapping_service().remove_id(file_id).await {
            error!("Error eliminando mapeo del archivo: {}", e);
            return Err(FileRepositoryError::MappingError(format!("Failed to remove mapping: {}", e)));
        }
        
        debug!("Archivo eliminado permanentemente con éxito: {}", file_id);
        Ok(())
    }
}

// Re-exportaciones necesarias para el compilador
use crate::domain::repositories::file_repository::FileRepositoryError;