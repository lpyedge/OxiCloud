use super::cache::{HttpCache, HttpCacheLayer, start_cache_cleanup_task};
use axum::{
    routing::get,
    Router,
    response::IntoResponse,
    Json,
    extract::State,
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::time::Duration;
use std::net::SocketAddr;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TestResponse {
    message: &'static str,
    timestamp: u64,
}

// Test handler for a simple GET endpoint
async fn test_handler() -> impl IntoResponse {
    // Create a simple response with a timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Simulate some processing time
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    let response = TestResponse {
        message: "Hello, this response is cacheable!",
        timestamp,
    };
    
    // Log the response generation
    tracing::info!("Generated fresh response with timestamp: {}", timestamp);
    
    Json(response)
}

// Run a test server with HTTP caching enabled
pub async fn run_test_server() {
    // Initialize HTTP cache with 10 seconds TTL
    let http_cache = HttpCache::with_max_age(10);
    
    // Start the cleanup task
    start_cache_cleanup_task(http_cache.clone());
    
    // Create a test router with the cache middleware
    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(HttpCacheLayer::new(http_cache));
    
    // Bind to a test port
    let addr = SocketAddr::from(([127, 0, 0, 1], 8086));
    tracing::info!("HTTP Cache test server listening on {}", addr);
    
    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}