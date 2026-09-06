//! WASM-compatible database for the biohack tracker.
//! Uses gloo_storage for local persistence.

use engine::models::*;
use gloo_storage::{LocalStorage, Storage};
use uuid::Uuid;

const STORAGE_KEY_LOG_ENTRIES: &str = "biohack2_log_entries";
const STORAGE_KEY_CATALOG_SEEDED: &str = "biohack2_catalog_seeded";
const STORAGE_KEY_CATALOG_ITEMS: &str = "biohack2_catalog_items";
const STORAGE_KEY_VITALS: &str = "biohack2_vitals";
const STORAGE_KEY_ALERTS: &str = "biohack2_alerts";
const STORAGE_KEY_STACKS: &str = "biohack2_stacks";

// ── LogEntry CRUD ─────────────────────────────────────────────────────────────

pub fn create_log_entry(entry: &LogEntry) -> Result<(), String> {
    let mut entries = get_log_entries()?;
    entries.push(entry.clone());
    LocalStorage::set(STORAGE_KEY_LOG_ENTRIES, &entries)
        .map_err(|e| format!("Failed to write log entries: {:?}", e))?;
    Ok(())
}

pub fn get_log_entries() -> Result<Vec<LogEntry>, String> {
    match LocalStorage::get::<Vec<LogEntry>>(STORAGE_KEY_LOG_ENTRIES) {
        Ok(mut entries) => {
            entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            Ok(entries)
        }
        Err(_) => Ok(Vec::new()),
    }
}

pub fn delete_log_entry(id: &str) -> Result<(), String> {
    let mut entries = get_log_entries()?;
    entries.retain(|e| e.id.to_string() != id);
    LocalStorage::set(STORAGE_KEY_LOG_ENTRIES, &entries)
        .map_err(|e| format!("Failed to delete log entry: {:?}", e))?;
    Ok(())
}

pub fn update_log_entry(entry: &LogEntry) -> Result<(), String> {
    let mut entries = get_log_entries()?;
    if let Some(idx) = entries.iter().position(|e| e.id == entry.id) {
        entries[idx] = entry.clone();
    }
    LocalStorage::set(STORAGE_KEY_LOG_ENTRIES, &entries)
        .map_err(|e| format!("Failed to update log entry: {:?}", e))?;
    Ok(())
}

// ── Catalog ───────────────────────────────────────────────────────────────────

pub fn catalog_seeded() -> bool {
    LocalStorage::get::<bool>(STORAGE_KEY_CATALOG_SEEDED).unwrap_or(false)
}

pub fn seed_catalog(items: &[CatalogItem]) -> Result<(), String> {
    LocalStorage::set(STORAGE_KEY_CATALOG_ITEMS, items)
        .map_err(|e| format!("Failed to seed catalog: {:?}", e))?;
    LocalStorage::set(STORAGE_KEY_CATALOG_SEEDED, &true)
        .map_err(|e| format!("Failed to mark catalog seeded: {:?}", e))?;
    Ok(())
}

pub fn search_catalog(query: &str) -> Result<Vec<CatalogItem>, String> {
    if !catalog_seeded() {
        let items = engine::catalog::seed_catalog();
        seed_catalog(&items)?;
    }
    let items: Vec<CatalogItem> = LocalStorage::get(STORAGE_KEY_CATALOG_ITEMS).unwrap_or_default();
    Ok(filter_catalog(&items, query))
}

fn filter_catalog(items: &[CatalogItem], query: &str) -> Vec<CatalogItem> {
    if query.is_empty() {
        return items.to_vec();
    }
    let q = query.to_lowercase();
    items
        .iter()
        .filter(|item| item.name.to_lowercase().contains(&q))
        .cloned()
        .collect()
}

// ── VitalsEntry CRUD ──────────────────────────────────────────────────────────

pub fn create_vitals_entry(entry: &VitalsEntry) -> Result<(), String> {
    let mut entries = get_vitals_entries(&Default::default())?;
    entries.push(entry.clone());
    LocalStorage::set(STORAGE_KEY_VITALS, &entries)
        .map_err(|e| format!("Failed to write vitals: {:?}", e))?;
    Ok(())
}

pub fn get_vitals_entries(filter: &VitalsEntryFilter) -> Result<Vec<VitalsEntry>, String> {
    let entries: Vec<VitalsEntry> = LocalStorage::get(STORAGE_KEY_VITALS).unwrap_or_default();
    let mut filtered = entries;

    if let Some(ref uid) = filter.user_id {
        filtered.retain(|e| e.user_id == *uid);
    }
    if let Some(ref start) = filter.start_date {
        filtered.retain(|e| e.timestamp >= *start);
    }
    if let Some(ref end) = filter.end_date {
        filtered.retain(|e| e.timestamp <= *end);
    }

    filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(filtered)
}

// ── Alert CRUD ────────────────────────────────────────────────────────────────

pub fn create_alert(alert: &Alert) -> Result<(), String> {
    let mut alerts = get_alerts(&Default::default())?;
    alerts.insert(0, alert.clone());
    LocalStorage::set(STORAGE_KEY_ALERTS, &alerts)
        .map_err(|e| format!("Failed to write alerts: {:?}", e))?;
    Ok(())
}

