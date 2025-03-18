use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::serve;
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

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Set up storage directory
    let storage_path = PathBuf::from("./storage");
    if !storage_path.exists() {
        std::fs::create_dir_all(&storage_path).expect("Failed to create storage directory");
    }

    // Set up locales directory
    let locales_path = PathBuf::from("./static/locales");
    if !locales_path.exists() {
        std::fs::create_dir_all(&locales_path).expect("Failed to create locales directory");
    }

    // Initialize path service
    let path_service = Arc::new(PathService::new(storage_path.clone()));
    
    // Initialize ID mapping service with optimizer
    let id_mapping_path = storage_path.join("folder_ids.json");
    let base_id_mapping_service = Arc::new(
        IdMappingService::new(id_mapping_path).await
            .expect("Failed to initialize ID mapping service")
    );
    
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
        base_id_mapping_service.clone(), // Use the base service, not the optimizer
        path_service.clone(),
        metadata_cache.clone(), // Clone to keep a reference for later use
        parallel_processor
    ));

    // Initialize application services
    let folder_service = Arc::new(FolderService::new(folder_repository));
    let file_service = Arc::new(FileService::new(file_repository));
    
    // Initialize i18n service
    let i18n_repository = Arc::new(FileSystemI18nService::new(locales_path));
    let i18n_service = Arc::new(I18nApplicationService::new(i18n_repository));
    
    // Preload translations
    if let Err(e) = i18n_service.load_translations(domain::services::i18n_service::Locale::English).await {
        tracing::warn!("Failed to load English translations: {}", e);
    }
    if let Err(e) = i18n_service.load_translations(domain::services::i18n_service::Locale::Spanish).await {
        tracing::warn!("Failed to load Spanish translations: {}", e);
    }
    
    tracing::info!("Compression service initialized with buffer pool support");

    // Build application router
    let api_routes = create_api_routes(folder_service, file_service, Some(i18n_service));
    let web_routes = create_web_routes();

    let app = Router::new()
        .nest("/api", api_routes)
        .merge(web_routes)
        .layer(TraceLayer::new_for_http());

    // Preload common directories to warm the cache
    tracing::info!("Preloading common directories to warm up cache...");
    if let Ok(count) = metadata_cache.preload_directory(&storage_path, true, 1).await {
        tracing::info!("Preloaded {} directory entries into cache", count);
    }
    
    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8085));
    tracing::info!("listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    serve::serve(listener, app).await.unwrap();
}