use axum::Router;
use tower_http::services::ServeDir;
use std::path::PathBuf;

/// Creates web routes for serving static files
pub fn create_web_routes() -> Router {
    Router::new()
        .fallback_service(
            ServeDir::new(PathBuf::from("static"))
        )
}