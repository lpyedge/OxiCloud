use std::time::Duration;

/// Configuración de caché
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// TTL para entradas de archivos en caché (ms)
    pub file_ttl_ms: u64,
    /// TTL para entradas de directorios en caché (ms)
    pub directory_ttl_ms: u64,
    /// Máximo número de entradas en caché
    pub max_entries: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            file_ttl_ms: 60_000,     // 1 minuto
            directory_ttl_ms: 120_000, // 2 minutos
            max_entries: 10_000,      // 10,000 entradas
        }
    }
}

/// Configuración de timeouts para diferentes operaciones
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// Timeout para operaciones de archivo (ms)
    pub file_operation_ms: u64,
    /// Timeout para operaciones de directorio (ms)
    pub dir_operation_ms: u64,
    /// Timeout para adquisición de locks (ms)
    pub lock_acquisition_ms: u64,
    /// Timeout para operaciones de red (ms)
    #[allow(dead_code)]
    pub network_operation_ms: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            file_operation_ms: 10000,    // 10 segundos
            dir_operation_ms: 30000,     // 30 segundos
            lock_acquisition_ms: 5000,   // 5 segundos
            network_operation_ms: 15000, // 15 segundos
        }
    }
}

impl TimeoutConfig {
    /// Obtiene un Duration para operaciones de archivo
    pub fn file_timeout(&self) -> Duration {
        Duration::from_millis(self.file_operation_ms)
    }

    /// Obtiene un Duration para operaciones de directorio
    pub fn dir_timeout(&self) -> Duration {
        Duration::from_millis(self.dir_operation_ms)
    }

    /// Obtiene un Duration para adquisición de locks
    pub fn lock_timeout(&self) -> Duration {
        Duration::from_millis(self.lock_acquisition_ms)
    }

    /// Obtiene un Duration para operaciones de red
    #[allow(dead_code)]
    pub fn network_timeout(&self) -> Duration {
        Duration::from_millis(self.network_operation_ms)
    }
}

/// Configuración para manejo de recursos grandes
#[derive(Debug, Clone)]
pub struct ResourceConfig {
    /// Umbral en MB para considerar un archivo como grande
    pub large_file_threshold_mb: u64,
    /// Umbral de entradas para considerar un directorio como grande
    #[allow(dead_code)]
    pub large_dir_threshold_entries: usize,
    /// Tamaño de chunk para procesamiento de archivos grandes (bytes)
    pub chunk_size_bytes: usize,
    /// Límite de tamaño de archivo para cargar en memoria (MB)
    pub max_in_memory_file_size_mb: u64,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            large_file_threshold_mb: 100,       // 100 MB
            large_dir_threshold_entries: 1000,  // 1000 entradas
            chunk_size_bytes: 1024 * 1024,      // 1 MB
            max_in_memory_file_size_mb: 50,     // 50 MB
        }
    }
}

impl ResourceConfig {
    /// Convierte un tamaño en bytes a MB
    pub fn bytes_to_mb(&self, bytes: u64) -> u64 {
        bytes / (1024 * 1024)
    }

    /// Determina si un archivo es considerado grande
    pub fn is_large_file(&self, size_bytes: u64) -> bool {
        self.bytes_to_mb(size_bytes) >= self.large_file_threshold_mb
    }
    
    /// Determina si un archivo es suficientemente grande para procesamiento paralelo
    pub fn needs_parallel_processing(&self, size_bytes: u64, config: &ConcurrencyConfig) -> bool {
        self.bytes_to_mb(size_bytes) >= config.min_size_for_parallel_chunks_mb
    }

    /// Determina si un archivo puede cargarse completo en memoria
    pub fn can_load_in_memory(&self, size_bytes: u64) -> bool {
        self.bytes_to_mb(size_bytes) <= self.max_in_memory_file_size_mb
    }

    /// Determina si un directorio es considerado grande
    #[allow(dead_code)]
    pub fn is_large_directory(&self, entry_count: usize) -> bool {
        entry_count >= self.large_dir_threshold_entries
    }
    
    /// Calcula el número de chunks para procesamiento paralelo
    pub fn calculate_optimal_chunks(&self, size_bytes: u64, config: &ConcurrencyConfig) -> usize {
        // Si el archivo no es suficientemente grande, retornar 1
        if !self.needs_parallel_processing(size_bytes, config) {
            return 1;
        }
        
        // Calcular el número de chunks basado en el tamaño
        let chunk_count = (size_bytes as usize + config.parallel_chunk_size_bytes - 1) 
                         / config.parallel_chunk_size_bytes;
                         
        // Limitar al máximo de chunks en paralelo
        chunk_count.min(config.max_parallel_chunks)
    }
    
    /// Calcula el tamaño óptimo de cada chunk para procesamiento paralelo
    pub fn calculate_chunk_size(&self, file_size: u64, chunk_count: usize) -> usize {
        if chunk_count <= 1 {
            return file_size as usize;
        }
        
        // Distribuir equitativamente el tamaño entre los chunks
        ((file_size as usize) + chunk_count - 1) / chunk_count
    }
}

/// Configuración para operaciones concurrentes
#[derive(Debug, Clone)]
pub struct ConcurrencyConfig {
    /// Máximo de tareas de archivo concurrentes
    pub max_concurrent_files: usize,
    /// Máximo de tareas de directorio concurrentes
    #[allow(dead_code)]
    pub max_concurrent_dirs: usize,
    /// Máximo de operaciones de IO concurrentes
    pub max_concurrent_io: usize,
    /// Máximo de chunks para procesar en paralelo por archivo
    pub max_parallel_chunks: usize,
    /// Tamaño mínimo de archivo (MB) para aplicar procesamiento paralelo de chunks
    pub min_size_for_parallel_chunks_mb: u64,
    /// Tamaño de chunk para procesamiento paralelo (bytes)
    pub parallel_chunk_size_bytes: usize,
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        Self {
            max_concurrent_files: 10,
            max_concurrent_dirs: 5,
            max_concurrent_io: 20,
            max_parallel_chunks: 8,
            min_size_for_parallel_chunks_mb: 200, // 200 MB
            parallel_chunk_size_bytes: 8 * 1024 * 1024, // 8 MB
        }
    }
}

/// Configuración global de la aplicación
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Configuración de caché
    pub cache: CacheConfig,
    /// Configuración de timeouts
    pub timeouts: TimeoutConfig,
    /// Configuración de recursos
    pub resources: ResourceConfig,
    /// Configuración de concurrencia
    pub concurrency: ConcurrencyConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            cache: CacheConfig::default(),
            timeouts: TimeoutConfig::default(),
            resources: ResourceConfig::default(),
            concurrency: ConcurrencyConfig::default(),
        }
    }
}

/// Obtenemos una configuración global por defecto
#[allow(dead_code)]
pub fn default_config() -> AppConfig {
    AppConfig::default()
}