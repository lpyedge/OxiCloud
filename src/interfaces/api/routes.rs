use std::sync::Arc;
use axum::{
    routing::{get, post, put, delete},
    Router,
    extract::{State, Query, Path},
};
use tower_http::{
    compression::CompressionLayer, 
    trace::TraceLayer,
};

use crate::interfaces::middleware::cache::{HttpCache, start_cache_cleanup_task};

use crate::application::services::folder_service::FolderService;
use crate::application::services::file_service::FileService;
use crate::application::services::i18n_application_service::I18nApplicationService;
use crate::application::services::batch_operations::BatchOperationService;

use crate::interfaces::api::handlers::folder_handler::FolderHandler;
use crate::interfaces::api::handlers::file_handler::FileHandler;
use crate::interfaces::api::handlers::i18n_handler::I18nHandler;
use crate::interfaces::api::handlers::batch_handler::{
    self, BatchHandlerState
};
use crate::application::dtos::pagination::PaginationRequestDto;

/// Creates API routes for the application
pub fn create_api_routes(
    folder_service: Arc<FolderService>, 
    file_service: Arc<FileService>,
    i18n_service: Option<Arc<I18nApplicationService>>,
) -> Router {
    // Inicializar el servicio de operaciones por lotes
    let batch_service = Arc::new(BatchOperationService::default(
        file_service.clone(),
        folder_service.clone()
    ));
    
    // Crear estado para el manejador de operaciones por lotes
    let batch_handler_state = BatchHandlerState {
        batch_service: batch_service.clone(),
    };
    
    // Implement HTTP Cache
    let http_cache = HttpCache::new();
    
    // Define TTL values for different resource types (in seconds)
    let _folders_ttl = 300;      // 5 minutes
    let _files_list_ttl = 300;   // 5 minutes
    let _i18n_ttl = 3600;        // 1 hour
    
    // Start the cleanup task for HTTP cache
    start_cache_cleanup_task(http_cache.clone());
    
    let folders_router = Router::new()
        .route("/", post(FolderHandler::create_folder))
        .route("/", get(|State(service): State<Arc<FolderService>>| async move {
            // No parent ID means list root folders
            FolderHandler::list_folders(State(service), None).await
        }))
        .route("/paginated", get(|
            State(service): State<Arc<FolderService>>,
            pagination: Query<PaginationRequestDto>
        | async move {
            // Paginación para carpetas raíz (sin parent)
            FolderHandler::list_folders_paginated(State(service), pagination, None).await
        }))
        .route("/{id}", get(FolderHandler::get_folder))
        .route("/{id}/contents", get(|
            State(service): State<Arc<FolderService>>,
            Path(id): Path<String>
        | async move {
            // Listar contenido de una carpeta por su ID
            FolderHandler::list_folders(State(service), Some(&id)).await
        }))
        .route("/{id}/contents/paginated", get(|
            State(service): State<Arc<FolderService>>,
            Path(id): Path<String>,
            pagination: Query<PaginationRequestDto>
        | async move {
            // Listar contenido paginado de una carpeta por su ID
            FolderHandler::list_folders_paginated(State(service), pagination, Some(&id)).await
        }))
        .route("/{id}/rename", put(FolderHandler::rename_folder))
        .route("/{id}/move", put(FolderHandler::move_folder))
        .route("/{id}", delete(FolderHandler::delete_folder))
        .with_state(folder_service);
        
    let files_router = Router::new()
        .route("/", get(|
            State(service): State<Arc<FileService>>,
            axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
        | async move {
            // Get folder_id from query parameter if present
            let folder_id = params.get("folder_id").map(|id| id.as_str());
            tracing::info!("API: Listando archivos con folder_id: {:?}", folder_id);
            FileHandler::list_files(State(service), folder_id).await
        }))
        .route("/upload", post(FileHandler::upload_file))
        .route("/{id}", get(FileHandler::download_file))
        .route("/{id}", delete(FileHandler::delete_file))
        .route("/{id}/move", put(FileHandler::move_file))
        .with_state(file_service);
    
    // Crear rutas para operaciones por lotes
    let batch_router = Router::new()
        // Operaciones de archivos
        .route("/files/move", post(batch_handler::move_files_batch))
        .route("/files/copy", post(batch_handler::copy_files_batch))
        .route("/files/delete", post(batch_handler::delete_files_batch))
        .route("/files/get", post(batch_handler::get_files_batch))
        // Operaciones de carpetas
        .route("/folders/delete", post(batch_handler::delete_folders_batch))
        .route("/folders/create", post(batch_handler::create_folders_batch))
        .route("/folders/get", post(batch_handler::get_folders_batch))
        .with_state(batch_handler_state);
    
    // Create a router without the i18n routes
    let mut router = Router::new()
        .nest("/folders", folders_router)
        .nest("/files", files_router)
        .nest("/batch", batch_router);
    
    // Add i18n routes if the service is provided
    if let Some(i18n_service) = i18n_service {
        let i18n_router = Router::new()
            .route("/locales", get(I18nHandler::get_locales))
            .route("/translate", get(I18nHandler::translate))
            .route("/locales/{locale_code}", get(|
                State(service): State<Arc<I18nApplicationService>>,
                axum::extract::Path(locale_code): axum::extract::Path<String>,
            | async move {
                I18nHandler::get_translations(State(service), locale_code).await
            }))
            .with_state(i18n_service);
        
        router = router.nest("/i18n", i18n_router);
    }
    
    // Apply compression and tracing layers
    router
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        // HTTP caching is disabled temporarily due to compatibility issues
        // .layer(HttpCacheLayer::new(http_cache.clone()).with_max_age(folders_ttl))
}