use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;
use tracing::{debug, error, info, instrument};

use crate::application::dtos::trash_dto::TrashedItemDto;
use crate::application::ports::trash_ports::TrashUseCase;
use crate::common::errors::{Result, DomainError, ErrorKind};
use crate::domain::entities::trashed_item::{TrashedItem, TrashedItemType};
use crate::domain::repositories::file_repository::FileRepository;
use crate::domain::repositories::folder_repository::FolderRepository;
use crate::domain::repositories::trash_repository::TrashRepository;

/**
 * Application service for trash operations.
 * 
 * The TrashService implements the trash management functionality in the application layer,
 * handling movement of files and folders to trash, restoration from trash, and permanent
 * deletion. It orchestrates interactions between the domain entities and infrastructure
 * repositories while enforcing business rules like retention policies.
 * 
 * This service follows the Clean Architecture pattern by:
 * - Depending on domain interfaces rather than concrete implementations
 * - Orchestrating domain operations without containing domain logic
 * - Exposing its functionality through the TrashUseCase port
 */
pub struct TrashService {
    /// Repository for trash-specific operations like listing and retrieving trashed items
    trash_repository: Arc<dyn TrashRepository>,
    
    /// Repository for file operations used when trashing, restoring, or deleting files
    file_repository: Arc<dyn FileRepository>,
    
    /// Repository for folder operations used when trashing, restoring, or deleting folders
    folder_repository: Arc<dyn FolderRepository>,
    
    /// Number of days items should be kept in trash before automatic cleanup
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
        debug!("User UUID validation: {}", user_id);
        
        // Validate user ownership
        debug!("Validando permisos de usuario");
        self.validate_user_ownership(item_id, user_id).await?;
        debug!("Permisos de usuario validados");
        
        // Parse UUIDs with detailed error handling
        debug!("Validando UUID del item: {}", item_id);
        let item_uuid = match Uuid::parse_str(item_id) {
            Ok(uuid) => {
                debug!("UUID del item válido: {}", uuid);
                uuid
            },
            Err(e) => {
                error!("UUID del item inválido: {} - Error: {}", item_id, e);
                return Err(DomainError::validation_error("Item", format!("Invalid item ID: {}", e)));
            }
        };
        
        debug!("Validando UUID del usuario: {}", user_id);
        let user_uuid = match Uuid::parse_str(user_id) {
            Ok(uuid) => {
                debug!("UUID del usuario válido: {}", uuid);
                uuid
            },
            Err(e) => {
                error!("UUID del usuario inválido: {} - Error: {}", user_id, e);
                return Err(DomainError::validation_error("User", format!("Invalid user ID: {}", e)));
            }
        };
        
