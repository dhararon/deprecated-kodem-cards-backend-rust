use sqlx::migrate;
use sqlx::postgres::PgPool;
use sqlx::migrate::Migrator;
use std::path::Path;

// static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn init_database(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPool::connect(database_url).await?;
    
    // Migraciones (sqlx 0.7)
    let migrator = Migrator::new(Path::new("./migrations")).await?;
    migrator.run(&pool).await?;
    
    Ok(pool)
} 