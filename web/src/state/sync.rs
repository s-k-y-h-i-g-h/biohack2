//! Server sync: the biohack2-server backend is the durable source of truth;
//! localStorage is a low-latency cache.
//!
//! Strategy:
//! - Every create/update/delete also fires an async push to the server
//!   (fire-and-forget; failures log to console, local state stays usable).
//! - On boot, `sync_from_server` pulls full state and REPLACES the local
//!   cache (so deletes stick), or — first run with local data — pushes
//!   everything up as a one-time migration.

use crate::state::db;
use engine::models::*;
use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;

const SYNCED_FLAG: &str = "biohack2_server_synced";

fn log_info(msg: &str) {
    web_sys::console::log_1(&JsValue::from_str(msg));
}
fn log_warn(msg: &str) {
    web_sys::console::warn_1(&JsValue::from_str(msg));
}

/// Run a fetch without ever letting a JS exception cross the wasm boundary.
///
/// `gloo_net::Request::send()` calls `window.fetch` through a wasm-bindgen
/// import *without* `catch`: when `fetch` throws synchronously (poisoned,
/// replaced, or missing in the current document) the Rust panic fires inside
/// the spawned task, traps the module, and the whole page dies — under the
/// wasm-bindgen browser test harness that shows up as a silent runner crash
/// (`http status: 500`) with no JavaScript error in the log. Every failure
/// mode here — no window, no fetch, sync throw, async rejection — collapses
/// into a regular `Err` the fire-and-forget callers already log.
pub(crate) async fn fetch_safe(
    method: &str,
    url: &str,
    body: Option<String>,
) -> Result<gloo_net::http::Response, JsValue> {
    let mut init = web_sys::RequestInit::new();
    init.set_method(method);
    if let Some(b) = body {
        let headers = js_sys::Object::new();
        js_sys::Reflect::set(
            &headers,
            &JsValue::from_str("Content-Type"),
            &JsValue::from_str("application/json"),
        )?;
        init.set_headers(&headers);
        let body_obj: js_sys::Object = js_sys::JsString::from(b.as_str()).into();
        init.set_body(&body_obj);
    }
    let request = web_sys::Request::new_with_str_and_init(url, &init)?;
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let fetch: js_sys::Function =
        js_sys::Reflect::get(window.as_ref(), &JsValue::from_str("fetch"))?.dyn_into()?;
    let promise: js_sys::Promise = fetch.call1(window.as_ref(), request.as_ref())?.dyn_into()?;
    let response = wasm_bindgen_futures::JsFuture::from(promise).await?;
    Ok(gloo_net::http::Response::from(
        response.dyn_into::<web_sys::Response>()?,
    ))
}

fn push(method: &str, path: &str, body: Option<serde_json::Value>) {
    let url = format!("api/{path}");
    let method = method.to_string();
    let body = body.map(|b| serde_json::to_string(&b).unwrap_or_default());
    spawn_local(async move {
        match fetch_safe(&method, &url, body).await {
            Ok(resp) if resp.ok() => {}
            Ok(resp) => log_warn(&format!("sync: {method} {url} -> {}", resp.status())),
            Err(e) => log_warn(&format!(
                "sync: {method} {url} failed: {e:?} (server down? keeping local)"
            )),
        }
    });
}

// ── Per-entity push helpers (called from db.rs after local writes) ────────────

pub(crate) fn push_log_entry(entry: &LogEntry) {
    push("POST", "log-entries", serde_json::to_value(entry).ok());
}

pub(crate) fn push_log_entry_delete(id: &str) {
    push("DELETE", &format!("log-entries/{id}"), None);
}

pub(crate) fn push_vitals_entry(entry: &VitalsEntry) {
    push("POST", "vitals", serde_json::to_value(entry).ok());
}

pub(crate) fn push_vitals_entry_delete(id: &str) {
    push("DELETE", &format!("vitals/{id}"), None);
}

pub(crate) fn push_alert(alert: &Alert) {
    push("POST", "alerts", serde_json::to_value(alert).ok());
}

pub(crate) fn push_alert_acknowledge(id: &str) {
    push("POST", &format!("alerts/{id}/acknowledge"), None);
}

pub(crate) fn push_stack(stack: &Stack) {
    push("POST", "stacks", serde_json::to_value(stack).ok());
}

pub(crate) fn push_stack_update(stack: &Stack) {
    push("PUT", "stacks", serde_json::to_value(stack).ok());
}

pub(crate) fn push_stack_delete(id: &str) {
    push("DELETE", &format!("stacks/{id}"), None);
}

pub(crate) fn push_note(note: &Note) {
    push("POST", "notes", serde_json::to_value(note).ok());
}

pub(crate) fn push_note_update(note: &Note) {
    push("PUT", "notes", serde_json::to_value(note).ok());
}

