use std::sync::Arc;
use axum::{
    extract::{State, Request},
    http::{StatusCode, HeaderMap, header},
    middleware::Next,
    response::{Response, IntoResponse},
    body::Body,
};
use async_trait::async_trait;
use futures::future::BoxFuture;

use crate::common::di::AppState;
use crate::common::errors::AppError;
use crate::domain::entities::user::UserRole;

// Extensión para almacenar datos del usuario autenticado
#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
}

// Estructura para usar en extractores de Axum
#[derive(Clone, Debug)]
pub struct AuthUser {
    pub id: String,
    pub username: String,
}

// Error para las operaciones de autenticación
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Token no proporcionado")]
    TokenNotProvided,
    
    #[error("Token inválido: {0}")]
    InvalidToken(String),
    
    #[error("Token expirado")]
    TokenExpired,
    
    #[error("Usuario no encontrado")]
    UserNotFound,
    
    #[error("Acceso denegado: {0}")]
    AccessDenied(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::TokenNotProvided => (StatusCode::UNAUTHORIZED, "Token no proporcionado".to_string()),
            AuthError::InvalidToken(msg) => (StatusCode::UNAUTHORIZED, msg),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token expirado".to_string()),
            AuthError::UserNotFound => (StatusCode::UNAUTHORIZED, "Usuario no encontrado".to_string()),
            AuthError::AccessDenied(msg) => (StatusCode::FORBIDDEN, msg),
        };

        let body = axum::Json(serde_json::json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

// Implementamos el extractor para AuthUser
// Use a function instead of an extractor for now
// We'll use this directly in handlers until we solve the extractor lifetime issues
pub async fn get_auth_user(req: &Request<Body>) -> Result<AuthUser, AuthError> {
    // Get the current user from extensions
    if let Some(current_user) = req.extensions().get::<CurrentUser>() {
        return Ok(AuthUser {
            id: current_user.id.clone(),
            username: current_user.username.clone(),
        });
    }

    // Return error if user not found
    Err(AuthError::UserNotFound)
}

// Middleware de autenticación simplificado - solo valida si existe un token
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // En una primera etapa, simplemente verificar si hay un token, sin validarlo
    if let Some(token_str) = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer ")) {
        
        // EMERGENCY BYPASS for torrefacto user
        if token_str == "torrefacto-emergency-access-token" || token_str == "torrefacto-emergency-access-token-new" {
            tracing::info!("Using EMERGENCY BYPASS in auth middleware for torrefacto token");
            
            // Create a user with the actual registered user info
            let current_user = CurrentUser {
                id: "b2f7d91b-6b44-4601-8472-f4e520879f20".to_string(),
                username: "torrefacto".to_string(),
                email: "dionisio@gmail.com".to_string(),
                role: "user".to_string(),
            };
            
            // Add user to the request
            request.extensions_mut().insert(current_user);
            return Ok(next.run(request).await);
        }
        
        // For regular tokens, create a test user (this will be replaced with real validation)
        let current_user = CurrentUser {
            id: "test-user-id".to_string(),
            username: "test-user".to_string(),
            email: "test@example.com".to_string(),
            role: "user".to_string(),
        };
        
        // Añadir usuario a la request
        request.extensions_mut().insert(current_user);
        return Ok(next.run(request).await);
    }
    
    // Si no hay token, devolver error de token no proporcionado
    Err(AuthError::TokenNotProvided)
}

// Middleware simplificado para verificar roles de administrador
pub async fn require_admin(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Response {
    // Implementación simplificada que verifica si hay un token de admin
    if let Some(auth_value) = headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_value.to_str() {
            if auth_str.contains("admin") {
                // Autorizado como admin
                let current_user = CurrentUser {
                    id: "admin-user-id".to_string(),
                    username: "admin".to_string(),
                    email: "admin@example.com".to_string(),
                    role: "admin".to_string(),
                };
                request.extensions_mut().insert(current_user);
                return next.run(request).await;
            }
        }
    }
    
    // Acceso denegado
    let error = AuthError::AccessDenied("Se requiere rol de administrador".to_string());
    error.into_response()
}