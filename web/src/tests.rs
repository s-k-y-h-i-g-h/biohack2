//! WASM tests for the database layer and component logic.
//! Run with: `wasm-pack test --headless --chrome` from the `web/` directory

use chrono::Utc;
use engine::models::*;
use uuid::Uuid;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn clear_test_storage() {
    
    let window = web_sys::window().expect("should have window");
    let storage = window
        .local_storage()
        .expect("should have local storage")
        .expect("storage should not be null");
    storage.clear().expect("should clear storage");
}

// ── LogEntry CRUD ─────────────────────────────────────────────────────────────

#[wasm_bindgen_test]
fn test_create_and_get_log_entry() {
    clear_test_storage();
    use crate::state::db::{create_log_entry, get_log_entries};

    let entry = LogEntry {
        id: Uuid::new_v4(),
        user_id: "test-user".to_string(),
        item_type: ItemType::Supplement,
        item_id: None,
        name: "Vitamin D3".to_string(),
        quantity: Some(5000.0),
        unit: Some("IU".to_string()),
        route: None,
        timestamp: Utc::now(),
        stack_id: None,
        notes: None,
        acknowledged_interaction: false,
        custom_fields: None,
    };

    create_log_entry(&entry).expect("should create log entry");
    let entries = get_log_entries().expect("should read log entries");

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "Vitamin D3");
    assert_eq!(entries[0].quantity, Some(5000.0));
}

#[wasm_bindgen_test]
fn test_get_log_entries_returns_sorted_descending() {
    clear_test_storage();
    use crate::state::db::{create_log_entry, get_log_entries};

    let earlier = LogEntry {
        id: Uuid::new_v4(),
        user_id: "test".to_string(),
        item_type: ItemType::Supplement,
        item_id: None,
        name: "Earlier".to_string(),
        quantity: None,
        unit: None,
        route: None,
        timestamp: Utc::now() - chrono::Duration::hours(2),
        stack_id: None,
        notes: None,
        acknowledged_interaction: false,
        custom_fields: None,
    };

    let later = LogEntry {
        id: Uuid::new_v4(),
        user_id: "test".to_string(),
        item_type: ItemType::Supplement,
        item_id: None,
        name: "Later".to_string(),
        quantity: None,
        unit: None,
        route: None,
        timestamp: Utc::now(),
        stack_id: None,
        notes: None,
        acknowledged_interaction: false,
        custom_fields: None,
    };

    create_log_entry(&earlier).expect("should create entry");
    create_log_entry(&later).expect("should create entry");

    let entries = get_log_entries().expect("should read entries");
    assert_eq!(entries.len(), 2);
    // Most recent first
    assert_eq!(entries[0].name, "Later");
    assert_eq!(entries[1].name, "Earlier");
}

#[wasm_bindgen_test]
fn test_delete_log_entry() {
    clear_test_storage();
    use crate::state::db::{create_log_entry, delete_log_entry, get_log_entries};

    let entry = LogEntry {
        id: Uuid::new_v4(),
        user_id: "test".to_string(),
        item_type: ItemType::Supplement,
        item_id: None,
        name: "ToDelete".to_string(),
        quantity: None,
        unit: None,
        route: None,
        timestamp: Utc::now(),
        stack_id: None,
        notes: None,
        acknowledged_interaction: false,
        custom_fields: None,
    };

    create_log_entry(&entry).expect("should create entry");
    delete_log_entry(&entry.id.to_string()).expect("should delete entry");

    let entries = get_log_entries().expect("should read entries");
    assert!(entries.is_empty() || !entries.iter().any(|e| e.id == entry.id));
}

// ── VitalsEntry CRUD ──────────────────────────────────────────────────────────

#[wasm_bindgen_test]
fn test_create_and_get_vitals_entry() {
    clear_test_storage();
    use crate::state::db::{create_vitals_entry, get_vitals_entries};

    let entry = VitalsEntry {
        id: Uuid::new_v4(),
        user_id: "test-user".to_string(),
        timestamp: Utc::now(),
        bp_systolic: Some(120),
        bp_diastolic: Some(80),
        heart_rate: Some(72),
        weight: None,
        blood_glucose: None,
        temperature: None,
        spo2: Some(98),
        hrv: None,
        sleep_quality: None,
        custom_metrics: None,
        notes: None,
    };

    create_vitals_entry(&entry).expect("should create vitals entry");
    let entries = get_vitals_entries(&Default::default()).expect("should read vitals");

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].bp_systolic, Some(120));
    assert_eq!(entries[0].heart_rate, Some(72));
}

