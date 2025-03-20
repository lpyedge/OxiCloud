use std::sync::Arc;
use crate::domain::entities::user::{User, UserRole};
use crate::domain::entities::session::Session;
use crate::domain::services::auth_service::AuthService;
use crate::application::ports::auth_ports::{UserStoragePort, SessionStoragePort};
use crate::application::dtos::user_dto::{UserDto, RegisterDto, LoginDto, AuthResponseDto, ChangePasswordDto, RefreshTokenDto};
use crate::common::errors::{DomainError, ErrorKind};

pub struct AuthApplicationService {
    user_storage: Arc<dyn UserStoragePort>,
    session_storage: Arc<dyn SessionStoragePort>,
    auth_service: Arc<AuthService>,
}

impl AuthApplicationService {
    pub fn new(
        user_storage: Arc<dyn UserStoragePort>,
        session_storage: Arc<dyn SessionStoragePort>,
        auth_service: Arc<AuthService>,
    ) -> Self {
        Self {
            user_storage,
            session_storage,
            auth_service,
        }
    }
    
    pub async fn register(&self, dto: RegisterDto) -> Result<UserDto, DomainError> {
        // Verificar usuario duplicado
        if self.user_storage.get_user_by_username(&dto.username).await.is_ok() {
            return Err(DomainError::new(
                ErrorKind::AlreadyExists,
                "User",
                format!("El usuario '{}' ya existe", dto.username)
            ));
        }
        
        if self.user_storage.get_user_by_email(&dto.email).await.is_ok() {
            return Err(DomainError::new(
                ErrorKind::AlreadyExists,
                "User",
                format!("El email '{}' ya está registrado", dto.email)
            ));
        }
        
        // Cuota predeterminada: 1GB (ajustable según plan)
        let default_quota = 1024 * 1024 * 1024; // 1GB
        
        // Crear usuario
        let user = User::new(
            dto.username,
            dto.email,
            dto.password,
            UserRole::User, // Por defecto: usuario normal
            default_quota,
        ).map_err(|e| DomainError::new(
            ErrorKind::InvalidInput,
            "User",
            format!("Error al crear usuario: {}", e)
        ))?;
        
        // Guardar usuario
        let created_user = self.user_storage.create_user(user).await?;
        
        tracing::info!("Usuario registrado: {}", created_user.id());
        Ok(UserDto::from(created_user))
    }
    
    pub async fn login(&self, dto: LoginDto) -> Result<AuthResponseDto, DomainError> {
        // Buscar usuario
        let mut user = self.user_storage
            .get_user_by_username(&dto.username)
            .await
            .map_err(|_| DomainError::new(
                ErrorKind::AccessDenied,
                "Auth",
                "Credenciales inválidas"
            ))?;
        
        // Verificar si usuario está activo
        if !user.is_active() {
            return Err(DomainError::new(
                ErrorKind::AccessDenied,
                "Auth",
                "Cuenta desactivada"
            ));
        }
        
        // Verificar contraseña
        let is_valid = user.verify_password(&dto.password)
            .map_err(|_| DomainError::new(
                ErrorKind::AccessDenied,
                "Auth",
                "Credenciales inválidas"
            ))?;
            
        if !is_valid {
            return Err(DomainError::new(
                ErrorKind::AccessDenied,
                "Auth",
                "Credenciales inválidas"
            ));
        }
        
        // Actualizar último login
        user.register_login();
        self.user_storage.update_user(user.clone()).await?;
        
        // Generar tokens
        let access_token = self.auth_service.generate_access_token(&user)
            .map_err(DomainError::from)?;
        
        let refresh_token = self.auth_service.generate_refresh_token();
        
        // Guardar sesión
        let session = Session::new(
            user.id().to_string(),
            refresh_token.clone(),
            None, // IP (se puede añadir desde la capa HTTP)
            None, // User-Agent (se puede añadir desde la capa HTTP)
            self.auth_service.refresh_token_expiry_days(),
        );
        
        self.session_storage.create_session(session).await?;
        
        // Respuesta de autenticación
        Ok(AuthResponseDto {
            user: UserDto::from(user),
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.auth_service.refresh_token_expiry_secs(),
        })
    }
    
