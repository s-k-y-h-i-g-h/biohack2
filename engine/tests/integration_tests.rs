use engine::{catalog::seed_catalog, db::{migrate, DbPool, seed_catalog as db_seed_catalog}, models::LogEntry};
use sqlx::sqlite::{Sqlite, SqlitePoolOptions};
use uuid::Uuid;

async fn test_pool() -> DbPool {
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap()
}

#[sqlx::test]
async fn test_seed_and_query_catalog(pool: DbPool) -> anyhow::Result<()> {
    migrate(&pool).await?;
    let catalog = seed_catalog();
    assert_eq!(catalog.len(), 27);
    
    // Seed the catalog
    let seeded = db_seed_catalog(&pool, &catalog).await;
    println!("Seed result: {:?}", seeded);
    
    // Check raw count
    let raw_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catalog_items")
        .fetch_one(&pool)
        .await?;
    println!("Raw count in DB: {}", raw_count);
    
    // Check what's actually in the table
    let raw_items: Vec<String> = sqlx::query_scalar("SELECT name FROM catalog_items")
        .fetch_all(&pool)
        .await?;
    println!("Raw items: {:?}", raw_items);
    
    // Check the exact SQL being used
    let raw_sql: Vec<String> = sqlx::query_scalar("SELECT sql FROM sqlite_master WHERE type='table' AND name='catalog_items'")
        .fetch_all(&pool)
        .await?;
    println!("Table schema: {:?}", raw_sql);
    
    // Query it back
    let items = engine::db::get_catalog_items(&pool).await?;
    println!("Items found: {}", items.len());
    for item in &items {
        println!("  - {}", item.name);
    }
    assert_eq!(items.len(), 27);

    // Verify a known item
    let vit_d3 = items.iter().find(|i| i.name == "Vitamin D3").expect("vit-d3 should exist");
    assert_eq!(vit_d3.name, "Vitamin D3");
    assert_eq!(vit_d3.dosage_range.as_ref().unwrap().unit, "IU");
    
    Ok(())
}

#[sqlx::test]
async fn test_create_and_query_log_entry(pool: DbPool) -> anyhow::Result<()> {
    migrate(&pool).await?;
    
    // First seed the catalog
    let catalog = seed_catalog();
    engine::db::seed_catalog(&pool, &catalog).await?;
    
    let item = catalog.iter().find(|i| i.name == "Vitamin D3").unwrap();
    println!("Item ID: {}", item.id);
    println!("Item name: {}", item.name);
    println!("Item category: {:?}", item.category);
    
    let entry = LogEntry {
        id: uuid::Uuid::new_v4(),
        user_id: "test-user".to_string(),
        item_type: engine::models::ItemType::Supplement,
        item_id: Some(item.id),
        name: "Vitamin D3".to_string(),
        quantity: Some(5000.0),
        unit: Some("IU".to_string()),
        route: None,
        timestamp: chrono::Utc::now(),
        stack_id: None,
        notes: None,
        acknowledged_interaction: false,
        custom_fields: None,
    };
    
    println!("Entry item_type: {:?}", entry.item_type);
    
    engine::db::create_log_entry(&pool, &entry).await?;
    
    let entries = engine::db::get_log_entries(&pool, &engine::models::LogEntryFilter {
        user_id: Some("test-user".to_string()),
        ..Default::default()
    }).await?;
    
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "Vitamin D3");
    assert_eq!(entries[0].quantity, Some(5000.0));
    
    Ok(())
}

#[sqlx::test]
async fn test_create_and_query_vitals(pool: DbPool) -> anyhow::Result<()> {
    migrate(&pool).await?;
    
    let entry = engine::models::VitalsEntry {
        id: uuid::Uuid::new_v4(),
        user_id: "test-user".to_string(),
        timestamp: chrono::Utc::now(),
        bp_systolic: Some(120),
        bp_diastolic: Some(80),
        heart_rate: Some(72),
        weight: None,
        blood_glucose: None,
        temperature: None,
        spo2: None,
        hrv: None,
        sleep_quality: None,
        custom_metrics: None,
        notes: None,
    };
    
    engine::db::create_vitals_entry(&pool, &entry).await?;
    
    let entries = engine::db::get_vitals_entries(&pool, &engine::models::VitalsEntryFilter {
        user_id: Some("test-user".to_string()),
        ..Default::default()
    }).await?;
    
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].heart_rate, Some(72));
    assert_eq!(entries[0].bp_systolic, Some(120));
    
    Ok(())
}