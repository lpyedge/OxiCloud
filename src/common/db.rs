use sqlx::{postgres::PgPoolOptions, PgPool};
use anyhow::Result;
use std::time::Duration;
use crate::common::config::AppConfig;

pub async fn create_database_pool(config: &AppConfig) -> Result<PgPool> {
    tracing::info!("Inicializando conexión a PostgreSQL con URL: {}", 
                  config.database.connection_string.replace("postgres://", "postgres://[user]:[pass]@"));
    
    // Add a more robust connection attempt with retries
    let mut attempt = 0;
    const MAX_ATTEMPTS: usize = 3;
    
    while attempt < MAX_ATTEMPTS {
        attempt += 1;
        tracing::info!("Intento de conexión a PostgreSQL #{}", attempt);
        
        // Crear el pool de conexiones con las opciones de configuración
        match PgPoolOptions::new()
            .max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(Duration::from_secs(config.database.connect_timeout_secs))
            .idle_timeout(Duration::from_secs(config.database.idle_timeout_secs))
            .max_lifetime(Duration::from_secs(config.database.max_lifetime_secs))
            .connect(&config.database.connection_string)
            .await {
                Ok(pool) => {
                    // Verificar la conexión
                    match sqlx::query("SELECT 1").execute(&pool).await {
                        Ok(_) => {
                            tracing::info!("Conexión a PostgreSQL establecida correctamente");
                            return Ok(pool);
                        },
                        Err(e) => {
                            tracing::error!("Error al verificar conexión: {}", e);
                            // Try creating the tables in this case - might be missing schema
                            tracing::info!("Intentando crear las tablas necesarias...");
                            
                            // Simple schema creation - this handles fresh installations
                            let create_tables_result = sqlx::query(r#"
                                CREATE TABLE IF NOT EXISTS users (
                                    id TEXT PRIMARY KEY,
                                    username TEXT UNIQUE NOT NULL,
                                    email TEXT UNIQUE NOT NULL,
                                    password_hash TEXT NOT NULL,
                                    role TEXT NOT NULL,
                                    is_active BOOLEAN NOT NULL DEFAULT TRUE,
                                    quota_bytes BIGINT NOT NULL DEFAULT 1073741824,
                                    last_login TIMESTAMP,
                                    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
                                );
                                
                                CREATE TABLE IF NOT EXISTS sessions (
                                    id TEXT PRIMARY KEY,
                                    user_id TEXT NOT NULL REFERENCES users(id),
                                    refresh_token TEXT UNIQUE NOT NULL,
                                    ip_address TEXT,
                                    user_agent TEXT,
                                    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                                    expires_at TIMESTAMP NOT NULL,
                                    is_revoked BOOLEAN NOT NULL DEFAULT FALSE
                                );
                            "#).execute(&pool).await;
                            
                            match create_tables_result {
                                Ok(_) => {
                                    tracing::info!("Tablas creadas correctamente");
                                    return Ok(pool);
                                },
                                Err(table_err) => {
                                    tracing::error!("Error al crear tablas: {}", table_err);
                                    if attempt >= MAX_ATTEMPTS {
                                        return Err(anyhow::anyhow!("Error en la conexión a PostgreSQL: {}", table_err));
                                    }
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    tracing::error!("Error al conectar a PostgreSQL: {}", e);
                    if attempt >= MAX_ATTEMPTS {
                        return Err(anyhow::anyhow!("Error en la conexión a PostgreSQL: {}", e));
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
    }
    
    Err(anyhow::anyhow!("No se pudo establecer la conexión a PostgreSQL después de {} intentos", MAX_ATTEMPTS))
}