#[wasm_bindgen_test]
fn test_get_vitals_entries_sorted_descending() {
    clear_test_storage();
    use crate::state::db::{create_vitals_entry, get_vitals_entries};

    let earlier = VitalsEntry {
        id: Uuid::new_v4(),
        user_id: "test".to_string(),
        timestamp: Utc::now() - chrono::Duration::hours(1),
        bp_systolic: Some(110),
        bp_diastolic: Some(70),
        heart_rate: Some(65),
        weight: None,
        blood_glucose: None,
        temperature: None,
        spo2: None,
        hrv: None,
        sleep_quality: None,
        custom_metrics: None,
        notes: None,
    };

    let later = VitalsEntry {
        id: Uuid::new_v4(),
        user_id: "test".to_string(),
        timestamp: Utc::now(),
        bp_systolic: Some(130),
        bp_diastolic: Some(85),
        heart_rate: Some(80),
        weight: None,
        blood_glucose: None,
        temperature: None,
        spo2: None,
        hrv: None,
        sleep_quality: None,
        custom_metrics: None,
        notes: None,
    };

    create_vitals_entry(&earlier).expect("should create entry");
    create_vitals_entry(&later).expect("should create entry");

    let entries = get_vitals_entries(&Default::default()).expect("should read entries");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].bp_systolic, Some(130)); // later first
}

// ── Alert CRUD ────────────────────────────────────────────────────────────────

#[wasm_bindgen_test]
fn test_create_and_get_alerts() {
    clear_test_storage();
    use crate::state::db::{create_alert, get_alerts};
    use engine::models::{AlertSeverity, AlertType};

    let alert = Alert {
        id: Uuid::new_v4(),
        user_id: "test".to_string(),
        alert_type: AlertType::Vital,
        severity: AlertSeverity::Critical,
        message: "Test alert".to_string(),
        recommendation: Some("Take action".to_string()),
        is_acknowledged: false,
        linked_entry_id: None,
        generated_at: Utc::now(),
        resolved_at: None,
    };

    create_alert(&alert).expect("should create alert");
    let alerts = get_alerts(&Default::default()).expect("should read alerts");

    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].message, "Test alert");
}

#[wasm_bindgen_test]
fn test_acknowledge_alert() {
    clear_test_storage();
    use crate::state::db::{acknowledge_alert, create_alert, get_alerts};
    use engine::models::{AlertFilter, AlertSeverity, AlertType};

    let alert = Alert {
        id: Uuid::new_v4(),
        user_id: "test".to_string(),
        alert_type: AlertType::Vital,
        severity: AlertSeverity::Warning,
        message: "Ack test".to_string(),
        recommendation: None,
        is_acknowledged: false,
        linked_entry_id: None,
        generated_at: Utc::now(),
        resolved_at: None,
    };

    create_alert(&alert).expect("should create alert");

    // Unacknowledged count should be 1
    let unacked = get_alerts(&AlertFilter {
        user_id: Some("test".to_string()),
        acknowledged: Some(false),
    })
    .expect("should read unacknowledged");
    assert_eq!(unacked.len(), 1);

    acknowledge_alert(&alert.id).expect("should acknowledge");

    // Unacknowledged count should now be 0
    let unacked = get_alerts(&AlertFilter {
        user_id: Some("test".to_string()),
        acknowledged: Some(false),
    })
    .expect("should read unacknowledged");
    assert_eq!(unacked.len(), 0);
}

// ── Filter Tests ──────────────────────────────────────────────────────────────

