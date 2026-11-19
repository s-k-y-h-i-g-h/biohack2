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

#[sqlx::test]
async fn test_tombstones_record_on_delete_and_clear_on_wipe(pool: DbPool) -> anyhow::Result<()> {
    use engine::models::Note;

    migrate(&pool).await?;
    db_seed_catalog(&pool, &seed_catalog()).await?;

    let note = Note {
        id: Uuid::new_v4(),
        user_id: "alice".into(),
        content: "doomed".into(),
        timestamp: chrono::Utc::now(),
        linked_entry_id: None,
    };
    engine::db::create_note(&pool, &note).await?;
    assert!(!engine::db::is_tombstoned(&pool, &note.id).await?);

    engine::db::delete_note(&pool, &note.id).await?;
    assert!(
        engine::db::is_tombstoned(&pool, &note.id).await?,
        "id-delete must leave a tombstone"
    );
    assert!(
        engine::db::list_tombstones(&pool)
            .await?
            .contains(&note.id.to_string())
    );

    // Wipe is a clean slate: tombstones are user data too.
    let deleted = engine::db::delete_all_user_data(&pool, "alice").await?;
    assert_eq!(deleted, 1, "wipe removes the tombstone row");
    assert!(
        !engine::db::is_tombstoned(&pool, &note.id).await?,
        "wipe clears tombstones"
    );
    assert!(engine::db::list_tombstones(&pool).await?.is_empty());
    Ok(())
}

#[sqlx::test]
async fn test_expired_tombstones_are_pruned(pool: DbPool) -> anyhow::Result<()> {
    migrate(&pool).await?;

    let fresh = Uuid::new_v4();
    let stale = Uuid::new_v4();
    engine::db::record_tombstone(&pool, &fresh).await?;
    let ancient = (chrono::Utc::now()
        - chrono::Duration::days(engine::db::TOMBSTONE_RETENTION_DAYS + 1))
    .to_rfc3339();
    sqlx::query("INSERT INTO deleted_tombstones (entry_id, deleted_at) VALUES (?1, ?2)")
        .bind(stale.to_string())
        .bind(&ancient)
        .execute(&pool)
        .await?;

    let live = engine::db::list_tombstones(&pool).await?;
    assert!(
        live.contains(&fresh.to_string()),
        "fresh tombstone survives"
    );
    assert!(
        !live.contains(&stale.to_string()),
        "tombstone older than retention is pruned"
    );
    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM deleted_tombstones")
        .fetch_one(&pool)
        .await?;
    assert_eq!(remaining, 1, "pruning physically deletes expired rows");
    Ok(())
}

#[test]
fn test_every_catalog_substance_has_an_assigned_model_class() {
    let cat: Vec<String> = engine::catalog::seed_catalog()
        .iter()
        .map(|c| c.name.clone())
        .collect();
    let missing: Vec<&String> = cat
        .iter()
        .filter(|n| engine::pk_assignments::assignment_for_name(n).is_none())
        .collect();
    println!("COVERAGE catalog={} missing={:?}", cat.len(), missing);
    assert!(missing.is_empty());
}

// ── spec-004 Phase 1: wearable ingest ────────────────────────────────────────

use engine::ingest::{RawReading, normalise_timestamp};

fn reading(record: &str, metric: &str, def: &str, value: f64, unit: &str, at: &str) -> RawReading {
    RawReading {
        source_id: "watch-1".to_string(),
        source_record_id: record.to_string(),
        metric: metric.to_string(),
        metric_definition: def.to_string(),
        value,
        unit: unit.to_string(),
        observed_at: at.parse::<chrono::DateTime<chrono::Utc>>().unwrap(),
        source_timezone: None,
        ingested_at: at.parse::<chrono::DateTime<chrono::Utc>>().unwrap(),
        payload: Some(format!("{{\"id\":\"{record}\"}}")),
    }
}

