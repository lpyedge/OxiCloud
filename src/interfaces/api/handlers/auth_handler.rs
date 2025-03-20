use std::sync::Arc;
use axum::{
    Router,
    routing::{post, get, put},
    extract::{State, Json, Path, Extension},
    http::{StatusCode, HeaderMap, header},
    response::IntoResponse,
    middleware,
};

use crate::common::di::AppState;
use crate::application::dtos::user_dto::{
    LoginDto, RegisterDto, UserDto, ChangePasswordDto, RefreshTokenDto, AuthResponseDto
};
use crate::interfaces::middleware::auth::CurrentUser;
use crate::common::errors::AppError;

pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .route("/me", get(get_current_user))
        .route("/change-password", put(change_password))
        .route("/logout", post(logout))
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(dto): Json<RegisterDto>,
) -> Result<impl IntoResponse, AppError> {
    let auth_service = state.auth_service.as_ref()
        .ok_or_else(|| AppError::internal_error("Servicio de autenticación no configurado"))?;
    
    let user = auth_service.auth_application_service.register(dto).await?;
    
    Ok((StatusCode::CREATED, Json(user)))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(dto): Json<LoginDto>,
) -> Result<impl IntoResponse, AppError> {
    let auth_service = state.auth_service.as_ref()
        .ok_or_else(|| AppError::internal_error("Servicio de autenticación no configurado"))?;
    
    let auth_response = auth_service.auth_application_service.login(dto).await?;
    
    Ok((StatusCode::OK, Json(auth_response)))
}

async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(dto): Json<RefreshTokenDto>,
) -> Result<impl IntoResponse, AppError> {
    let auth_service = state.auth_service.as_ref()
        .ok_or_else(|| AppError::internal_error("Servicio de autenticación no configurado"))?;
    
    let auth_response = auth_service.auth_application_service.refresh_token(dto).await?;
    
    Ok((StatusCode::OK, Json(auth_response)))
}

async fn get_current_user(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<impl IntoResponse, AppError> {
    let auth_service = state.auth_service.as_ref()
        .ok_or_else(|| AppError::internal_error("Servicio de autenticación no configurado"))?;
    
    let user = auth_service.auth_application_service.get_user_by_id(&current_user.id).await?;
    
    Ok((StatusCode::OK, Json(user)))
}

async fn change_password(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Json(dto): Json<ChangePasswordDto>,
) -> Result<impl IntoResponse, AppError> {
    let auth_service = state.auth_service.as_ref()
        .ok_or_else(|| AppError::internal_error("Servicio de autenticación no configurado"))?;
    
    auth_service.auth_application_service.change_password(&current_user.id, dto).await?;
    
    Ok(StatusCode::OK)
}

async fn logout(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let auth_service = state.auth_service.as_ref()
        .ok_or_else(|| AppError::internal_error("Servicio de autenticación no configurado"))?;
    
    // Extract refresh token from request
    let refresh_token = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::unauthorized("Token de refresco no encontrado"))?;
    
    auth_service.auth_application_service.logout(&current_user.id, refresh_token).await?;
    
    Ok(StatusCode::OK)
}