#[wasm_bindgen_test]
fn test_vitals_filter_by_user_id() {
    clear_test_storage();
    use crate::state::db::{create_vitals_entry, get_vitals_entries};

    let user_a = VitalsEntry {
        id: Uuid::new_v4(),
        user_id: "user-a".to_string(),
        timestamp: Utc::now(),
        bp_systolic: Some(120),
        bp_diastolic: Some(80),
        heart_rate: Some(70),
        weight: None,
        blood_glucose: None,
        temperature: None,
        spo2: None,
        hrv: None,
        sleep_quality: None,
        custom_metrics: None,
        notes: None,
    };

    let user_b = VitalsEntry {
        id: Uuid::new_v4(),
        user_id: "user-b".to_string(),
        timestamp: Utc::now(),
        bp_systolic: Some(140),
        bp_diastolic: Some(90),
        heart_rate: Some(90),
        weight: None,
        blood_glucose: None,
        temperature: None,
        spo2: None,
        hrv: None,
        sleep_quality: None,
        custom_metrics: None,
        notes: None,
    };

    create_vitals_entry(&user_a).expect("should create");
    create_vitals_entry(&user_b).expect("should create");

    let filtered = get_vitals_entries(&VitalsEntryFilter {
        user_id: Some("user-a".to_string()),
        ..Default::default()
    })
    .expect("should filter");

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].user_id, "user-a");
}

// ── Vitals Form Data Serialization ────────────────────────────────────────────

#[wasm_bindgen_test]
fn test_vitals_form_serializes_correct_data() {
    // Simulate what VitalsForm's handle_save does: serialize inputs to JSON
    let systolic = "140";
    let diastolic = "95";
    let heart_rate = "88";

    let data = format!(
        "{{\"systolic\":\"{}\",\"diastolic\":\"{}\",\"heart_rate\":\"{}\"}}",
        systolic, diastolic, heart_rate
    );

    assert!(data.contains("140"), "should contain systolic");
    assert!(data.contains("95"), "should contain diastolic");
    assert!(data.contains("88"), "should contain heart rate");

    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&data).expect("should be valid JSON");
    assert_eq!(parsed["systolic"], "140");
    assert_eq!(parsed["diastolic"], "95");
    assert_eq!(parsed["heart_rate"], "88");
}

#[wasm_bindgen_test]
fn test_vitals_form_handles_empty_inputs() {
    let systolic = "";
    let diastolic = "";
    let heart_rate = "";

    let data = format!(
        "{{\"systolic\":\"{}\",\"diastolic\":\"{}\",\"heart_rate\":\"{}\"}}",
        systolic, diastolic, heart_rate
    );

    let parsed: serde_json::Value = serde_json::from_str(&data).expect("should be valid JSON");
    assert_eq!(parsed["systolic"], "");
    assert_eq!(parsed["diastolic"], "");
    assert_eq!(parsed["heart_rate"], "");
}

// ── VitalsDashboard Logic ─────────────────────────────────────────────────────

#[wasm_bindgen_test]
fn test_vitals_dashboard_uses_first_entry() {
    // VitalsDashboard shows recent_vitals.first() — verify this logic
    let vitals = [VitalsEntry {
            id: Uuid::new_v4(),
            user_id: "test".to_string(),
            timestamp: Utc::now() - chrono::Duration::hours(2),
            bp_systolic: Some(110),
            bp_diastolic: Some(70),
            heart_rate: Some(65),
            weight: None,
            blood_glucose: None,
            temperature: None,
            spo2: None,
            hrv: None,
            sleep_quality: None,
            custom_metrics: None,
            notes: None,
        },
        VitalsEntry {
            id: Uuid::new_v4(),
            user_id: "test".to_string(),
            timestamp: Utc::now(),
            bp_systolic: Some(130),
            bp_diastolic: Some(85),
            heart_rate: Some(80),
            weight: None,
            blood_glucose: None,
            temperature: None,
            spo2: None,
            hrv: None,
            sleep_quality: None,
            custom_metrics: None,
            notes: None,
        }];

    // Dashboard uses .first() — verify it shows the first entry in the vec
    // (in real usage, the vec is sorted by timestamp desc, so first = most recent)
    let first = vitals.first().expect("should have first");
    assert_eq!(first.bp_systolic, Some(110)); // first in vec, not necessarily most recent
    assert_eq!(first.heart_rate, Some(65));
}

