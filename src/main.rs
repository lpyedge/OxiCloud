use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::serve;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod domain;
mod application;
mod infrastructure;
mod interfaces;

use application::services::folder_service::FolderService;
use application::services::file_service::FileService;
use application::services::i18n_application_service::I18nApplicationService;
use infrastructure::repositories::folder_fs_repository::FolderFsRepository;
use infrastructure::repositories::file_fs_repository::FileFsRepository;
use infrastructure::services::file_system_i18n_service::FileSystemI18nService;
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

    // Initialize repositories
    let folder_repository = Arc::new(FolderFsRepository::new(storage_path.clone()));
    let file_repository = Arc::new(FileFsRepository::new(storage_path.clone(), folder_repository.clone()));

    // Initialize services
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

    // Build application router
    let api_routes = create_api_routes(folder_service, file_service, Some(i18n_service));
    let web_routes = create_web_routes();

    let app = Router::new()
        .nest("/api", api_routes)
        .merge(web_routes)
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8085));
    tracing::info!("listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    serve::serve(listener, app).await.unwrap();
}