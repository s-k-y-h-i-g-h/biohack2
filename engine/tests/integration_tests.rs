//! Integration tests for the engine.

use engine::{catalog::seed_catalog, db::{migrate, DbPool}, models::{LogEntryFilter, VitalsEntryFilter}};
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
    engine::db::seed_catalog(&pool, &catalog).await?;
    
    // Query it back
    let items = engine::db::get_catalog_items(&pool).await?;
    assert_eq!(items.len(), 27);
    
    // Verify a known item
    let vit_d3 = items.iter().find(|i| i.id.to_string() == "vit-d3").expect("vit-d3 should exist");
    assert_eq!(vit_d3.name, "Vitamin D3");
    assert_eq!(vit_d3.dosage_range.as_ref().unwrap().unit, "IU");
    
    Ok(())
}

#[sqlx::test]
async fn test_create_and_query_log_entry(pool: DbPool) -> anyhow::Result<()> {
    migrate(&pool).await?;
    
    let entry = LogEntry {
        id: Uuid::new_v4(),
        user_id: "test-user".to_string(),
        item_type: engine::models::ItemType::Supplement,
        item_id: Some(Uuid::parse_str("vit-d3").unwrap()),
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
    
    engine::db::create_log_entry(&pool, &entry).await?;
    
    let entries = engine::db::get_log_entries(&pool, &LogEntryFilter {
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
        id: Uuid::new_v4(),
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
    
    let entries = engine::db::get_vitals_entries(&pool, &VitalsEntryFilter {
        user_id: Some("test-user".to_string()),
        ..Default::default()
    }).await?;
    
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].heart_rate, Some(72));
    assert_eq!(entries[0].bp_systolic, Some(120));
    
    Ok(())
}