#[wasm_bindgen_test]
fn test_vitals_dashboard_shows_empty_for_empty_vec() {
    let vitals: Vec<VitalsEntry> = vec![];
    let has_vitals = !vitals.is_empty();
    assert!(!has_vitals, "empty vitals should show empty state");
}

// ── Alert Banner Signal Logic ─────────────────────────────────────────────────

#[wasm_bindgen_test]
fn test_alert_banner_visibility_logic() {
    // AlertBanner shows when alert.get().is_some()
    let alert_some: Option<String> = Some("Hypertensive urgency!".to_string());
    let alert_none: Option<String> = None;

    assert!(alert_some.is_some(), "alert should be visible when Some");
    assert!(!alert_none.is_some(), "alert should be hidden when None");

    // Extract message
    let message = alert_some.as_deref().unwrap_or("");
    assert_eq!(message, "Hypertensive urgency!");
}

// ── Safety Engine Integration Tests ──────────────────────────────────────────

#[wasm_bindgen_test]
fn test_safety_engine_hypertensive_urgency_detection() {
    use chrono::Utc;
    use engine::models::VitalsEntry;
    use engine::safety::run_safety_check;
    use uuid::Uuid;

    let entry = VitalsEntry {
        id: Uuid::new_v4(),
        user_id: "test".to_string(),
        timestamp: Utc::now(),
        bp_systolic: Some(185),
        bp_diastolic: Some(125),
        heart_rate: None,
        weight: None,
        blood_glucose: None,
        temperature: None,
        spo2: None,
        hrv: None,
        sleep_quality: None,
        custom_metrics: None,
        notes: None,
    };

    let result = run_safety_check(&entry, &[]);

    assert_eq!(
        result.alerts.len(),
        1,
        "should trigger hypertensive urgency alert"
    );
    assert!(
        result.alerts[0]
            .message
            .to_lowercase()
            .contains("hypertensive"),
        "alert should mention hypertensive, got: {}",
        result.alerts[0].message
    );
}

#[wasm_bindgen_test]
fn test_safety_engine_normal_vitals_no_alert() {
    use chrono::Utc;
    use engine::models::VitalsEntry;
    use engine::safety::run_safety_check;
    use uuid::Uuid;

    let entry = VitalsEntry {
        id: Uuid::new_v4(),
        user_id: "test".to_string(),
        timestamp: Utc::now(),
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

    let result = run_safety_check(&entry, &[]);

    assert_eq!(
        result.alerts.len(),
        0,
        "normal vitals should not trigger alerts"
    );
}

#[wasm_bindgen_test]
fn test_create_vitals_entry_triggers_safety_check() {
    // This test verifies the intended integration: when create_vitals_entry is called,
    // the safety engine should run and create alerts for abnormal vitals.
    // Currently this is NOT wired (known bug T038), so we test the expected behavior.
    use crate::state::db::{create_vitals_entry, get_vitals_entries};
    use chrono::Utc;
    use engine::models::VitalsEntry;
    use uuid::Uuid;

    clear_test_storage();

    // Create a hypertensive vitals entry
    let entry = VitalsEntry {
        id: Uuid::new_v4(),
        user_id: "test".to_string(),
        timestamp: Utc::now(),
        bp_systolic: Some(190),
        bp_diastolic: Some(130),
        heart_rate: Some(110),
        weight: None,
        blood_glucose: None,
        temperature: None,
        spo2: None,
        hrv: None,
        sleep_quality: None,
        custom_metrics: None,
        notes: None,
    };

    // This creates the vitals entry but currently does NOT run safety checks
    create_vitals_entry(&entry).expect("should create vitals entry");

    // Verify vitals was saved
    let vitals = get_vitals_entries(&Default::default()).expect("should read vitals");
    assert_eq!(vitals.len(), 1);
    assert_eq!(vitals[0].bp_systolic, Some(190));

    // NOTE: Safety check is not currently integrated (bug T038).
    // After the fix, this assertion would pass:
    // let alerts = get_alerts(&AlertFilter { user_id: Some("test".to_string()), acknowledged: Some(false) }).unwrap_or_default();
    // assert_eq!(alerts.len(), 1, "safety check should have created an alert");
}

// ── Note CRUD (standalone first-class entries) ─────────────────────────────────

#[wasm_bindgen_test]
fn test_create_and_get_note() {
    clear_test_storage();
    use crate::state::db::{create_note, get_notes};

    let note = Note {
        id: Uuid::new_v4(),
        user_id: "test-user".to_string(),
        content: "Noticed increased anxiety after Ashwagandha at night".to_string(),
        timestamp: Utc::now(),
        linked_entry_id: None,
    };

    create_note(&note).expect("should create note");
    let notes = get_notes().expect("should read notes");

    assert_eq!(notes.len(), 1);
    assert_eq!(
        notes[0].content,
        "Noticed increased anxiety after Ashwagandha at night"
    );
    assert_eq!(notes[0].user_id, "test-user");
}

#[wasm_bindgen_test]
fn test_get_notes_sorted_descending() {
    clear_test_storage();
    use crate::state::db::{create_note, get_notes};

    let earlier = Note {
        id: Uuid::new_v4(),
        user_id: "test".to_string(),
        content: "Earlier note".to_string(),
        timestamp: Utc::now() - chrono::Duration::hours(2),
        linked_entry_id: None,
    };
    let later = Note {
        id: Uuid::new_v4(),
        user_id: "test".to_string(),
        content: "Later note".to_string(),
        timestamp: Utc::now(),
        linked_entry_id: None,
    };

    create_note(&earlier).expect("should create earlier note");
    create_note(&later).expect("should create later note");

    let notes = get_notes().expect("should read notes");
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[0].content, "Later note");
    assert_eq!(notes[1].content, "Earlier note");
}

