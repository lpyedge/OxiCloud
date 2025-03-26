use axum::{
    routing::get,
    Router,
    response::Html,
};
use tower_http::services::ServeDir;
use crate::common::di::AppState;
use crate::common::config::AppConfig;

/// Creates web routes for serving static files
pub fn create_web_routes() -> Router<AppState> {
    // Get config to access static path
    let config = AppConfig::from_env();
    let static_path = config.static_path.clone();

    Router::new()
        // Add specific route for login
        .route("/login", get(serve_login_page))
        // Serve static files
        .fallback_service(
            ServeDir::new(static_path)
        )
}

/// Serve the login page
async fn serve_login_page() -> Html<&'static str> {
    Html(include_str!("../../../static/login.html"))
}