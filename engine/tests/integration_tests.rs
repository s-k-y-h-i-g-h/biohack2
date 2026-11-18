use engine::{
    catalog::seed_catalog,
    db::{DbPool, migrate, seed_catalog as db_seed_catalog},
    models::LogEntry,
};
use uuid::Uuid;

#[sqlx::test]
async fn test_seed_and_query_catalog(pool: DbPool) -> anyhow::Result<()> {
    migrate(&pool).await?;
    let catalog = seed_catalog();
    assert_eq!(catalog.len(), 37);

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
    let raw_sql: Vec<String> = sqlx::query_scalar(
        "SELECT sql FROM sqlite_master WHERE type='table' AND name='catalog_items'",
    )
    .fetch_all(&pool)
    .await?;
    println!("Table schema: {:?}", raw_sql);

    // Query it back
    let items = engine::db::get_catalog_items(&pool).await?;
    println!("Items found: {}", items.len());
    for item in &items {
        println!("  - {}", item.name);
    }
    assert_eq!(items.len(), 37);

    // Verify a known item
    let vit_d3 = items
        .iter()
        .find(|i| i.name == "Vitamin D3")
        .expect("vit-d3 should exist");
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

    let entries = engine::db::get_log_entries(
        &pool,
        &engine::models::LogEntryFilter {
            user_id: Some("test-user".to_string()),
            ..Default::default()
        },
    )
    .await?;

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

    let entries = engine::db::get_vitals_entries(
        &pool,
        &engine::models::VitalsEntryFilter {
            user_id: Some("test-user".to_string()),
            ..Default::default()
        },
    )
    .await?;

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].heart_rate, Some(72));
    assert_eq!(entries[0].bp_systolic, Some(120));

    Ok(())
}

// ── spec-003 Phase 3 ─────────────────────────────────────────────────────────

// T228/T226: a pre-existing database with the OLD alerts CHECK constraint
// must survive migrate() and accept the new alert types afterwards.
#[sqlx::test]
async fn legacy_alerts_table_migrated_for_new_types(pool: DbPool) -> anyhow::Result<()> {
    // Recreate the pre-spec-003 schema by hand (no migrate yet).
    sqlx::query(
        "CREATE TABLE alerts (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            type TEXT NOT NULL CHECK(type IN ('vital','interaction','warning')),
            severity TEXT NOT NULL CHECK(severity IN ('info','warning','critical')),
            message TEXT NOT NULL,
            recommendation TEXT,
            is_acknowledged INTEGER NOT NULL DEFAULT 0,
            linked_entry_id TEXT,
            generated_at TEXT NOT NULL,
            resolved_at TEXT
        )",
    )
    .execute(&pool)
    .await?;
    let legacy = engine::models::Alert {
        id: Uuid::new_v4(),
        user_id: "u1".into(),
        alert_type: engine::models::AlertType::Warning,
        severity: engine::models::AlertSeverity::Info,
        message: "kept across migration".into(),
        recommendation: None,
        is_acknowledged: false,
        linked_entry_id: None,
        generated_at: chrono::Utc::now(),
        resolved_at: None,
    };
    engine::db::create_alert(&pool, &legacy).await?;

    migrate(&pool).await?; // rebuilds alerts, preserves rows

    let fresh = engine::models::Alert {
        id: Uuid::new_v4(),
        alert_type: engine::models::AlertType::Redose,
        ..legacy.clone()
    };
    engine::db::create_alert(&pool, &fresh).await?; // would hit the old CHECK
    let all = engine::db::get_alerts(&pool, &Default::default()).await?;
    assert_eq!(all.len(), 2);
    assert!(
        all.iter()
            .any(|a| a.alert_type == engine::models::AlertType::Redose)
    );

    // T232: resolve without acknowledging.
    engine::db::resolve_alert(&pool, &fresh.id, chrono::Utc::now()).await?;
    let all = engine::db::get_alerts(&pool, &Default::default()).await?;
    let r = all.iter().find(|a| a.id == fresh.id).unwrap();
    assert!(r.resolved_at.is_some() && !r.is_acknowledged);
    Ok(())
}