#[sqlx::test]
async fn test_ingest_idempotent_and_grouped(pool: DbPool) -> anyhow::Result<()> {
    migrate(&pool).await?;
    let batch = vec![
        reading(
            "r1",
            "heart_rate",
            "instantaneous",
            62.0,
            "bpm",
            "2026-11-18T07:00:00Z",
        ),
        reading(
            "r2",
            "bp_systolic",
            "korotkoff_1",
            118.0,
            "mmHg",
            "2026-11-18T07:00:00Z",
        ),
        reading(
            "r3",
            "bp_diastolic",
            "korotkoff_5",
            76.0,
            "mmHg",
            "2026-11-18T07:00:00Z",
        ),
    ];
    let rep = engine::db::ingest_readings(&pool, "u1", &batch).await?;
    assert_eq!(rep.ingested, 3);
    assert_eq!(rep.vitals_rows, 1, "same moment must merge into one row");

    // Re-ingesting the identical batch is a full no-op (FR-052).
    let rep2 = engine::db::ingest_readings(&pool, "u1", &batch).await?;
    assert_eq!(rep2.ingested, 0);
    assert_eq!(rep2.duplicates_skipped, 3);
    assert_eq!(rep2.vitals_rows, 0);

    // One new metric for the same moment updates the existing row (COALESCE).
    let late = vec![reading(
        "r4",
        "spo2",
        "pulse_oximetry",
        98.0,
        "%",
        "2026-11-18T07:00:00Z",
    )];
    let rep3 = engine::db::ingest_readings(&pool, "u1", &late).await?;
    assert_eq!(rep3.vitals_rows, 1);

    let rows = engine::db::get_vitals_entries(&pool, &engine::models::VitalsEntryFilter::default())
        .await?;
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.heart_rate, Some(62));
    assert_eq!(row.bp_systolic, Some(118));
    assert_eq!(row.bp_diastolic, Some(76));
    assert_eq!(row.spo2, Some(98), "late reading must land on the same row");
    Ok(())
}

#[sqlx::test]
async fn test_ingest_definition_gating_and_custom(pool: DbPool) -> anyhow::Result<()> {
    migrate(&pool).await?;
    let batch = vec![
        reading("h1", "hrv", "rmssd", 42.0, "ms", "2026-11-18T06:00:00Z"),
        reading("h2", "hrv", "sdnn", 60.0, "ms", "2026-11-18T06:00:00Z"),
        reading(
            "w1",
            "weight",
            "body_weight",
            150.0,
            "lb",
            "2026-11-18T06:00:00Z",
        ),
    ];
    let rep = engine::db::ingest_readings(&pool, "u1", &batch).await?;
    assert_eq!(rep.custom_metrics, 1, "sdnn is custom, rmssd native");

    let rows = engine::db::get_vitals_entries(&pool, &engine::models::VitalsEntryFilter::default())
        .await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].hrv, Some(42.0));
    assert!(
        (rows[0].weight.unwrap() - 68.0388555).abs() < 1e-6,
        "lb normalised to kg"
    );
    let custom = rows[0].custom_metrics.clone().unwrap();
    assert_eq!(custom["hrv:sdnn"]["value"], 60.0);
    assert_eq!(custom["hrv:sdnn"]["definition"], "sdnn");
    Ok(())
}

#[sqlx::test]
async fn test_ingest_quarantines_implausible_never_stores(pool: DbPool) -> anyhow::Result<()> {
    migrate(&pool).await?;
    let future = reading(
        "bad1",
        "heart_rate",
        "instantaneous",
        62.0,
        "bpm",
        "2099-01-01T00:00:00Z",
    );
    let ancient = reading(
        "bad2",
        "heart_rate",
        "instantaneous",
        62.0,
        "bpm",
        "1970-01-01T00:00:00Z",
    );
    let good = reading(
        "ok1",
        "heart_rate",
        "instantaneous",
        62.0,
        "bpm",
        "2026-11-18T06:00:00Z",
    );
    let rep = engine::db::ingest_readings(&pool, "u1", &[future, ancient, good]).await?;
    assert_eq!(rep.quarantined.len(), 2);
    assert_eq!(rep.ingested, 1);

    let rows = engine::db::get_vitals_entries(&pool, &engine::models::VitalsEntryFilter::default())
        .await?;
    assert_eq!(rows.len(), 1);
    let ledger: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM raw_reading_ledger WHERE source_record_id LIKE 'bad%'",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(ledger, 0, "quarantined records never enter the ledger");
    Ok(())
}

#[sqlx::test]
async fn test_ingest_sources_upsert_advances_last_seen(pool: DbPool) -> anyhow::Result<()> {
    migrate(&pool).await?;
    use engine::ingest::{IngestSource, SourceKind};
    let t1 = "2026-11-01T00:00:00Z".parse().unwrap();
    let t2 = "2026-11-18T00:00:00Z".parse().unwrap();
    engine::db::upsert_ingest_sources(
        &pool,
        &[IngestSource {
            id: "watch-1".into(),
            kind: SourceKind::Companion,
            device_label: Some("Pixel Watch".into()),
            platform: Some("android".into()),
            first_seen: t1,
            last_seen: t1,
        }],
    )
    .await?;
    engine::db::upsert_ingest_sources(
        &pool,
        &[IngestSource {
            id: "watch-1".into(),
            kind: SourceKind::Companion,
            device_label: Some("Pixel Watch".into()),
            platform: Some("android".into()),
            first_seen: t2, // stale first_seen must NOT move first_seen backwards
            last_seen: t2,
        }],
    )
    .await?;
    let (first, last): (String, String) =
        sqlx::query_as("SELECT first_seen, last_seen FROM ingest_sources WHERE id = 'watch-1'")
            .fetch_one(&pool)
            .await?;
    assert!(
        first.starts_with("2026-11-01"),
        "first_seen preserved: {first}"
    );
    assert!(last.starts_with("2026-11-18"), "last_seen advanced: {last}");
    Ok(())
}

