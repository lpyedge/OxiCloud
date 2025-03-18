use std::path::PathBuf;
use std::sync::Arc;
use std::io::{self, SeekFrom};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::task;
use tokio::sync::{Semaphore, Mutex};
use futures::future::join_all;
use tracing::{info, debug, error};
use bytes::{Bytes, BytesMut};

use crate::common::config::AppConfig;
use crate::domain::repositories::file_repository::FileRepositoryError;
use crate::infrastructure::services::buffer_pool::BufferPool;

/// Estructura para el rango de bytes a procesar
#[derive(Debug, Clone, Copy)]
pub struct ChunkRange {
    /// Índice del chunk
    pub index: usize,
    /// Posición de inicio en bytes
    pub start: u64,
    /// Tamaño del chunk en bytes
    pub size: usize,
}

/// Buffer pooling específico para BytesMut
pub struct BytesBufferPool {
    buffers: Mutex<Vec<BytesMut>>,
    buffer_size: usize,
    max_buffers: usize,
}

impl BytesBufferPool {
    pub fn new(buffer_size: usize, max_buffers: usize) -> Self {
        Self {
            buffers: Mutex::new(Vec::with_capacity(max_buffers)),
            buffer_size,
            max_buffers,
        }
    }
    
    /// Obtener un buffer del pool o crear uno nuevo
    pub async fn get_buffer(&self) -> BytesMut {
        let mut buffers = self.buffers.lock().await;
        
        if let Some(mut buffer) = buffers.pop() {
            // Reutilizar buffer existente
            buffer.clear(); // Mantener capacidad, limpiar contenido
            buffer
        } else {
            // Crear nuevo buffer si el pool está vacío
            BytesMut::with_capacity(self.buffer_size)
        }
    }
    
    /// Devolver un buffer al pool para reutilización
    pub async fn return_buffer(&self, mut buffer: BytesMut) {
        // Restablece el buffer para reutilización
        buffer.clear();
        
        let mut buffers = self.buffers.lock().await;
        
        // Solo mantener hasta max_buffers
        if buffers.len() < self.max_buffers {
            buffers.push(buffer);
        }
        // Si ya tenemos suficientes buffers, este se descartará
    }
}

/// Procesador paralelo de archivos para operaciones IO intensivas
pub struct ParallelFileProcessor {
    /// Configuración de la aplicación
    config: AppConfig,
    /// Semáforo para limitar concurrencia global
    concurrency_limiter: Arc<Semaphore>,
    /// Pool de buffers para optimizar memoria
    buffer_pool: Option<Arc<BufferPool>>,
    /// Pool de buffers BytesMut para operaciones zero-copy
    bytes_pool: Arc<BytesBufferPool>,
}

impl ParallelFileProcessor {
    /// Crea una nueva instancia del procesador
    pub fn new(config: AppConfig) -> Self {
        let concurrency_limiter = Arc::new(Semaphore::new(config.concurrency.max_concurrent_io));
        
        // Crear pool de BytesMut para operaciones eficientes
        let chunk_size = config.resources.chunk_size_bytes;
        let max_chunks = config.concurrency.max_parallel_chunks;
        let bytes_pool = Arc::new(BytesBufferPool::new(chunk_size, max_chunks * 2));
        
        Self {
            config,
            concurrency_limiter,
            buffer_pool: None,
            bytes_pool,
        }
    }
    
    /// Crea una nueva instancia del procesador con un pool de buffers
    pub fn new_with_buffer_pool(config: AppConfig, buffer_pool: Arc<BufferPool>) -> Self {
        let concurrency_limiter = Arc::new(Semaphore::new(config.concurrency.max_concurrent_io));
        
        // Crear pool de BytesMut para operaciones eficientes
        let chunk_size = config.resources.chunk_size_bytes;
        let max_chunks = config.concurrency.max_parallel_chunks;
        let bytes_pool = Arc::new(BytesBufferPool::new(chunk_size, max_chunks * 2));
        
        Self {
            config,
            concurrency_limiter,
            buffer_pool: Some(buffer_pool),
            bytes_pool,
        }
    }
    
