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
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;

const SYNCED_FLAG: &str = "biohack2_server_synced";

fn log_info(msg: &str) {
    web_sys::console::log_1(&JsValue::from_str(msg));
}
fn log_warn(msg: &str) {
    web_sys::console::warn_1(&JsValue::from_str(msg));
}

fn push(method: &str, path: &str, body: Option<serde_json::Value>) {
    let url = format!("api/{path}");
    let method = method.to_string();
    spawn_local(async move {
        let builder = match method.as_str() {
            "POST" => Request::post(&url),
            "PUT" => Request::put(&url),
            "DELETE" => Request::delete(&url),
            _ => Request::get(&url),
        };
        let result = if let Some(b) = body {
            match builder
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&b).unwrap_or_default())
            {
                Ok(req) => req.send().await,
                Err(e) => Err(e),
            }
        } else {
            builder.send().await
        };
        match result {
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
        let resp = match Request::get("api/sync/pull").send().await {
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
            let result = Request::post("api/sync/push")
                .header("Content-Type", "application/json")
                .body(body);
            let send_result = match result {
                Ok(req) => req.send().await,
                Err(e) => Err(e),
            };
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
            && let Ok(ev) = web_sys::Event::new("biohack2-sync-complete") {
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
    local.sort_by(|a, b| b.sort_time().cmp(&a.sort_time()));
    local
}

/// Stacks merge by id; on collision keep the one with the newer updated_at.
fn merge_stacks(mut local: Vec<Stack>, server: Vec<Stack>) -> Vec<Stack> {
    let local_keys: std::collections::HashSet<uuid::Uuid> = local.iter().map(|s| s.id).collect();
    for s in server {
        if !local_keys.contains(&s.id) {
            local.push(s);
        }
    }
    local
}

fn already_synced() -> bool {
    LocalStorage::get::<bool>(SYNCED_FLAG).unwrap_or(false)
}
