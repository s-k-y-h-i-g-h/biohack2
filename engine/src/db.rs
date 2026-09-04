use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqliteRow, SqlitePool, Row};
use uuid::Uuid;

use crate::models::*;

pub type DbPool = SqlitePool;

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

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_catalog_category ON catalog_items(category)"
    )
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
        )"
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
        )"
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
        )"
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
            type TEXT NOT NULL CHECK(type IN ('vital','interaction','warning')),
            severity TEXT NOT NULL CHECK(severity IN ('info','warning','critical')),
            message TEXT NOT NULL,
            recommendation TEXT,
            is_acknowledged INTEGER NOT NULL DEFAULT 0,
            linked_entry_id TEXT,
            generated_at TEXT NOT NULL,
            resolved_at TEXT
        )"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_alerts_user_unack ON alerts(user_id, is_acknowledged)"
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
        )"
    )
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
    .bind(serde_json::to_string(&entry.item_type)?)
    .bind(entry.item_id.map(|u| u.to_string()))
    .bind(&entry.name)
    .bind(entry.quantity)
    .bind(entry.unit.clone())
    .bind(serde_json::to_string(&entry.route)?)
    .bind(entry.timestamp.to_rfc3339())
    .bind(entry.stack_id.map(|u| u.to_string()))
    .bind(entry.notes.clone())
    .bind(entry.acknowledged_interaction as i32)
    .bind(serde_json::to_string(&entry.custom_fields)?)
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

pub async fn get_log_entries(pool: &DbPool, filter: &LogEntryFilter) -> anyhow::Result<Vec<LogEntry>> {
    let mut query = String::from("SELECT id, user_id, item_type, item_id, name, quantity, unit, route, timestamp, stack_id, notes, acknowledged_interaction, custom_fields FROM log_entries WHERE 1=1");
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
        bind_vars.push(serde_json::to_string(cat)?);
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
    sqlx::query(
        "UPDATE log_entries SET notes = ?2, acknowledged_interaction = ?3 WHERE id = ?1"
    )
    .bind(entry.id.to_string())
    .bind(entry.notes.clone())
    .bind(entry.acknowledged_interaction as i32)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_log_entry(pool: &DbPool, id: &Uuid) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM log_entries WHERE id = ?1").bind(id.to_string()).execute(pool).await?;
    Ok(())
}

// ── CatalogItem CRUD ──────────────────────────────────────────────────────────