    /// Divide un archivo en chunks para procesamiento paralelo
    pub fn calculate_chunks(&self, file_size: u64) -> Vec<ChunkRange> {
        // Determinar si el archivo necesita procesamiento paralelo
        let needs_parallel = self.config.resources.needs_parallel_processing(
            file_size, &self.config.concurrency
        );
        
        if !needs_parallel {
            // Para archivos pequeños, usar un solo chunk
            return vec![ChunkRange { 
                index: 0,
                start: 0,
                size: file_size as usize 
            }];
        }
        
        // Calcular número óptimo de chunks
        let chunk_count = self.config.resources.calculate_optimal_chunks(
            file_size, &self.config.concurrency
        );
        
        // Calcular tamaño de cada chunk
        let chunk_size = self.config.resources.calculate_chunk_size(file_size, chunk_count);
        
        // Crear los rangos de chunks
        let mut chunks = Vec::with_capacity(chunk_count);
        
        let mut start = 0;
        for i in 0..chunk_count {
            let current_chunk_size = if i == chunk_count - 1 {
                // Último chunk puede ser más pequeño
                (file_size - start) as usize
            } else {
                chunk_size
            };
            
            chunks.push(ChunkRange {
                index: i,
                start,
                size: current_chunk_size,
            });
            
            start += current_chunk_size as u64;
        }
        
        debug!("File size: {} bytes, divided into {} chunks of ~{} bytes each", 
              file_size, chunks.len(), chunk_size);
        
        chunks
    }
    
    /// Lee un archivo en paralelo y devuelve el contenido completo
    /// Implementación optimizada usando BytesMut para reducir copias de memoria
    pub async fn read_file_parallel(&self, file_path: &PathBuf) -> Result<Vec<u8>, FileRepositoryError> {
        // Obtener tamaño del archivo
        let metadata = tokio::fs::metadata(file_path).await
            .map_err(FileRepositoryError::IoError)?;
        
        let file_size = metadata.len();
        
        // Verificar si el archivo es demasiado grande para memoria
        if !self.config.resources.can_load_in_memory(file_size) {
            return Err(FileRepositoryError::Other(
                format!("File too large to load in memory: {} MB (max: {} MB)", 
                       file_size / (1024 * 1024), 
                       self.config.resources.max_in_memory_file_size_mb)
            ));
        }
        
        // Calcular chunks
        let chunks = self.calculate_chunks(file_size);
        
        if chunks.len() == 1 {
            // Para un solo chunk, usar lectura simple con buffer pool si está disponible
            info!("Reading file with size {}MB as a single chunk", file_size / (1024 * 1024));
            
            if let Some(pool) = &self.buffer_pool {
                // Usar buffer del pool para lectura eficiente
                debug!("Using buffer pool for single chunk read");
                let mut buffer = pool.get_buffer().await;
                
                // Si el buffer es demasiado pequeño, revertir a la implementación estándar
                if buffer.capacity() < file_size as usize {
                    debug!("Buffer from pool too small ({}), using standard read", buffer.capacity());
                    let content = tokio::fs::read(file_path).await
                        .map_err(FileRepositoryError::IoError)?;
                    
                    return Ok(content);
                }
                
                // Usar el buffer de memoria del pool
                let mut file = File::open(file_path).await
                    .map_err(FileRepositoryError::IoError)?;
                
                let read_size = file.read(buffer.as_mut_slice()).await
                    .map_err(FileRepositoryError::IoError)?;
                
                buffer.set_used(read_size);
                
                // Convertir en Vec<u8>
                let content = buffer.into_vec();
                return Ok(content);
            } else {
                // Implementación estándar sin pool
                let content = tokio::fs::read(file_path).await
                    .map_err(FileRepositoryError::IoError)?;
                
                return Ok(content);
            }
        }
        
        // Para múltiples chunks, usar lectura paralela
        info!("Reading file with size {}MB in {} parallel chunks using BytesMut", 
             file_size / (1024 * 1024), chunks.len());
        
        // Crear buffer de resultado final (pre-allocated)
        let mut result = BytesMut::with_capacity(file_size as usize);
        result.resize(file_size as usize, 0);
        let result_mutex = Arc::new(Mutex::new(result));
        
        // Crear tareas para cada chunk
        let mut tasks = Vec::with_capacity(chunks.len());
        
        // Abrir archivo una sola vez y compartirlo
        let file = Arc::new(File::open(file_path).await
            .map_err(FileRepositoryError::IoError)?);
        
        // Referencia al pool de BytesMut
        let bytes_pool = self.bytes_pool.clone();
        
        // Procesar chunks en paralelo
        for chunk in chunks {
            let file_clone = file.clone();
            let result_clone = result_mutex.clone();
            let semaphore_clone = self.concurrency_limiter.clone();
            let bytes_pool_clone = bytes_pool.clone();
            
            // Spawn task para este chunk - no hay necesidad de copiar los datos originales
            let task = task::spawn(async move {
                // Adquirir permiso del semáforo
                let _permit = semaphore_clone.acquire().await.unwrap();
                
                // Obtener un buffer reusable del pool de BytesMut
                let mut chunk_buffer = bytes_pool_clone.get_buffer().await;
                
                // Asegurar que tenga suficiente capacidad
                if chunk_buffer.capacity() < chunk.size {
                    chunk_buffer = BytesMut::with_capacity(chunk.size);
                }
                // Resize al tamaño exacto necesario
                chunk_buffer.resize(chunk.size, 0);
                
                // Crear un descriptor de archivo duplicado para uso independiente
                let mut file_handle = file_clone.try_clone().await?;
                
                // Posicionar y leer directamente en el BytesMut
                file_handle.seek(SeekFrom::Start(chunk.start)).await?;
                let bytes_read = file_handle.read_exact(&mut chunk_buffer[..chunk.size]).await?;
                
                if bytes_read != chunk.size {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        format!("Expected to read {} bytes but got {}", chunk.size, bytes_read)
                    ));
                }
                
                // Escribir en resultado final
                let mut result_lock = result_clone.lock().await;
                let start_pos = chunk.start as usize;
                let end_pos = start_pos + chunk.size;
                
                // Usar copy_from_slice para copiar desde BytesMut al buffer de resultado
                result_lock[start_pos..end_pos].copy_from_slice(&chunk_buffer[..chunk.size]);
                
                // Devolver el buffer al pool para su reutilización
                bytes_pool_clone.return_buffer(chunk_buffer).await;
                
                // Registrar progreso
                debug!("Chunk {} processed: {} bytes from offset {}", 
                      chunk.index, chunk.size, chunk.start);
                
                Ok::<_, io::Error>(())
            });
            
