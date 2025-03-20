use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use chrono::Utc;

use crate::domain::entities::session::Session;
use crate::domain::repositories::session_repository::{SessionRepository, SessionRepositoryError, SessionRepositoryResult};
use crate::application::ports::auth_ports::SessionStoragePort;
use crate::common::errors::DomainError;

pub struct SessionPgRepository {
    pool: Arc<PgPool>,
}

impl SessionPgRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
    
    // Método auxiliar para mapear errores SQL a errores de dominio
    fn map_sqlx_error(err: sqlx::Error) -> SessionRepositoryError {
        match err {
            sqlx::Error::RowNotFound => {
                SessionRepositoryError::NotFound("Sesión no encontrada".to_string())
            },
            _ => SessionRepositoryError::DatabaseError(
                format!("Error de base de datos: {}", err)
            ),
        }
    }
}

#[async_trait]
impl SessionRepository for SessionPgRepository {
    /// Crea una nueva sesión
    async fn create_session(&self, session: Session) -> SessionRepositoryResult<Session> {
        sqlx::query(
            r#"
            INSERT INTO auth.sessions (
                id, user_id, refresh_token, expires_at, 
                ip_address, user_agent, created_at, revoked
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8
            )
            "#
        )
        .bind(session.id())
        .bind(session.user_id())
        .bind(session.refresh_token())
        .bind(session.expires_at())
        .bind(&session.ip_address)
        .bind(&session.user_agent)
        .bind(session.created_at())
        .bind(session.is_revoked())
        .execute(&*self.pool)
        .await
        .map_err(Self::map_sqlx_error)?;

        Ok(session)
    }
    
    /// Obtiene una sesión por ID
    async fn get_session_by_id(&self, id: &str) -> SessionRepositoryResult<Session> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, user_id, refresh_token, expires_at, 
                ip_address, user_agent, created_at, revoked
            FROM auth.sessions
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_one(&*self.pool)
        .await
        .map_err(Self::map_sqlx_error)?;

        Ok(Session {
            id: row.get("id"),
            user_id: row.get("user_id"),
            refresh_token: row.get("refresh_token"),
            expires_at: row.get("expires_at"),
            ip_address: row.get("ip_address"),
            user_agent: row.get("user_agent"),
            created_at: row.get("created_at"),
            revoked: row.get("revoked"),
        })
    }
    
    /// Obtiene una sesión por token de actualización
    async fn get_session_by_refresh_token(&self, refresh_token: &str) -> SessionRepositoryResult<Session> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, user_id, refresh_token, expires_at, 
                ip_address, user_agent, created_at, revoked
            FROM auth.sessions
            WHERE refresh_token = $1
            "#
        )
        .bind(refresh_token)
        .fetch_one(&*self.pool)
        .await
        .map_err(Self::map_sqlx_error)?;

        Ok(Session {
            id: row.get("id"),
            user_id: row.get("user_id"),
            refresh_token: row.get("refresh_token"),
            expires_at: row.get("expires_at"),
            ip_address: row.get("ip_address"),
            user_agent: row.get("user_agent"),
            created_at: row.get("created_at"),
            revoked: row.get("revoked"),
        })
    }
    
    /// Obtiene todas las sesiones de un usuario
    async fn get_sessions_by_user_id(&self, user_id: &str) -> SessionRepositoryResult<Vec<Session>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id, user_id, refresh_token, expires_at, 
                ip_address, user_agent, created_at, revoked
            FROM auth.sessions
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(&*self.pool)
        .await
        .map_err(Self::map_sqlx_error)?;

        let sessions = rows.into_iter()
            .map(|row| {
                Session {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    refresh_token: row.get("refresh_token"),
                    expires_at: row.get("expires_at"),
                    ip_address: row.get("ip_address"),
                    user_agent: row.get("user_agent"),
                    created_at: row.get("created_at"),
                    revoked: row.get("revoked"),
                }
            })
            .collect();

        Ok(sessions)
    }
    
    /// Revoca una sesión específica
    async fn revoke_session(&self, session_id: &str) -> SessionRepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE auth.sessions
            SET revoked = true
            WHERE id = $1
            "#
        )
        .bind(session_id)
        .execute(&*self.pool)
        .await
        .map_err(Self::map_sqlx_error)?;

        Ok(())
    }
    
    /// Revoca todas las sesiones de un usuario
    async fn revoke_all_user_sessions(&self, user_id: &str) -> SessionRepositoryResult<u64> {
        let result = sqlx::query(
            r#"
            UPDATE auth.sessions
            SET revoked = true
            WHERE user_id = $1 AND revoked = false
            "#
        )
        .bind(user_id)
        .execute(&*self.pool)
        .await
        .map_err(Self::map_sqlx_error)?;

        Ok(result.rows_affected())
    }
    
    /// Elimina sesiones expiradas
    async fn delete_expired_sessions(&self) -> SessionRepositoryResult<u64> {
        let now = Utc::now();
        
        let result = sqlx::query(
            r#"
            DELETE FROM auth.sessions
            WHERE expires_at < $1
            "#
        )
        .bind(now)
        .execute(&*self.pool)
        .await
        .map_err(Self::map_sqlx_error)?;

        Ok(result.rows_affected())
    }
}

// Implementación del puerto de almacenamiento para la capa de aplicación
#[async_trait]
impl SessionStoragePort for SessionPgRepository {
    async fn create_session(&self, session: Session) -> Result<Session, DomainError> {
        SessionRepository::create_session(self, session).await.map_err(DomainError::from)
    }
    
    async fn get_session_by_refresh_token(&self, refresh_token: &str) -> Result<Session, DomainError> {
        SessionRepository::get_session_by_refresh_token(self, refresh_token)
            .await
            .map_err(DomainError::from)
    }
    
    async fn revoke_session(&self, session_id: &str) -> Result<(), DomainError> {
        SessionRepository::revoke_session(self, session_id).await.map_err(DomainError::from)
    }
    
    async fn revoke_all_user_sessions(&self, user_id: &str) -> Result<u64, DomainError> {
        SessionRepository::revoke_all_user_sessions(self, user_id)
            .await
            .map_err(DomainError::from)
    }
}