#[test]
fn test_timestamp_contract_edge_cases() {
    let now: chrono::DateTime<chrono::Utc> = "2026-11-18T12:00:00Z".parse().unwrap();
    // Offset in the label is recorded, not silently dropped (T306).
    let (utc, off) = normalise_timestamp("2026-11-18T09:30:00+02:00", None, now).unwrap();
    assert_eq!(utc.to_rfc3339(), "2026-11-18T07:30:00+00:00");
    assert_eq!(off.as_deref(), Some("+02:00"));
    // Naive local time with explicit zone (the Garmin CSV case).
    let (utc2, _) = normalise_timestamp("2026-11-18 09:30", Some("+02:00"), now).unwrap();
    assert_eq!(utc2, utc);
}

#[sqlx::test]
async fn test_ingest_failure_retention_clears_on_retry(pool: DbPool) -> anyhow::Result<()> {
    migrate(&pool).await?;
    // A quarantined record must be retained verbatim, not dropped.
    let bad = reading(
        "f1",
        "heart_rate",
        "instantaneous",
        61.0,
        "bpm",
        "2099-01-01T00:00:00Z",
    );
    engine::db::ingest_readings(&pool, "u1", std::slice::from_ref(&bad)).await?;
    let fails = engine::db::list_ingest_failures(&pool).await?;
    assert_eq!(fails.len(), 1);
    assert_eq!(fails[0].source_record_id, "f1");
    assert!(
        fails[0].payload.as_deref().unwrap().contains("f1"),
        "verbatim payload retained"
    );

    // The same record, fixed and re-submitted: ingested, and its retained
    // failure retires automatically (FR-059 retry loop closes).
    let fixed = reading(
        "f1",
        "heart_rate",
        "instantaneous",
        61.0,
        "bpm",
        "2026-11-18T06:00:00Z",
    );
    let rep = engine::db::ingest_readings(&pool, "u1", std::slice::from_ref(&fixed)).await?;
    assert_eq!(rep.ingested, 1);
    assert!(engine::db::list_ingest_failures(&pool).await?.is_empty());
    Ok(())
}

// ── spec-004 T325/T326/T329: ingested rows reach the safety engine ──────────

#[sqlx::test]
async fn test_ingest_triggers_existing_alert_path(pool: DbPool) -> anyhow::Result<()> {
    migrate(&pool).await?;
    let batch = vec![
        reading(
            "b1",
            "bp_systolic",
            "korotkoff_1",
            192.0,
            "mmHg",
            "2026-11-18T09:00:00Z",
        ),
        reading(
            "b2",
            "bp_diastolic",
            "korotkoff_5",
            124.0,
            "mmHg",
            "2026-11-18T09:00:00Z",
        ),
    ];
    let rep = engine::db::ingest_readings(&pool, "u1", &batch).await?;
    assert_eq!(
        rep.safety_alerts, 1,
        "hypertensive row must alert on landing (T326)"
    );

    let alerts = engine::db::get_alerts(
        &pool,
        &engine::models::AlertFilter {
            user_id: Some("u1".into()),
            acknowledged: None,
        },
    )
    .await?;
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].alert_type, engine::models::AlertType::Vital);
    assert!(alerts[0].message.starts_with("Hypertensive urgency"));
    assert_eq!(alerts[0].severity, engine::models::AlertSeverity::Critical);
    // The alert must point at the ingested row it came from.
    let rows = engine::db::get_vitals_entries(&pool, &engine::models::VitalsEntryFilter::default())
        .await?;
    assert_eq!(alerts[0].linked_entry_id, Some(rows[0].id));

    // Re-ingest: ledger skips everything, so nothing re-alerts (FR-052 + T329).
    let rep2 = engine::db::ingest_readings(&pool, "u1", &batch).await?;
    assert_eq!(rep2.safety_alerts, 0);
    assert_eq!(
        engine::db::get_alerts(
            &pool,
            &engine::models::AlertFilter {
                user_id: Some("u1".into()),
                acknowledged: None
            },
        )
        .await?
        .len(),
        1
    );
    Ok(())
}

