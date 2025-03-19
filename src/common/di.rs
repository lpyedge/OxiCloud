use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

use crate::domain::services::path_service::PathService;
use crate::infrastructure::repositories::folder_fs_repository::FolderFsRepository;
use crate::infrastructure::repositories::file_fs_repository::FileFsRepository;
use crate::infrastructure::services::file_system_i18n_service::FileSystemI18nService;
use crate::infrastructure::services::id_mapping_service::IdMappingService;
use crate::infrastructure::services::cache_manager::StorageCacheManager;
use crate::infrastructure::services::file_metadata_cache::FileMetadataCache;
use crate::application::services::folder_service::FolderService;
use crate::application::services::file_service::FileService;
use crate::application::services::i18n_application_service::I18nApplicationService;
use crate::application::services::storage_mediator::{StorageMediator, FileSystemStorageMediator};
use crate::application::ports::inbound::{FileUseCase, FolderUseCase, UseCaseFactory};
use crate::application::ports::outbound::{FileStoragePort, FolderStoragePort};
use crate::application::ports::file_ports::{FileUploadUseCase, FileRetrievalUseCase, FileManagementUseCase, FileUseCaseFactory};
use crate::application::ports::storage_ports::{FileReadPort, FileWritePort, FilePathResolutionPort};
use crate::infrastructure::repositories::{FileMetadataManager, FilePathResolver, FileFsReadRepository, FileFsWriteRepository};
use crate::application::services::{FileUploadService, FileRetrievalService, FileManagementService, AppFileUseCaseFactory};
use crate::common::errors::DomainError;
use crate::domain::services::i18n_service::I18nService;
use crate::common::config::AppConfig;
use crate::domain::repositories::folder_repository::FolderRepository;

/// Fábrica para los diferentes componentes de la aplicación
#[allow(dead_code)]
pub struct AppServiceFactory {
    storage_path: PathBuf,
    locales_path: PathBuf,
    config: AppConfig,
}

impl AppServiceFactory {
    /// Crea una nueva fábrica de servicios
    #[allow(dead_code)]
    pub fn new(storage_path: PathBuf, locales_path: PathBuf) -> Self {
        Self {
            storage_path,
            locales_path,
            config: AppConfig::default(),
        }
    }
    
    /// Crea una nueva fábrica de servicios con configuración personalizada
    #[allow(dead_code)]
    pub fn with_config(storage_path: PathBuf, locales_path: PathBuf, config: AppConfig) -> Self {
        Self {
            storage_path,
            locales_path,
            config,
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
            config: self.config.clone(),
        })
    }
    
    /// Inicializa los servicios de repositorio utilizando el patrón Builder mejorado
    #[allow(dead_code)]
    pub fn create_repository_services(&self, core: &CoreServices) -> RepositoryServices {
        // Storage mediator - con inicialización diferida para folder repository
        let folder_repository_holder = Arc::new(RwLock::new(None));
        
        let storage_mediator = Arc::new(FileSystemStorageMediator::new_with_lazy_folder(
            folder_repository_holder.clone(),
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
        
        // Actualizar el holder para el mediador una vez que el repository está creado
        if let Ok(mut holder) = folder_repository_holder.write() {
            *holder = Some(folder_repository.clone());
        }
        
        // Metadata cache
        let metadata_cache = Arc::new(
            FileMetadataCache::default_with_config(core.config.clone())
        );
        
        // Componentes refactorizados
        let metadata_manager = Arc::new(FileMetadataManager::new(
            metadata_cache.clone(),
            core.config.clone()
        ));
        
        let path_resolver = Arc::new(FilePathResolver::new(
            core.path_service.clone(),
            storage_mediator.clone(),
            core.id_mapping_service.clone()
        ));
        
        // File repositories separados para lectura y escritura
        let file_read_repository = Arc::new(FileFsReadRepository::new(
            self.storage_path.clone(),
            metadata_manager.clone(),
            path_resolver.clone(),
            core.config.clone(),
            None // processor will be added later if needed
        ));
        
        let file_write_repository = Arc::new(FileFsWriteRepository::new(
            self.storage_path.clone(),
            metadata_manager.clone(),
            path_resolver.clone(),
            storage_mediator.clone(),
            core.config.clone(),
            None // processor will be added later if needed
        ));
        
        // Legacy file repository - mantenido por compatibilidad
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
            file_read_repository,
            file_write_repository,
            i18n_repository,
            storage_mediator,
            metadata_manager,
            path_resolver,
        }
    }
    
    /// Inicializa los servicios de aplicación
    #[allow(dead_code)]
    pub fn create_application_services(&self, repos: &RepositoryServices) -> ApplicationServices {
        // Servicios principales
        let folder_service = Arc::new(FolderService::new(
            repos.folder_repository.clone()
        ));
        
        // Antiguo servicio único
        let file_service = Arc::new(FileService::new(
            repos.file_repository.clone()
        ));
        
        // Nuevos servicios refactorizados
        let file_upload_service = Arc::new(FileUploadService::new(
            repos.file_write_repository.clone()
        ));
        
        let file_retrieval_service = Arc::new(FileRetrievalService::new(
            repos.file_read_repository.clone()
        ));
        
        let file_management_service = Arc::new(FileManagementService::new(
            repos.file_write_repository.clone()
        ));
        
        let file_use_case_factory = Arc::new(AppFileUseCaseFactory::new(
            repos.file_read_repository.clone(),
            repos.file_write_repository.clone()
        ));
        
        let i18n_service = Arc::new(I18nApplicationService::new(
            repos.i18n_repository.clone()
        ));
        
        ApplicationServices {
            folder_service,
            file_service,
            file_upload_service,
            file_retrieval_service,
            file_management_service,
            file_use_case_factory,
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
    pub config: AppConfig,
}

/// Contenedor para servicios de repositorio
#[allow(dead_code)]
pub struct RepositoryServices {
    pub folder_repository: Arc<dyn FolderStoragePort>,
    pub file_repository: Arc<dyn FileStoragePort>,
    pub file_read_repository: Arc<dyn FileReadPort>,
    pub file_write_repository: Arc<dyn FileWritePort>,
    pub i18n_repository: Arc<dyn I18nService>,
    pub storage_mediator: Arc<dyn StorageMediator>,
    pub metadata_manager: Arc<FileMetadataManager>,
    pub path_resolver: Arc<FilePathResolver>,
}

/// Contenedor para servicios de aplicación
#[allow(dead_code)]
pub struct ApplicationServices {
    pub folder_service: Arc<dyn FolderUseCase>,
    pub file_service: Arc<dyn FileUseCase>,
    pub file_upload_service: Arc<dyn FileUploadUseCase>,
    pub file_retrieval_service: Arc<dyn FileRetrievalUseCase>,
    pub file_management_service: Arc<dyn FileManagementUseCase>,
    pub file_use_case_factory: Arc<dyn FileUseCaseFactory>,
    pub i18n_service: Arc<I18nApplicationService>,
}