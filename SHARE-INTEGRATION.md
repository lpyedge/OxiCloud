# Documentación Técnica: Sistema de Compartición en OxiCloud

## Resumen Ejecutivo

La funcionalidad de compartición de archivos y carpetas en OxiCloud permite a los usuarios generar enlaces de acceso para compartir sus recursos con otros usuarios, incluso aquellos sin cuenta en el sistema. La implementación sigue los principios de Arquitectura Hexagonal, manteniendo una clara separación entre dominio, aplicación e infraestructura.

## Arquitectura y Componentes

### 1. Entidades de Dominio

**Share (src/domain/entities/share.rs)**

La entidad principal que representa un recurso compartido:

```rust
pub struct Share {
    pub id: String,                         // Identificador único del enlace
    pub item_id: String,                    // ID del archivo o carpeta compartido
    pub item_type: ShareItemType,           // Tipo (File o Folder)
    pub token: String,                      // Token único para acceso público
    pub password_hash: Option<String>,      // Hash de contraseña opcional
    pub expires_at: Option<u64>,            // Timestamp de expiración opcional
    pub permissions: SharePermissions,      // Permisos otorgados
    pub created_at: u64,                    // Timestamp de creación
    pub created_by: String,                 // ID del usuario creador
    pub access_count: u64,                  // Contador de accesos
}

pub enum ShareItemType {
    File,
    Folder
}

pub struct SharePermissions {
    pub read: bool,     // Permiso de lectura
    pub write: bool,    // Permiso de escritura
    pub reshare: bool,  // Permiso para volver a compartir
}
```

La entidad implementa métodos para:
- Validar la expiración del enlace
- Verificar contraseñas
- Incrementar el contador de accesos
- Modificar propiedades (permisos, contraseña, expiración)

### 2. Interfaces del Repositorio

**ShareRepository (src/domain/repositories/share_repository.rs)**

Define las operaciones de persistencia para los enlaces compartidos:

```rust
#[async_trait]
pub trait ShareRepository: Send + Sync + 'static {
    async fn save(&self, share: &Share) -> Result<Share, ShareRepositoryError>;
    async fn find_by_id(&self, id: &str) -> Result<Share, ShareRepositoryError>;
    async fn find_by_token(&self, token: &str) -> Result<Share, ShareRepositoryError>;
    async fn find_by_item(&self, item_id: &str, item_type: &ShareItemType) -> Result<Vec<Share>, ShareRepositoryError>;
    async fn update(&self, share: &Share) -> Result<Share, ShareRepositoryError>;
    async fn delete(&self, id: &str) -> Result<(), ShareRepositoryError>;
    async fn find_by_user(&self, user_id: &str, offset: usize, limit: usize) -> Result<(Vec<Share>, usize), ShareRepositoryError>;
}
```

### 3. Puertos de Aplicación

**ShareUseCase y ShareStoragePort (src/application/ports/share_ports.rs)**

Define las interfaces para la capa de aplicación:

