use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use tracing::{debug, error, instrument};

use crate::application::ports::trash_ports::TrashUseCase;
use crate::common::di::AppState;
use crate::interfaces::middleware::auth::AuthUser;

/// Obtiene todos los elementos en la papelera para el usuario actual
#[instrument(skip(state))]
pub async fn get_trash_items(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    debug!("Solicitud para listar elementos en papelera para usuario {}", auth_user.id);
    
    let trash_service = match state.trash_service.as_ref() {
        Some(service) => service,
        None => {
            return (StatusCode::NOT_IMPLEMENTED, Json(json!({
                "error": "Trash feature is not enabled"
            }))).into_response();
        }
    };
    
    let result = trash_service.get_trash_items(&auth_user.id).await;
    
    match result {
        Ok(items) => {
            debug!("Encontrados {} elementos en la papelera", items.len());
            (StatusCode::OK, Json(items)).into_response()
        },
        Err(e) => {
            error!("Error al obtener elementos de la papelera: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": format!("Error retrieving trash items: {}", e)
            }))).into_response()
        }
    }
}

/// Mueve un elemento (archivo o carpeta) a la papelera
#[instrument(skip(state))]
pub async fn move_to_trash(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((item_type, item_id)): Path<(String, String)>,
) -> impl IntoResponse {
    debug!("Solicitud para mover a papelera: tipo={}, id={}, usuario={}", 
           item_type, item_id, auth_user.id);
    
    let trash_service = match state.trash_service.as_ref() {
        Some(service) => service,
        None => {
            return (StatusCode::NOT_IMPLEMENTED, Json(json!({
                "error": "Trash feature is not enabled"
            }))).into_response();
        }
    };
    let result = trash_service.move_to_trash(&item_id, &item_type, &auth_user.id).await;
    
    match result {
        Ok(_) => {
            debug!("Elemento movido a papelera con éxito");
            (StatusCode::OK, Json(json!({
                "success": true,
                "message": "Item moved to trash successfully"
            }))).into_response()
        },
        Err(e) => {
            error!("Error al mover elemento a papelera: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": format!("Error moving item to trash: {}", e)
            }))).into_response()
        }
    }
}

/// Restaura un elemento desde la papelera a su ubicación original
#[instrument(skip(state))]
pub async fn restore_from_trash(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(trash_id): Path<String>,
) -> impl IntoResponse {
    debug!("Solicitud para restaurar elemento {} de papelera", trash_id);
    
    let trash_service = match state.trash_service.as_ref() {
        Some(service) => service,
        None => {
            return (StatusCode::NOT_IMPLEMENTED, Json(json!({
                "error": "Trash feature is not enabled"
            }))).into_response();
        }
    };
    let result = trash_service.restore_item(&trash_id, &auth_user.id).await;
    
    match result {
        Ok(_) => {
            debug!("Elemento restaurado con éxito");
            (StatusCode::OK, Json(json!({
                "success": true,
                "message": "Item restored successfully"
            }))).into_response()
        },
        Err(e) => {
            error!("Error al restaurar elemento de papelera: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": format!("Error restoring item from trash: {}", e)
            }))).into_response()
        }
    }
}

/// Elimina permanentemente un elemento de la papelera
#[instrument(skip(state))]
pub async fn delete_permanently(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(trash_id): Path<String>,
) -> impl IntoResponse {
    debug!("Solicitud para eliminar permanentemente elemento {}", trash_id);
    
    let trash_service = match state.trash_service.as_ref() {
        Some(service) => service,
        None => {
            return (StatusCode::NOT_IMPLEMENTED, Json(json!({
                "error": "Trash feature is not enabled"
            }))).into_response();
        }
    };
    let result = trash_service.delete_permanently(&trash_id, &auth_user.id).await;
    
    match result {
        Ok(_) => {
            debug!("Elemento eliminado permanentemente");
            (StatusCode::OK, Json(json!({
                "success": true,
                "message": "Item deleted permanently"
            }))).into_response()
        },
        Err(e) => {
            error!("Error al eliminar permanentemente elemento: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": format!("Error deleting item permanently: {}", e)
            }))).into_response()
        }
    }
}

/// Vacía la papelera completamente para el usuario actual
#[instrument(skip(state))]
pub async fn empty_trash(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    debug!("Solicitud para vaciar papelera del usuario {}", auth_user.id);
    
    let trash_service = match state.trash_service.as_ref() {
        Some(service) => service,
        None => {
            return (StatusCode::NOT_IMPLEMENTED, Json(json!({
                "error": "Trash feature is not enabled"
            }))).into_response();
        }
    };
    let result = trash_service.empty_trash(&auth_user.id).await;
    
    match result {
        Ok(_) => {
            debug!("Papelera vaciada con éxito");
            (StatusCode::OK, Json(json!({
                "success": true,
                "message": "Trash emptied successfully"
            }))).into_response()
        },
        Err(e) => {
            error!("Error al vaciar papelera: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": format!("Error emptying trash: {}", e)
            }))).into_response()
        }
    }
}