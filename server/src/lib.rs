//! biohack2-server: persistent backend for the biohack2 web app.
//!
//! Serves the static frontend from `dist/` and a REST API backed by the
//! engine's SQLite layer. This process owns the SQLite file — the frontend's
//! localStorage becomes a cache, not the source of truth.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use engine::db::{self, DbPool};
use engine::models::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

pub const DEFAULT_USER_ID: &str = "local-device";
pub const DEFAULT_DB_FILE: &str = "biohack.db";

#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn err_response(status: StatusCode, msg: &str) -> Response {
    (status, Json(serde_json::json!({ "error": msg }))).into_response()
}

fn internal(e: anyhow::Error) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    )
        .into_response()
}

fn bad_request(msg: &str) -> Response {
    err_response(StatusCode::BAD_REQUEST, msg)
}

// ── query param structs ───────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct EntriesQuery {
    pub user_id: Option<String>,
    pub stack_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub category: Option<String>,
}

fn parse_filter(q: &EntriesQuery) -> Result<LogEntryFilter, Response> {
    let mut f = LogEntryFilter {
        user_id: q.user_id.clone().or(Some(DEFAULT_USER_ID.into())),
        ..Default::default()
    };
    if let Some(ref sid) = q.stack_id {
        f.stack_id = Some(sid.parse().map_err(|_| bad_request("invalid stack_id"))?);
    }
    if let Some(ref sd) = q.start_date {
        f.start_date = Some(
            sd.parse()
                .map_err(|_| bad_request("invalid start_date (RFC3339 expected)"))?,
        );
    }
    if let Some(ref ed) = q.end_date {
        f.end_date = Some(
            ed.parse()
                .map_err(|_| bad_request("invalid end_date (RFC3339 expected)"))?,
        );
    }
    if let Some(ref cat) = q.category {
        let cat = cat.to_lowercase();
        f.category = Some(ItemType::from_str(&cat).ok_or_else(|| bad_request(&format!("unknown category: {cat}")))?);
    }
    Ok(f)
}

#[derive(Deserialize)]
pub struct VitalsQuery {
    pub user_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Deserialize)]
pub struct AlertsQuery {
    pub user_id: Option<String>,
    pub acknowledged: Option<bool>,
}

// ── handlers: log entries ─────────────────────────────────────────────────────

async fn list_log_entries(
    State(state): State<AppState>,
    Query(q): Query<EntriesQuery>,
) -> Result<Json<Vec<LogEntry>>, Response> {
    let filter = parse_filter(&q)?;
    db::get_log_entries(&state.pool, &filter)
        .await
        .map(Json)
        .map_err(internal)
}

async fn create_log_entry(
    State(state): State<AppState>,
    Json(entry): Json<LogEntry>,
) -> Result<(StatusCode, Json<LogEntry>), Response> {
    db::create_log_entry(&state.pool, &entry)
        .await
        .map(|_| (StatusCode::CREATED, Json(entry)))
        .map_err(internal)
}

