//! Database rollback tool
//!
//! Run with: cargo run --bin rollback
//!
//! This will show applied migrations and let you remove the last one.

use sqlx::postgres::PgPoolOptions;
use sqlx::Row;

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

    println!("Connected successfully!\n");

    // Get list of applied migrations
    let rows = sqlx::query("SELECT id, name, applied_at FROM _migrations ORDER BY id DESC")
        .fetch_all(&pool)
        .await?;

    if rows.is_empty() {
        println!("No migrations have been applied.");
        return Ok(());
    }

    println!("Applied migrations (newest first):");
    for row in &rows {
        let id: i32 = row.get("id");
        let name: String = row.get("name");
        let applied_at: chrono::DateTime<chrono::Utc> = row.get("applied_at");
        println!("  [{}] {} ({})", id, name, applied_at.format("%Y-%m-%d %H:%M:%S"));
    }

    let last: &sqlx::postgres::PgRow = &rows[0];
    let last_name: String = last.get("name");
    let last_id: i32 = last.get("id");

    println!("\nTo remove the last migration record ({}):", last_name);
    println!("  Run: cargo run --bin rollback -- --confirm {}", last_id);

    // Check for --confirm flag
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 && args[1] == "--confirm" {
        let id_to_remove: i32 = args[2].parse()?;

        if id_to_remove == last_id {
            sqlx::query("DELETE FROM _migrations WHERE id = $1")
                .bind(id_to_remove)
                .execute(&pool)
                .await?;

            println!("\nRemoved migration record: {}", last_name);
            println!("NOTE: This only removes the tracking record.");
            println!("      You may need to manually revert database changes.");
        } else {
            println!("\nError: Can only remove the most recent migration (id={}).", last_id);
        }
    }

    Ok(())
}