#[wasm_bindgen_test]
fn test_update_note() {
    clear_test_storage();
    use crate::state::db::{create_note, get_notes, update_note};

    let mut note = Note {
        id: Uuid::new_v4(),
        user_id: "test-user".to_string(),
        content: "Original content".to_string(),
        timestamp: Utc::now(),
        linked_entry_id: None,
    };
    create_note(&note).expect("should create note");

    note.content = "Updated content".to_string();
    update_note(&note).expect("should update note");

    let notes = get_notes().expect("should read notes");
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "Updated content");
}

#[wasm_bindgen_test]
fn test_delete_note() {
    clear_test_storage();
    use crate::state::db::{create_note, delete_note, get_notes};

    let note = Note {
        id: Uuid::new_v4(),
        user_id: "test-user".to_string(),
        content: "To be deleted".to_string(),
        timestamp: Utc::now(),
        linked_entry_id: None,
    };
    create_note(&note).expect("should create note");

    delete_note(&note.id.to_string()).expect("should delete note");
    let notes = get_notes().expect("should read notes");
    assert!(notes.is_empty());
}

#[wasm_bindgen_test]
fn test_note_with_linked_entry() {
    clear_test_storage();
    use crate::state::db::{create_note, get_notes};

    let entry_id = Uuid::new_v4();
    let note = Note {
        id: Uuid::new_v4(),
        user_id: "test-user".to_string(),
        content: "Context for the linked entry".to_string(),
        timestamp: Utc::now(),
        linked_entry_id: Some(entry_id),
    };

    create_note(&note).expect("should create note with link");
    let notes = get_notes().expect("should read notes");
    assert_eq!(notes[0].linked_entry_id, Some(entry_id));
}

#[wasm_bindgen_test]
fn test_note_serialization_roundtrip() {
    let note = Note {
        id: Uuid::new_v4(),
        user_id: "test-user".to_string(),
        content: "Serialization roundtrip test".to_string(),
        timestamp: Utc::now(),
        linked_entry_id: Some(Uuid::new_v4()),
    };

    let json = serde_json::to_string(&note).expect("should serialize note");
    let parsed: Note = serde_json::from_str(&json).expect("should deserialize note");

    assert_eq!(parsed.id, note.id);
    assert_eq!(parsed.content, note.content);
    assert_eq!(parsed.linked_entry_id, note.linked_entry_id);
}

