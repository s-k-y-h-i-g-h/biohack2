//! Safety protocol tests — the 3 deterministic protocols from the biohack CLI,
//! plus stimulant/serotonergic classification and contextual advice (FR-009).

use engine::models::*;
use engine::safety::{SafetyEngine, RecentSubstance, contextual_advice, is_stimulant_name, is_serotonergic_name};
use chrono::{Duration, Utc};

fn make_vitals(hr: Option<i32>, sbp: Option<i32>, dbp: Option<i32>) -> VitalsEntry {
    VitalsEntry {
        id: uuid::Uuid::new_v4(),
        user_id: "test-user".to_string(),
        timestamp: Utc::now(),
        bp_systolic: sbp,
        bp_diastolic: dbp,
        heart_rate: hr,
        weight: None,
        blood_glucose: None,
        temperature: None,
        spo2: None,
        hrv: None,
        sleep_quality: None,
        custom_metrics: None,
        notes: None,
    }
}

fn make_substance(name: &str, is_stimulant: bool, is_serotonergic: bool) -> RecentSubstance {
    RecentSubstance {
        name: name.to_string(),
        category: ItemType::Supplement,
        taken_at: Utc::now() - Duration::hours(1),
        is_stimulant,
        is_serotonergic,
    }
}

fn make_log(name: &str, item_type: ItemType, hours_ago: i64) -> LogEntry {
    LogEntry {
        id: uuid::Uuid::new_v4(),
        user_id: "test".to_string(),
        item_type,
        item_id: None,
        name: name.to_string(),
        quantity: None,
        unit: None,
        route: None,
        timestamp: Utc::now() - Duration::hours(hours_ago),
        stack_id: None,
        notes: None,
        acknowledged_interaction: false,
        custom_fields: None,
    }
}

// ── Protocol 1: stimulant tachycardia ────────────────────────────────────────

#[test]
fn test_no_alerts_for_normal_vitals() {
    let engine = SafetyEngine::new();
    let vitals = make_vitals(Some(72), Some(120), Some(80));
    let result = engine.check_vitals(&vitals, &[]);
    assert!(result.alerts.is_empty());
    assert!(result.protocols_triggered.is_empty());
}

#[test]
fn test_tachycardia_alert_with_stimulant() {
    let engine = SafetyEngine::new();
    let vitals = make_vitals(Some(110), None, None);
    let substances = vec![make_substance("Caffeine", true, false)];
    let result = engine.check_vitals(&vitals, &substances);
    assert_eq!(result.alerts.len(), 1);
    assert_eq!(result.protocols_triggered.len(), 1);
    assert!(result.alerts[0].message.contains("tachycardia"));
}

#[test]
fn test_no_tachycardia_alert_without_stimulant() {
    let engine = SafetyEngine::new();
    let vitals = make_vitals(Some(110), None, None);
    let substances = vec![make_substance("Melatonin", false, false)];
    let result = engine.check_vitals(&vitals, &substances);
    assert!(result.alerts.is_empty());
}

#[test]
fn test_stimulant_window_excludes_old_intake() {
    let engine = SafetyEngine::new();
    let vitals = make_vitals(Some(110), None, None);
    // Stimulant 6h ago — outside the 4h window
    let old = RecentSubstance {
        name: "Caffeine".to_string(),
        category: ItemType::Supplement,
        taken_at: Utc::now() - Duration::hours(6),
        is_stimulant: true,
        is_serotonergic: false,
    };
    let result = engine.check_vitals(&vitals, &[old]);
    assert!(result.alerts.is_empty());
}

// ── Protocol 2: hypertensive urgency ─────────────────────────────────────────

#[test]
fn test_hypertensive_urgency_alert() {
    let engine = SafetyEngine::new();
    let vitals = make_vitals(None, Some(185), Some(125));
    let result = engine.check_vitals(&vitals, &[]);
    assert_eq!(result.alerts.len(), 1);
    assert!(result.alerts[0].message.to_lowercase().contains("hypertensive"));
}

#[test]
fn test_no_hypertension_alert_for_normal_bp() {
    let engine = SafetyEngine::new();
    let vitals = make_vitals(None, Some(120), Some(80));
    let result = engine.check_vitals(&vitals, &[]);
    assert!(result.alerts.is_empty());
}

