use std::fmt::{Display, Formatter, Result as FmtResult};
use std::error::Error as StdError;
use thiserror::Error;

/// Tipos de errores comunes en toda la aplicación
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// Entidad no encontrada
    NotFound,
    /// Entidad ya existe
    AlreadyExists,
    /// Entrada inválida o validación fallida
    InvalidInput,
    /// Error de acceso o permisos
    AccessDenied,
    /// Tiempo de espera agotado
    Timeout,
    /// Error interno del sistema
    InternalError,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ErrorKind::NotFound => write!(f, "Not Found"),
            ErrorKind::AlreadyExists => write!(f, "Already Exists"),
            ErrorKind::InvalidInput => write!(f, "Invalid Input"),
            ErrorKind::AccessDenied => write!(f, "Access Denied"),
            ErrorKind::Timeout => write!(f, "Timeout"),
            ErrorKind::InternalError => write!(f, "Internal Error"),
        }
    }
}

/// Error base de dominio que proporciona contexto detallado
#[derive(Error, Debug)]
#[error("{kind}: {message}")]
pub struct DomainError {
    /// Tipo de error
    pub kind: ErrorKind,
    /// Tipo de entidad afectada (ej: "File", "Folder")
    pub entity_type: &'static str,
    /// Identificador de la entidad si está disponible
    pub entity_id: Option<String>,
    /// Mensaje descriptivo del error
    pub message: String,
    /// Error fuente (opcional)
    #[source]
    pub source: Option<Box<dyn StdError + Send + Sync>>,
}

impl DomainError {
    /// Crea un nuevo error de dominio
    pub fn new<S: Into<String>>(
        kind: ErrorKind,
        entity_type: &'static str,
        message: S,
    ) -> Self {
        Self {
            kind,
            entity_type,
            entity_id: None,
            message: message.into(),
            source: None,
        }
    }

    /// Crea un error de entidad no encontrada
    pub fn not_found<S: Into<String>>(entity_type: &'static str, entity_id: S) -> Self {
        let id = entity_id.into();
        Self {
            kind: ErrorKind::NotFound,
            entity_type,
            entity_id: Some(id.clone()),
            message: format!("{} not found: {}", entity_type, id),
            source: None,
        }
    }

    /// Crea un error de entidad ya existente
    pub fn already_exists<S: Into<String>>(entity_type: &'static str, entity_id: S) -> Self {
        let id = entity_id.into();
        Self {
            kind: ErrorKind::AlreadyExists,
            entity_type,
            entity_id: Some(id.clone()),
            message: format!("{} already exists: {}", entity_type, id),
            source: None,
        }
    }

    /// Crea un error de tiempo agotado
    pub fn timeout<S: Into<String>>(entity_type: &'static str, message: S) -> Self {
        Self {
            kind: ErrorKind::Timeout,
            entity_type,
            entity_id: None,
            message: message.into(),
            source: None,
        }
    }
    
    /// Crea un error interno
    pub fn internal_error<S: Into<String>>(entity_type: &'static str, message: S) -> Self {
        Self {
            kind: ErrorKind::InternalError,
            entity_type,
            entity_id: None,
            message: message.into(),
            source: None,
        }
    }
    
    /// Crea un error de acceso denegado
    pub fn access_denied<S: Into<String>>(entity_type: &'static str, message: S) -> Self {
        Self {
            kind: ErrorKind::AccessDenied,
            entity_type,
            entity_id: None,
            message: message.into(),
            source: None,
        }
    }
    
    /// Crea un error de validación
    pub fn validation_error<S: Into<String>>(entity_type: &'static str, message: S) -> Self {
        Self {
            kind: ErrorKind::InvalidInput,
            entity_type,
            entity_id: None,
            message: message.into(),
            source: None,
        }
    }

    /// Establece el ID de la entidad
    #[allow(dead_code)]
    pub fn with_id<S: Into<String>>(mut self, entity_id: S) -> Self {
        self.entity_id = Some(entity_id.into());
        self
    }

    /// Establece el error fuente
    pub fn with_source<E: StdError + Send + Sync + 'static>(mut self, source: E) -> Self {
        self.source = Some(Box::new(source));
        self
    }
}

/// Trait para añadir contexto a los errores
pub trait ErrorContext<T, E> {
    fn with_context<C, F>(self, context: F) -> Result<T, DomainError>
    where
        C: Into<String>,
        F: FnOnce() -> C;

    #[allow(dead_code)]
    fn with_error_kind(self, kind: ErrorKind, entity_type: &'static str) -> Result<T, DomainError>;
}

impl<T, E: StdError + Send + Sync + 'static> ErrorContext<T, E> for Result<T, E> {
    fn with_context<C, F>(self, context: F) -> Result<T, DomainError>
    where
        C: Into<String>,
        F: FnOnce() -> C,
    {
        self.map_err(|e| {
            DomainError {
                kind: ErrorKind::InternalError,
                entity_type: "Unknown",
                entity_id: None,
                message: context().into(),
                source: Some(Box::new(e)),
            }
        })
    }

    fn with_error_kind(self, kind: ErrorKind, entity_type: &'static str) -> Result<T, DomainError> {
        self.map_err(|e| {
            DomainError {
                kind,
                entity_type,
                entity_id: None,
                message: format!("{}", e),
                source: Some(Box::new(e)),
            }
        })
    }
}

/// Macro para convertir errores específicos a DomainError
#[macro_export]
macro_rules! impl_from_error {
    ($error_type:ty, $entity_type:expr) => {
        impl From<$error_type> for DomainError {
            fn from(err: $error_type) -> Self {
                DomainError {
                    kind: ErrorKind::InternalError,
                    entity_type: $entity_type,
                    entity_id: None,
                    message: format!("{}", err),
                    source: Some(Box::new(err)),
                }
            }
        }
    };
}

// Implementación para errores estándar comunes
impl_from_error!(std::io::Error, "IO");
impl_from_error!(serde_json::Error, "Serialization");