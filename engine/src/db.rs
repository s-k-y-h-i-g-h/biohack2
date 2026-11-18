use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool, sqlite::SqliteRow};
use uuid::Uuid;

use crate::models::*;

pub type DbPool = SqlitePool;

/// Canonical enum <-> bare-string conversion lives in `models.rs`
/// (`ItemType::as_str` / `from_str`). SQLite stores bare words and the schema
/// CHECK constraints match on exactly those words, so DB (de)serialization must
/// NOT go through serde_json — serde's representation quotes them.
///
/// Helper to parse DateTime<Utc> from RFC3339 string
fn parse_datetime(s: &str) -> Option<DateTime<Utc>> {
    Some(DateTime::parse_from_rfc3339(s).ok()?.with_timezone(&Utc))
}

/// Initializes the database schema. Call once at startup.
pub async fn migrate(pool: &DbPool) -> anyhow::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS log_entries (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            item_type TEXT NOT NULL CHECK(item_type IN ('supplement','medication','drug','food','action')),
            item_id TEXT,
            name TEXT NOT NULL,
            quantity REAL,
            unit TEXT,
            route TEXT,
            timestamp TEXT NOT NULL,
            stack_id TEXT,
            notes TEXT,
            acknowledged_interaction INTEGER NOT NULL DEFAULT 0,
            custom_fields TEXT
        )"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_log_entries_user_timestamp ON log_entries(user_id, timestamp)"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS catalog_items (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL CHECK(category IN ('supplement','medication','drug','food','action')),
            dosage_range TEXT,
            half_life TEXT,
            contraindications TEXT,
            warnings TEXT,
            is_custom INTEGER NOT NULL DEFAULT 0,
            source TEXT,
            version INTEGER NOT NULL DEFAULT 1
        )"
    )
    .execute(pool)
    .await?;

    // mechanism_tags added 2026-10-02 (mechanism-class taxonomy). SQLite has no
    // IF NOT EXISTS for ADD COLUMN, so tolerate the duplicate-column error.
    let _ = sqlx::query("ALTER TABLE catalog_items ADD COLUMN mechanism_tags TEXT")
        .execute(pool)
        .await;

    // Numeric half-life columns added 2026-11-14 (spec 003 T207). Same
    // additive pattern: tolerate the duplicate-column error on re-run.
    for ddl in [
        "ALTER TABLE catalog_items ADD COLUMN half_life_hours REAL",
        "ALTER TABLE catalog_items ADD COLUMN half_life_range_lo REAL",
        "ALTER TABLE catalog_items ADD COLUMN half_life_range_hi REAL",
    ] {
        let _ = sqlx::query(ddl).execute(pool).await;
    }

    // SubstanceModel persistence (spec 003 T206). Keyed by NAME for the same
    // reason seed_catalog is name-idempotent: catalog UUIDs regenerate on
    // every seed run.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS substance_models (
            substance TEXT PRIMARY KEY,
            model_class TEXT NOT NULL,
            half_life_hours REAL,
            half_life_range TEXT,
            params TEXT NOT NULL DEFAULT 'null',
            provenance TEXT NOT NULL,
            catalog_version INTEGER NOT NULL DEFAULT 1,
            rationale TEXT
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_catalog_category ON catalog_items(category)")
        .execute(pool)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS stacks (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS stack_items (
            stack_id TEXT NOT NULL,
            item_id TEXT NOT NULL,
            quantity REAL,
            unit TEXT,
            note TEXT,
            PRIMARY KEY (stack_id, item_id),
            FOREIGN KEY(stack_id) REFERENCES stacks(id) ON DELETE CASCADE
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS vitals_entries (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            bp_systolic INTEGER,
            bp_diastolic INTEGER,
            heart_rate INTEGER,
            weight REAL,
            blood_glucose REAL,
            temperature REAL,
            spo2 INTEGER,
            hrv REAL,
            sleep_quality TEXT,
            custom_metrics TEXT,
            notes TEXT
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_vitals_user_timestamp ON vitals_entries(user_id, timestamp)"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS alerts (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            type TEXT NOT NULL CHECK(type IN ('vital','interaction','warning','redose','cumulative_load')),
            severity TEXT NOT NULL CHECK(severity IN ('info','warning','critical')),
            message TEXT NOT NULL,
            recommendation TEXT,
            is_acknowledged INTEGER NOT NULL DEFAULT 0,
            linked_entry_id TEXT,
            generated_at TEXT NOT NULL,
            resolved_at TEXT
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_alerts_user_unack ON alerts(user_id, is_acknowledged)",
    )
    .execute(pool)
    .await?;

    // spec-003 T228: alerts.type gained 'redose' and 'cumulative_load'.
    // SQLite cannot ALTER a CHECK constraint in place, so pre-existing
    // databases get the table rebuilt exactly once; the sqlite_master guard
    // makes this a no-op on fresh databases and on already-migrated ones.
    let alerts_ddl: Option<(String,)> =
        sqlx::query_as("SELECT sql FROM sqlite_master WHERE type='table' AND name='alerts'")
            .fetch_optional(pool)
            .await?;
    if alerts_ddl.is_some_and(|(sql,)| !sql.contains("redose")) {
        // Canonical SQLite recipe: rename the original aside, create the
        // table under its real name with the new constraint, copy, drop.
        // (Renaming the NEW table TO `alerts` hits a schema-namespace clash
        // after the DROP on some SQLite builds.)
        sqlx::query("ALTER TABLE alerts RENAME TO alerts_legacy")
            .execute(pool)
            .await?;
        sqlx::query(
            "CREATE TABLE alerts (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                type TEXT NOT NULL CHECK(type IN ('vital','interaction','warning','redose','cumulative_load')),
                severity TEXT NOT NULL CHECK(severity IN ('info','warning','critical')),
                message TEXT NOT NULL,
                recommendation TEXT,
                is_acknowledged INTEGER NOT NULL DEFAULT 0,
                linked_entry_id TEXT,
                generated_at TEXT NOT NULL,
                resolved_at TEXT
            )",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "INSERT INTO alerts SELECT id, user_id, type, severity, message, recommendation, is_acknowledged, linked_entry_id, generated_at, resolved_at FROM alerts_legacy",
        )
        .execute(pool)
        .await?;
        sqlx::query("DROP TABLE alerts_legacy")
            .execute(pool)
            .await?;
        // The index rode along with the rename (same name is fine) — ensure
        // it exists either way.
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_alerts_user_unack ON alerts(user_id, is_acknowledged)",
        )
        .execute(pool)
        .await?;
    }

    // spec-003 T226: user-configured therapeutic target bands (FR-034).
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS target_bands (
            user_id TEXT NOT NULL,
            substance TEXT NOT NULL,
            low REAL NOT NULL CHECK(low >= 0),
            high REAL NOT NULL,
            window_hours REAL NOT NULL CHECK(window_hours > 0),
            updated_at TEXT NOT NULL,
            PRIMARY KEY (user_id, substance)
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS insights (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            type TEXT NOT NULL CHECK(type IN ('correlation','trend','pattern')),
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            confidence REAL NOT NULL,
            supporting_data_points INTEGER NOT NULL,
            generated_at TEXT NOT NULL,
            related_entry_ids TEXT
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            content TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            linked_entry_id TEXT
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_notes_user_timestamp ON notes(user_id, timestamp)")
        .execute(pool)
        .await?;

    Ok(())
}

// ── LogEntry CRUD ─────────────────────────────────────────────────────────────

pub async fn create_log_entry(pool: &DbPool, entry: &LogEntry) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO log_entries (id, user_id, item_type, item_id, name, quantity, unit, route, timestamp, stack_id, notes, acknowledged_interaction, custom_fields)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"
    )
    .bind(entry.id.to_string())
    .bind(&entry.user_id)
    .bind(entry.item_type.as_str().to_string())
    .bind(entry.item_id.map(|u| u.to_string()))
    .bind(&entry.name)
    .bind(entry.quantity)
    .bind(entry.unit.clone())
    .bind(serde_json::to_string(&entry.route)?)
    .bind(entry.timestamp.to_rfc3339())
    .bind(entry.stack_id.map(|u| u.to_string()))
    .bind(entry.notes.clone())
    .bind(entry.acknowledged_interaction as i32)
    // None must persist as SQL NULL, never the four-byte string "null":
    // a stored "null" came back as Some(Value::Null) and broke round-trip
    // fidelity (cross_module_tests, 2026-11-18).
    .bind(match &entry.custom_fields {
        Some(v) => Some(serde_json::to_string(v)?),
        None => None,
    })
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_log_entry(pool: &DbPool, id: &Uuid) -> anyhow::Result<Option<LogEntry>> {
    let row = sqlx::query(
        "SELECT id, user_id, item_type, item_id, name, quantity, unit, route, timestamp, stack_id, notes, acknowledged_interaction, custom_fields
         FROM log_entries WHERE id = ?1"
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await?;
    Ok(row.map(row_to_log_entry))
}

pub async fn get_log_entries(
    pool: &DbPool,
    filter: &LogEntryFilter,
) -> anyhow::Result<Vec<LogEntry>> {
    let mut query = String::from(
        "SELECT id, user_id, item_type, item_id, name, quantity, unit, route, timestamp, stack_id, notes, acknowledged_interaction, custom_fields FROM log_entries WHERE 1=1",
    );
    let mut bind_vars: Vec<String> = Vec::new();

    if let Some(ref uid) = filter.user_id {
        query.push_str(" AND user_id = ?");
        bind_vars.push(uid.clone());
    }
    if let Some(sid) = filter.stack_id {
        query.push_str(" AND stack_id = ?");
        bind_vars.push(sid.to_string());
    }
    if let Some(ref start) = filter.start_date {
        query.push_str(" AND timestamp >= ?");
        bind_vars.push(start.to_rfc3339());
    }
    if let Some(ref end) = filter.end_date {
        query.push_str(" AND timestamp <= ?");
        bind_vars.push(end.to_rfc3339());
    }
    if let Some(cat) = &filter.category {
        query.push_str(" AND item_type = ?");
        // NOT serde_json — that adds quotes around the value ("\"supplement\"")
        // which never matches the stored bare string.
        bind_vars.push(cat.as_str().to_string());
    }

    query.push_str(" ORDER BY timestamp DESC");

    let mut q = sqlx::query(&query);
    for v in bind_vars {
        q = q.bind(v);
    }
    let rows = q.fetch_all(pool).await?;

    Ok(rows.into_iter().map(row_to_log_entry).collect())
}

pub async fn update_log_entry(pool: &DbPool, entry: &LogEntry) -> anyhow::Result<()> {
    sqlx::query("UPDATE log_entries SET notes = ?2, acknowledged_interaction = ?3 WHERE id = ?1")
        .bind(entry.id.to_string())
        .bind(entry.notes.clone())
        .bind(entry.acknowledged_interaction as i32)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_log_entry(pool: &DbPool, id: &Uuid) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM log_entries WHERE id = ?1")
        .bind(id.to_string())
        .execute(pool)
        .await?;
    Ok(())
}

// ── CatalogItem CRUD ──────────────────────────────────────────────────────────

pub async fn seed_catalog(pool: &DbPool, items: &[CatalogItem]) -> anyhow::Result<()> {
    for item in items {
        let cat_str = item.category.as_str().to_string();
        // Idempotent by NAME (not id): seed_catalog() regenerates UUIDs on
        // every call, so a fresh seed run against an existing database must
        // match on name or it would duplicate every substance.
        let result = sqlx::query(
            "INSERT OR IGNORE INTO catalog_items (id, name, category, dosage_range, half_life, contraindications, warnings, mechanism_tags, is_custom, source, version, half_life_hours, half_life_range_lo, half_life_range_hi)
             SELECT ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14
             WHERE NOT EXISTS (SELECT 1 FROM catalog_items WHERE name = ?2)"
        )
        .bind(item.id.to_string())
        .bind(&item.name)
        .bind(&cat_str)
        .bind(serde_json::to_string(&item.dosage_range)?)
        .bind(item.half_life.as_deref())
        .bind(serde_json::to_string(&item.contraindications)?)
        .bind(serde_json::to_string(&item.warnings)?)
        .bind(serde_json::to_string(&item.mechanism_tags)?)
        .bind(item.is_custom as i32)
        .bind(item.source.as_deref())
        .bind(item.version)
        .bind(item.half_life_hours)
        .bind(item.half_life_range.map(|(lo, _)| lo))
        .bind(item.half_life_range.map(|(_, hi)| hi))
        .execute(pool)
        .await;

        match &result {
            Ok(res) => {
                println!(
                    "Inserted {}: rows_affected={}",
                    item.name,
                    res.rows_affected()
                );
            }
            Err(e) => {
                println!("Error inserting {}: {:?}", item.name, e);
            }
        }
    }
    Ok(())
}

pub async fn get_catalog_items(pool: &DbPool) -> anyhow::Result<Vec<CatalogItem>> {
    let rows = sqlx::query(
        "SELECT id, name, category, dosage_range, half_life, contraindications, warnings, mechanism_tags, is_custom, source, version, half_life_hours, half_life_range_lo, half_life_range_hi FROM catalog_items"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(row_to_catalog_item).collect())
}

pub async fn search_catalog(pool: &DbPool, query: &str) -> anyhow::Result<Vec<CatalogItem>> {
    let rows = sqlx::query(
        "SELECT id, name, category, dosage_range, half_life, contraindications, warnings, mechanism_tags, is_custom, source, version, half_life_hours, half_life_range_lo, half_life_range_hi FROM catalog_items WHERE name LIKE ?1"
    )
    .bind(format!("%{}%", query))
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(row_to_catalog_item).collect())
}