    pub async fn refresh_token(&self, dto: RefreshTokenDto) -> Result<AuthResponseDto, DomainError> {
        // Obtener sesión válida
        let session = self.session_storage
            .get_session_by_refresh_token(&dto.refresh_token)
            .await?;
        
        // Verificar si la sesión está expirada o revocada
        if session.is_expired() || session.is_revoked() {
            return Err(DomainError::new(
                ErrorKind::AccessDenied,
                "Auth",
                "Sesión expirada o inválida"
            ));
        }
        
        // Obtener usuario
        let user = self.user_storage
            .get_user_by_id(session.user_id())
            .await?;
        
        // Verificar si usuario está activo
        if !user.is_active() {
            return Err(DomainError::new(
                ErrorKind::AccessDenied,
                "Auth",
                "Cuenta desactivada"
            ));
        }
        
        // Revocar sesión actual
        self.session_storage.revoke_session(session.id()).await?;
        
        // Generar nuevos tokens
        let access_token = self.auth_service.generate_access_token(&user)
            .map_err(DomainError::from)?;
        
        let new_refresh_token = self.auth_service.generate_refresh_token();
        
        // Crear nueva sesión
        let new_session = Session::new(
            user.id().to_string(),
            new_refresh_token.clone(),
            None,
            None,
            self.auth_service.refresh_token_expiry_days(),
        );
        
        self.session_storage.create_session(new_session).await?;
        
        Ok(AuthResponseDto {
            user: UserDto::from(user),
            access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.auth_service.refresh_token_expiry_secs(),
        })
    }
    
    pub async fn logout(&self, user_id: &str, refresh_token: &str) -> Result<(), DomainError> {
        // Obtener sesión
        let session = match self.session_storage.get_session_by_refresh_token(refresh_token).await {
            Ok(s) => s,
            // Si la sesión no existe, consideramos el logout como exitoso
            Err(_) => return Ok(()),
        };
        
        // Verificar que la sesión pertenece al usuario
        if session.user_id() != user_id {
            return Err(DomainError::new(
                ErrorKind::AccessDenied,
                "Auth",
                "La sesión no pertenece al usuario"
            ));
        }
        
        // Revocar sesión
        self.session_storage.revoke_session(session.id()).await?;
        
        Ok(())
    }
    
    pub async fn logout_all(&self, user_id: &str) -> Result<u64, DomainError> {
        // Revocar todas las sesiones del usuario
        let revoked_count = self.session_storage.revoke_all_user_sessions(user_id).await?;
        
        Ok(revoked_count)
    }
    
    pub async fn change_password(&self, user_id: &str, dto: ChangePasswordDto) -> Result<(), DomainError> {
        // Obtener usuario
        let mut user = self.user_storage.get_user_by_id(user_id).await?;
        
        // Verificar contraseña actual
        let is_valid = user.verify_password(&dto.current_password)
            .map_err(|_| DomainError::new(
                ErrorKind::AccessDenied,
                "Auth",
                "Contraseña actual incorrecta"
            ))?;
            
        if !is_valid {
            return Err(DomainError::new(
                ErrorKind::AccessDenied,
                "Auth",
                "Contraseña actual incorrecta"
            ));
        }
        
        // Actualizar contraseña
        user.update_password(dto.new_password.clone())
            .map_err(|e| DomainError::new(
                ErrorKind::InvalidInput,
                "User",
                format!("Error al cambiar contraseña: {}", e)
            ))?;
        
        // Guardar usuario actualizado
        self.user_storage.update_user(user).await?;
        
        // Opcional: revocar todas las sesiones para forzar re-login con nueva contraseña
        self.session_storage.revoke_all_user_sessions(user_id).await?;
        
        Ok(())
    }
    
    pub async fn get_user(&self, user_id: &str) -> Result<UserDto, DomainError> {
        let user = self.user_storage.get_user_by_id(user_id).await?;
        Ok(UserDto::from(user))
    }
    
    // Alias for consistency with handler method
    pub async fn get_user_by_id(&self, user_id: &str) -> Result<UserDto, DomainError> {
        self.get_user(user_id).await
    }
    
    pub async fn list_users(&self, limit: i64, offset: i64) -> Result<Vec<UserDto>, DomainError> {
        let users = self.user_storage.list_users(limit, offset).await?;
        Ok(users.into_iter().map(UserDto::from).collect())
    }
}