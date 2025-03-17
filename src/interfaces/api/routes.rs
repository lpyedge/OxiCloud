use std::sync::Arc;
use axum::{
    routing::{get, post, put, delete},
    Router,
    extract::State,
};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::application::services::folder_service::FolderService;
use crate::application::services::file_service::FileService;
use crate::application::services::i18n_application_service::I18nApplicationService;
use crate::interfaces::api::handlers::folder_handler::FolderHandler;
use crate::interfaces::api::handlers::file_handler::FileHandler;
use crate::interfaces::api::handlers::i18n_handler::I18nHandler;

/// Creates API routes for the application
pub fn create_api_routes(
    folder_service: Arc<FolderService>, 
    file_service: Arc<FileService>,
    i18n_service: Option<Arc<I18nApplicationService>>,
) -> Router {
    let folders_router = Router::new()
        .route("/", post(FolderHandler::create_folder))
        .route("/", get(|State(service): State<Arc<FolderService>>| async move {
            // No parent ID means list root folders
            FolderHandler::list_folders(State(service), None).await
        }))
        .route("/{id}", get(FolderHandler::get_folder))
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
    
    // Create a router without the i18n routes
    let mut router = Router::new()
        .nest("/folders", folders_router)
        .nest("/files", files_router);
    
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
    
    router
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
}