// ── SubstanceModel persistence (spec 003 T206/T207) ──────────────────────────

/// Upsert PK model rows, keyed by substance NAME. Re-running with the same
/// assignments is a no-op-equivalent replace — seed is idempotent.
pub async fn upsert_substance_models(
    pool: &DbPool,
    models: &[crate::pk_params::SubstanceModel],
) -> anyhow::Result<()> {
    for m in models {
        let class_str = serde_json::to_string(&m.model_class)?
            .trim_matches('"')
            .to_string();
        let range_str = m
            .half_life_range
            .map(|(lo, hi)| serde_json::to_string(&[lo, hi]))
            .transpose()?;
        sqlx::query(
            "INSERT OR REPLACE INTO substance_models
             (substance, model_class, half_life_hours, half_life_range, params, provenance, catalog_version, rationale)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .bind(&m.substance)
        .bind(&class_str)
        .bind(m.half_life_hours)
        .bind(&range_str)
        .bind(serde_json::to_string(&m.params)?)
        .bind(&m.provenance)
        .bind(m.catalog_version)
        .bind(m.rationale.as_deref())
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn get_substance_models(
    pool: &DbPool,
) -> anyhow::Result<Vec<crate::pk_params::SubstanceModel>> {
    let rows = sqlx::query(
        "SELECT substance, model_class, half_life_hours, half_life_range, params, provenance, catalog_version, rationale FROM substance_models",
    )
    .fetch_all(pool)
    .await?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let class_str: String = row.try_get("model_class")?;
        let model_class: crate::models::ModelClass =
            serde_json::from_value(serde_json::Value::String(class_str))?;
        let range_str: Option<String> = row
            .try_get::<Option<String>, _>("half_life_range")
            .unwrap_or(None);
        let half_life_range = range_str
            .as_deref()
            .and_then(|s| serde_json::from_str::<[f64; 2]>(s).ok())
            .map(|[lo, hi]| (lo, hi));
        out.push(crate::pk_params::SubstanceModel {
            substance: row.try_get("substance")?,
            model_class,
            half_life_hours: row
                .try_get::<Option<f64>, _>("half_life_hours")
                .unwrap_or(None),
            half_life_range,
            params: row
                .try_get::<String, _>("params")
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or(serde_json::Value::Null),
            provenance: row.try_get("provenance")?,
            catalog_version: row.try_get::<i32, _>("catalog_version").unwrap_or(1),
            rationale: row
                .try_get::<Option<String>, _>("rationale")
                .unwrap_or(None),
        });
    }
    Ok(out)
}

/// Rebuild the `SubstanceModel` set from the PERSISTED catalog and upsert it.
/// Called after catalog seeding so the model table always reflects the
/// catalog actually in the database, not the one compiled into the binary.
pub async fn ensure_substance_models(pool: &DbPool) -> anyhow::Result<usize> {
    let catalog = get_catalog_items(pool).await?;
    let models = crate::pk_assignments::build_models(&catalog);
    let n = models.len();
    upsert_substance_models(pool, &models).await?;
    Ok(n)
}

pub async fn create_stack(pool: &DbPool, stack: &Stack) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO stacks (id, user_id, name, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(stack.id.to_string())
    .bind(&stack.user_id)
    .bind(&stack.name)
    .bind(stack.description.as_deref())
    .bind(stack.created_at.to_rfc3339())
    .bind(stack.updated_at.to_rfc3339())
    .execute(pool)
    .await?;

    for item in &stack.items {
        sqlx::query(
            "INSERT INTO stack_items (stack_id, item_id, quantity, unit, note) VALUES (?1, ?2, ?3, ?4, ?5)"
        )
        .bind(stack.id.to_string())
        .bind(item.item_id.to_string())
        .bind(item.quantity)
        .bind(item.unit.clone())
        .bind(item.note.clone())
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn get_stack(pool: &DbPool, id: &Uuid) -> anyhow::Result<Option<Stack>> {
    let row = sqlx::query(
        "SELECT id, user_id, name, description, created_at, updated_at FROM stacks WHERE id = ?1",
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await?;

    match row {
        None => Ok(None),
        Some(r) => {
            let items_rows = sqlx::query(
                "SELECT item_id, quantity, unit, note FROM stack_items WHERE stack_id = ?1",
            )
            .bind(id.to_string())
            .fetch_all(pool)
            .await?;

            let stack = Stack {
                id: Uuid::parse_str(&r.try_get::<String, _>("id")?).unwrap_or_default(),
                user_id: r.try_get::<String, _>("user_id")?,
                name: r.try_get::<String, _>("name")?,
                description: r.try_get::<Option<String>, _>("description")?,
                created_at: parse_datetime(&r.try_get::<String, _>("created_at")?)
                    .unwrap_or_default(),
                updated_at: parse_datetime(&r.try_get::<String, _>("updated_at")?)
                    .unwrap_or_default(),
                items: items_rows
                    .into_iter()
                    .map(|row| StackItem {
                        item_id: Uuid::parse_str(
                            &row.try_get::<String, _>("item_id").unwrap_or_default(),
                        )
                        .unwrap_or_default(),
                        quantity: row.try_get::<Option<f64>, _>("quantity").unwrap_or(None),
                        unit: row.try_get::<Option<String>, _>("unit").unwrap_or(None),
                        note: row.try_get::<Option<String>, _>("note").unwrap_or(None),
                    })
                    .collect(),
            };
            Ok(Some(stack))
        }
    }
}

pub async fn get_stacks(pool: &DbPool, user_id: &str) -> anyhow::Result<Vec<Stack>> {
    let rows = sqlx::query(
        "SELECT id, user_id, name, description, created_at, updated_at FROM stacks WHERE user_id = ?1 ORDER BY updated_at DESC"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut stacks = Vec::new();
    for r in rows {
        let id = Uuid::parse_str(&r.try_get::<String, _>("id")?).unwrap_or_default();
        let items_rows = sqlx::query(
            "SELECT item_id, quantity, unit, note FROM stack_items WHERE stack_id = ?1",
        )
        .bind(id.to_string())
        .fetch_all(pool)
        .await?;

        stacks.push(Stack {
            id,
            user_id: r.try_get::<String, _>("user_id")?,
            name: r.try_get::<String, _>("name")?,
            description: r.try_get::<Option<String>, _>("description")?,
            created_at: parse_datetime(&r.try_get::<String, _>("created_at")?).unwrap_or_default(),
            updated_at: parse_datetime(&r.try_get::<String, _>("updated_at")?).unwrap_or_default(),
            items: items_rows
                .into_iter()
                .map(|row| StackItem {
                    item_id: Uuid::parse_str(
                        &row.try_get::<String, _>("item_id").unwrap_or_default(),
                    )
                    .unwrap_or_default(),
                    quantity: row.try_get::<Option<f64>, _>("quantity").unwrap_or(None),
                    unit: row.try_get::<Option<String>, _>("unit").unwrap_or(None),
                    note: row.try_get::<Option<String>, _>("note").unwrap_or(None),
                })
                .collect(),
        });
    }
    Ok(stacks)
}

pub async fn update_stack(pool: &DbPool, stack: &Stack) -> anyhow::Result<()> {
    sqlx::query("UPDATE stacks SET name = ?2, description = ?3, updated_at = ?4 WHERE id = ?1")
        .bind(stack.id.to_string())
        .bind(&stack.name)
        .bind(stack.description.as_deref())
        .bind(stack.updated_at.to_rfc3339())
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM stack_items WHERE stack_id = ?1")
        .bind(stack.id.to_string())
        .execute(pool)
        .await?;
    for item in &stack.items {
        sqlx::query(
            "INSERT INTO stack_items (stack_id, item_id, quantity, unit, note) VALUES (?1, ?2, ?3, ?4, ?5)"
        )
        .bind(stack.id.to_string())
        .bind(item.item_id.to_string())
        .bind(item.quantity)
        .bind(item.unit.clone())
        .bind(item.note.clone())
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn delete_stack(pool: &DbPool, id: &Uuid) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM stack_items WHERE stack_id = ?1")
        .bind(id.to_string())
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM stacks WHERE id = ?1")
        .bind(id.to_string())
        .execute(pool)
        .await?;
    Ok(())
}

// ── VitalsEntry CRUD ──────────────────────────────────────────────────────────

pub async fn create_vitals_entry(pool: &DbPool, entry: &VitalsEntry) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO vitals_entries (id, user_id, timestamp, bp_systolic, bp_diastolic, heart_rate, weight, blood_glucose, temperature, spo2, hrv, sleep_quality, custom_metrics, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)"
    )
    .bind(entry.id.to_string())
    .bind(&entry.user_id)
    .bind(entry.timestamp.to_rfc3339())
    .bind(entry.bp_systolic)
    .bind(entry.bp_diastolic)
    .bind(entry.heart_rate)
    .bind(entry.weight)
    .bind(entry.blood_glucose)
    .bind(entry.temperature)
    .bind(entry.spo2)
    .bind(entry.hrv)
    .bind(serde_json::to_string(&entry.sleep_quality)?)
    .bind(serde_json::to_string(&entry.custom_metrics)?)
    .bind(entry.notes.clone())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_vitals_entry(pool: &DbPool, id: &Uuid) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM vitals_entries WHERE id = ?1")
        .bind(id.to_string())
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_vitals_entries(
    pool: &DbPool,
    filter: &VitalsEntryFilter,
) -> anyhow::Result<Vec<VitalsEntry>> {
    let mut query = String::from(
        "SELECT id, user_id, timestamp, bp_systolic, bp_diastolic, heart_rate, weight, blood_glucose, temperature, spo2, hrv, sleep_quality, custom_metrics, notes FROM vitals_entries WHERE 1=1",
    );
    let mut bind_vars: Vec<String> = Vec::new();

    if let Some(ref uid) = filter.user_id {
        query.push_str(" AND user_id = ?");
        bind_vars.push(uid.clone());
    }
    if let Some(ref start) = filter.start_date {
        query.push_str(" AND timestamp >= ?");
        bind_vars.push(start.to_rfc3339());
    }
    if let Some(ref end) = filter.end_date {
        query.push_str(" AND timestamp <= ?");
        bind_vars.push(end.to_rfc3339());
    }

    query.push_str(" ORDER BY timestamp DESC");

    let mut q = sqlx::query(&query);
    for v in bind_vars {
        q = q.bind(v);
    }
    let rows = q.fetch_all(pool).await?;
    Ok(rows.into_iter().map(row_to_vitals_entry).collect())
}

// ── Alert CRUD ────────────────────────────────────────────────────────────────

pub async fn create_alert(pool: &DbPool, alert: &Alert) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO alerts (id, user_id, type, severity, message, recommendation, is_acknowledged, linked_entry_id, generated_at, resolved_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
    )
    .bind(alert.id.to_string())
    .bind(&alert.user_id)
    // Bare strings — the schema's CHECK constraints require 'vital'/'warning'/etc
    // without JSON quotes (serde_json::to_string would write "\"vital\"" and fail).
    .bind(alert.alert_type.as_str().to_string())
    .bind(alert.severity.as_str().to_string())
    .bind(&alert.message)
    .bind(alert.recommendation.as_deref())
    .bind(alert.is_acknowledged as i32)
    .bind(alert.linked_entry_id.map(|u| u.to_string()))
    .bind(alert.generated_at.to_rfc3339())
    .bind(alert.resolved_at.as_ref().map(|d| d.to_rfc3339()))
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_alerts(pool: &DbPool, filter: &AlertFilter) -> anyhow::Result<Vec<Alert>> {
    let mut query = String::from(
        "SELECT id, user_id, type, severity, message, recommendation, is_acknowledged, linked_entry_id, generated_at, resolved_at FROM alerts WHERE 1=1",
    );
    let mut bind_vars: Vec<String> = Vec::new();

    if let Some(ref uid) = filter.user_id {
        query.push_str(" AND user_id = ?");
        bind_vars.push(uid.clone());
    }
    if let Some(ack) = filter.acknowledged {
        query.push_str(" AND is_acknowledged = ?");
        // Integer, not string — the column is INTEGER; 'true' would never match 1.
        bind_vars.push((ack as i32).to_string());
    }

    query.push_str(" ORDER BY generated_at DESC");

    let mut q = sqlx::query(&query);
    for v in bind_vars {
        q = q.bind(v);
    }
    let rows = q.fetch_all(pool).await?;
    Ok(rows.into_iter().map(row_to_alert).collect())
}

pub async fn acknowledge_alert(pool: &DbPool, id: &Uuid) -> anyhow::Result<()> {
    sqlx::query("UPDATE alerts SET is_acknowledged = 1, resolved_at = ?1 WHERE id = ?2")
        .bind(Utc::now().to_rfc3339())
        .bind(id.to_string())
        .execute(pool)
        .await?;
    Ok(())
}

/// spec-003 T232: mark an alert resolved by external circumstance (a logged
/// dose restoring the target band) without treating it as user-acknowledged.
/// Idempotent: re-resolving just refreshes the timestamp.
pub async fn resolve_alert(pool: &DbPool, id: &Uuid, at: DateTime<Utc>) -> anyhow::Result<()> {
    sqlx::query("UPDATE alerts SET resolved_at = ?1 WHERE id = ?2")
        .bind(at.to_rfc3339())
        .bind(id.to_string())
        .execute(pool)
        .await?;
    Ok(())
}

// ── Target band CRUD (spec-003 T226) ─────────────────────────────────────────

/// Insert-or-replace the user's target band for a substance (one band per
/// (user, substance) by table PK). Validation is the caller-model's job
/// (`TargetBand::validate`); the table CHECKs mirror the numeric bounds.
pub async fn upsert_target_band(
    pool: &DbPool,
    user_id: &str,
    band: &TargetBand,
) -> anyhow::Result<()> {
    band.validate().map_err(anyhow::Error::msg)?;
    sqlx::query(
        "INSERT INTO target_bands (user_id, substance, low, high, window_hours, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(user_id, substance) DO UPDATE SET
            low = excluded.low, high = excluded.high,
            window_hours = excluded.window_hours, updated_at = excluded.updated_at",
    )
    .bind(user_id)
    .bind(&band.substance)
    .bind(band.low)
    .bind(band.high)
    .bind(band.window_hours)
    .bind(band.updated_at.to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_target_bands(pool: &DbPool, user_id: &str) -> anyhow::Result<Vec<TargetBand>> {
    let rows = sqlx::query(
        "SELECT substance, low, high, window_hours, updated_at FROM target_bands WHERE user_id = ?1 ORDER BY substance",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    rows.iter()
        .map(|row| {
            let updated: String = row.try_get("updated_at")?;
            Ok(TargetBand {
                substance: row.try_get("substance")?,
                low: row.try_get("low")?,
                high: row.try_get("high")?,
                window_hours: row.try_get("window_hours")?,
                updated_at: parse_datetime(&updated)
                    .ok_or_else(|| sqlx::Error::Decode("bad updated_at rfc3339".into()))?,
            })
        })
        .collect()
}

/// Returns whether a row was removed.
pub async fn delete_target_band(
    pool: &DbPool,
    user_id: &str,
    substance: &str,
) -> anyhow::Result<bool> {
    let res = sqlx::query("DELETE FROM target_bands WHERE user_id = ?1 AND substance = ?2")
        .bind(user_id)
        .bind(substance)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

// ── Insight CRUD ──────────────────────────────────────────────────────────────

pub async fn create_insight(pool: &DbPool, insight: &Insight) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO insights (id, user_id, type, title, description, confidence, supporting_data_points, generated_at, related_entry_ids)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
    )
    .bind(insight.id.to_string())
    .bind(&insight.user_id)
    // Bare string — schema CHECK requires 'correlation'/'trend'/'pattern' unquoted
    .bind(insight.insight_type.as_str().to_string())
    .bind(&insight.title)
    .bind(&insight.description)
    .bind(insight.confidence)
    .bind(insight.supporting_data_points)
    .bind(insight.generated_at.to_rfc3339())
    .bind(serde_json::to_string(&insight.related_entry_ids)?)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_insights(pool: &DbPool, user_id: &str) -> anyhow::Result<Vec<Insight>> {
    let rows = sqlx::query(
        "SELECT id, user_id, type, title, description, confidence, supporting_data_points, generated_at, related_entry_ids FROM insights WHERE user_id = ?1 ORDER BY generated_at DESC"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(row_to_insight).collect())
}

// ── Row converters ────────────────────────────────────────────────────────────

fn row_to_log_entry(row: SqliteRow) -> LogEntry {
    LogEntry {
        id: Uuid::parse_str(&row.try_get::<String, _>("id").unwrap_or_default())
            .unwrap_or_default(),
        user_id: row.try_get::<String, _>("user_id").unwrap_or_default(),
        item_type: ItemType::from_str(&row.try_get::<String, _>("item_type").unwrap_or_default())
            .unwrap_or_default(),
        item_id: row
            .try_get::<Option<String>, _>("item_id")
            .unwrap_or(None)
            .and_then(|s| Uuid::parse_str(&s).ok()),
        name: row.try_get::<String, _>("name").unwrap_or_default(),
        quantity: row.try_get::<Option<f64>, _>("quantity").unwrap_or(None),
        unit: row.try_get::<Option<String>, _>("unit").unwrap_or(None),
        route: row
            .try_get::<Option<String>, _>("route")
            .unwrap_or(None)
            .and_then(|s| serde_json::from_str(&s).ok()),
        timestamp: parse_datetime(&row.try_get::<String, _>("timestamp").unwrap_or_default())
            .unwrap_or_default(),
        stack_id: row
            .try_get::<Option<String>, _>("stack_id")
            .unwrap_or(None)
            .and_then(|s| Uuid::parse_str(&s).ok()),
        notes: row.try_get::<Option<String>, _>("notes").unwrap_or(None),
        acknowledged_interaction: row
            .try_get::<i32, _>("acknowledged_interaction")
            .unwrap_or(0)
            != 0,
        custom_fields: row
            .try_get::<Option<String>, _>("custom_fields")
            .unwrap_or(None)
            // Deserialize as Option<Value> so a legacy stored "null" reads
            // back as None rather than Some(Value::Null).
            .and_then(|s| serde_json::from_str::<Option<serde_json::Value>>(&s).ok())
            .flatten(),
    }
}

fn row_to_catalog_item(row: SqliteRow) -> CatalogItem {
    let half_life_str: Option<String> = row
        .try_get::<Option<String>, _>("half_life")
        .unwrap_or(None);
    // Stored numerics take precedence; rows written BEFORE the T207 columns
    // (or written with NULL numerics) fall back to the same canonical
    // derivation the seed path uses — one parser, no drift.
    let derived = crate::pharmacokinetics::half_life_fields(half_life_str.as_deref());
    CatalogItem {
        id: Uuid::parse_str(&row.try_get::<String, _>("id").unwrap_or_default())
            .unwrap_or_default(),
        name: row.try_get::<String, _>("name").unwrap_or_default(),
        category: ItemType::from_str(&row.try_get::<String, _>("category").unwrap_or_default())
            .unwrap_or_default(),
        dosage_range: row
            .try_get::<Option<String>, _>("dosage_range")
            .unwrap_or(None)
            .and_then(|s| serde_json::from_str(&s).ok()),
        half_life: half_life_str,
        half_life_hours: row
            .try_get::<Option<f64>, _>("half_life_hours")
            .unwrap_or(None)
            .or(derived.0),
        half_life_range: match (
            row.try_get::<Option<f64>, _>("half_life_range_lo")
                .unwrap_or(None),
            row.try_get::<Option<f64>, _>("half_life_range_hi")
                .unwrap_or(None),
        ) {
            (Some(lo), Some(hi)) => Some((lo, hi)),
            _ => derived.1,
        },
        contraindications: row
            .try_get::<Option<String>, _>("contraindications")
            .unwrap_or(None)
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default(),
        warnings: row
            .try_get::<Option<String>, _>("warnings")
            .unwrap_or(None)
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default(),
        mechanism_tags: row
            .try_get::<Option<String>, _>("mechanism_tags")
            .unwrap_or(None)
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default(),
        is_custom: row.try_get::<i32, _>("is_custom").unwrap_or(0) != 0,
        source: row.try_get::<Option<String>, _>("source").unwrap_or(None),
        version: row.try_get::<i32, _>("version").unwrap_or(1),
    }
}

fn row_to_vitals_entry(row: SqliteRow) -> VitalsEntry {
    VitalsEntry {
        id: Uuid::parse_str(&row.try_get::<String, _>("id").unwrap_or_default())
            .unwrap_or_default(),
        user_id: row.try_get::<String, _>("user_id").unwrap_or_default(),
        timestamp: parse_datetime(&row.try_get::<String, _>("timestamp").unwrap_or_default())
            .unwrap_or_default(),
        bp_systolic: row.try_get::<Option<i32>, _>("bp_systolic").unwrap_or(None),
        bp_diastolic: row
            .try_get::<Option<i32>, _>("bp_diastolic")
            .unwrap_or(None),
        heart_rate: row.try_get::<Option<i32>, _>("heart_rate").unwrap_or(None),
        weight: row.try_get::<Option<f64>, _>("weight").unwrap_or(None),
        blood_glucose: row
            .try_get::<Option<f64>, _>("blood_glucose")
            .unwrap_or(None),
        temperature: row.try_get::<Option<f64>, _>("temperature").unwrap_or(None),
        spo2: row.try_get::<Option<i32>, _>("spo2").unwrap_or(None),
        hrv: row.try_get::<Option<f64>, _>("hrv").unwrap_or(None),
        sleep_quality: row
            .try_get::<Option<String>, _>("sleep_quality")
            .unwrap_or(None)
            .and_then(|s| serde_json::from_str(&s).ok()),
        custom_metrics: row
            .try_get::<Option<String>, _>("custom_metrics")
            .unwrap_or(None)
            .and_then(|s| serde_json::from_str(&s).ok()),
        notes: row.try_get::<Option<String>, _>("notes").unwrap_or(None),
    }
}

fn row_to_alert(row: SqliteRow) -> Alert {
    Alert {
        id: Uuid::parse_str(&row.try_get::<String, _>("id").unwrap_or_default())
            .unwrap_or_default(),
        user_id: row.try_get::<String, _>("user_id").unwrap_or_default(),
        alert_type: AlertType::from_str(&row.try_get::<String, _>("type").unwrap_or_default())
            .unwrap_or_default(),
        severity: AlertSeverity::from_str(
            &row.try_get::<String, _>("severity").unwrap_or_default(),
        )
        .unwrap_or_default(),
        message: row.try_get::<String, _>("message").unwrap_or_default(),
        recommendation: row
            .try_get::<Option<String>, _>("recommendation")
            .unwrap_or(None),
        is_acknowledged: row.try_get::<i32, _>("is_acknowledged").unwrap_or(0) != 0,
        linked_entry_id: row
            .try_get::<Option<String>, _>("linked_entry_id")
            .unwrap_or(None)
            .and_then(|s| Uuid::parse_str(&s).ok()),
        generated_at: parse_datetime(&row.try_get::<String, _>("generated_at").unwrap_or_default())
            .unwrap_or_default(),
        resolved_at: row
            .try_get::<Option<String>, _>("resolved_at")
            .unwrap_or(None)
            .and_then(|s| parse_datetime(&s)),
    }
}

// ── Notes (US7 — standalone first-class entries) ────────────────────────────

pub async fn create_note(pool: &DbPool, note: &Note) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO notes (id, user_id, content, timestamp, linked_entry_id)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(note.id.to_string())
    .bind(&note.user_id)
    .bind(&note.content)
    .bind(note.timestamp.to_rfc3339())
    .bind(note.linked_entry_id.map(|u| u.to_string()))
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_notes(pool: &DbPool, user_id: &str) -> anyhow::Result<Vec<Note>> {
    let rows = sqlx::query(
        "SELECT id, user_id, content, timestamp, linked_entry_id
         FROM notes WHERE user_id = ?1 ORDER BY timestamp DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| Note {
            id: Uuid::parse_str(&row.try_get::<String, _>("id").unwrap_or_default())
                .unwrap_or_default(),
            user_id: row.try_get::<String, _>("user_id").unwrap_or_default(),
            content: row.try_get::<String, _>("content").unwrap_or_default(),
            timestamp: parse_datetime(&row.try_get::<String, _>("timestamp").unwrap_or_default())
                .unwrap_or_default(),
            linked_entry_id: row
                .try_get::<Option<String>, _>("linked_entry_id")
                .unwrap_or(None)
                .and_then(|s| Uuid::parse_str(&s).ok()),
        })
        .collect())
}

pub async fn update_note(pool: &DbPool, note: &Note) -> anyhow::Result<()> {
    sqlx::query("UPDATE notes SET content = ?2, linked_entry_id = ?3 WHERE id = ?1")
        .bind(note.id.to_string())
        .bind(&note.content)
        .bind(note.linked_entry_id.map(|u| u.to_string()))
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_note(pool: &DbPool, id: &Uuid) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM notes WHERE id = ?1")
        .bind(id.to_string())
        .execute(pool)
        .await?;
    Ok(())
}

/// Erase every user-generated collection (FR-020 cloud data deletion).
/// Shared seed content (catalog, substance models) is deliberately kept so a
/// wipe-and-migrate restart still has a working app; `stack_items` disappears
/// via its ON DELETE CASCADE from `stacks`. Returns the number of rows deleted.
pub async fn delete_all_user_data(pool: &DbPool, user_id: &str) -> anyhow::Result<u64> {
    let mut total = 0u64;
    for sql in [
        "DELETE FROM log_entries WHERE user_id = ?1",
        "DELETE FROM vitals_entries WHERE user_id = ?1",
        "DELETE FROM alerts WHERE user_id = ?1",
        "DELETE FROM insights WHERE user_id = ?1",
        "DELETE FROM notes WHERE user_id = ?1",
        "DELETE FROM target_bands WHERE user_id = ?1",
        "DELETE FROM stacks WHERE user_id = ?1",
    ] {
        total += sqlx::query(sql)
            .bind(user_id)
            .execute(pool)
            .await?
            .rows_affected();
    }
    Ok(total)
}

fn row_to_insight(row: SqliteRow) -> Insight {
    Insight {
        id: Uuid::parse_str(&row.try_get::<String, _>("id").unwrap_or_default())
            .unwrap_or_default(),
        user_id: row.try_get::<String, _>("user_id").unwrap_or_default(),
        insight_type: InsightType::from_str(&row.try_get::<String, _>("type").unwrap_or_default())
            .unwrap_or_default(),
        title: row.try_get::<String, _>("title").unwrap_or_default(),
        description: row.try_get::<String, _>("description").unwrap_or_default(),
        confidence: row.try_get::<f64, _>("confidence").unwrap_or(0.0),
        supporting_data_points: row.try_get::<i32, _>("supporting_data_points").unwrap_or(0),
        generated_at: parse_datetime(&row.try_get::<String, _>("generated_at").unwrap_or_default())
            .unwrap_or_default(),
        related_entry_ids: row
            .try_get::<Option<String>, _>("related_entry_ids")
            .unwrap_or(None)
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default(),
    }
}
