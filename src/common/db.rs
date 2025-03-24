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
                                -- Create the auth schema if not exists
                                CREATE SCHEMA IF NOT EXISTS auth;
                                
                                -- Create UserRole enum type if not exists
                                DO $$ 
                                BEGIN
                                    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'userrole') THEN
                                        CREATE TYPE auth.userrole AS ENUM ('admin', 'user');
                                    END IF;
                                END $$;
                                
                                -- Create the auth.users table
                                CREATE TABLE IF NOT EXISTS auth.users (
                                    id VARCHAR(36) PRIMARY KEY,
                                    username VARCHAR(32) NOT NULL UNIQUE,
                                    email VARCHAR(255) NOT NULL UNIQUE,
                                    password_hash TEXT NOT NULL,
                                    role auth.userrole NOT NULL,
                                    storage_quota_bytes BIGINT NOT NULL,
                                    storage_used_bytes BIGINT NOT NULL DEFAULT 0,
                                    created_at TIMESTAMPTZ NOT NULL,
                                    updated_at TIMESTAMPTZ NOT NULL,
                                    last_login_at TIMESTAMPTZ,
                                    active BOOLEAN NOT NULL DEFAULT TRUE
                                );
                                
                                -- Create an index on username and email for fast lookups
                                CREATE INDEX IF NOT EXISTS idx_users_username ON auth.users(username);
                                CREATE INDEX IF NOT EXISTS idx_users_email ON auth.users(email);
                                
                                -- Create the sessions table
                                CREATE TABLE IF NOT EXISTS auth.sessions (
                                    id VARCHAR(36) PRIMARY KEY,
                                    user_id VARCHAR(36) NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
                                    refresh_token VARCHAR(255) NOT NULL UNIQUE,
                                    expires_at TIMESTAMPTZ NOT NULL,
                                    ip_address VARCHAR(45),
                                    user_agent TEXT,
                                    created_at TIMESTAMPTZ NOT NULL,
                                    revoked BOOLEAN NOT NULL DEFAULT FALSE
                                );
                                
                                -- Create indexes on user_id and refresh_token for fast lookups
                                CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON auth.sessions(user_id);
                                CREATE INDEX IF NOT EXISTS idx_sessions_refresh_token ON auth.sessions(refresh_token);
                                CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON auth.sessions(expires_at);
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