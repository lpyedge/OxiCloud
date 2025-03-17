use std::path::PathBuf;
use std::sync::Arc;

use crate::domain::repositories::folder_repository::{FolderRepository, FolderRepositoryResult};
use crate::application::dtos::folder_dto::{CreateFolderDto, RenameFolderDto, MoveFolderDto, FolderDto};

/// Service for folder operations
pub struct FolderService {
    folder_repository: Arc<dyn FolderRepository>,
}

impl FolderService {
    /// Creates a new folder service
    pub fn new(folder_repository: Arc<dyn FolderRepository>) -> Self {
        Self { folder_repository }
    }
    
    /// Creates a new folder
    pub async fn create_folder(&self, dto: CreateFolderDto) -> FolderRepositoryResult<FolderDto> {
        let parent_path = match &dto.parent_id {
            Some(parent_id) => {
                let parent = self.folder_repository.get_folder_by_id(parent_id).await?;
                Some(parent.path)
            },
            None => None
        };
        
        let folder = self.folder_repository.create_folder(dto.name, parent_path).await?;
        Ok(FolderDto::from(folder))
    }
    
    /// Gets a folder by ID
    pub async fn get_folder(&self, id: &str) -> FolderRepositoryResult<FolderDto> {
        let folder = self.folder_repository.get_folder_by_id(id).await?;
        Ok(FolderDto::from(folder))
    }
    
    /// Gets a folder by path
    #[allow(dead_code)]
    pub async fn get_folder_by_path(&self, path: &str) -> FolderRepositoryResult<FolderDto> {
        let path_buf = PathBuf::from(path);
        let folder = self.folder_repository.get_folder_by_path(&path_buf).await?;
        Ok(FolderDto::from(folder))
    }
    
    /// Lists folders in a parent folder
    pub async fn list_folders(&self, parent_id: Option<&str>) -> FolderRepositoryResult<Vec<FolderDto>> {
        let folders = self.folder_repository.list_folders(parent_id).await?;
        Ok(folders.into_iter().map(FolderDto::from).collect())
    }
    
    /// Renames a folder
    pub async fn rename_folder(&self, id: &str, dto: RenameFolderDto) -> FolderRepositoryResult<FolderDto> {
        let folder = self.folder_repository.rename_folder(id, dto.name).await?;
        Ok(FolderDto::from(folder))
    }
    
    /// Moves a folder to a new parent
    pub async fn move_folder(&self, id: &str, dto: MoveFolderDto) -> FolderRepositoryResult<FolderDto> {
        let folder = self.folder_repository.move_folder(id, dto.parent_id.as_deref()).await?;
        Ok(FolderDto::from(folder))
    }
    
    /// Deletes a folder
    pub async fn delete_folder(&self, id: &str) -> FolderRepositoryResult<()> {
        self.folder_repository.delete_folder(id).await
    }
}