use sqlx::sqlite::SqlitePool;
use sqlx::migrate::Migrator;

static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn init_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let path = db_path.strip_prefix("sqlite:").unwrap_or(db_path);
    let absolute_path = std::env::current_dir()?.join(path);
    
    if let Some(parent) = absolute_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    
    if !absolute_path.exists() {
        std::fs::File::create(&absolute_path)?;
    }
    
    let pool = SqlitePool::connect(&format!("sqlite:{}", absolute_path.display())).await?;
    MIGRATOR.run(&pool).await?;
    Ok(pool)
}
