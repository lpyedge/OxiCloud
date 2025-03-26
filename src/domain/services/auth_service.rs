use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::Utc;

use crate::domain::entities::user::User;
use crate::common::errors::{DomainError, ErrorKind};

// Reclamaciones JWT
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,     // user ID
    pub exp: i64,        // expiration timestamp
    pub iat: i64,        // issued at timestamp
    pub jti: String,     // JWT ID
    pub username: String, // username
    pub email: String,   // email
    pub role: String,    // role as string
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Credenciales inválidas")]
    InvalidCredentials,
    
    #[error("Token expirado")]
    TokenExpired,
    
    #[error("Token inválido: {0}")]
    InvalidToken(String),
    
    #[error("Acceso denegado: {0}")]
    AccessDenied(String),
    
    #[error("Operación no permitida: {0}")]
    OperationNotAllowed(String),
    
    #[error("Error interno: {0}")]
    InternalError(String),
}

impl From<AuthError> for DomainError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidCredentials => {
                DomainError::new(ErrorKind::AccessDenied, "Auth", "Credenciales inválidas")
            },
            AuthError::TokenExpired => {
                DomainError::new(ErrorKind::AccessDenied, "Auth", "Token expirado")
            },
            AuthError::InvalidToken(msg) => {
                DomainError::new(ErrorKind::AccessDenied, "Auth", format!("Token inválido: {}", msg))
            },
            AuthError::AccessDenied(msg) => {
                DomainError::new(ErrorKind::AccessDenied, "Auth", msg)
            },
            AuthError::OperationNotAllowed(msg) => {
                DomainError::new(ErrorKind::AccessDenied, "Auth", msg)
            },
            AuthError::InternalError(msg) => {
                DomainError::new(ErrorKind::InternalError, "Auth", msg)
            },
        }
    }
}

pub struct AuthService {
    jwt_secret: String,
    access_token_expiry: i64,  // segundos
    refresh_token_expiry: i64, // segundos
}

impl AuthService {
    pub fn new(jwt_secret: String, access_token_expiry_secs: i64, refresh_token_expiry_secs: i64) -> Self {
        Self {
            jwt_secret,
            access_token_expiry: access_token_expiry_secs,
            refresh_token_expiry: refresh_token_expiry_secs,
        }
    }
    
    pub fn generate_access_token(&self, user: &User) -> Result<String, AuthError> {
        let now = Utc::now().timestamp();
        
        // Log information for debugging
        tracing::debug!(
            "Generating token for user: {}, id: {}, role: {}", 
            user.username(), 
            user.id(), 
            user.role()
        );
        
        let claims = TokenClaims {
            sub: user.id().to_string(),
            exp: now + self.access_token_expiry,
            iat: now,
            jti: Uuid::new_v4().to_string(),
            username: user.username().to_string(),
            email: user.email().to_string(),
            role: format!("{}", user.role()),
        };
        
        // Log JWT claims for debugging
        tracing::debug!("JWT claims: sub={}, exp={}, iat={}", claims.sub, claims.exp, claims.iat);
        
        match encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes())
        ) {
            Ok(token) => {
                tracing::debug!("Token generated successfully, length: {}", token.len());
                Ok(token)
            },
            Err(e) => {
                tracing::error!("Error generating token: {}", e);
                Err(AuthError::InternalError(format!("Error al generar token: {}", e)))
            }
        }
    }
    
    pub fn generate_refresh_token(&self) -> String {
        Uuid::new_v4().to_string()
    }
    
    pub fn validate_token(&self, token: &str) -> Result<TokenClaims, AuthError> {
        let validation = Validation::new(Algorithm::HS256);
        
        let token_data = decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation
        )
        .map_err(|e| {
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::InvalidToken(format!("Error al validar token: {}", e)),
            }
        })?;
        
        Ok(token_data.claims)
    }
    
    // Duración del refresh token en segundos
    pub fn refresh_token_expiry_secs(&self) -> i64 {
        self.refresh_token_expiry
    }
    
    // Duración del refresh token en días (para la entidad Session)
    pub fn refresh_token_expiry_days(&self) -> i64 {
        self.refresh_token_expiry / (24 * 3600)
    }
}