#[sqlx::test]
async fn test_ingest_alert_storm_coalesced_per_day(pool: DbPool) -> anyhow::Result<()> {
    migrate(&pool).await?;
    // Relative days so the test never depends on wall-clock position: batch 1
    // is yesterday (every row hypertensive, spread across the day), batch 2 is
    // today. Both fully in the past — no future-skew rejection.
    let yesterday = (chrono::Utc::now() - chrono::Duration::days(1))
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    let today = chrono::Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();

    // The 288-readings-a-day scenario compressed: 12 same-day rows, all
    // hypertensive.
    let mut batch = Vec::new();
    for i in 0..12 {
        let at = (yesterday + chrono::Duration::hours(i)).to_rfc3339();
        batch.push(reading(
            &format!("s{i}"),
            "bp_systolic",
            "korotkoff_1",
            185.0 + i as f64,
            "mmHg",
            &at,
        ));
        batch.push(reading(
            &format!("d{i}"),
            "bp_diastolic",
            "korotkoff_5",
            112.0,
            "mmHg",
            &at,
        ));
    }
    let rep = engine::db::ingest_readings(&pool, "u1", &batch).await?;
    assert_eq!(rep.vitals_rows, 12);
    assert_eq!(
        rep.safety_alerts, 1,
        "one protocol, one UTC day -> one alert (T329)"
    );

    // A new day re-alerts: coalescing bounds volume, it is not a mute.
    // (today + 10s keeps this deterministic — a run in the first 10 seconds of
    // a UTC day is acceptable flake-space, but nothing wider.)
    let at = (today + chrono::Duration::seconds(10)).to_rfc3339();
    let next_day = vec![
        reading("e1", "bp_systolic", "korotkoff_1", 190.0, "mmHg", &at),
        reading("e2", "bp_diastolic", "korotkoff_5", 121.0, "mmHg", &at),
    ];
    let rep2 = engine::db::ingest_readings(&pool, "u1", &next_day).await?;
    assert_eq!(rep2.safety_alerts, 1);

    let alerts = engine::db::get_alerts(
        &pool,
        &engine::models::AlertFilter {
            user_id: Some("u1".into()),
            acknowledged: None,
        },
    )
    .await?;
    assert_eq!(alerts.len(), 2, "at most one per observation day");
    Ok(())
}

#[sqlx::test]
async fn test_ingest_tachycardia_uses_logged_stimulant(pool: DbPool) -> anyhow::Result<()> {
    migrate(&pool).await?;
    // Caffeine logged in the app 30 minutes ago — inside the 4h stimulant
    // window the existing protocol uses.
    let entry = LogEntry {
        id: Uuid::new_v4(),
        user_id: "u1".to_string(),
        item_type: engine::models::ItemType::Supplement,
        item_id: None,
        name: "caffeine".to_string(),
        quantity: Some(200.0),
        unit: Some("mg".to_string()),
        route: None,
        timestamp: chrono::Utc::now() - chrono::Duration::minutes(30),
        stack_id: None,
        notes: None,
        acknowledged_interaction: false,
        custom_fields: None,
    };
    engine::db::create_log_entry(&pool, &entry).await?;

    let at = (chrono::Utc::now() - chrono::Duration::minutes(5)).to_rfc3339();
    let batch = vec![reading(
        "h1",
        "heart_rate",
        "instantaneous",
        132.0,
        "bpm",
        &at,
    )];
    let rep = engine::db::ingest_readings(&pool, "u1", &batch).await?;
    assert_eq!(
        rep.safety_alerts, 1,
        "logged stimulant + HR 132 must fire protocol 1"
    );

    let alerts = engine::db::get_alerts(
        &pool,
        &engine::models::AlertFilter {
            user_id: Some("u1".into()),
            acknowledged: None,
        },
    )
    .await?;
    assert!(
        alerts[0]
            .message
            .starts_with("Stimulant-associated tachycardia")
    );
    Ok(())
}