pub fn get_alerts(filter: &AlertFilter) -> Result<Vec<Alert>, String> {
    let alerts: Vec<Alert> = LocalStorage::get(STORAGE_KEY_ALERTS).unwrap_or_default();
    let mut filtered = alerts;

    if let Some(ref uid) = filter.user_id {
        filtered.retain(|a| a.user_id == *uid);
    }
    if let Some(ack) = filter.acknowledged {
        filtered.retain(|a| a.is_acknowledged == ack);
    }

    filtered.sort_by(|a, b| b.generated_at.cmp(&a.generated_at));
    Ok(filtered)
}

pub fn acknowledge_alert(id: &Uuid) -> Result<(), String> {
    let mut alerts = get_alerts(&Default::default())?;
    if let Some(alert) = alerts.iter_mut().find(|a| a.id == *id) {
        alert.is_acknowledged = true;
        alert.resolved_at = Some(chrono::Utc::now());
    }
    LocalStorage::set(STORAGE_KEY_ALERTS, &alerts)
        .map_err(|e| format!("Failed to acknowledge alert: {:?}", e))?;
    Ok(())
}

// ── Stack CRUD ────────────────────────────────────────────────────────────────

pub fn create_stack(stack: &Stack) -> Result<(), String> {
    let mut stacks = get_stacks()?;
    stacks.insert(0, stack.clone());
    LocalStorage::set(STORAGE_KEY_STACKS, &stacks)
        .map_err(|e| format!("Failed to write stacks: {:?}", e))?;
    Ok(())
}

pub fn get_stacks() -> Result<Vec<Stack>, String> {
    match LocalStorage::get::<Vec<Stack>>(STORAGE_KEY_STACKS) {
        Ok(stacks) => Ok(stacks),
        Err(_) => Ok(Vec::new()),
    }
}

pub fn delete_stack(id: &str) -> Result<(), String> {
    let mut stacks = get_stacks()?;
    stacks.retain(|s| s.id.to_string() != id);
    LocalStorage::set(STORAGE_KEY_STACKS, &stacks)
        .map_err(|e| format!("Failed to delete stack: {:?}", e))?;
    Ok(())
}

/// Log a stack by creating individual LogEntries for each item with the same timestamp.
pub fn log_stack(stack: &Stack) -> Result<Vec<Uuid>, String> {
    let timestamp = chrono::Utc::now();
    let mut created_ids = Vec::new();

    for item in &stack.items {
        // Look up the catalog item to get the name
        let catalog_items = engine::catalog::seed_catalog();
        let catalog_item = catalog_items.iter()
            .find(|c| c.id == item.item_id);

        let name = catalog_item
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let entry = LogEntry {
            id: Uuid::new_v4(),
            user_id: stack.user_id.clone(),
            item_type: catalog_item
                .map(|c| c.category.clone())
                .unwrap_or(ItemType::Supplement),
            item_id: Some(item.item_id),
            name,
            quantity: item.quantity,
            unit: item.unit.clone(),
            route: None,
            timestamp,
            stack_id: Some(stack.id),
            notes: item.note.clone(),
            acknowledged_interaction: false,
            custom_fields: None,
        };

        create_log_entry(&entry)?;
        created_ids.push(entry.id);
    }

    Ok(created_ids)
}

// ── Data Export ────────────────────────────────────────────────────────────────

pub fn export_data() -> Result<String, String> {
    let log_entries = get_log_entries().unwrap_or_default();
    let vitals = get_vitals_entries(&Default::default()).unwrap_or_default();

    let mut csv = String::from("type,id,name,item_type,quantity,unit,timestamp,notes\n");

    // Log entries
    for entry in &log_entries {
        let qty = entry.quantity.map(|q| q.to_string()).unwrap_or_default();
        let unit = entry.unit.as_ref().map(|u| u.as_str()).unwrap_or("");
        let notes = entry.notes.as_deref().unwrap_or("").replace(',', ";");
        let item_type_str = match &entry.item_type {
            ItemType::Supplement => "supplement",
            ItemType::Medication => "medication",
            ItemType::Drug => "drug",
            ItemType::Food => "food",
            ItemType::Action => "action",
        };
        csv.push_str(&format!(
            "log,{},{},{},{},{} {},{}\n",
            entry.id,
            entry.name,
            item_type_str,
            qty,
            unit,
            entry.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
            notes,
        ));
    }

    // Vitals entries
    for entry in &vitals {
        let sbp = entry.bp_systolic.map(|v| v.to_string()).unwrap_or_default();
        let hr = entry.heart_rate.map(|v| v.to_string()).unwrap_or_default();
        let notes = entry.notes.as_deref().unwrap_or("").replace(',', ";");
        csv.push_str(&format!(
            "vitals,{},{},Vitals,{},{} {},{}\n",
            entry.id,
            "Vitals Reading",
            sbp,
            hr,
            entry.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
            notes,
        ));
    }

    // Write to localStorage and show in console (fallback for WASM)
    if let Some(win) = web_sys::window() {
        let storage = gloo_storage::LocalStorage::raw();
        let _ = storage.set_item("biohack_export_csv", &csv);
        // Alert user that data is exported
        let _ = win.alert_with_message("Data exported! Check browser console for CSV content.");
    }

    Ok(csv)
}

// Helper to convert string category to ItemType
pub fn parse_item_type(s: &str) -> Result<ItemType, String> {
    match s.to_lowercase().as_str() {
        "supplement" => Ok(ItemType::Supplement),
        "medication" => Ok(ItemType::Medication),
        "drug" => Ok(ItemType::Drug),
        "food" => Ok(ItemType::Food),
        "action" => Ok(ItemType::Action),
        _ => Err(format!("Invalid item type: {}", s)),
    }
}
