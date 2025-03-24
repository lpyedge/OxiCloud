use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;
use tracing::{debug, error, info, instrument};

use crate::application::dtos::trash_dto::TrashedItemDto;
use crate::application::ports::trash_ports::TrashUseCase;
use crate::common::errors::{Result, DomainError, ErrorKind};
use crate::domain::entities::trashed_item::{TrashedItem, TrashedItemType};
use crate::domain::repositories::file_repository::{FileRepository, FileRepositoryResult};
use crate::domain::repositories::folder_repository::{FolderRepository, FolderRepositoryResult};
use crate::domain::repositories::trash_repository::TrashRepository;

/// Servicio de aplicación para operaciones de papelera
pub struct TrashService {
    trash_repository: Arc<dyn TrashRepository>,
    file_repository: Arc<dyn FileRepository>,
    folder_repository: Arc<dyn FolderRepository>,
    retention_days: u32,
}

impl TrashService {
    pub fn new(
        trash_repository: Arc<dyn TrashRepository>,
        file_repository: Arc<dyn FileRepository>,
        folder_repository: Arc<dyn FolderRepository>,
        retention_days: u32,
    ) -> Self {
        Self {
            trash_repository,
            file_repository,
            folder_repository,
            retention_days,
        }
    }

    /// Convierte una entidad TrashedItem a un DTO
    fn to_dto(&self, item: TrashedItem) -> TrashedItemDto {
        // Calcular days_until_deletion antes de mover item.original_path
        let days_until_deletion = item.days_until_deletion();
        
        TrashedItemDto {
            id: item.id.to_string(),
            original_id: item.original_id.to_string(),
            item_type: match item.item_type {
                TrashedItemType::File => "file".to_string(),
                TrashedItemType::Folder => "folder".to_string(),
            },
            name: item.name,
            original_path: item.original_path,
            trashed_at: item.trashed_at,
            days_until_deletion,
        }
    }

    /// Valida los permisos del usuario sobre un elemento
    #[instrument(skip(self))]
    async fn validate_user_ownership(&self, _item_id: &str, _user_id: &str) -> Result<()> {
        // Aquí implementaríamos la validación de permisos
        // Por ahora, simplemente devolvemos Ok ya que no tenemos una implementación completa
        // de permisos por usuario
        Ok(())
    }
}

#[async_trait]
impl TrashUseCase for TrashService {
    #[instrument(skip(self))]
    async fn get_trash_items(&self, user_id: &str) -> Result<Vec<TrashedItemDto>> {
        debug!("Obteniendo elementos en papelera para usuario: {}", user_id);
        
        let user_uuid = Uuid::parse_str(user_id)
            .map_err(|e| DomainError::validation_error("User", format!("Invalid user ID: {}", e)))?;
            
        let items = self.trash_repository.get_trash_items(&user_uuid).await?;
        
        let dtos = items.into_iter()
            .map(|item| self.to_dto(item))
            .collect();
            
        Ok(dtos)
    }

    #[instrument(skip(self))]
    async fn move_to_trash(&self, item_id: &str, item_type: &str, user_id: &str) -> Result<()> {
        info!("Moviendo a papelera: tipo={}, id={}, usuario={}", item_type, item_id, user_id);
        
        self.validate_user_ownership(item_id, user_id).await?;
        
        let item_uuid = Uuid::parse_str(item_id)
            .map_err(|e| DomainError::validation_error("Item", format!("Invalid item ID: {}", e)))?;
            
        let user_uuid = Uuid::parse_str(user_id)
            .map_err(|e| DomainError::validation_error("User", format!("Invalid user ID: {}", e)))?;
        
        match item_type {
            "file" => {
                // Obtener el archivo para verificar que existe y capturar sus datos
                let file = self.file_repository.get_file_by_id(item_id).await
                    .map_err(|e| DomainError::new(
                        ErrorKind::NotFound,
                        "File",
                        format!("Error retrieving file {}: {}", item_id, e)
                    ))?;
                
                let original_path = file.storage_path().to_string();
                
                // Crear el elemento de papelera
                let trashed_item = TrashedItem::new(
                    item_uuid,
                    user_uuid,
                    TrashedItemType::File,
                    file.name().to_string(),
                    original_path,
                    self.retention_days,
                );
                
                // Primero añadimos a la papelera para registrar el elemento
                self.trash_repository.add_to_trash(&trashed_item).await?;
                
                // Luego movemos el archivo físicamente a la papelera
                self.file_repository.move_to_trash(item_id).await
                    .map_err(|e| DomainError::new(
                        ErrorKind::InternalError,
                        "File",
                        format!("Error moving file {} to trash: {}", item_id, e)
                    ))?;
                
                debug!("Archivo movido a papelera: {}", item_id);
                Ok(())
            },
            "folder" => {
                // Obtener la carpeta para verificar que existe y capturar sus datos
                let folder = self.folder_repository.get_folder_by_id(item_id).await
                    .map_err(|e| DomainError::new(
                        ErrorKind::NotFound,
                        "Folder",
                        format!("Error retrieving folder {}: {}", item_id, e)
                    ))?;
                
                let original_path = folder.storage_path().to_string();
                
                // Crear el elemento de papelera
                let trashed_item = TrashedItem::new(
                    item_uuid,
                    user_uuid,
                    TrashedItemType::Folder,
                    folder.name().to_string(),
                    original_path,
                    self.retention_days,
                );
                
                // Primero añadimos a la papelera para registrar el elemento
                self.trash_repository.add_to_trash(&trashed_item).await?;
                
                // Luego movemos la carpeta físicamente a la papelera
                self.folder_repository.move_to_trash(item_id).await
                    .map_err(|e| DomainError::new(
                        ErrorKind::InternalError,
                        "Folder",
                        format!("Error moving folder {} to trash: {}", item_id, e)
                    ))?;
                
                debug!("Carpeta movida a papelera: {}", item_id);
                Ok(())
            },
            _ => Err(DomainError::validation_error("Item", format!("Invalid item type: {}", item_type))),
        }
    }

