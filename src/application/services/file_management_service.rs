use std::sync::Arc;
use async_trait::async_trait;

use crate::application::dtos::file_dto::FileDto;
use crate::application::ports::file_ports::FileManagementUseCase;
use crate::application::ports::storage_ports::FileWritePort;
use crate::common::errors::DomainError;

/// Servicio para operaciones de gestión de archivos
pub struct FileManagementService {
    file_repository: Arc<dyn FileWritePort>,
}

impl FileManagementService {
    /// Crea un nuevo servicio de gestión de archivos
    pub fn new(file_repository: Arc<dyn FileWritePort>) -> Self {
        Self { file_repository }
    }
    
    /// Crea un stub para pruebas
    pub fn default_stub() -> Self {
        Self {
            file_repository: Arc::new(crate::infrastructure::repositories::FileFsWriteRepository::default_stub())
        }
    }
}

#[async_trait]
impl FileManagementUseCase for FileManagementService {
    async fn move_file(&self, file_id: &str, folder_id: Option<String>) -> Result<FileDto, DomainError> {
        tracing::info!("Moviendo archivo con ID: {} a carpeta: {:?}", file_id, folder_id);
        
        let moved_file = self.file_repository.move_file(file_id, folder_id).await
            .map_err(|e| {
                tracing::error!("Error al mover archivo (ID: {}): {}", file_id, e);
                e
            })?;
        
        tracing::info!("Archivo movido exitosamente: {} (ID: {}) a carpeta: {:?}", 
                       moved_file.name(), moved_file.id(), moved_file.folder_id());
        
        Ok(FileDto::from(moved_file))
    }
    
    async fn delete_file(&self, id: &str) -> Result<(), DomainError> {
        self.file_repository.delete_file(id).await
    }
}