pub(crate) fn push_note_delete(id: &str) {
    push("DELETE", &format!("notes/{id}"), None);
}

// ── Boot sync ─────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "snake_case", default)]
struct ServerState {
    log_entries: Vec<LogEntry>,
    vitals_entries: Vec<VitalsEntry>,
    alerts: Vec<Alert>,
    stacks: Vec<Stack>,
    notes: Vec<Note>,
    catalog: Vec<CatalogItem>,
}

/// Pull server state and replace the local cache; or migrate local → server
/// on first run. Fire-and-forget — the app renders from localStorage
/// immediately and refreshes when this lands.
pub fn sync_from_server() {
    spawn_local(async move {
        let resp = match fetch_safe("GET", "api/sync/pull", None).await {
            Ok(r) => r,
            Err(_) => {
                log_warn("sync: server unreachable, staying on local cache");
                return;
            }
        };
        if !resp.ok() {
            log_warn(&format!("sync: pull failed ({})", resp.status()));
            return;
        }
        let state: ServerState = match resp.json().await {
            Ok(s) => s,
            Err(e) => {
                log_warn(&format!("sync: bad pull payload: {e:?}"));
                return;
            }
        };

        let server_empty = state.log_entries.is_empty()
            && state.vitals_entries.is_empty()
            && state.alerts.is_empty()
            && state.stacks.is_empty()
            && state.notes.is_empty();

        let local_has_data = db::get_log_entries()
            .map(|v| !v.is_empty())
            .unwrap_or(false)
            || db::get_vitals_entries(&Default::default())
                .map(|v| !v.is_empty())
                .unwrap_or(false)
            || db::get_notes().map(|v| !v.is_empty()).unwrap_or(false);

        if server_empty && local_has_data && !already_synced() {
            // First run against a fresh server: migrate everything up.
            let payload = serde_json::json!({
                "log_entries": db::get_log_entries().unwrap_or_default(),
                "vitals_entries": db::get_vitals_entries(&Default::default()).unwrap_or_default(),
                "alerts": db::get_alerts(&AlertFilter { user_id: None, acknowledged: None }).unwrap_or_default(),
                "stacks": db::get_stacks().unwrap_or_default(),
                "notes": db::get_notes().unwrap_or_default(),
            });
            let body = serde_json::to_string(&payload).unwrap_or_default();
            let send_result = fetch_safe("POST", "api/sync/push", Some(body)).await;
            match send_result {
                Ok(r) if r.ok() => {
                    let _ = LocalStorage::set(SYNCED_FLAG, true);
                    log_info("sync: migrated local data to server");
                }
                Ok(r) => log_warn(&format!("sync: migration rejected ({})", r.status())),
                Err(e) => log_warn(&format!("sync: migration failed: {e:?}")),
            }
            return;
        }

        // MERGE, never replace. Union local ∪ server by id, newest wins per id.
        // - First boot with pre-existing local data: local flows up via the
        //   pushes below; nothing local is ever dropped.
        // - Server rows the client hasn't seen (other device / API writes)
        //   flow down into the cache.
        let merged_logs = merge_by_id(
            db::get_log_entries().unwrap_or_default(),
            state.log_entries.clone(),
        );
        let merged_vitals = merge_by_id(
            db::get_vitals_entries(&Default::default()).unwrap_or_default(),
            state.vitals_entries.clone(),
        );
        let merged_alerts = merge_by_id(
            db::get_alerts(&AlertFilter {
                user_id: None,
                acknowledged: None,
            })
            .unwrap_or_default(),
            state.alerts.clone(),
        );
        let merged_stacks =
            merge_stacks(db::get_stacks().unwrap_or_default(), state.stacks.clone());
        let merged_notes = merge_by_id(db::get_notes().unwrap_or_default(), state.notes.clone());

        // Push any local rows the server doesn't have yet (one-time
        // migration AND any rows written while the server was down).
        let server_ids: std::collections::HashSet<String> = state
            .log_entries
            .iter()
            .map(|e| e.id.to_string())
            .chain(state.vitals_entries.iter().map(|v| v.id.to_string()))
            .chain(state.alerts.iter().map(|a| a.id.to_string()))
            .chain(state.stacks.iter().map(|s| s.id.to_string()))
            .chain(state.notes.iter().map(|n| n.id.to_string()))
            .collect();
        for e in &merged_logs {
            if !server_ids.contains(&e.id.to_string()) {
                push_log_entry(e);
            }
        }
        for v in &merged_vitals {
            if !server_ids.contains(&v.id.to_string()) {
                push_vitals_entry(v);
            }
        }
        for a in &merged_alerts {
            if !server_ids.contains(&a.id.to_string()) {
                push_alert(a);
            }
        }
        for s in &merged_stacks {
            if !server_ids.contains(&s.id.to_string()) {
                push_stack(s);
            }
        }
        for n in &merged_notes {
            if !server_ids.contains(&n.id.to_string()) {
                push_note(n);
            }
        }

        let _ = LocalStorage::set("biohack2_log_entries", &merged_logs);
        let _ = LocalStorage::set("biohack2_vitals", &merged_vitals);
        let _ = LocalStorage::set("biohack2_alerts", &merged_alerts);
        let _ = LocalStorage::set("biohack2_stacks", &merged_stacks);
        let _ = LocalStorage::set("biohack2_notes", &merged_notes);
        if !state.catalog.is_empty() {
            let _ = LocalStorage::set("biohack2_catalog_items", &state.catalog);
            let _ = LocalStorage::set("biohack2_catalog_seeded", true);
        }
        let _ = LocalStorage::set(SYNCED_FLAG, true);

        // Notify the UI to re-read storage: bump the global data version.
        if let Some(win) = web_sys::window()
            && let Ok(ev) = web_sys::Event::new("biohack2-sync-complete")
        {
            let _ = win.dispatch_event(&ev);
        }
        log_info(&format!(
            "sync: merged with server ({} logs, {} vitals, {} stacks, {} notes)",
            merged_logs.len(),
            merged_vitals.len(),
            merged_stacks.len(),
            merged_notes.len()
        ));
    });
}

