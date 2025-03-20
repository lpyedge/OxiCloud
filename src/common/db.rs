use sqlx::{postgres::PgPoolOptions, PgPool};
use anyhow::Result;
use std::time::Duration;
use crate::common::config::AppConfig;

pub async fn create_database_pool(config: &AppConfig) -> Result<PgPool> {
    tracing::info!("Inicializando conexión a PostgreSQL...");
    
    // Crear el pool de conexiones con las opciones de configuración
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .acquire_timeout(Duration::from_secs(config.database.connect_timeout_secs))
        .idle_timeout(Duration::from_secs(config.database.idle_timeout_secs))
        .max_lifetime(Duration::from_secs(config.database.max_lifetime_secs))
        .connect(&config.database.connection_string)
        .await?;
    
    // Verificar la conexión
    sqlx::query("SELECT 1").execute(&pool).await?;
    
    tracing::info!("Conexión a PostgreSQL establecida correctamente");
    Ok(pool)
}