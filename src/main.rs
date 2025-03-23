use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod common;
mod domain;
mod application;
mod infrastructure;
mod interfaces;

use application::services::folder_service::FolderService;
use application::services::file_service::FileService;
use application::services::i18n_application_service::I18nApplicationService;
use application::services::storage_mediator::FileSystemStorageMediator;
use domain::services::path_service::PathService;
use infrastructure::repositories::folder_fs_repository::FolderFsRepository;
use infrastructure::repositories::file_fs_repository::FileFsRepository;
use infrastructure::repositories::parallel_file_processor::ParallelFileProcessor;
use infrastructure::services::file_system_i18n_service::FileSystemI18nService;
use infrastructure::services::id_mapping_service::IdMappingService;
use infrastructure::services::id_mapping_optimizer::IdMappingOptimizer;
use infrastructure::services::file_metadata_cache::FileMetadataCache;
use infrastructure::services::buffer_pool::BufferPool;
use infrastructure::services::compression_service::GzipCompressionService;
use interfaces::{create_api_routes, web::create_web_routes};
use common::db::create_database_pool;
use common::auth_factory::create_auth_services;
use common::di::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration from environment variables
    let config = common::config::AppConfig::from_env();
    
    // Set up storage directory
    let storage_path = config.storage_path.clone();
    if !storage_path.exists() {
        std::fs::create_dir_all(&storage_path).expect("Failed to create storage directory");
    }

    // Set up locales directory
    let locales_path = PathBuf::from("./static/locales");
    if !locales_path.exists() {
        std::fs::create_dir_all(&locales_path).expect("Failed to create locales directory");
    }
    
    // Initialize database if auth is enabled
    let db_pool = if config.features.enable_auth {
        match create_database_pool(&config).await {
            Ok(pool) => {
                tracing::info!("PostgreSQL database pool initialized successfully");
                Some(Arc::new(pool))
            },
            Err(e) => {
                tracing::error!("Failed to initialize database pool: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Initialize path service
    let path_service = Arc::new(PathService::new(storage_path.clone()));
    
    // Initialize ID mapping service for folders
    let folder_id_mapping_path = storage_path.join("folder_ids.json");
    let folder_id_mapping_service = Arc::new(
        IdMappingService::new(folder_id_mapping_path).await
            .expect("Failed to initialize folder ID mapping service")
    );
    
    // Initialize ID mapping service for files
    let file_id_mapping_path = storage_path.join("file_ids.json");
    let file_id_mapping_service = Arc::new(
        IdMappingService::new(file_id_mapping_path).await
            .expect("Failed to initialize file ID mapping service")
    );
    
    // For backward compatibility, use folder ID service as the base ID mapping service
    let base_id_mapping_service = folder_id_mapping_service.clone();
    
    // Create optimized ID mapping service with batch processing and caching
    let id_mapping_optimizer = Arc::new(
        IdMappingOptimizer::new(base_id_mapping_service.clone())
    );
    
    // Initialize folder repository with all required components
    let folder_repository = Arc::new(FolderFsRepository::new(
        storage_path.clone(),
        Arc::new(FileSystemStorageMediator::new_stub()), // Temporary stub (will be replaced)
        base_id_mapping_service.clone(),
        path_service.clone()
    ));
    
    // Initialize storage mediator
    let storage_mediator = Arc::new(FileSystemStorageMediator::new(
        folder_repository.clone(),
        path_service.clone(),
        id_mapping_optimizer.clone()
    ));
    
    // Update folder repository with proper storage mediator
    // This replaces the stub we initialized it with
    let folder_repository = Arc::new(FolderFsRepository::new(
        storage_path.clone(),
        storage_mediator.clone(),
        base_id_mapping_service.clone(),
        path_service.clone()
    ));
    
    // Start cleanup task for ID mapping optimizer
    IdMappingOptimizer::start_cleanup_task(id_mapping_optimizer.clone());
    
    tracing::info!("ID mapping optimizer initialized with batch processing and caching");
    
    // Initialize the metadata cache 
    let config = common::config::AppConfig::default();
    let metadata_cache = Arc::new(FileMetadataCache::default_with_config(config.clone()));
    
    // Start the periodic cleanup task for cache maintenance
    let cache_clone = metadata_cache.clone();
    tokio::spawn(async move {
        FileMetadataCache::start_cleanup_task(cache_clone).await;
    });
    
    // Initialize the buffer pool for memory optimization
    // Use larger buffer size for better performance with large files
    let buffer_pool = BufferPool::new(256 * 1024, 50, 120); // 256KB buffers, 50 max, 2 min TTL
    
    // Start the buffer pool cleanup task
    BufferPool::start_cleaner(buffer_pool.clone());
    
    tracing::info!("Buffer pool initialized with 50 buffers of 256KB each");
    
    // Initialize parallel file processor with buffer pool
    let parallel_processor = Arc::new(ParallelFileProcessor::new_with_buffer_pool(
        config.clone(),
        buffer_pool.clone()
    ));
    
    // Initialize compression service with buffer pool
    let _compression_service = Arc::new(GzipCompressionService::new_with_buffer_pool(
        buffer_pool.clone()
    ));
    
    // Initialize file repository with mediator, ID mapping service, metadata cache, and parallel processor
    let file_repository = Arc::new(FileFsRepository::new_with_processor(
        storage_path.clone(), 
        storage_mediator,
        file_id_mapping_service.clone(), // Use the file-specific ID mapping service
        path_service.clone(),
        metadata_cache.clone(), // Clone to keep a reference for later use
        parallel_processor
    ));

    // Initialize application services
    let folder_service = Arc::new(FolderService::new(folder_repository));
    let file_service = Arc::new(FileService::new(file_repository));
    
    // Initialize i18n service
    let i18n_repository = Arc::new(FileSystemI18nService::new(locales_path.clone()));
    let i18n_service = Arc::new(I18nApplicationService::new(i18n_repository.clone()));
    
    // Preload translations
    if let Err(e) = i18n_service.load_translations(domain::services::i18n_service::Locale::English).await {
        tracing::warn!("Failed to load English translations: {}", e);
    }
    if let Err(e) = i18n_service.load_translations(domain::services::i18n_service::Locale::Spanish).await {
        tracing::warn!("Failed to load Spanish translations: {}", e);
    }
    
    tracing::info!("Compression service initialized with buffer pool support");
    
    // Initialize auth services if enabled and database connection is available
    let auth_services = if config.features.enable_auth && db_pool.is_some() {
        match create_auth_services(
            &config, 
            db_pool.as_ref().unwrap().clone(),
            Some(folder_service.clone())  // Pasar el servicio de carpetas para creación automática de carpetas de usuario
        ).await {
            Ok(services) => {
                tracing::info!("Authentication services initialized successfully with folder service");
                Some(services)
            },
            Err(e) => {
                tracing::error!("Failed to initialize authentication services: {}", e);
                None
            }
        }
    } else {
        None
    };
    
    // Create AppState for DI container
    let core_services = common::di::CoreServices {
        path_service: path_service.clone(),
        cache_manager: Arc::new(infrastructure::services::cache_manager::StorageCacheManager::default()),
        id_mapping_service: base_id_mapping_service.clone(), // We keep using the folder ID mapping service for core services
        config: config.clone(),
    };
    
    // Crear stubs para los repositorios
    let file_read_stub = Arc::new(infrastructure::repositories::FileFsReadRepository::default_stub());
    let file_write_stub = Arc::new(infrastructure::repositories::FileFsWriteRepository::default_stub());
    let storage_mediator_stub = Arc::new(application::services::storage_mediator::FileSystemStorageMediator::new_stub());
    let metadata_manager = Arc::new(infrastructure::repositories::FileMetadataManager::default());
    let path_resolver_stub = Arc::new(infrastructure::repositories::FilePathResolver::default_stub());
    
    let repository_services = common::di::RepositoryServices {
        folder_repository: Arc::new(FolderFsRepository::new(
            storage_path.clone(),
            storage_mediator_stub.clone(),
            folder_id_mapping_service.clone(),
            path_service.clone()
        )),
        file_repository: Arc::new(FileFsRepository::new(
            storage_path.clone(), 
            storage_mediator_stub.clone(),
            file_id_mapping_service.clone(),
            path_service.clone(),
            metadata_cache.clone(),
        )),
        file_read_repository: file_read_stub,
        file_write_repository: file_write_stub,
        i18n_repository: i18n_repository.clone(),
        storage_mediator: storage_mediator_stub,
        metadata_manager,
        path_resolver: path_resolver_stub,
    };
    
    let application_services = common::di::ApplicationServices {
        folder_service: folder_service.clone(),
        file_service: file_service.clone(),
        file_upload_service: Arc::new(application::services::file_upload_service::FileUploadService::default_stub()),
        file_retrieval_service: Arc::new(application::services::file_retrieval_service::FileRetrievalService::default_stub()),
        file_management_service: Arc::new(application::services::file_management_service::FileManagementService::default_stub()),
        file_use_case_factory: Arc::new(application::services::file_use_case_factory::AppFileUseCaseFactory::default_stub()),
        i18n_service: i18n_service.clone(),
    };
    
    // Create the AppState without Arc first
    let mut app_state = AppState::new(
        core_services,
        repository_services,
        application_services,
    );
    
    // Add database pool if available
    if let Some(pool) = db_pool {
        app_state = app_state.with_database(pool);
    }
    
    // Add auth services if available
    let have_auth_services = auth_services.is_some();
    if let Some(services) = auth_services {
        app_state = app_state.with_auth_services(services);
    }
    
    // Wrap in Arc after all modifications
    let app_state = Arc::new(app_state);

    // Build application router
    let api_routes = create_api_routes(folder_service, file_service, Some(i18n_service));
    let web_routes = create_web_routes();
    
    // Build the app router
    // Import auth handler
    use interfaces::api::handlers::auth_handler::auth_routes;
    
    // Create basic app router
    let mut app = Router::new()
        .nest("/api", api_routes)
        .merge(web_routes)
        .layer(TraceLayer::new_for_http());
    
    // Add auth routes if auth is enabled
    if config.features.enable_auth && have_auth_services {
        // Create auth routes with app state
        let auth_router = auth_routes().with_state(app_state.clone());
        
        // Add auth routes at /api/auth
        app = app.nest("/api/auth", auth_router);
    }

    // Preload common directories to warm the cache
    tracing::info!("Preloading common directories to warm up cache...");
    if let Ok(count) = metadata_cache.preload_directory(&storage_path, true, 1).await {
        tracing::info!("Preloaded {} directory entries into cache", count);
    }
    
    // Start server with clear message
    let addr = SocketAddr::from(([127, 0, 0, 1], 8086));
    tracing::info!("Starting OxiCloud server on http://{}", addr);
    
    // Start the server
    tracing::info!("Authentication system initialized successfully");
    
    // Import the redirect middleware
    use crate::interfaces::middleware::redirect::redirect_middleware;
    
    // Apply the redirect middleware to handle legacy routes
    app = app.layer(axum::middleware::from_fn(redirect_middleware));
    
    // Create a standard TCP listener
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Server binding to http://{}", addr);
    tracing::info!("Starting server with Axum routes...");
    
    // For Axum 0.8, we need to properly handle state
    // Add global state to the router
    let app = app.with_state(app_state);
    
    // Use axum's serve function with the router with state
    axum::serve(listener, app).await?;
    
    tracing::info!("Server shutdown completed");
    
    Ok(())
}