            tasks.push(task);
        }
        
        // Esperar a que todas las tareas terminen
        let results = join_all(tasks).await;
        
        // Verificar errores
        for (i, task_result) in results.into_iter().enumerate() {
            match task_result {
                Ok(Ok(())) => {},
                Ok(Err(e)) => {
                    error!("Error in chunk {}: {}", i, e);
                    return Err(FileRepositoryError::IoError(e));
                },
                Err(e) => {
                    error!("Task error in chunk {}: {}", i, e);
                    return Err(FileRepositoryError::Other(format!("Task error: {}", e)));
                }
            }
        }
        
        // Obtener el resultado final y convertir a Vec<u8>
        let result_buffer = result_mutex.lock().await;
        let result_vec = result_buffer.to_vec();
        
        info!("Successfully read file of {}MB in parallel with optimized BytesMut", file_size / (1024 * 1024));
        Ok(result_vec)
    }
    
    /// Escribe un archivo en paralelo desde un buffer
    /// Implementación optimizada usando BytesMut/Bytes para reducir copias de memoria
    pub async fn write_file_parallel(
        &self, 
        file_path: &PathBuf, 
        content: &[u8]
    ) -> Result<(), FileRepositoryError> {
        let file_size = content.len() as u64;
        
        // Calcular chunks
        let chunks = self.calculate_chunks(file_size);
        
        if chunks.len() == 1 {
            // Para un solo chunk, usar escritura simple
            info!("Writing file with size {}MB as a single chunk", file_size / (1024 * 1024));
            
            // Implementación estándar (el buffer pooling no ofrece ventajas para escritura simple)
            tokio::fs::write(file_path, content).await
                .map_err(FileRepositoryError::IoError)?;
            
            return Ok(());
        }
        
        // Para múltiples chunks, usar escritura paralela
        info!("Writing file with size {}MB in {} parallel chunks using Bytes", 
             file_size / (1024 * 1024), chunks.len());
        
        // Crear archivo (no usamos Mutex para reducir contención)
        let file = File::create(file_path).await
            .map_err(FileRepositoryError::IoError)?;
        
        // Convertir contenido a Bytes (un solo paso de copia)
        let content_bytes = Bytes::copy_from_slice(content);
        
        // Crear tareas para cada chunk
        let mut tasks = Vec::with_capacity(chunks.len());
        
        // Procesar chunks en paralelo
        for chunk in chunks {
            let file_clone = file.try_clone().await
                .map_err(FileRepositoryError::IoError)?;
            let semaphore_clone = self.concurrency_limiter.clone();
            
            // Crear slice de Bytes (no copia datos, solo referencia)
            let start_idx = chunk.start as usize;
            let end_idx = start_idx + chunk.size;
            let chunk_data = content_bytes.slice(start_idx..end_idx);
            
            // Crear y lanzar tarea
            let task = task::spawn(async move {
                // Adquirir permiso del semáforo
                let _permit = semaphore_clone.acquire().await.unwrap();
                
                // Posicionar y escribir
                let mut file_handle = file_clone;
                file_handle.seek(SeekFrom::Start(chunk.start)).await?;
                file_handle.write_all(&chunk_data).await?;
                
                // Registrar progreso
                debug!("Chunk {} written: {} bytes at offset {}", 
                      chunk.index, chunk.size, chunk.start);
                
                Ok::<_, io::Error>(())
            });
            
            tasks.push(task);
        }
        
        // Esperar a que todas las tareas terminen
        let results = join_all(tasks).await;
        
        // Verificar errores
        for (i, task_result) in results.into_iter().enumerate() {
            match task_result {
                Ok(Ok(())) => {},
                Ok(Err(e)) => {
                    error!("Error in chunk {}: {}", i, e);
                    return Err(FileRepositoryError::IoError(e));
                },
                Err(e) => {
                    error!("Task error in chunk {}: {}", i, e);
                    return Err(FileRepositoryError::Other(format!("Task error: {}", e)));
                }
            }
        }
        
        // Garantizar que todo se ha escrito correctamente
        let mut file_handle = file;
        file_handle.flush().await.map_err(FileRepositoryError::IoError)?;
        
        info!("Successfully wrote file of {}MB in parallel with optimized Bytes", file_size / (1024 * 1024));
        Ok(())
    }
    
    /// Escribe un chunk en un archivo en una posición específica
    #[allow(dead_code)]
    async fn write_chunk_optimized(
        file: &mut File, 
        offset: u64, 
        data: Bytes
    ) -> Result<(), std::io::Error> {
        // Preparar la escritura en la posición correcta
        file.seek(SeekFrom::Start(offset)).await?;
        
        // Escribir datos sin copias adicionales
        file.write_all(&data).await?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_parallel_read_write() {
        // Crear configuración con umbral bajo para testing
        let mut config = AppConfig::default();
        config.concurrency.min_size_for_parallel_chunks_mb = 1; // 1MB para testing
        config.concurrency.max_parallel_chunks = 4;
        
        let processor = ParallelFileProcessor::new(config);
        
        // Crear directorio temporal
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.bin");
        
        // Crear datos de prueba (2MB)
        let size = 2 * 1024 * 1024;
        let mut test_data = Vec::with_capacity(size);
        for i in 0..size {
            test_data.push((i % 256) as u8);
        }
        
        // Escribir archivo en paralelo
        processor.write_file_parallel(&file_path, &test_data).await.unwrap();
        
        // Leer archivo en paralelo
        let read_data = processor.read_file_parallel(&file_path).await.unwrap();
        
        // Verificar que los datos son idénticos
        assert_eq!(test_data.len(), read_data.len());
        assert_eq!(test_data, read_data);
    }
    
    #[tokio::test]
    async fn test_bytesmut_pool() {
        // Crear pool
        let pool = BytesBufferPool::new(1024, 5);
        
        // Obtener buffer
        let mut buffer1 = pool.get_buffer().await;
        buffer1.put_slice(b"test data");
        assert_eq!(&buffer1[..9], b"test data");
        
        // Devolver buffer al pool
        pool.return_buffer(buffer1).await;
        
        // Obtener otro buffer (debería ser el mismo)
        let buffer2 = pool.get_buffer().await;
        assert_eq!(buffer2.capacity(), 1024);
        
        // El buffer debería estar vacío (clear)
        assert_eq!(buffer2.len(), 0);
    }
    
    #[test]
    fn test_chunk_calculation() {
        // Crear configuración de prueba
        let mut config = AppConfig::default();
        config.concurrency.min_size_for_parallel_chunks_mb = 100; // 100MB
        config.concurrency.max_parallel_chunks = 4;
        config.concurrency.parallel_chunk_size_bytes = 50 * 1024 * 1024; // 50MB
        
        let processor = ParallelFileProcessor::new(config);
        
        // Archivo pequeño (10MB)
        let small_file_size = 10 * 1024 * 1024;
        let chunks = processor.calculate_chunks(small_file_size);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].size as u64, small_file_size);
        
        // Archivo grande (300MB)
        let large_file_size = 300 * 1024 * 1024;
        let chunks = processor.calculate_chunks(large_file_size);
        assert_eq!(chunks.len(), 4); // Limitado a max_parallel_chunks
        
        // Verificar que todos los chunks suman el tamaño total
        let total_size: u64 = chunks.iter().map(|c| c.size as u64).sum();
        assert_eq!(total_size, large_file_size);
    }
}