```rust
#[async_trait]
pub trait ShareUseCase: Send + Sync + 'static {
    // Crear un nuevo enlace compartido
    async fn create_shared_link(&self, user_id: &str, dto: CreateShareDto) -> Result<ShareDto, DomainError>;
    
    // Obtener un enlace compartido por ID
    async fn get_shared_link(&self, id: &str) -> Result<ShareDto, DomainError>;
    
    // Obtener un enlace compartido por token
    async fn get_shared_link_by_token(&self, token: &str) -> Result<ShareDto, DomainError>;
    
    // Obtener todos los enlaces compartidos para un elemento
    async fn get_shared_links_for_item(&self, item_id: &str, item_type: &ShareItemType) -> Result<Vec<ShareDto>, DomainError>;
    
    // Actualizar un enlace compartido
    async fn update_shared_link(&self, id: &str, dto: UpdateShareDto) -> Result<ShareDto, DomainError>;
    
    // Eliminar un enlace compartido
    async fn delete_shared_link(&self, id: &str) -> Result<(), DomainError>;
    
    // Obtener enlaces compartidos de un usuario con paginación
    async fn get_user_shared_links(&self, user_id: &str, page: usize, per_page: usize) -> Result<PaginatedResponseDto<ShareDto>, DomainError>;
    
    // Verificar la contraseña de un enlace protegido
    async fn verify_shared_link_password(&self, token: &str, password: &str) -> Result<bool, DomainError>;
    
    // Registrar un acceso a un enlace compartido
    async fn register_shared_link_access(&self, token: &str) -> Result<(), DomainError>;
}

#[async_trait]
pub trait ShareStoragePort: Send + Sync + 'static {
    // Métodos para interactuar con el almacenamiento
    async fn save_share(&self, share: &Share) -> Result<Share, DomainError>;
    async fn find_share_by_id(&self, id: &str) -> Result<Share, DomainError>;
    // ... otros métodos
}
```

### 4. Objetos de Transferencia de Datos (DTOs)

**DTOs (src/application/dtos/share_dto.rs)**

```rust
// DTO para la creación de enlaces compartidos
pub struct CreateShareDto {
    pub item_id: String,
    pub item_type: String,
    pub password: Option<String>,
    pub expires_at: Option<u64>,
    pub permissions: Option<SharePermissionsDto>,
}

// DTO para actualizar enlaces compartidos
pub struct UpdateShareDto {
    pub password: Option<String>,
    pub expires_at: Option<u64>,
    pub permissions: Option<SharePermissionsDto>,
}

// DTO de permisos
pub struct SharePermissionsDto {
    pub read: bool,
    pub write: bool,
    pub reshare: bool,
}

// DTO para respuestas
pub struct ShareDto {
    pub id: String,
    pub item_id: String,
    pub item_type: String,
    pub token: String,
    pub url: String,
    pub password_protected: bool,
    pub expires_at: Option<u64>,
    pub permissions: SharePermissionsDto,
    pub created_at: u64,
    pub created_by: String,
    pub access_count: u64,
}
```

### 5. Servicios de Aplicación

**ShareService (src/application/services/share_service.rs)**

Implementa la lógica de negocio para la compartición de archivos:

```rust
pub struct ShareService {
    config: Arc<AppConfig>,
    share_repository: Arc<dyn ShareStoragePort>,
    file_repository: Arc<dyn FileStoragePort>,
    folder_repository: Arc<dyn FolderStoragePort>,
}
```

El servicio implementa:
- Validación de elementos compartidos
- Gestión de permisos
- Generación de enlaces y tokens únicos
- Protección con contraseña
- Control de expiración
- Seguimiento de accesos

### 6. Implementación de Infraestructura

**ShareFsRepository (src/infrastructure/repositories/share_fs_repository.rs)**

Implementa la persistencia de enlaces compartidos usando el sistema de archivos:

```rust
pub struct ShareFsRepository {
    config: Arc<AppConfig>,
}

// Almacena los enlaces en un archivo JSON
struct ShareRecord {
    id: String,
    item_id: String,
    item_type: String,
    token: String,
    password_hash: Option<String>,
    expires_at: Option<u64>,
    permissions_read: bool,
    permissions_write: bool,
    permissions_reshare: bool,
    created_at: u64,
    created_by: String,
    access_count: u64,
}
```

La implementación:
- Guarda los enlaces compartidos en un archivo JSON
- Gestiona consultas y actualizaciones
- Proporciona búsqueda por ID, token o usuario
- Implementa paginación

### 7. Controladores API y Rutas

**Manejadores (src/interfaces/api/handlers/share_handler.rs)**

