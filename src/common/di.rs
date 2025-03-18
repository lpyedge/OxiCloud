use std::path::PathBuf;
use std::sync::Arc;
use async_trait::async_trait;

use crate::domain::services::path_service::PathService;
use crate::infrastructure::repositories::folder_fs_repository::FolderFsRepository;
use crate::infrastructure::repositories::file_fs_repository::FileFsRepository;
use crate::infrastructure::services::file_system_i18n_service::FileSystemI18nService;
use crate::infrastructure::services::id_mapping_service::IdMappingService;
use crate::infrastructure::services::cache_manager::StorageCacheManager;
use crate::application::services::folder_service::FolderService;
use crate::application::services::file_service::FileService;
use crate::application::services::i18n_application_service::I18nApplicationService;
use crate::application::services::storage_mediator::{StorageMediator, FileSystemStorageMediator};
use crate::application::ports::inbound::{FileUseCase, FolderUseCase, UseCaseFactory};
use crate::application::ports::outbound::{FileStoragePort, FolderStoragePort};
use crate::common::errors::DomainError;
use crate::domain::services::i18n_service::I18nService;

/// Fábrica para los diferentes componentes de la aplicación
#[allow(dead_code)]
pub struct AppServiceFactory {
    storage_path: PathBuf,
    locales_path: PathBuf,
}

impl AppServiceFactory {
    /// Crea una nueva fábrica de servicios
    #[allow(dead_code)]
    pub fn new(storage_path: PathBuf, locales_path: PathBuf) -> Self {
        Self {
            storage_path,
            locales_path,
        }
    }
    
    /// Inicializa los servicios base del sistema
    #[allow(dead_code)]
    pub async fn create_core_services(&self) -> Result<CoreServices, DomainError> {
        // Path service
        let path_service = Arc::new(PathService::new(self.storage_path.clone()));
        
        // Cache manager
        // TTL values in milliseconds and max entries for cache
        let file_ttl_ms = 60_000; // 1 minute for files
        let dir_ttl_ms = 120_000; // 2 minutes for directories
        let max_entries = 10_000; // Maximum cache entries
        let cache_manager = Arc::new(StorageCacheManager::new(file_ttl_ms, dir_ttl_ms, max_entries));
        
        // Iniciar tarea de limpieza de caché en segundo plano
        let cache_manager_clone = cache_manager.clone();
        tokio::spawn(async move {
            StorageCacheManager::start_cleanup_task(cache_manager_clone).await;
        });
        
        // ID mapping service
        let id_mapping_path = self.storage_path.join("folder_ids.json");
        let id_mapping_service = Arc::new(
            IdMappingService::new(id_mapping_path).await?
        );
        
        Ok(CoreServices {
            path_service,
            cache_manager,
            id_mapping_service,
        })
    }
    
    /// Inicializa los servicios de repositorio
    #[allow(dead_code)]
    pub fn create_repository_services(&self, core: &CoreServices) -> RepositoryServices {
        // Storage mediator - create first because it's needed by folder repository
        // (temporarily using a placeholder for folder repository, will update later)
        let placeholder_folder_repo = Arc::new(FolderFsRepository::new_stub());
        
        let storage_mediator = Arc::new(FileSystemStorageMediator::new(
            placeholder_folder_repo.clone(),
            core.path_service.clone(),
            core.id_mapping_service.clone()
        ));
        
        // Folder repository
        let folder_repository = Arc::new(FolderFsRepository::new(
            self.storage_path.clone(),
            storage_mediator.clone(),
            core.id_mapping_service.clone(),
            core.path_service.clone(),
        ));
        
        // Create a file metadata cache with default configuration
        let metadata_cache = Arc::new(
            crate::infrastructure::services::file_metadata_cache::FileMetadataCache::default_with_config(
                crate::common::config::AppConfig::default()
            )
        );
        
        // File repository
        let file_repository = Arc::new(FileFsRepository::new(
            self.storage_path.clone(), 
            storage_mediator.clone(),
            core.id_mapping_service.clone(),
            core.path_service.clone(),
            metadata_cache,
        ));
        
        // I18n repository
        let i18n_repository = Arc::new(FileSystemI18nService::new(
            self.locales_path.clone()
        ));
        
        RepositoryServices {
            folder_repository,
            file_repository,
            i18n_repository,
            storage_mediator,
        }
    }
    
    /// Inicializa los servicios de aplicación
    #[allow(dead_code)]
    pub fn create_application_services(&self, repos: &RepositoryServices) -> ApplicationServices {
        // Servicios principales
        let folder_service = Arc::new(FolderService::new(
            repos.folder_repository.clone()
        ));
        
        let file_service = Arc::new(FileService::new(
            repos.file_repository.clone()
        ));
        
        let i18n_service = Arc::new(I18nApplicationService::new(
            repos.i18n_repository.clone()
        ));
        
        ApplicationServices {
            folder_service,
            file_service,
            i18n_service,
        }
    }
}

/// Contenedor para servicios base
#[allow(dead_code)]
pub struct CoreServices {
    pub path_service: Arc<PathService>,
    pub cache_manager: Arc<StorageCacheManager>,
    pub id_mapping_service: Arc<IdMappingService>,
}

/// Contenedor para servicios de repositorio
#[allow(dead_code)]
pub struct RepositoryServices {
    pub folder_repository: Arc<dyn FolderStoragePort>,
    pub file_repository: Arc<dyn FileStoragePort>,
    pub i18n_repository: Arc<dyn I18nService>,
    pub storage_mediator: Arc<dyn StorageMediator>,
}

/// Contenedor para servicios de aplicación
#[allow(dead_code)]
pub struct ApplicationServices {
    pub folder_service: Arc<dyn FolderUseCase>,
    pub file_service: Arc<dyn FileUseCase>,
    pub i18n_service: Arc<I18nApplicationService>,
}

/// Fábrica de casos de uso para la inyección de dependencias
#[allow(dead_code)]
pub struct AppUseCaseFactory {
    services: ApplicationServices,
}

impl AppUseCaseFactory {
    #[allow(dead_code)]
    pub fn new(services: ApplicationServices) -> Self {
        Self { services }
    }
}

#[async_trait]
impl UseCaseFactory for AppUseCaseFactory {
    fn create_file_use_case(&self) -> Arc<dyn FileUseCase> {
        self.services.file_service.clone()
    }
    
    fn create_folder_use_case(&self) -> Arc<dyn FolderUseCase> {
        self.services.folder_service.clone()
    }
}