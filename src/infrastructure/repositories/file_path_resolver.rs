use std::path::PathBuf;
use std::sync::Arc;
use async_trait::async_trait;

use crate::domain::services::path_service::{PathService, StoragePath};
use crate::application::services::storage_mediator::StorageMediator;
use crate::infrastructure::services::id_mapping_service::{IdMappingService, IdMappingError};
use crate::domain::repositories::file_repository::FileRepositoryError;
use crate::common::errors::DomainError;
use crate::application::ports::storage_ports::FilePathResolutionPort;

/// Resuelve rutas de archivos y gestiona el mapeo de IDs a rutas
pub struct FilePathResolver {
    path_service: Arc<PathService>,
    storage_mediator: Arc<dyn StorageMediator>,
    id_mapping_service: Arc<IdMappingService>,
}

impl FilePathResolver {
    /// Crea un nuevo resolver de rutas
    pub fn new(
        path_service: Arc<PathService>,
        storage_mediator: Arc<dyn StorageMediator>,
        id_mapping_service: Arc<IdMappingService>,
    ) -> Self {
        Self {
            path_service,
            storage_mediator,
            id_mapping_service,
        }
    }
    
    /// Resuelve una ruta de dominio a una ruta física absoluta
    pub fn resolve_storage_path(&self, storage_path: &StoragePath) -> PathBuf {
        self.path_service.resolve_path(storage_path)
    }
    
    /// Resuelve una ruta PathBuf a una ruta física absoluta (legacy)
    pub fn resolve_legacy_path(&self, relative_path: &std::path::Path) -> PathBuf {
        self.storage_mediator.resolve_path(relative_path)
    }
    
    /// Obtiene la ruta de un archivo por su ID
    pub async fn get_path_by_id(&self, id: &str) -> Result<StoragePath, FileRepositoryError> {
        self.id_mapping_service.get_path_by_id(id).await
            .map_err(FileRepositoryError::from)
    }
    
    /// Actualiza la ruta para un ID existente
    pub async fn update_path(&self, id: &str, storage_path: &StoragePath) -> Result<(), FileRepositoryError> {
        self.id_mapping_service.update_path(id, storage_path).await
            .map_err(FileRepositoryError::from)
    }
    
    /// Obtiene o crea un ID para una ruta
    pub async fn get_or_create_id(&self, storage_path: &StoragePath) -> Result<String, FileRepositoryError> {
        self.id_mapping_service.get_or_create_id(storage_path).await
            .map_err(FileRepositoryError::from)
    }
    
    /// Elimina un ID del mapeo
    pub async fn remove_id(&self, id: &str) -> Result<(), FileRepositoryError> {
        self.id_mapping_service.remove_id(id).await
            .map_err(FileRepositoryError::from)
    }
    
    /// Guarda cambios pendientes
    pub async fn save_changes(&self) -> Result<(), FileRepositoryError> {
        self.id_mapping_service.save_pending_changes().await
            .map_err(FileRepositoryError::from)
    }
}

// Implementación de FilePathResolutionPort
#[async_trait]
impl FilePathResolutionPort for FilePathResolver {
    async fn get_file_path(&self, id: &str) -> Result<StoragePath, DomainError> {
        self.get_path_by_id(id).await
            .map_err(|e| match e {
                FileRepositoryError::NotFound(id) => DomainError::not_found("File", id),
                FileRepositoryError::IoError(e) => DomainError::internal_error("FilePath", e.to_string()),
                FileRepositoryError::Timeout(msg) => DomainError::internal_error("FilePath", msg),
                _ => DomainError::internal_error("FilePath", e.to_string()),
            })
    }
    
    fn resolve_path(&self, storage_path: &StoragePath) -> PathBuf {
        self.resolve_storage_path(storage_path)
    }
}