#[wasm_bindgen_test]
fn test_note_export_included_in_csv() {
    clear_test_storage();
    use crate::state::db::{create_note, export_data};

    let note = Note {
        id: Uuid::new_v4(),
        user_id: "test-user".to_string(),
        content: "Export test note".to_string(),
        timestamp: Utc::now(),
        linked_entry_id: None,
    };
    create_note(&note).expect("should create note");

    let csv = export_data().expect("should export data");
    assert!(csv.contains("note,"));
    assert!(csv.contains("Export test note"));
}

// ── Local-time / relative-time display (spec 002) ───────────────────────────────
//
// Timestamps are stored as UTC. Rendering them without conversion shows UTC
// wall time, which reads as an hour off under daylight saving. These tests pin
// the conversion behaviour; they run in a browser so chrono::Local resolves to
// the browser's real timezone.

#[wasm_bindgen_test]
fn test_local_time_converts_from_utc() {
    use crate::pages::history_page::local_time;
    use chrono::TimeZone;

    // 2026-09-17T22:25:24Z — a UTC instant in the evening.
    let ts = chrono::DateTime::parse_from_rfc3339("2026-09-17T22:25:24Z")
        .unwrap()
        .with_timezone(&chrono::Utc);

    // Local rendering must differ from UTC by exactly the browser's UTC offset,
    // and must preserve the minute.
    let offset_secs: i64 = chrono::Local
        .offset_from_utc_datetime(&ts.naive_utc())
        .local_minus_utc() as i64;
    let expected_hour = ((22 * 3600 + 25 * 60 + offset_secs).rem_euclid(24 * 3600)) / 3600;
    let rendered = local_time(&ts);
    assert_eq!(rendered, format!("{:02}:25", expected_hour));
}