    #[instrument(skip(self))]
    async fn restore_item(&self, trash_id: &str, user_id: &str) -> Result<()> {
        info!("Restaurando elemento {} para usuario {}", trash_id, user_id);
        
        let trash_uuid = Uuid::parse_str(trash_id)
            .map_err(|e| DomainError::validation_error("Trash", format!("Invalid trash ID: {}", e)))?;
            
        let user_uuid = Uuid::parse_str(user_id)
            .map_err(|e| DomainError::validation_error("User", format!("Invalid user ID: {}", e)))?;
        
        // Obtener el elemento de la papelera
        let item = self.trash_repository.get_trash_item(&trash_uuid, &user_uuid).await?
            .ok_or_else(|| DomainError::not_found("TrashedItem", trash_id.to_string()))?;
        
        // Restaurar según tipo
        match item.item_type {
            TrashedItemType::File => {
                // Restaurar el archivo a su ubicación original
                let file_id = item.original_id.to_string();
                self.file_repository.restore_from_trash(&file_id, &item.original_path).await
                    .map_err(|e| DomainError::new(
                        ErrorKind::InternalError,
                        "File",
                        format!("Error restoring file {} from trash: {}", file_id, e)
                    ))?;
                debug!("Archivo restaurado desde papelera: {}", file_id);
            },
            TrashedItemType::Folder => {
                // Restaurar la carpeta a su ubicación original
                let folder_id = item.original_id.to_string();
                self.folder_repository.restore_from_trash(&folder_id, &item.original_path).await
                    .map_err(|e| DomainError::new(
                        ErrorKind::InternalError,
                        "Folder",
                        format!("Error restoring folder {} from trash: {}", folder_id, e)
                    ))?;
                debug!("Carpeta restaurada desde papelera: {}", folder_id);
            }
        }
        
        // Eliminar el item de la papelera
        self.trash_repository.restore_from_trash(&trash_uuid, &user_uuid).await?;
        
        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete_permanently(&self, trash_id: &str, user_id: &str) -> Result<()> {
        info!("Eliminando permanentemente elemento {} para usuario {}", trash_id, user_id);
        
        let trash_uuid = Uuid::parse_str(trash_id)
            .map_err(|e| DomainError::validation_error("Trash", format!("Invalid trash ID: {}", e)))?;
            
        let user_uuid = Uuid::parse_str(user_id)
            .map_err(|e| DomainError::validation_error("User", format!("Invalid user ID: {}", e)))?;
        
        // Obtener el elemento de la papelera
        let item = self.trash_repository.get_trash_item(&trash_uuid, &user_uuid).await?
            .ok_or_else(|| DomainError::not_found("TrashedItem", trash_id.to_string()))?;
        
        // Eliminar permanentemente según tipo
        match item.item_type {
            TrashedItemType::File => {
                // Eliminar el archivo permanentemente
                let file_id = item.original_id.to_string();
                self.file_repository.delete_file_permanently(&file_id).await
                    .map_err(|e| DomainError::new(
                        ErrorKind::InternalError,
                        "File",
                        format!("Error deleting file {} permanently: {}", file_id, e)
                    ))?;
                debug!("Archivo eliminado permanentemente: {}", file_id);
            },
            TrashedItemType::Folder => {
                // Eliminar la carpeta permanentemente
                let folder_id = item.original_id.to_string();
                self.folder_repository.delete_folder_permanently(&folder_id).await
                    .map_err(|e| DomainError::new(
                        ErrorKind::InternalError,
                        "Folder",
                        format!("Error deleting folder {} permanently: {}", folder_id, e)
                    ))?;
                debug!("Carpeta eliminada permanentemente: {}", folder_id);
            }
        }
        
        // Eliminar el item de la papelera
        self.trash_repository.delete_permanently(&trash_uuid, &user_uuid).await?;
        
        Ok(())
    }

    #[instrument(skip(self))]
    async fn empty_trash(&self, user_id: &str) -> Result<()> {
        info!("Vaciando papelera para usuario {}", user_id);
        
        let user_uuid = Uuid::parse_str(user_id)
            .map_err(|e| DomainError::validation_error("User", format!("Invalid user ID: {}", e)))?;
        
        // Obtener todos los elementos en la papelera del usuario
        let items = self.trash_repository.get_trash_items(&user_uuid).await?;
        
        // Eliminar permanentemente cada elemento
        for item in items {
            match item.item_type {
                TrashedItemType::File => {
                    // Eliminar el archivo permanentemente
                    let file_id = item.original_id.to_string();
                    if let Err(e) = self.file_repository.delete_file_permanently(&file_id).await {
                        error!("Error al eliminar archivo {} permanentemente: {}", file_id, e);
                    }
                },
                TrashedItemType::Folder => {
                    // Eliminar la carpeta permanentemente
                    let folder_id = item.original_id.to_string();
                    if let Err(e) = self.folder_repository.delete_folder_permanently(&folder_id).await {
                        error!("Error al eliminar carpeta {} permanentemente: {}", folder_id, e);
                    }
                }
            }
        }
        
        // Limpiar todos los registros de la papelera para este usuario
        self.trash_repository.clear_trash(&user_uuid).await?;
        
        info!("Papelera vaciada completamente para usuario {}", user_id);
        Ok(())
    }
}