async fn delete_log_entry(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, Response> {
    let uuid = id.parse().map_err(|_| bad_request("invalid id"))?;
    db::delete_log_entry(&state.pool, &uuid)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(internal)
}

// ── handlers: catalog ─────────────────────────────────────────────────────────

async fn list_catalog(State(state): State<AppState>) -> Result<Json<Vec<CatalogItem>>, Response> {
    db::get_catalog_items(&state.pool)
        .await
        .map(Json)
        .map_err(internal)
}

async fn search_catalog(
    State(state): State<AppState>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Json<Vec<CatalogItem>>, Response> {
    let query = q.get("q").cloned().unwrap_or_default();
    db::search_catalog(&state.pool, &query)
        .await
        .map(Json)
        .map_err(internal)
}

// ── handlers: stacks ──────────────────────────────────────────────────────────

async fn list_stacks(State(state): State<AppState>) -> Result<Json<Vec<Stack>>, Response> {
    db::get_stacks(&state.pool, DEFAULT_USER_ID)
        .await
        .map(Json)
        .map_err(internal)
}

async fn create_stack(
    State(state): State<AppState>,
    Json(stack): Json<Stack>,
) -> Result<(StatusCode, Json<Stack>), Response> {
    db::create_stack(&state.pool, &stack)
        .await
        .map(|_| (StatusCode::CREATED, Json(stack)))
        .map_err(internal)
}

async fn update_stack(
    State(state): State<AppState>,
    Json(stack): Json<Stack>,
) -> Result<StatusCode, Response> {
    db::update_stack(&state.pool, &stack)
        .await
        .map(|_| StatusCode::OK)
        .map_err(internal)
}

async fn delete_stack(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, Response> {
    let uuid = id.parse().map_err(|_| bad_request("invalid id"))?;
    db::delete_stack(&state.pool, &uuid)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(internal)
}

// ── handlers: vitals ──────────────────────────────────────────────────────────

async fn list_vitals(
    State(state): State<AppState>,
    Query(q): Query<VitalsQuery>,
) -> Result<Json<Vec<VitalsEntry>>, Response> {
    let mut filter = VitalsEntryFilter {
        user_id: q.user_id.clone().or(Some(DEFAULT_USER_ID.into())),
        ..Default::default()
    };
    if let Some(ref sd) = q.start_date {
        filter.start_date = Some(sd.parse().map_err(|_| bad_request("invalid start_date"))?);
    }
    if let Some(ref ed) = q.end_date {
        filter.end_date = Some(ed.parse().map_err(|_| bad_request("invalid end_date"))?);
    }
    db::get_vitals_entries(&state.pool, &filter)
        .await
        .map(Json)
        .map_err(internal)
}

async fn create_vitals_entry(
    State(state): State<AppState>,
    Json(entry): Json<VitalsEntry>,
) -> Result<(StatusCode, Json<VitalsEntry>), Response> {
    db::create_vitals_entry(&state.pool, &entry)
        .await
        .map(|_| (StatusCode::CREATED, Json(entry)))
        .map_err(internal)
}

// ── handlers: alerts ──────────────────────────────────────────────────────────

async fn list_alerts(
    State(state): State<AppState>,
    Query(q): Query<AlertsQuery>,
) -> Result<Json<Vec<Alert>>, Response> {
    let filter = AlertFilter {
        user_id: q.user_id.clone().or(Some(DEFAULT_USER_ID.into())),
        acknowledged: q.acknowledged,
    };
    db::get_alerts(&state.pool, &filter)
        .await
        .map(Json)
        .map_err(internal)
}

async fn create_alert(
    State(state): State<AppState>,
    Json(alert): Json<Alert>,
) -> Result<(StatusCode, Json<Alert>), Response> {
    db::create_alert(&state.pool, &alert)
        .await
        .map(|_| (StatusCode::CREATED, Json(alert)))
        .map_err(internal)
}

async fn acknowledge_alert(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, Response> {
    let uuid = id.parse().map_err(|_| bad_request("invalid id"))?;
    db::acknowledge_alert(&state.pool, &uuid)
        .await
        .map(|_| StatusCode::OK)
        .map_err(internal)
}

// ── handlers: notes ───────────────────────────────────────────────────────────

async fn list_notes(State(state): State<AppState>) -> Result<Json<Vec<Note>>, Response> {
    db::get_notes(&state.pool, DEFAULT_USER_ID)
        .await
        .map(Json)
        .map_err(internal)
}

async fn create_note(
    State(state): State<AppState>,
    Json(note): Json<Note>,
) -> Result<(StatusCode, Json<Note>), Response> {
    db::create_note(&state.pool, &note)
        .await
        .map(|_| (StatusCode::CREATED, Json(note)))
        .map_err(internal)
}

async fn update_note(
    State(state): State<AppState>,
    Json(note): Json<Note>,
) -> Result<StatusCode, Response> {
    db::update_note(&state.pool, &note)
        .await
        .map(|_| StatusCode::OK)
        .map_err(internal)
}

async fn delete_note(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, Response> {
    let uuid = id.parse().map_err(|_| bad_request("invalid id"))?;
    db::delete_note(&state.pool, &uuid)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(internal)
}

// ── handlers: safety (server-side check so rules stay in the engine) ─────────

async fn check_vitals_safety(
    State(state): State<AppState>,
    Json(entry): Json<VitalsEntry>,
) -> Result<Json<serde_json::Value>, Response> {
    let recent = db::get_log_entries(
        &state.pool,
        &LogEntryFilter {
            user_id: Some(DEFAULT_USER_ID.into()),
            ..Default::default()
        },
    )
    .await
    .map_err(internal)?;

    let substances = engine::safety::RecentSubstance::from_log_entries(&recent);

    let engine_check = engine::safety::SafetyEngine::new();
    let result = engine_check.check_vitals(&entry, &substances);

    // Enrich with contextual advice from the same log set
    let alerts: Vec<Alert> = result
        .alerts
        .into_iter()
        .map(|mut alert| {
            if let Some(advice) = engine::safety::contextual_advice(&alert, &substances) {
                let rec = alert.recommendation.clone().unwrap_or_default();
                alert.recommendation = Some(format!("{rec} Context: {advice}"));
            }
            alert
        })
        .collect();

    Ok(Json(serde_json::json!({ "alerts": alerts })))
}

// ── handlers: system ──────────────────────────────────────────────────────────

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn sync_pull(State(state): State<AppState>) -> Result<Json<serde_json::Value>, Response> {
    // Full-state pull for frontend hydration and localStorage migration.
    let log_filter = LogEntryFilter {
        user_id: Some(DEFAULT_USER_ID.into()),
        ..Default::default()
    };
    let vitals_filter = VitalsEntryFilter {
        user_id: Some(DEFAULT_USER_ID.into()),
        ..Default::default()
    };
    let alert_filter = AlertFilter {
        user_id: Some(DEFAULT_USER_ID.into()),
        acknowledged: None,
    };
    let (entries, vitals, alerts, stacks, notes, catalog) = tokio::try_join!(
        db::get_log_entries(&state.pool, &log_filter),
        db::get_vitals_entries(&state.pool, &vitals_filter),
        db::get_alerts(&state.pool, &alert_filter),
        db::get_stacks(&state.pool, DEFAULT_USER_ID),
        db::get_notes(&state.pool, DEFAULT_USER_ID),
        db::get_catalog_items(&state.pool),
    )
    .map_err(internal)?;

    Ok(Json(serde_json::json!({
        "log_entries": entries,
        "vitals_entries": vitals,
        "alerts": alerts,
        "stacks": stacks,
        "notes": notes,
        "catalog": catalog,
    })))
}

async fn sync_push(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, Response> {
    // Bulk upsert: one-time localStorage → server migration.
    // Entries are keyed by id; existing rows with the same id are replaced.
    let mut counts = serde_json::Map::new();

    counts.insert(
        "log_entries".into(),
        upsert_entity::<LogEntry, _>(&state.pool, &payload, "log_entries", |p, e| {
            Box::pin(async move { db::create_log_entry(p, &e).await })
        })
        .await
        .into(),
    );
    counts.insert(
        "vitals_entries".into(),
        upsert_entity::<VitalsEntry, _>(&state.pool, &payload, "vitals_entries", |p, e| {
            Box::pin(async move { db::create_vitals_entry(p, &e).await })
        })
        .await
        .into(),
    );
    counts.insert(
        "alerts".into(),
        upsert_entity::<Alert, _>(&state.pool, &payload, "alerts", |p, e| {
            Box::pin(async move { db::create_alert(p, &e).await })
        })
        .await
        .into(),
    );
    counts.insert(
        "stacks".into(),
        upsert_entity::<Stack, _>(&state.pool, &payload, "stacks", |p, e| {
            Box::pin(async move { db::create_stack(p, &e).await })
        })
        .await
        .into(),
    );
    counts.insert(
        "notes".into(),
        upsert_entity::<Note, _>(&state.pool, &payload, "notes", |p, e| {
            Box::pin(async move { db::create_note(p, &e).await })
        })
        .await
        .into(),
    );
    if let Some(catalog) = payload.get("catalog").and_then(|v| v.as_array()) {
        let items: Vec<CatalogItem> = catalog
            .iter()
            .filter_map(|c| serde_json::from_value(c.clone()).ok())
            .collect();
        if db::seed_catalog(&state.pool, &items).await.is_ok() {
            counts.insert("catalog".into(), items.len().into());
        }
    }

    Ok(Json(serde_json::Value::Object(counts)))
}

/// Deserialize every element under `key` and insert it via `insert`.
///
/// The five entity collections in the migration payload differ only in their
/// type and the insert function; the count-and-skip-bad-rows loop is identical.
/// Returns the number of rows successfully inserted so the caller can report it.
async fn upsert_entity<'a, T, Fut>(
    pool: &'a DbPool,
    payload: &serde_json::Value,
    key: &str,
    insert: impl Fn(&'a DbPool, T) -> Fut + 'a,
) -> u64
where
    T: serde::de::DeserializeOwned,
    Fut: std::future::Future<Output = anyhow::Result<()>> + 'a,
{
    let Some(arr) = payload.get(key).and_then(|v| v.as_array()) else {
        return 0;
    };
    let mut n = 0u64;
    for e in arr {
        if let Ok(row) = serde_json::from_value::<T>(e.clone())
            && insert(pool, row).await.is_ok()
        {
            n += 1;
        }
    }
    n
}

// ── SPA static serving with API fallback discipline ──────────────────────────

async fn serve_index(State(state): State<AppState>) -> Response {
    let _ = state;
    let path = std::path::Path::new("dist/index.html");
    match tokio::fs::read(path).await {
        Ok(bytes) => {
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, "text/html".parse().unwrap());
            (StatusCode::OK, headers, bytes).into_response()
        }
        Err(_) => err_response(
            StatusCode::NOT_FOUND,
            "dist/index.html not found — run `bash build-web.sh` first",
        ),
    }
}

// ── router ────────────────────────────────────────────────────────────────────

pub fn build_router(pool: DbPool, dist_dir: Option<&str>) -> Router {
    let state = AppState { pool };

    let api = Router::new()
        .route("/log-entries", get(list_log_entries).post(create_log_entry))
        .route("/log-entries/{id}", delete(delete_log_entry))
        .route("/catalog", get(list_catalog))
        .route("/catalog/search", get(search_catalog))
        .route(
            "/stacks",
            get(list_stacks).post(create_stack).put(update_stack),
        )
        .route("/stacks/{id}", delete(delete_stack))
        .route("/vitals", get(list_vitals).post(create_vitals_entry))
        .route("/alerts", get(list_alerts).post(create_alert))
        .route("/alerts/{id}/acknowledge", post(acknowledge_alert))
        .route("/notes", get(list_notes).post(create_note).put(update_note))
        .route("/notes/{id}", delete(delete_note))
        .route("/safety/check-vitals", post(check_vitals_safety))
        .route("/sync/pull", get(sync_pull))
        .route("/sync/push", post(sync_push))
        .route("/health", get(health));

    let mut app = Router::new().nest("/api", api);

    match dist_dir {
        Some(dir) => {
            app = app.fallback_service(ServeDir::new(dir).append_index_html_on_directories(true));
        }
        None => {
            app = app.route("/", get(serve_index)).fallback(serve_index);
        }
    }

    app.with_state(state)
}

pub async fn run(pool: DbPool, addr: SocketAddr, dist_dir: Option<&str>) -> anyhow::Result<()> {
    let app = build_router(pool, dist_dir);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
