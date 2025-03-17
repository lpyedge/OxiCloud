use std::sync::Arc;

use crate::domain::repositories::file_repository::{FileRepository, FileRepositoryResult};
use crate::application::dtos::file_dto::FileDto;

/// Service for file operations
pub struct FileService {
    file_repository: Arc<dyn FileRepository>,
}

impl FileService {
    /// Creates a new file service
    pub fn new(file_repository: Arc<dyn FileRepository>) -> Self {
        Self { file_repository }
    }
    
    /// Uploads a new file from bytes
    pub async fn upload_file_from_bytes(
        &self,
        name: String,
        folder_id: Option<String>,
        content_type: String,
        content: Vec<u8>,
    ) -> FileRepositoryResult<FileDto>
    {
        let file = self.file_repository.save_file_from_bytes(name, folder_id, content_type, content).await?;
        Ok(FileDto::from(file))
    }
    
    /// Gets a file by ID
    pub async fn get_file(&self, id: &str) -> FileRepositoryResult<FileDto> {
        let file = self.file_repository.get_file_by_id(id).await?;
        Ok(FileDto::from(file))
    }
    
    /// Lists files in a folder
    pub async fn list_files(&self, folder_id: Option<&str>) -> FileRepositoryResult<Vec<FileDto>> {
        let files = self.file_repository.list_files(folder_id).await?;
        Ok(files.into_iter().map(FileDto::from).collect())
    }
    
    /// Deletes a file
    pub async fn delete_file(&self, id: &str) -> FileRepositoryResult<()> {
        self.file_repository.delete_file(id).await
    }
    
    /// Gets file content
    pub async fn get_file_content(&self, id: &str) -> FileRepositoryResult<Vec<u8>> {
        self.file_repository.get_file_content(id).await
    }
    
    /// Moves a file to a new folder implementing direct save with new location without deleting first
    pub async fn move_file(&self, file_id: &str, folder_id: Option<String>) -> FileRepositoryResult<FileDto> {
        // Get the current file complete info
        let source_file = match self.file_repository.get_file_by_id(file_id).await {
            Ok(f) => f,
            Err(e) => {
                tracing::error!("Error al obtener archivo (ID: {}): {}", file_id, e);
                return Err(e);
            }
        };
        
        tracing::info!("Moviendo archivo: {} (ID: {}) de carpeta: {:?} a carpeta: {:?}", 
                      source_file.name, file_id, source_file.folder_id, folder_id);
        
        // Special handling for PDF files
        let is_pdf = source_file.name.to_lowercase().ends_with(".pdf");
        if is_pdf {
            tracing::info!("Moviendo un archivo PDF: {}", source_file.name);
        }
        
        // No hacer nada si ya estamos en la carpeta de destino
        if source_file.folder_id == folder_id {
            tracing::info!("El archivo ya está en la carpeta de destino, no es necesario moverlo");
            return Ok(FileDto::from(source_file));
        }
        
        // Step 1: Get file content
        tracing::info!("Leyendo contenido del archivo: {}", source_file.name);
        let content = match self.file_repository.get_file_content(file_id).await {
            Ok(content) => {
                tracing::info!("Contenido del archivo leído correctamente: {} bytes", content.len());
                content
            },
            Err(e) => {
                tracing::error!("Error al leer el contenido del archivo {}: {}", file_id, e);
                return Err(e);
            }
        };
        
        // Step 2: Save the file to the new location with a new ID
        tracing::info!("Guardando archivo en nueva ubicación: {} en carpeta: {:?}", source_file.name, folder_id);
        let new_file = match self.file_repository.save_file_from_bytes(
            source_file.name.clone(),
            folder_id.clone(),
            source_file.mime_type.clone(),
            content
        ).await {
            Ok(file) => {
                tracing::info!("Archivo guardado en nueva ubicación con ID: {}", file.id);
                file
            },
            Err(e) => {
                tracing::error!("Error al guardar archivo en nueva ubicación: {}", e);
                return Err(e);
            }
        };
        
        // Step 3: Only after ensuring new file is saved, try to delete the old file
        // If this fails, it's not critical - we already have the file in the new location
        tracing::info!("Eliminando archivo original con ID: {}", file_id);
        match self.file_repository.delete_file(file_id).await {
            Ok(_) => tracing::info!("Archivo original eliminado correctamente"),
            Err(e) => {
                tracing::warn!("Error al eliminar archivo original (ID: {}): {} - archivo duplicado posible", file_id, e);
                // Continue even if delete fails - at worst we'll have duplicate files
            }
        }
        
        tracing::info!("Archivo movido exitosamente: {} (ID: {}) a carpeta: {:?}", 
                       new_file.name, new_file.id, folder_id);
        
        Ok(FileDto::from(new_file))
    }
}