// T226: target band upsert / list / delete roundtrip.
#[sqlx::test]
async fn target_band_crud(pool: DbPool) -> anyhow::Result<()> {
    migrate(&pool).await?;
    let band = engine::models::TargetBand {
        substance: "Magnesium".into(),
        low: 40.0,
        high: 90.0,
        window_hours: 24.0,
        updated_at: chrono::Utc::now(),
    };
    engine::db::upsert_target_band(&pool, "u1", &band).await?;
    let mut raised = band.clone();
    raised.low = 55.0;
    engine::db::upsert_target_band(&pool, "u1", &raised).await?; // update path
    engine::db::upsert_target_band(&pool, "u2", &band).await?; // other user

    let u1 = engine::db::list_target_bands(&pool, "u1").await?;
    assert_eq!(u1.len(), 1);
    assert_eq!(u1[0].low, 55.0);
    assert_eq!(engine::db::list_target_bands(&pool, "u2").await?.len(), 1);

    // Validation rejects nonsense before it reaches SQL.
    let bad = engine::models::TargetBand {
        low: 100.0,
        high: 50.0,
        ..band.clone()
    };
    assert!(
        engine::db::upsert_target_band(&pool, "u1", &bad)
            .await
            .is_err()
    );

    assert!(engine::db::delete_target_band(&pool, "u1", "Magnesium").await?);
    assert!(!engine::db::delete_target_band(&pool, "u1", "Magnesium").await?);
    Ok(())
}

#[sqlx::test]
async fn test_delete_all_user_data_wipes_user_and_keeps_others(pool: DbPool) -> anyhow::Result<()> {
    use engine::models::LogEntryFilter;
    use engine::models::{ItemType, Note};

    migrate(&pool).await?;
    db_seed_catalog(&pool, &seed_catalog()).await?;

    for user in ["alice", "bob"] {
        let entry = LogEntry {
            id: Uuid::new_v4(),
            user_id: user.to_string(),
            item_type: ItemType::Supplement,
            item_id: None,
            name: "Test Substance".to_string(),
            quantity: Some(100.0),
            unit: Some("mg".to_string()),
            route: None,
            timestamp: chrono::Utc::now(),
            stack_id: None,
            notes: None,
            acknowledged_interaction: false,
            custom_fields: None,
        };
        engine::db::create_log_entry(&pool, &entry).await?;
        let note = Note {
            id: Uuid::new_v4(),
            user_id: user.to_string(),
            content: "a private note".to_string(),
            timestamp: chrono::Utc::now(),
            linked_entry_id: None,
        };
        engine::db::create_note(&pool, &note).await?;
    }

    let deleted = engine::db::delete_all_user_data(&pool, "alice").await?;
    assert_eq!(deleted, 2, "both alice rows (log + note) must be counted");

    let alice_logs = engine::db::get_log_entries(
        &pool,
        &LogEntryFilter {
            user_id: Some("alice".into()),
            ..Default::default()
        },
    )
    .await?;
    let alice_notes = engine::db::get_notes(&pool, "alice").await?;
    assert!(alice_logs.is_empty() && alice_notes.is_empty());

    let bob_logs = engine::db::get_log_entries(
        &pool,
        &LogEntryFilter {
            user_id: Some("bob".into()),
            ..Default::default()
        },
    )
    .await?;
    let bob_notes = engine::db::get_notes(&pool, "bob").await?;
    assert_eq!(bob_logs.len(), 1, "bob's log must survive");
    assert_eq!(bob_notes.len(), 1, "bob's note must survive");

    // Shared seed content survives: wipe-and-migrate restarts keep a catalog.
    let catalog = engine::db::get_catalog_items(&pool).await?;
    assert_eq!(catalog.len(), 37);

    // Idempotent: a second wipe deletes nothing and reports 0.
    assert_eq!(engine::db::delete_all_user_data(&pool, "alice").await?, 0);
    Ok(())
}
