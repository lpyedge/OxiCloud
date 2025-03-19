use std::sync::Arc;
use async_trait::async_trait;

use crate::application::dtos::file_dto::FileDto;
use crate::application::ports::file_ports::FileUploadUseCase;
use crate::application::ports::storage_ports::FileWritePort;
use crate::common::errors::DomainError;

/// Servicio para operaciones de subida de archivos
pub struct FileUploadService {
    file_repository: Arc<dyn FileWritePort>,
}

impl FileUploadService {
    /// Crea un nuevo servicio de subida de archivos
    pub fn new(file_repository: Arc<dyn FileWritePort>) -> Self {
        Self { file_repository }
    }
}

#[async_trait]
impl FileUploadUseCase for FileUploadService {
    async fn upload_file(
        &self,
        name: String,
        folder_id: Option<String>,
        content_type: String,
        content: Vec<u8>,
    ) -> Result<FileDto, DomainError> {
        let file = self.file_repository.save_file(name, folder_id, content_type, content).await?;
        Ok(FileDto::from(file))
    }
}