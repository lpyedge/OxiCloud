use std::path::PathBuf;
use std::sync::Arc;
use async_trait::async_trait;

use crate::domain::entities::file::File;
use crate::application::ports::storage_ports::FileWritePort;
use crate::common::errors::DomainError;
use crate::domain::repositories::file_repository::FileRepositoryResult;
use crate::infrastructure::repositories::file_metadata_manager::{FileMetadataManager, MetadataError};
use crate::infrastructure::repositories::file_path_resolver::FilePathResolver;
use crate::domain::services::path_service::StoragePath;
use crate::infrastructure::repositories::parallel_file_processor::ParallelFileProcessor;
use crate::common::config::AppConfig;
use crate::application::services::storage_mediator::StorageMediator;

/// Implementación de repositorio para operaciones de escritura de archivos
pub struct FileFsWriteRepository {
    root_path: PathBuf,
    metadata_manager: Arc<FileMetadataManager>,
    path_resolver: Arc<FilePathResolver>,
    storage_mediator: Arc<dyn StorageMediator>,
    config: AppConfig,
    parallel_processor: Option<Arc<ParallelFileProcessor>>,
}

impl FileFsWriteRepository {
    /// Crea un nuevo repositorio de escritura de archivos
    pub fn new(
        root_path: PathBuf,
        metadata_manager: Arc<FileMetadataManager>,
        path_resolver: Arc<FilePathResolver>,
        storage_mediator: Arc<dyn StorageMediator>,
        config: AppConfig,
        parallel_processor: Option<Arc<ParallelFileProcessor>>,
    ) -> Self {
        Self {
            root_path,
            metadata_manager,
            path_resolver,
            storage_mediator,
            config,
            parallel_processor,
        }
    }
    
    /// Crea directorios padres si es necesario
    async fn ensure_parent_directory(&self, abs_path: &PathBuf) -> FileRepositoryResult<()> {
        if let Some(parent) = abs_path.parent() {
            tokio::time::timeout(
                self.config.timeouts.dir_timeout(),
                tokio::fs::create_dir_all(parent)
            ).await
            .map_err(|_| crate::domain::repositories::file_repository::FileRepositoryError::Timeout(
                format!("Timeout creating parent directory: {}", parent.display())
            ))?
            .map_err(crate::domain::repositories::file_repository::FileRepositoryError::IoError)?;
        }
        Ok(())
    }
    
    /// Crea una entidad de archivo a partir de metadatos
    async fn create_file_entity(
        &self,
        id: String,
        name: String,
        storage_path: StoragePath,
        size: u64,
        mime_type: String,
        folder_id: Option<String>,
        created_at: Option<u64>,
        modified_at: Option<u64>,
    ) -> FileRepositoryResult<File> {
        // If timestamps are provided, use them; otherwise, let File::new create default timestamps
        if let (Some(created), Some(modified)) = (created_at, modified_at) {
            File::with_timestamps(
                id, 
                name, 
                storage_path, 
                size, 
                mime_type, 
                folder_id,
                created,
                modified,
            )
            .map_err(|e| crate::domain::repositories::file_repository::FileRepositoryError::Other(e.to_string()))
        } else {
            File::new(
                id, 
                name, 
                storage_path, 
                size, 
                mime_type, 
                folder_id,
            )
            .map_err(|e| crate::domain::repositories::file_repository::FileRepositoryError::Other(e.to_string()))
        }
    }
    
    /// Elimina un archivo de forma no bloqueante
    async fn delete_file_non_blocking(&self, _abs_path: PathBuf) -> FileRepositoryResult<()> {
        // Implementación real debe eliminar el archivo
        // Por ahora, devolvemos OK
        Ok(())
    }
}

#[async_trait]
impl FileWritePort for FileFsWriteRepository {
    async fn save_file(
        &self,
        name: String,
        folder_id: Option<String>,
        content_type: String,
        content: Vec<u8>,
    ) -> Result<File, DomainError> {
        // Implementación real debe guardar el archivo en disco
        // Por ahora, devolvemos un error
        Err(DomainError::internal_error("File save", "Save functionality not yet implemented"))
    }
    
    async fn move_file(&self, file_id: &str, target_folder_id: Option<String>) -> Result<File, DomainError> {
        // Implementación real debe mover el archivo a otra carpeta
        // Por ahora, devolvemos un error
        Err(DomainError::internal_error("File move", "Move functionality not yet implemented"))
    }
    
    async fn delete_file(&self, id: &str) -> Result<(), DomainError> {
        // Implementación real debe eliminar el archivo
        // Por ahora, devolvemos un error
        Err(DomainError::internal_error("File delete", "Delete functionality not yet implemented"))
    }
}