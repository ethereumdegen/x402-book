//! Database migration tool
//!
//! Run with: cargo run --bin migrate
//!
//! This will run all pending migrations from the migrations/ directory.

use sqlx::postgres::PgPoolOptions;
use sqlx::{Row, PgPool};
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in environment");

    println!("Connecting to database...");

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;

    println!("Connected successfully!");

    // Create migrations tracking table if it doesn't exist
    create_migrations_table(&pool).await?;

    // Get list of already applied migrations
    let applied = get_applied_migrations(&pool).await?;
    println!("Applied migrations: {:?}", applied);

    // Find migration files
    let migrations_dir = find_migrations_dir()?;
    println!("Migrations directory: {}", migrations_dir.display());

    let mut migration_files: Vec<_> = fs::read_dir(&migrations_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension().map(|ext| ext == "sql").unwrap_or(false)
        })
        .collect();

    // Sort by filename
    migration_files.sort_by_key(|entry| entry.file_name());

    let mut applied_count = 0;

    for entry in migration_files {
        let filename = entry.file_name().to_string_lossy().to_string();

        if applied.contains(&filename) {
            println!("  [SKIP] {} (already applied)", filename);
            continue;
        }

        println!("  [RUN]  {}", filename);

        let sql = fs::read_to_string(entry.path())?;

        // Run the migration
        sqlx::raw_sql(&sql).execute(&pool).await?;

        // Record the migration
        record_migration(&pool, &filename).await?;

        println!("  [DONE] {}", filename);
        applied_count += 1;
    }

    if applied_count == 0 {
        println!("\nNo new migrations to apply.");
    } else {
        println!("\nApplied {} migration(s).", applied_count);
    }

    Ok(())
}

async fn create_migrations_table(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::raw_sql(
        r#"
        CREATE TABLE IF NOT EXISTS _migrations (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) UNIQUE NOT NULL,
            applied_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn get_applied_migrations(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query("SELECT name FROM _migrations ORDER BY id")
        .fetch_all(pool)
        .await?;

    Ok(rows.iter().map(|row| row.get("name")).collect())
}

async fn record_migration(pool: &PgPool, name: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO _migrations (name) VALUES ($1)")
        .bind(name)
        .execute(pool)
        .await?;
    Ok(())
}

fn find_migrations_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    // Check multiple possible locations
    let candidates = [
        "./migrations",
        "../migrations",
        "./forum-backend/migrations",
    ];

    for candidate in candidates {
        let path = Path::new(candidate);
        if path.exists() && path.is_dir() {
            return Ok(path.to_path_buf());
        }
    }

    Err("Could not find migrations directory".into())
}