// ── Protocol 3: serotonin syndrome ───────────────────────────────────────────

#[test]
fn test_serotonin_syndrome_detection() {
    let engine = SafetyEngine::new();
    let new_entry = make_log("Fluoxetine", ItemType::Medication, 0);
    let existing = vec![
        make_log("5-HTP", ItemType::Supplement, 1),
        make_log("St. John's Wort", ItemType::Supplement, 2),
    ];
    let alerts = engine.check_interactions(&new_entry, &existing);
    assert!(!alerts.is_empty());
    assert_eq!(alerts[0].alert_type, AlertType::Interaction);
}

#[test]
fn test_no_serotonin_alert_for_non_serotonergic() {
    let engine = SafetyEngine::new();
    let new_entry = make_log("Vitamin D3", ItemType::Supplement, 0);
    let existing = vec![make_log("Magnesium", ItemType::Supplement, 1)];
    let alerts = engine.check_interactions(&new_entry, &existing);
    assert!(alerts.is_empty());
}

// ── Name classification ──────────────────────────────────────────────────────

#[test]
fn test_stimulant_name_classification() {
    assert!(is_stimulant_name("Coffee"));
    assert!(is_stimulant_name("Espresso"));
    assert!(is_stimulant_name("Caffeine"));
    assert!(is_stimulant_name("Nicotine Patch"));
    assert!(is_stimulant_name("Modafinil"));
    assert!(!is_stimulant_name("Magnesium Glycinate"));
    assert!(!is_stimulant_name("Melatonin"));
    assert!(!is_stimulant_name("Vitamin D3"));
}

#[test]
fn test_serotonergic_name_classification() {
    assert!(is_serotonergic_name("5-HTP"));
    assert!(is_serotonergic_name("L-Tryptophan"));
    assert!(is_serotonergic_name("St. John's Wort"));
    assert!(is_serotonergic_name("Tramadol"));
    assert!(!is_serotonergic_name("Creatine"));
    assert!(!is_serotonergic_name("Magnesium Glycinate"));
}

// ── Contextual advice (FR-009) ────────────────────────────────────────────────

fn make_alert(message: &str) -> Alert {
    Alert {
        id: uuid::Uuid::new_v4(),
        user_id: "test-user".to_string(),
        alert_type: AlertType::Vital,
        severity: AlertSeverity::Critical,
        message: message.to_string(),
        recommendation: Some("base recommendation".to_string()),
        is_acknowledged: false,
        linked_entry_id: None,
        generated_at: Utc::now(),
        resolved_at: None,
    }
}

#[test]
fn test_advice_low_magnesium_for_hypertension() {
    let alert = make_alert("Hypertensive urgency: BP 185/125 mmHg");
    let substances = vec![make_substance("Vitamin D3", false, false)];
    let advice = contextual_advice(&alert, &substances);
    assert!(advice.is_some());
    assert!(advice.unwrap().to_lowercase().contains("magnesium"));
}

#[test]
fn test_advice_none_when_magnesium_logged() {
    let alert = make_alert("Hypertensive urgency: BP 185/125 mmHg");
    let substances = vec![make_substance("Magnesium Glycinate", false, false)];
    let advice = contextual_advice(&alert, &substances);
    assert!(advice.is_none());
}

#[test]
fn test_advice_lists_stimulants_for_tachycardia() {
    let alert = make_alert("Stimulant-associated tachycardia: HR 110 bpm");
    let substances = vec![
        make_substance("Coffee", true, false),
        make_substance("Melatonin", false, false),
    ];
    let advice = contextual_advice(&alert, &substances);
    let advice = advice.unwrap();
    assert!(advice.contains("Coffee"));
    assert!(!advice.contains("Melatonin"));
}

#[test]
fn test_advice_ignores_substances_older_than_week() {
    let alert = make_alert("Hypertensive urgency: BP 185/125 mmHg");
    let old = RecentSubstance {
        name: "Magnesium Glycinate".to_string(),
        category: ItemType::Supplement,
        taken_at: Utc::now() - Duration::days(10),
        is_stimulant: false,
        is_serotonergic: false,
    };
    let advice = contextual_advice(&alert, &[old]);
    // Old magnesium doesn't count — still advises low intake
    assert!(advice.unwrap().to_lowercase().contains("magnesium"));
}