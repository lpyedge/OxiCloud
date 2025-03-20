use std::sync::Arc;
use anyhow::Result;
use sqlx::PgPool;

use crate::domain::services::auth_service::AuthService;
use crate::application::services::auth_application_service::AuthApplicationService;
use crate::infrastructure::repositories::{UserPgRepository, SessionPgRepository};
use crate::common::config::AppConfig;
use crate::common::di::AuthServices;

pub async fn create_auth_services(config: &AppConfig, pool: Arc<PgPool>) -> Result<AuthServices> {
    // Crear servicio de dominio de autenticación
    let auth_service = Arc::new(AuthService::new(
        config.auth.jwt_secret.clone(),
        config.auth.access_token_expiry_secs,
        config.auth.refresh_token_expiry_secs,
    ));
    
    // Crear repositorios PostgreSQL
    let user_repository = Arc::new(UserPgRepository::new(pool.clone()));
    let session_repository = Arc::new(SessionPgRepository::new(pool.clone()));
    
    // Crear servicio de aplicación de autenticación
    let auth_application_service = Arc::new(AuthApplicationService::new(
        user_repository,
        session_repository,
        auth_service.clone(),
    ));
    
    Ok(AuthServices {
        auth_service,
        auth_application_service,
    })
}