pub async fn seed_catalog(pool: &DbPool, items: &[CatalogItem]) -> anyhow::Result<()> {
    for item in items {
        sqlx::query(
            "INSERT OR IGNORE INTO catalog_items (id, name, category, dosage_range, half_life, contraindications, warnings, is_custom, source, version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
        )
        .bind(item.id.to_string())
        .bind(&item.name)
        .bind(serde_json::to_string(&item.category)?)
        .bind(serde_json::to_string(&item.dosage_range)?)
        .bind(item.half_life.as_ref().map(|s| s.as_str()))
        .bind(serde_json::to_string(&item.contraindications)?)
        .bind(serde_json::to_string(&item.warnings)?)
        .bind(item.is_custom as i32)
        .bind(item.source.as_ref().map(|s| s.as_str()))
        .bind(item.version)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn get_catalog_items(pool: &DbPool) -> anyhow::Result<Vec<CatalogItem>> {
    let rows = sqlx::query(
        "SELECT id, name, category, dosage_range, half_life, contraindications, warnings, is_custom, source, version FROM catalog_items"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(row_to_catalog_item).collect())
}

pub async fn search_catalog(pool: &DbPool, query: &str) -> anyhow::Result<Vec<CatalogItem>> {
    let rows = sqlx::query(
        "SELECT id, name, category, dosage_range, half_life, contraindications, warnings, is_custom, source, version FROM catalog_items WHERE name LIKE ?1"
    )
    .bind(format!("%{}%", query))
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(row_to_catalog_item).collect())
}

// ── Stack CRUD ────────────────────────────────────────────────────────────────

pub async fn create_stack(pool: &DbPool, stack: &Stack) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO stacks (id, user_id, name, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
    )
    .bind(stack.id.to_string())
    .bind(&stack.user_id)
    .bind(&stack.name)
    .bind(stack.description.as_ref().map(|s| s.as_str()))
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
        "SELECT id, user_id, name, description, created_at, updated_at FROM stacks WHERE id = ?1"
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await?;

    match row {
        None => Ok(None),
        Some(r) => {
            let items_rows = sqlx::query(
                "SELECT item_id, quantity, unit, note FROM stack_items WHERE stack_id = ?1"
            )
            .bind(id.to_string())
            .fetch_all(pool)
            .await?;

            let stack = Stack {
                id: Uuid::parse_str(&r.try_get::<String, _>("id")?).unwrap_or_default(),
                user_id: r.try_get::<String, _>("user_id")?,
                name: r.try_get::<String, _>("name")?,
                description: r.try_get::<Option<String>, _>("description")?,
                created_at: parse_datetime(&r.try_get::<String, _>("created_at")?).unwrap_or_default(),
                updated_at: parse_datetime(&r.try_get::<String, _>("updated_at")?).unwrap_or_default(),
                items: items_rows
                    .into_iter()
                    .map(|row| StackItem {
                        item_id: Uuid::parse_str(&row.try_get::<String, _>("item_id").unwrap_or_default()).unwrap_or_default(),
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
            "SELECT item_id, quantity, unit, note FROM stack_items WHERE stack_id = ?1"
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
                    item_id: Uuid::parse_str(&row.try_get::<String, _>("item_id").unwrap_or_default()).unwrap_or_default(),
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
    sqlx::query(
        "UPDATE stacks SET name = ?2, description = ?3, updated_at = ?4 WHERE id = ?1"
    )
    .bind(stack.id.to_string())
    .bind(&stack.name)
    .bind(stack.description.as_ref().map(|s| s.as_str()))
    .bind(stack.updated_at.to_rfc3339())
    .execute(pool)
    .await?;

    sqlx::query("DELETE FROM stack_items WHERE stack_id = ?1").bind(stack.id.to_string()).execute(pool).await?;
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
    sqlx::query("DELETE FROM stack_items WHERE stack_id = ?1").bind(id.to_string()).execute(pool).await?;
    sqlx::query("DELETE FROM stacks WHERE id = ?1").bind(id.to_string()).execute(pool).await?;
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

pub async fn get_vitals_entries(pool: &DbPool, filter: &VitalsEntryFilter) -> anyhow::Result<Vec<VitalsEntry>> {
    let mut query = String::from("SELECT id, user_id, timestamp, bp_systolic, bp_diastolic, heart_rate, weight, blood_glucose, temperature, spo2, hrv, sleep_quality, custom_metrics, notes FROM vitals_entries WHERE 1=1");
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
    .bind(serde_json::to_string(&alert.alert_type)?)
    .bind(serde_json::to_string(&alert.severity)?)
    .bind(&alert.message)
    .bind(alert.recommendation.as_ref().map(|s| s.as_str()))
    .bind(alert.is_acknowledged as i32)
    .bind(alert.linked_entry_id.map(|u| u.to_string()))
    .bind(alert.generated_at.to_rfc3339())
    .bind(alert.resolved_at.as_ref().map(|d| d.to_rfc3339()))
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_alerts(pool: &DbPool, filter: &AlertFilter) -> anyhow::Result<Vec<Alert>> {
    let mut query = String::from("SELECT id, user_id, type, severity, message, recommendation, is_acknowledged, linked_entry_id, generated_at, resolved_at FROM alerts WHERE 1=1");
    let mut bind_vars: Vec<String> = Vec::new();

    if let Some(ref uid) = filter.user_id {
        query.push_str(" AND user_id = ?");
        bind_vars.push(uid.clone());
    }
    if let Some(ack) = filter.acknowledged {
        query.push_str(" AND is_acknowledged = ?");
        bind_vars.push(ack.to_string());
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
    sqlx::query(
        "UPDATE alerts SET is_acknowledged = 1, resolved_at = ?1 WHERE id = ?2"
    )
    .bind(Utc::now().to_rfc3339())
    .bind(id.to_string())
    .execute(pool)
    .await?;
    Ok(())
}

// ── Insight CRUD ──────────────────────────────────────────────────────────────

pub async fn create_insight(pool: &DbPool, insight: &Insight) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO insights (id, user_id, type, title, description, confidence, supporting_data_points, generated_at, related_entry_ids)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
    )
    .bind(insight.id.to_string())
    .bind(&insight.user_id)
    .bind(serde_json::to_string(&insight.insight_type)?)
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
        id: Uuid::parse_str(&row.try_get::<String, _>("id").unwrap_or_default()).unwrap_or_default(),
        user_id: row.try_get::<String, _>("user_id").unwrap_or_default(),
        item_type: serde_json::from_str(&row.try_get::<String, _>("item_type").unwrap_or_default()).unwrap_or_default(),
        item_id: row.try_get::<Option<String>, _>("item_id").unwrap_or(None).and_then(|s| Uuid::parse_str(&s).ok()),
        name: row.try_get::<String, _>("name").unwrap_or_default(),
        quantity: row.try_get::<Option<f64>, _>("quantity").unwrap_or(None),
        unit: row.try_get::<Option<String>, _>("unit").unwrap_or(None),
        route: row.try_get::<Option<String>, _>("route").unwrap_or(None).and_then(|s| serde_json::from_str(&s).ok()),
        timestamp: parse_datetime(&row.try_get::<String, _>("timestamp").unwrap_or_default()).unwrap_or_default(),
        stack_id: row.try_get::<Option<String>, _>("stack_id").unwrap_or(None).and_then(|s| Uuid::parse_str(&s).ok()),
        notes: row.try_get::<Option<String>, _>("notes").unwrap_or(None),
        acknowledged_interaction: row.try_get::<i32, _>("acknowledged_interaction").unwrap_or(0) != 0,
        custom_fields: row.try_get::<Option<String>, _>("custom_fields").unwrap_or(None).and_then(|s| serde_json::from_str(&s).ok()),
    }
}

fn row_to_catalog_item(row: SqliteRow) -> CatalogItem {
    CatalogItem {
        id: Uuid::parse_str(&row.try_get::<String, _>("id").unwrap_or_default()).unwrap_or_default(),
        name: row.try_get::<String, _>("name").unwrap_or_default(),
        category: serde_json::from_str(&row.try_get::<String, _>("category").unwrap_or_default()).unwrap_or_default(),
        dosage_range: row.try_get::<Option<String>, _>("dosage_range").unwrap_or(None).and_then(|s| serde_json::from_str(&s).ok()),
        half_life: row.try_get::<Option<String>, _>("half_life").unwrap_or(None),
        contraindications: row.try_get::<Option<String>, _>("contraindications").unwrap_or(None).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default(),
        warnings: row.try_get::<Option<String>, _>("warnings").unwrap_or(None).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default(),
        is_custom: row.try_get::<i32, _>("is_custom").unwrap_or(0) != 0,
        source: row.try_get::<Option<String>, _>("source").unwrap_or(None),
        version: row.try_get::<i32, _>("version").unwrap_or(1),
    }
}

fn row_to_vitals_entry(row: SqliteRow) -> VitalsEntry {
    VitalsEntry {
        id: Uuid::parse_str(&row.try_get::<String, _>("id").unwrap_or_default()).unwrap_or_default(),
        user_id: row.try_get::<String, _>("user_id").unwrap_or_default(),
        timestamp: parse_datetime(&row.try_get::<String, _>("timestamp").unwrap_or_default()).unwrap_or_default(),
        bp_systolic: row.try_get::<Option<i32>, _>("bp_systolic").unwrap_or(None),
        bp_diastolic: row.try_get::<Option<i32>, _>("bp_diastolic").unwrap_or(None),
        heart_rate: row.try_get::<Option<i32>, _>("heart_rate").unwrap_or(None),
        weight: row.try_get::<Option<f64>, _>("weight").unwrap_or(None),
        blood_glucose: row.try_get::<Option<f64>, _>("blood_glucose").unwrap_or(None),
        temperature: row.try_get::<Option<f64>, _>("temperature").unwrap_or(None),
        spo2: row.try_get::<Option<i32>, _>("spo2").unwrap_or(None),
        hrv: row.try_get::<Option<f64>, _>("hrv").unwrap_or(None),
        sleep_quality: row.try_get::<Option<String>, _>("sleep_quality").unwrap_or(None).and_then(|s| serde_json::from_str(&s).ok()),
        custom_metrics: row.try_get::<Option<String>, _>("custom_metrics").unwrap_or(None).and_then(|s| serde_json::from_str(&s).ok()),
        notes: row.try_get::<Option<String>, _>("notes").unwrap_or(None),
    }
}

fn row_to_alert(row: SqliteRow) -> Alert {
    Alert {
        id: Uuid::parse_str(&row.try_get::<String, _>("id").unwrap_or_default()).unwrap_or_default(),
        user_id: row.try_get::<String, _>("user_id").unwrap_or_default(),
        alert_type: serde_json::from_str(&row.try_get::<String, _>("type").unwrap_or_default()).unwrap_or_default(),
        severity: serde_json::from_str(&row.try_get::<String, _>("severity").unwrap_or_default()).unwrap_or_default(),
        message: row.try_get::<String, _>("message").unwrap_or_default(),
        recommendation: row.try_get::<Option<String>, _>("recommendation").unwrap_or(None),
        is_acknowledged: row.try_get::<i32, _>("is_acknowledged").unwrap_or(0) != 0,
        linked_entry_id: row.try_get::<Option<String>, _>("linked_entry_id").unwrap_or(None).and_then(|s| Uuid::parse_str(&s).ok()),
        generated_at: parse_datetime(&row.try_get::<String, _>("generated_at").unwrap_or_default()).unwrap_or_default(),
        resolved_at: row.try_get::<Option<String>, _>("resolved_at").unwrap_or(None).and_then(|s| parse_datetime(&s)),
    }
}

fn row_to_insight(row: SqliteRow) -> Insight {
    Insight {
        id: Uuid::parse_str(&row.try_get::<String, _>("id").unwrap_or_default()).unwrap_or_default(),
        user_id: row.try_get::<String, _>("user_id").unwrap_or_default(),
        insight_type: serde_json::from_str(&row.try_get::<String, _>("type").unwrap_or_default()).unwrap_or_default(),
        title: row.try_get::<String, _>("title").unwrap_or_default(),
        description: row.try_get::<String, _>("description").unwrap_or_default(),
        confidence: row.try_get::<f64, _>("confidence").unwrap_or(0.0),
        supporting_data_points: row.try_get::<i32, _>("supporting_data_points").unwrap_or(0),
        generated_at: parse_datetime(&row.try_get::<String, _>("generated_at").unwrap_or_default()).unwrap_or_default(),
        related_entry_ids: row.try_get::<Option<String>, _>("related_entry_ids").unwrap_or(None).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default(),
    }
}