        match item_type {
            "file" => {
                info!("Procesando archivo para mover a papelera: {}", item_id);
                
                // Obtener el archivo para verificar que existe y capturar sus datos
                debug!("Obteniendo datos del archivo: {}", item_id);
                let file = match self.file_repository.get_file_by_id(item_id).await {
                    Ok(file) => {
                        debug!("Archivo encontrado: {} ({})", file.name(), item_id);
                        file
                    },
                    Err(e) => {
                        error!("Error al obtener archivo: {} - {}", item_id, e);
                        return Err(DomainError::new(
                            ErrorKind::NotFound,
                            "File",
                            format!("Error retrieving file {}: {}", item_id, e)
                        ));
                    }
                };
                
                let original_path = file.storage_path().to_string();
                debug!("Ruta original del archivo: {}", original_path);
                
                // Crear el elemento de papelera
                debug!("Creando objeto TrashedItem para el archivo");
                let trashed_item = TrashedItem::new(
                    item_uuid,
                    user_uuid,
                    TrashedItemType::File,
                    file.name().to_string(),
                    original_path,
                    self.retention_days,
                );
                debug!("TrashedItem creado con éxito: {} -> {}", file.name(), trashed_item.id);
                
                // Primero añadimos a la papelera para registrar el elemento
                info!("Añadiendo archivo {} a índice de papelera", item_id);
                match self.trash_repository.add_to_trash(&trashed_item).await {
                    Ok(_) => {
                        debug!("Archivo añadido al índice de papelera con éxito");
                    },
                    Err(e) => {
                        error!("Error al añadir archivo al índice de papelera: {}", e);
                        return Err(DomainError::internal_error("TrashRepository", format!("Failed to add file to trash: {}", e)));
                    }
                };
                
                // Luego movemos el archivo físicamente a la papelera
                info!("Moviendo archivo físicamente a la papelera: {}", item_id);
                match self.file_repository.move_to_trash(item_id).await {
                    Ok(_) => {
                        debug!("Archivo movido físicamente a papelera con éxito: {}", item_id);
                    },
                    Err(e) => {
                        error!("Error al mover archivo físicamente a papelera: {} - {}", item_id, e);
                        return Err(DomainError::new(
                            ErrorKind::InternalError,
                            "File",
                            format!("Error moving file {} to trash: {}", item_id, e)
                        ));
                    }
                }
                
                info!("Archivo movido a papelera completamente: {}", item_id);
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
                debug!("Adding folder {} to trash repository", item_id);
                match self.trash_repository.add_to_trash(&trashed_item).await {
                    Ok(_) => debug!("Successfully added folder to trash repository"),
                    Err(e) => {
                        error!("Failed to add folder to trash repository: {}", e);
                        return Err(DomainError::internal_error("TrashRepository", format!("Failed to add folder to trash: {}", e)));
                    }
                };
                
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
        
        let trash_uuid = match Uuid::parse_str(trash_id) {
            Ok(id) => {
                info!("Trash UUID parsed successfully: {}", id);
                id
            },
            Err(e) => {
                error!("Invalid trash ID format: {} - {}", trash_id, e);
                return Err(DomainError::validation_error("Trash", format!("Invalid trash ID: {}", e)));
            }
        };
            
        let user_uuid = match Uuid::parse_str(user_id) {
            Ok(id) => {
                info!("User UUID parsed successfully: {}", id);
                id
            },
            Err(e) => {
                error!("Invalid user ID format: {} - {}", user_id, e);
                return Err(DomainError::validation_error("User", format!("Invalid user ID: {}", e)));
            }
        };
        
        // Obtener el elemento de la papelera
        info!("Retrieving trash item from repository: ID={}", trash_id);
        let item_result = self.trash_repository.get_trash_item(&trash_uuid, &user_uuid).await;
        
        match item_result {
            Ok(Some(item)) => {
                info!("Found item in trash: ID={}, Type={:?}, OriginalID={}", 
                    trash_id, item.item_type, item.original_id);
                
                // Restaurar según tipo
                match item.item_type {
                    TrashedItemType::File => {
                        // Restaurar el archivo a su ubicación original
                        let file_id = item.original_id.to_string();
                        let original_path = item.original_path.clone();
                        
                        info!("Restoring file from trash: ID={}, OriginalPath={}", file_id, original_path);
                        match self.file_repository.restore_from_trash(&file_id, &original_path).await {
                            Ok(_) => {
                                info!("Successfully restored file from trash: {}", file_id);
                            },
                            Err(e) => {
                                // Check if the error is because the file is not found
                                if format!("{}", e).contains("not found") {
                                    info!("File not found in trash, may already have been restored: {}", file_id);
                                    // We continue so we can clean up the trash entry
                                } else {
                                    // Return error for other kinds of errors
                                    error!("Error restoring file from trash: {} - {}", file_id, e);
                                    return Err(DomainError::new(
                                        ErrorKind::InternalError,
                                        "File",
                                        format!("Error restoring file {} from trash: {}", file_id, e)
                                    ));
                                }
                            }
                        }
                    },
                    TrashedItemType::Folder => {
                        // Restaurar la carpeta a su ubicación original
                        let folder_id = item.original_id.to_string();
                        let original_path = item.original_path.clone();
                        
                        info!("Restoring folder from trash: ID={}, OriginalPath={}", folder_id, original_path);
                        match self.folder_repository.restore_from_trash(&folder_id, &original_path).await {
                            Ok(_) => {
                                info!("Successfully restored folder from trash: {}", folder_id);
                            },
                            Err(e) => {
                                // Check if the error is because the folder is not found
                                if format!("{}", e).contains("not found") {
                                    info!("Folder not found in trash, may already have been restored: {}", folder_id);
                                    // We continue so we can clean up the trash entry
                                } else {
                                    // Return error for other kinds of errors
                                    error!("Error restoring folder from trash: {} - {}", folder_id, e);
                                    return Err(DomainError::new(
                                        ErrorKind::InternalError,
                                        "Folder",
                                        format!("Error restoring folder {} from trash: {}", folder_id, e)
                                    ));
                                }
                            }
                        }
                    }
                }
                
                // Always remove the item from the trash index to maintain consistency
                info!("Removing item from trash index after restoration: {}", trash_id);
                match self.trash_repository.restore_from_trash(&trash_uuid, &user_uuid).await {
                    Ok(_) => {
                        info!("Successfully removed entry from trash index: {}", trash_id);
                    },
                    Err(e) => {
                        error!("Error removing entry from trash index: {} - {}", trash_id, e);
                        return Err(DomainError::new(
                            ErrorKind::InternalError,
                            "Trash",
                            format!("Error removing trash entry after restoration: {}", e)
                        ));
                    }
                }
                
                info!("Item successfully restored from trash: {}", trash_id);
                Ok(())
            },
            Ok(None) => {
                // If the item isn't found in trash, we can just return success
                info!("Item not found in trash index, considering as already restored: {}", trash_id);
                Ok(())
            },
            Err(e) => {
                // Something went wrong with the repository
                error!("Error retrieving item from trash repository: {} - {}", trash_id, e);
                Err(e)
            }
        }
    }

    #[instrument(skip(self))]
    async fn delete_permanently(&self, trash_id: &str, user_id: &str) -> Result<()> {
        info!("Permanently deleting item {} for user {}", trash_id, user_id);
        
        let trash_uuid = match Uuid::parse_str(trash_id) {
            Ok(id) => {
                info!("Trash UUID parsed successfully: {}", id);
                id
            },
            Err(e) => {
                error!("Invalid trash ID format: {} - {}", trash_id, e);
                return Err(DomainError::validation_error("Trash", format!("Invalid trash ID: {}", e)));
            }
        };
            
        let user_uuid = match Uuid::parse_str(user_id) {
            Ok(id) => {
                info!("User UUID parsed successfully: {}", id);
                id
            },
            Err(e) => {
                error!("Invalid user ID format: {} - {}", user_id, e);
                return Err(DomainError::validation_error("User", format!("Invalid user ID: {}", e)));
            }
        };
        
        // Obtener el elemento de la papelera
        info!("Retrieving trash item from repository: ID={}", trash_id);
        let item_result = self.trash_repository.get_trash_item(&trash_uuid, &user_uuid).await;
        
        match item_result {
            Ok(Some(item)) => {
                info!("Found item in trash: ID={}, Type={:?}, OriginalID={}", 
                      trash_id, item.item_type, item.original_id);
                
                // Eliminar permanentemente según tipo
                match item.item_type {
                    TrashedItemType::File => {
                        // Eliminar el archivo permanentemente
                        let file_id = item.original_id.to_string();
                        
                        info!("Permanently deleting file: {}", file_id);
                        match self.file_repository.delete_file_permanently(&file_id).await {
                            Ok(_) => {
                                info!("Successfully deleted file permanently: {}", file_id);
                            },
                            Err(e) => {
                                // Check if the file is not found - in that case, we can continue
                                // because we still want to remove the item from the trash index
                                if format!("{}", e).contains("not found") {
                                    info!("File not found, may already have been deleted: {}", file_id);
                                } else {
                                    // Return error for other types of errors
                                    error!("Error permanently deleting file: {} - {}", file_id, e);
                                    return Err(DomainError::new(
                                        ErrorKind::InternalError,
                                        "File",
                                        format!("Error deleting file {} permanently: {}", file_id, e)
                                    ));
                                }
                            }
                        }
                    },
                    TrashedItemType::Folder => {
                        // Eliminar la carpeta permanentemente
                        let folder_id = item.original_id.to_string();
                        
                        info!("Permanently deleting folder: {}", folder_id);
                        match self.folder_repository.delete_folder_permanently(&folder_id).await {
                            Ok(_) => {
                                info!("Successfully deleted folder permanently: {}", folder_id);
                            },
                            Err(e) => {
                                // Check if the folder is not found - in that case, we can continue
                                if format!("{}", e).contains("not found") {
                                    info!("Folder not found, may already have been deleted: {}", folder_id);
                                } else {
                                    // Return error for other types of errors
                                    error!("Error permanently deleting folder: {} - {}", folder_id, e);
                                    return Err(DomainError::new(
                                        ErrorKind::InternalError,
                                        "Folder",
                                        format!("Error deleting folder {} permanently: {}", folder_id, e)
                                    ));
                                }
                            }
                        }
                    }
                }
                
                // Eliminar el item de la papelera siempre, para mantener consistencia
                info!("Removing entry from trash index: {}", trash_id);
                match self.trash_repository.delete_permanently(&trash_uuid, &user_uuid).await {
                    Ok(_) => {
                        info!("Successfully removed entry from trash index: {}", trash_id);
                    },
                    Err(e) => {
                        error!("Error removing entry from trash index: {} - {}", trash_id, e);
                        return Err(DomainError::new(
                            ErrorKind::InternalError,
                            "Trash",
                            format!("Error removing trash entry: {}", e)
                        ));
                    }
                };
                
                info!("Item permanently deleted from trash: {}", trash_id);
                Ok(())
            },
            Ok(None) => {
                // If the item isn't found in trash, we can just return success
                info!("Item not found in trash, considering as already deleted: {}", trash_id);
                Ok(())
            },
            Err(e) => {
                // Something went wrong with the repository
                error!("Error retrieving item from trash repository: {} - {}", trash_id, e);
                Err(e)
            }
        }
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