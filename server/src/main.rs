//! biohack2-server entry point: open SQLite, migrate, seed catalog, serve.

use biohack2_server::{DEFAULT_DB_FILE, DEFAULT_USER_ID, run};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_file = std::env::var("BIOHACK2_DB").unwrap_or_else(|_| DEFAULT_DB_FILE.to_string());
    let port: u16 = std::env::var("BIOHACK2_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8082);
    let dist_dir = std::env::var("BIOHACK2_DIST").unwrap_or_else(|_| "dist".to_string());

    let url = format!("sqlite://{db_file}?mode=rwc");
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(8)
        .connect(&url)
        .await?;

    engine::db::migrate(&pool).await?;

    // Seed the catalog on every boot (name-idempotent — only inserts
    // substances missing from the database, so new catalog additions
    // land in existing databases without duplicating existing rows).
    let items = engine::catalog::seed_catalog();
    engine::db::seed_catalog(&pool, &items).await?;
    println!("Catalog ensured: {} seed substances available", items.len());

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("biohack2-server: http://{addr}");
    println!("  db:    {db_file}");
    println!("  dist:  {dist_dir}");
    println!("  user:  {DEFAULT_USER_ID}");

    run(pool, addr, Some(&dist_dir)).await
}