```rust
// Crear un nuevo enlace compartido
pub async fn create_shared_link(
    State(share_use_case): State<Arc<dyn ShareUseCase>>,
    Json(dto): Json<CreateShareDto>,
) -> impl IntoResponse {
    // Implementación...
}

// Obtener un enlace compartido
pub async fn get_shared_link(
    State(share_use_case): State<Arc<dyn ShareUseCase>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Implementación...
}

// Obtener enlaces compartidos de un usuario
pub async fn get_user_shares(
    State(share_use_case): State<Arc<dyn ShareUseCase>>,
    Query(query): Query<GetSharesQuery>,
) -> impl IntoResponse {
    // Implementación...
}

// Actualizar un enlace compartido
pub async fn update_shared_link(
    State(share_use_case): State<Arc<dyn ShareUseCase>>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateShareDto>,
) -> impl IntoResponse {
    // Implementación...
}

// Eliminar un enlace compartido
pub async fn delete_shared_link(
    State(share_use_case): State<Arc<dyn ShareUseCase>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Implementación...
}

// Acceder a un elemento compartido a través de su token
pub async fn access_shared_item(
    State(share_use_case): State<Arc<dyn ShareUseCase>>,
    Path(token): Path<String>,
) -> impl IntoResponse {
    // Implementación...
}

// Verificar la contraseña de un elemento compartido protegido
pub async fn verify_shared_item_password(
    State(share_use_case): State<Arc<dyn ShareUseCase>>,
    Path(token): Path<String>,
    Json(req): Json<VerifyPasswordRequest>,
) -> impl IntoResponse {
    // Implementación...
}
```

**Rutas (src/interfaces/api/routes.rs)**

```rust
// Rutas privadas para la gestión de enlaces compartidos
let share_router = Router::new()
    .route("/", post(share_handler::create_shared_link))
    .route("/", get(share_handler::get_user_shares))
    .route("/{id}", get(share_handler::get_shared_link))
    .route("/{id}", put(share_handler::update_shared_link))
    .route("/{id}", delete(share_handler::delete_shared_link));

// Rutas públicas para acceder a los enlaces compartidos
let public_share_router = Router::new()
    .route("/{token}", get(share_handler::access_shared_item))
    .route("/{token}/verify", post(share_handler::verify_shared_item_password));

// Configuración en el router principal
router
    .nest("/shares", share_router)       // API privada: /api/shares/...
    .nest("/s", public_share_router);    // API pública: /api/s/...
```

### 8. Integración en el Sistema

La funcionalidad de compartición está integrada con:

1. **Configuración del sistema**: Se puede habilitar/deshabilitar mediante la configuración:
```rust
pub struct FeaturesConfig {
    // ...
    pub enable_file_sharing: bool,
    // ...
}
```

2. **Inyección de dependencias**: El servicio se instancia en main.rs y se inyecta en las rutas:
```rust
// Inicializar el repositorio y servicio de compartición
let share_service: Option<Arc<dyn ShareUseCase>> = if config.features.enable_file_sharing {
    let share_repository = Arc::new(ShareFsRepository::new(Arc::new(config.clone())));
    let share_service = Arc::new(ShareService::new(
        Arc::new(config.clone()),
        share_repository,
        file_repository.clone(),
        folder_repository.clone()
    ));
    Some(share_service)
} else {
    None
};

// Agregar a los servicios de aplicación
let application_services = ApplicationServices {
    // ...
    share_service: share_service.clone(),
};

// Configurar las rutas
let api_routes = create_api_routes(
    folder_service, 
    file_service,
    Some(i18n_service),
    trash_service,
    search_service,
    share_service
);
```

## Flujos de Trabajo

### 1. Creación de un Enlace Compartido

1. El usuario selecciona un archivo o carpeta para compartir
2. El frontend envía una petición POST a `/api/shares/` con los detalles (contraseña opcional, expiración, permisos)
3. `ShareService.create_shared_link()` valida los datos y verifica que el elemento existe
4. Se genera un token único y una URL de acceso
5. El enlace se guarda en el repositorio
6. Se devuelve la URL y detalles del enlace compartido