#[sqlx::test]
async fn test_ingest_in_range_readings_stay_silent(pool: DbPool) -> anyhow::Result<()> {
    migrate(&pool).await?;
    let batch = vec![
        reading(
            "n1",
            "bp_systolic",
            "korotkoff_1",
            118.0,
            "mmHg",
            "2026-11-18T09:00:00Z",
        ),
        reading(
            "n2",
            "bp_diastolic",
            "korotkoff_5",
            76.0,
            "mmHg",
            "2026-11-18T09:00:00Z",
        ),
        reading(
            "n3",
            "heart_rate",
            "instantaneous",
            62.0,
            "bpm",
            "2026-11-18T09:00:00Z",
        ),
    ];
    let rep = engine::db::ingest_readings(&pool, "u1", &batch).await?;
    assert_eq!(rep.safety_alerts, 0);
    assert!(
        engine::db::get_alerts(
            &pool,
            &engine::models::AlertFilter {
                user_id: Some("u1".into()),
                acknowledged: None
            },
        )
        .await?
        .is_empty()
    );
    Ok(())
}

#[sqlx::test]
async fn test_ingest_advice_cites_measured_sleep(pool: DbPool) -> anyhow::Result<()> {
    migrate(&pool).await?;
    let entry = LogEntry {
        id: Uuid::new_v4(),
        user_id: "u1".to_string(),
        item_type: engine::models::ItemType::Supplement,
        item_id: None,
        name: "caffeine".to_string(),
        quantity: Some(200.0),
        unit: Some("mg".to_string()),
        route: None,
        timestamp: chrono::Utc::now() - chrono::Duration::minutes(30),
        stack_id: None,
        notes: None,
        acknowledged_interaction: false,
        custom_fields: None,
    };
    engine::db::create_log_entry(&pool, &entry).await?;

    // B4 independent test: high HR on a night that measured 4.2 h of sleep.
    // Sleep duration lands as a custom metric (no native field); same
    // observed_at as the HR row so they merge into one vitals entry.
    let at = (chrono::Utc::now() - chrono::Duration::minutes(5)).to_rfc3339();
    let batch = vec![
        reading("h1", "heart_rate", "instantaneous", 132.0, "bpm", &at),
        reading("h2", "sleep_hours", "total_sleep_time", 4.2, "h", &at),
    ];
    let rep = engine::db::ingest_readings(&pool, "u1", &batch).await?;
    assert_eq!(rep.safety_alerts, 1);
    assert_eq!(rep.custom_metrics, 1, "sleep_hours must land as custom");

    let alerts = engine::db::get_alerts(
        &pool,
        &engine::models::AlertFilter {
            user_id: Some("u1".into()),
            acknowledged: None,
        },
    )
    .await?;
    let rec = alerts[0].recommendation.clone().unwrap_or_default();
    // AC-1: cites the measured value with its source (date is asserted by
    // the "on DD Mon" fragment around the same clause).
    assert!(
        rec.contains("Recovery context: measured sleep 4.2 h on "),
        "{rec}"
    );
    assert!(rec.contains("(from watch-1)"), "{rec}");
    // Log-derived context still rides along (advice is additive, not replaced).
    assert!(
        rec.contains("Recent stimulants in your log: caffeine"),
        "{rec}"
    );
    Ok(())
}

#[sqlx::test]
async fn test_advice_falls_back_without_recovery_data(pool: DbPool) -> anyhow::Result<()> {
    migrate(&pool).await?;
    // AC-2 / T328: no ingested recovery data → advice is exactly the
    // log-derived text of before, with no Recovery context clause.
    let at = (chrono::Utc::now() - chrono::Duration::minutes(5)).to_rfc3339();
    let batch = vec![reading(
        "h1",
        "heart_rate",
        "instantaneous",
        132.0,
        "bpm",
        &at,
    )];
    let rep = engine::db::ingest_readings(&pool, "u1", &batch).await?;
    assert_eq!(
        rep.safety_alerts, 0,
        "no stimulant logged -> protocol 1 stays quiet"
    );

    // Hypertensive row with no sleep/HRV data: magnesium advice, no recovery clause.
    let batch2 = vec![
        reading("b1", "bp_systolic", "korotkoff_1", 192.0, "mmHg", &at),
        reading("b2", "bp_diastolic", "korotkoff_5", 124.0, "mmHg", &at),
    ];
    assert_eq!(
        engine::db::ingest_readings(&pool, "u1", &batch2)
            .await?
            .safety_alerts,
        1
    );
    let alerts = engine::db::get_alerts(
        &pool,
        &engine::models::AlertFilter {
            user_id: Some("u1".into()),
            acknowledged: None,
        },
    )
    .await?;
    let rec = alerts[0].recommendation.clone().unwrap_or_default();
    assert!(!rec.contains("Recovery context"), "{rec}");
    assert!(
        rec.contains("Context:"),
        "log-derived advice still applies: {rec}"
    );
    Ok(())
}