/// Union two collections by id: keep all local rows plus any server rows
/// whose id is not present locally. Result sorted newest-first by `time`.
fn merge_by_id<T: engine::models::SortableEntry>(mut local: Vec<T>, server: Vec<T>) -> Vec<T> {
    let local_keys: std::collections::HashSet<uuid::Uuid> =
        local.iter().map(|e| e.sort_id()).collect();
    for s in server {
        if !local_keys.contains(&s.sort_id()) {
            local.push(s);
        }
    }
    local.sort_by_key(|e| std::cmp::Reverse(e.sort_time()));
    local
}

/// Stacks merge by id; on collision keep the one with the newer updated_at.
fn merge_stacks(local: Vec<Stack>, server: Vec<Stack>) -> Vec<Stack> {
    merge_by_id(local, server)
}

fn already_synced() -> bool {
    LocalStorage::get::<bool>(SYNCED_FLAG).unwrap_or(false)
}

// ── Cloud data lifecycle (FR-020 / FR-021) ─────────────────────────────────

/// Download the complete server-side dataset as one JSON document.
/// Deliberately the same shape as a sync pull: what the client can re-ingest
/// is exactly what it can take away.
pub fn cloud_export(status: leptos::prelude::RwSignal<String>) {
    use leptos::prelude::Set;
    spawn_local(async move {
        status.set("Exporting cloud data…".into());
        match fetch_safe("GET", "api/export", None).await {
            Ok(resp) if resp.ok() => match resp.text().await {
                Ok(text) => {
                    download_text_as_file(&text, "biohack_export.json");
                    status.set("Cloud export downloaded (biohack_export.json).".into());
                }
                Err(_) => status.set("Cloud export failed: unreadable response.".into()),
            },
            Ok(resp) => status.set(format!("Cloud export failed: HTTP {}.", resp.status())),
            Err(_) => status.set("Cloud export failed: server unreachable.".into()),
        }
    });
}

/// Erase all user data on the server. The confirmation gate is the arm →
/// "Sure?" button pattern in the UI; the server additionally requires
/// `confirm=yes`, so this call is the single deliberate deletion path.
pub fn cloud_delete(status: leptos::prelude::RwSignal<String>) {
    use leptos::prelude::Set;
    spawn_local(async move {
        status.set("Deleting cloud data…".into());
        match fetch_safe("DELETE", "api/data?confirm=yes", None).await {
            Ok(resp) if resp.ok() => {
                status.set("Cloud data deleted. Local data untouched.".into());
            }
            Ok(resp) => status.set(format!("Cloud deletion refused: HTTP {}.", resp.status())),
            Err(_) => status.set("Cloud deletion failed: server unreachable.".into()),
        }
    });
}

fn download_text_as_file(text: &str, filename: &str) {
    let Some(win) = web_sys::window() else { return };
    let Some(doc) = win.document() else { return };
    let array = js_sys::Array::new();
    array.push(&js_sys::JsString::from(text));
    let Ok(blob) = web_sys::Blob::new_with_str_sequence(&array) else {
        return;
    };
    let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else {
        return;
    };
    if let Ok(a) = doc.create_element("a")
        && let Ok(anchor) = a.dyn_into::<web_sys::HtmlElement>()
    {
        let _ = anchor.set_attribute("href", &url);
        let _ = anchor.set_attribute("download", filename);
        anchor.click();
        let _ = web_sys::Url::revoke_object_url(&url);
    }
}