### 2. Acceso a un Recurso Compartido

1. El usuario recibe y accede a un enlace compartido (ej: `http://oxicloud.example/api/s/{token}`)
2. El backend verifica:
   - Que el token es válido
   - Que el enlace no ha expirado
   - Si está protegido por contraseña
3. Si requiere contraseña, se solicita al usuario
4. El contador de accesos se incrementa
5. Se devuelven los metadatos del recurso compartido para mostrar en la interfaz
6. El usuario puede acceder al contenido según los permisos otorgados

## Seguridad

### Protección por Contraseña

- Las contraseñas se almacenan como hashes en lugar de texto plano
- El sistema utiliza un hash simple por ahora, pero está diseñado para implementar algoritmos más seguros como bcrypt

### Control de Expiración

- Los enlaces pueden configurarse para expirar automáticamente
- El sistema verifica la expiración antes de permitir accesos

### Control de Permisos

- El sistema implementa un modelo de permisos granular (lectura, escritura, recompartir)
- Cada operación valida los permisos antes de permitir la acción

## Manejo de Errores

El sistema implementa manejo de errores consistente:

```rust
pub enum ShareServiceError {
    #[error("Share not found: {0}")]
    NotFound(String),
    
    #[error("Item not found: {0}")]
    ItemNotFound(String),
    
    #[error("Access denied: {0}")]
    AccessDenied(String),
    
    #[error("Invalid password: {0}")]
    InvalidPassword(String),
    
    #[error("Share expired")]
    Expired,
    
    #[error("Repository error: {0}")]
    Repository(String),
    
    #[error("Invalid item type: {0}")]
    InvalidItemType(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
}
```

Estos errores se mapean a códigos HTTP apropiados en los controladores:
- `NotFound` → HTTP 404 Not Found
- `PasswordRequired` → HTTP 401 Unauthorized + metadata
- `Expired` → HTTP 410 Gone
- `AccessDenied` → HTTP 403 Forbidden
- `ValidationError` → HTTP 400 Bad Request

## Extensibilidad y Futuras Mejoras

La arquitectura está diseñada para permitir futuras mejoras:

1. **Notificaciones**: Integración con un sistema de notificaciones para alertar a los usuarios cuando se accede a sus recursos compartidos.

2. **Registro de Actividad**: Implementación de un registro detallado de actividades para auditar quién accedió a qué recursos y cuándo.

3. **Límites de Uso**: Establecer límites de uso (número máximo de accesos, ancho de banda) para enlaces compartidos.

4. **Estadísticas Avanzadas**: Proporcionar métricas detalladas sobre el uso de recursos compartidos.

5. **Persistencia Alternativa**: La arquitectura permite implementar fácilmente alternativas de almacenamiento (base de datos, servicios en la nube) manteniendo la misma interfaz.

## Estado Actual

La funcionalidad de compartición está completamente implementada en el backend y lista para integrarse con el frontend. La característica está habilitada por defecto en la configuración actual.

## Consideraciones Técnicas

- **Rendimiento**: El sistema utiliza un enfoque de almacenamiento basado en archivos JSON, lo que es adecuado para un volumen moderado de enlaces compartidos. Para una carga mayor, se recomienda migrar a una base de datos.

- **Escalabilidad**: El diseño permite escalar horizontalmente la funcionalidad implementando repositorios distribuidos o basados en la nube.

- **Mantenimiento**: La clara separación de responsabilidades facilita el mantenimiento y las pruebas de la funcionalidad.

## Conclusión

La implementación del sistema de compartición en OxiCloud sigue los principios de la Arquitectura Hexagonal, permitiendo una clara separación entre el dominio, la aplicación y la infraestructura. Esto facilita la evolución del sistema y la adaptación a requisitos cambiantes. La funcionalidad proporciona todas las características básicas esperadas de un sistema de compartición moderno, incluyendo protección por contraseña, expiración y permisos granulares.