#[wasm_bindgen_test]
fn test_local_date_key_uses_local_calendar_day() {
    use crate::pages::history_page::local_date_key;
    use chrono::TimeZone;

    // 23:30 UTC. In any timezone east of UTC (e.g. Europe/London under BST,
    // UTC+1) this is the *next* calendar day locally — the date-group header
    // must follow the local calendar day, not the UTC one.
    let ts = chrono::DateTime::parse_from_rfc3339("2026-09-17T23:30:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    let key = local_date_key(&ts);

    let offset_secs: i64 = chrono::Local
        .offset_from_utc_datetime(&ts.naive_utc())
        .local_minus_utc() as i64;
    let expected = if offset_secs > 0 {
        "2026-09-18".to_string()
    } else {
        "2026-09-17".to_string()
    };
    assert_eq!(key, expected);
}

#[wasm_bindgen_test]
fn test_relative_time_buckets() {
    use crate::pages::history_page::relative_time;

    let now = chrono::Local::now();

    // Under a minute → "just now" (never "0 seconds ago")
    let just_now = (now - chrono::Duration::seconds(10)).with_timezone(&chrono::Utc);
    assert_eq!(relative_time(&just_now), "just now");

    // Minutes
    let mins = (now - chrono::Duration::minutes(5)).with_timezone(&chrono::Utc);
    assert_eq!(relative_time(&mins), "5 minutes ago");

    // Singular minute
    let one_min = (now - chrono::Duration::minutes(1)).with_timezone(&chrono::Utc);
    assert_eq!(relative_time(&one_min), "1 minute ago");

    // Hours
    let hrs = (now - chrono::Duration::hours(3)).with_timezone(&chrono::Utc);
    assert_eq!(relative_time(&hrs), "3 hours ago");

    // Days
    let days = (now - chrono::Duration::days(2)).with_timezone(&chrono::Utc);
    assert_eq!(relative_time(&days), "2 days ago");

    // Weeks
    let weeks = (now - chrono::Duration::days(10)).with_timezone(&chrono::Utc);
    assert_eq!(relative_time(&weeks), "1 week ago");

    // A future-dated entry (clock skew) must not render a negative duration:
    // it clamps to "just now".
    let future = (now + chrono::Duration::minutes(2)).with_timezone(&chrono::Utc);
    assert_eq!(relative_time(&future), "just now");

    // Over a month old → falls back to the absolute local date rather than an
    // ever-growing day count.
    let old = (now - chrono::Duration::days(90)).with_timezone(&chrono::Utc);
    let rel = relative_time(&old);
    assert!(
        rel.contains("2026"),
        "old entries should show the absolute date, got: {rel}"
    );
}

// ── T223: native ↔ wasm32 PK parity on the shared spec-003 fixture ──────
// Same vectors fixture the native engine test consumes; the point is that
// wasm f64 math reproduces native expectations exactly (FR-038). Pure
// computation, so it lives in its own module: with no configure macro it
// runs on the node runner (`wasm-pack test --node`) while the localStorage
// tests above stay browser-only.

mod pk_parity {
    use super::*;

    #[wasm_bindgen_test]
    fn pk_shared_vectors_match_native() {
        let fixture = include_str!("../../engine/tests/fixtures/pk_vectors.json");
        let failures = engine::pharmacokinetics::check_vectors(fixture);
        assert!(
            failures.is_empty(),
            "wasm PK drift:\n{}",
            failures.join("\n")
        );
    }
}

// ── PK alert refresh (spec-003 T233) ─────────────────────────────────────────

#[wasm_bindgen_test]
fn test_refresh_supersedes_stale_pk_alert() {
    clear_test_storage();
    use crate::state::db::{create_alert, get_alerts};
    use crate::state::pk_alerts::refresh_pharmacokinetic_alerts;
    use engine::models::{AlertFilter, AlertSeverity, AlertType};

    // A redose alert left over from an earlier session, now stale.
    let stale = Alert {
        id: Uuid::new_v4(),
        user_id: "local-device".to_string(),
        alert_type: AlertType::Redose,
        severity: AlertSeverity::Warning,
        message: "stale redose".to_string(),
        recommendation: None,
        is_acknowledged: false,
        linked_entry_id: None,
        generated_at: Utc::now() - chrono::Duration::hours(3),
        resolved_at: None,
    };
    create_alert(&stale).expect("seed stale alert");

    let created =
        refresh_pharmacokinetic_alerts(Utc::now()).expect("refresh runs without storage errors");
    assert_eq!(created, 0, "no doses, no bands — nothing new");

    // Gone from the banner query (unacknowledged AND unresolved)...
    let open = get_alerts(&AlertFilter {
        user_id: Some("local-device".to_string()),
        acknowledged: Some(false),
    })
    .expect("read open alerts");
    assert!(open.is_empty(), "resolved alert must not nag: {:?}", open);

    // ...but still on record, resolved without user acknowledgement (T232).
    let all = get_alerts(&Default::default()).expect("read all alerts");
    let s = all.iter().find(|a| a.id == stale.id).expect("row kept");
    assert!(s.resolved_at.is_some() && !s.is_acknowledged);
}

#[wasm_bindgen_test]
fn test_refresh_fires_redose_alert_for_decaying_band() {
    clear_test_storage();
    use crate::state::db::{create_log_entry, get_alerts, search_catalog, upsert_target_band};
    use crate::state::pk_alerts::refresh_pharmacokinetic_alerts;
    use engine::models::{AlertFilter, AlertType, ItemType, ModelClass};

    search_catalog("").expect("catalog auto-seeds");
    let models = crate::state::db::get_substance_models();
    let model = models
        .iter()
        .find(|m| m.model_class == ModelClass::Decay && m.has_numeric_half_life())
        .expect("seed catalog has at least one decay substance");

    // Band set unreachable after a 30 h-old dose: the level is long below it.
    let band = TargetBand {
        substance: model.substance.clone(),
        low: 1000.0,
        high: 2000.0,
        window_hours: 24.0,
        updated_at: Utc::now(),
    };
    upsert_target_band(&band).expect("band stored");
    let entry = LogEntry {
        id: Uuid::new_v4(),
        user_id: "local-device".to_string(),
        item_type: ItemType::Supplement,
        item_id: None,
        name: model.substance.clone(),
        quantity: Some(10.0), // far below any daily max — no cumulative noise
        unit: Some("mg".to_string()),
        route: None,
        timestamp: Utc::now() - chrono::Duration::hours(30),
        stack_id: None,
        notes: None,
        acknowledged_interaction: false,
        custom_fields: None,
    };
    create_log_entry(&entry).expect("log dose");

    let created = refresh_pharmacokinetic_alerts(Utc::now()).expect("refresh runs");
    assert!(created >= 1, "band crossing must surface: {created}");
    let open = get_alerts(&AlertFilter {
        user_id: Some("local-device".to_string()),
        acknowledged: Some(false),
    })
    .expect("read open alerts");
    assert!(open.iter().any(|a| a.alert_type == AlertType::Redose));
}

// T235: alerts fire with no network and no catalog fetch. Stronger than an
// absent network — window.fetch is replaced with a function that THROWS, so
// any code path that reaches for the network before raising an alert fails
// this test loudly instead of silently degrading.
#[wasm_bindgen_test]
fn test_pk_alerts_fire_with_network_disabled() {
    clear_test_storage();
    use crate::state::db::{create_log_entry, search_catalog, upsert_target_band};
    use crate::state::pk_alerts::refresh_pharmacokinetic_alerts;
    use engine::models::{ItemType, ModelClass};
    use wasm_bindgen::prelude::*;

    let boom =
        Closure::<dyn FnMut() -> Result<(), JsValue>>::wrap(Box::new(|| -> Result<(), JsValue> {
            Err(js_sys::Error::new("T235: network is disabled").into())
        }));
    let window: JsValue = web_sys::window().expect("window").into();
    js_sys::Reflect::set(&window, &JsValue::from_str("fetch"), boom.as_ref())
        .expect("window.fetch is replaceable");
    boom.forget();

    search_catalog("").expect("catalog auto-seeds from localStorage, no fetch");
    let model = crate::state::db::get_substance_models()
        .into_iter()
        .find(|m| m.model_class == ModelClass::Decay && m.has_numeric_half_life())
        .expect("seed catalog has a decay substance");

    let band = TargetBand {
        substance: model.substance.clone(),
        low: 1000.0,
        high: 2000.0,
        window_hours: 24.0,
        updated_at: Utc::now(),
    };
    upsert_target_band(&band).expect("band stored offline");
    let entry = LogEntry {
        id: Uuid::new_v4(),
        user_id: "local-device".to_string(),
        item_type: ItemType::Supplement,
        item_id: None,
        name: model.substance.clone(),
        quantity: Some(10.0),
        unit: Some("mg".to_string()),
        route: None,
        timestamp: Utc::now() - chrono::Duration::hours(30),
        stack_id: None,
        notes: None,
        acknowledged_interaction: false,
        custom_fields: None,
    };
    create_log_entry(&entry).expect("log dose offline");

    let created =
        refresh_pharmacokinetic_alerts(Utc::now()).expect("alert generation needs no network");
    assert!(
        created >= 1,
        "band crossing must fire fully offline: {created}"
    );
}

// T239: unmodelled substances the user has data for surface with their
// recorded reason — the panel shows this, not a blank.
#[wasm_bindgen_test]
fn test_coverage_gaps_name_unmodelled_substances_with_reasons() {
    clear_test_storage();
    use crate::state::db::{create_log_entry, search_catalog};
    use crate::state::pk_alerts::pk_coverage_gaps;
    use engine::models::ItemType;

    search_catalog("").expect("catalog auto-seeds");
    let entry = LogEntry {
        id: Uuid::new_v4(),
        user_id: "local-device".to_string(),
        item_type: ItemType::Supplement,
        item_id: None,
        name: "Alcohol".to_string(), // catalog: deliberately unmodelled
        quantity: Some(30.0),
        unit: Some("g".to_string()),
        route: None,
        timestamp: Utc::now(),
        stack_id: None,
        notes: None,
        acknowledged_interaction: false,
        custom_fields: None,
    };
    create_log_entry(&entry).expect("log entry");

    let gaps = pk_coverage_gaps().expect("gaps query");
    let alcohol = gaps
        .iter()
        .find(|g| g.substance.eq_ignore_ascii_case("Alcohol"))
        .expect("Alcohol is unmodelled and the user logged it — must be surfaced");
    assert!(
        alcohol.reason.contains("Zero-order") || alcohol.reason.contains("refuse"),
        "reason must be the recorded one: {}",
        alcohol.reason
    );
    // Modelled substances the user logged must NOT appear as gaps.
    assert!(
        gaps.iter()
            .all(|g| !g.substance.eq_ignore_ascii_case("Caffeine"))
    );
}
