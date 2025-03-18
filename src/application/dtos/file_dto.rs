use serde::Serialize;
use crate::domain::entities::file::File;

/// DTO for file responses
#[derive(Debug, Serialize)]
pub struct FileDto {
    /// File ID
    pub id: String,
    
    /// File name
    pub name: String,
    
    /// Path to the file (relative)
    pub path: String,
    
    /// Size in bytes
    pub size: u64,
    
    /// MIME type
    pub mime_type: String,
    
    /// Parent folder ID
    pub folder_id: Option<String>,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Last modification timestamp
    pub modified_at: u64,
}

impl From<File> for FileDto {
    fn from(file: File) -> Self {
        Self {
            id: file.id().to_string(),
            name: file.name().to_string(),
            path: file.path_string().to_string(),
            size: file.size(),
            mime_type: file.mime_type().to_string(),
            folder_id: file.folder_id().map(String::from),
            created_at: file.created_at(),
            modified_at: file.modified_at(),
        }
    }
}

// Para convertir de FileDto a File para los batch handlers
impl From<FileDto> for File {
    fn from(dto: FileDto) -> Self {
        // Usar constructor para crear una entidad desde DTO
        // Nota: esto debe simplificarse si File tiene un constructor adecuado
        // Si no, deberías hacer la conversión de la mejor manera posible
        File::from_dto(
            dto.id, 
            dto.name, 
            dto.path,
            dto.size,
            dto.mime_type,
            dto.folder_id,
            dto.created_at,
            dto.modified_at
        )
    }
}