//! Safety protocol tests — the 3 deterministic protocols from the biohack CLI,
//! plus stimulant/serotonergic classification and contextual advice (FR-009).

use chrono::{Duration, Utc};
use engine::catalog::seed_catalog;
use engine::models::*;
use engine::safety::{
    RecentSubstance, SafetyEngine, contextual_advice, is_serotonergic_name, is_stimulant_name,
};

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

fn make_dosed_log(name: &str, qty: f64, unit: &str) -> LogEntry {
    let mut e = make_log(name, ItemType::Supplement, 0);
    e.quantity = Some(qty);
    e.unit = Some(unit.to_string());
    e
}

// ── Dose vs catalog range ────────────────────────────────────────────────────

#[test]
fn test_dose_over_catalog_max_fires() {
    let engine = SafetyEngine::new();
    let catalog = seed_catalog();
    // L-theanine catalog max is 200 mg; 1000 mg is 5x -> critical
    let entry = make_dosed_log("L-Theanine", 1000.0, "mg");
    let alerts = engine.check_dose(&entry, &catalog);
    assert_eq!(alerts.len(), 1, "1000mg L-theanine must alert: {alerts:?}");
    assert_eq!(alerts[0].severity, AlertSeverity::Critical);
    assert!(alerts[0].message.contains("5.0x"));
}

#[test]
fn test_dose_mild_over_max_is_warning() {
    let engine = SafetyEngine::new();
    let catalog = seed_catalog();
    // Kratom max 5 g; 8 g = 1.6x -> warning, not critical
    let entry = make_dosed_log("Kratom", 8.0, "g");
    let alerts = engine.check_dose(&entry, &catalog);
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].severity, AlertSeverity::Warning);
}

#[test]
fn test_dose_within_catalog_max_is_silent() {
    let engine = SafetyEngine::new();
    let catalog = seed_catalog();
    let entry = make_dosed_log("L-Theanine", 200.0, "mg");
    assert!(engine.check_dose(&entry, &catalog).is_empty());
}

#[test]
fn test_dose_small_over_max_below_threshold_is_silent() {
    // 1.2x max: over, but below the 1.5x floor — noise, stay silent.
    let engine = SafetyEngine::new();
    let catalog = seed_catalog();
    let entry = make_dosed_log("Kratom", 6.0, "g");
    assert!(engine.check_dose(&entry, &catalog).is_empty());
}

#[test]
fn test_dose_unit_mismatch_is_silent() {
    // 1000 "g" of L-theanine is a unit error, not a dose violation.
    let engine = SafetyEngine::new();
    let catalog = seed_catalog();
    let entry = make_dosed_log("L-Theanine", 1000.0, "g");
    assert!(engine.check_dose(&entry, &catalog).is_empty());
}

#[test]
fn test_dose_unknown_substance_is_silent() {
    let engine = SafetyEngine::new();
    let catalog = seed_catalog();
    let entry = make_dosed_log("Unobtainium", 5000.0, "mg");
    assert!(engine.check_dose(&entry, &catalog).is_empty());
}

#[test]
fn test_dose_missing_quantity_is_silent() {
    let engine = SafetyEngine::new();
    let catalog = seed_catalog();
    let entry = make_log("L-Theanine", ItemType::Supplement, 0);
    assert!(engine.check_dose(&entry, &catalog).is_empty());
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
    assert!(
        result.alerts[0]
            .message
            .to_lowercase()
            .contains("hypertensive")
    );
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

#[test]
fn test_repeated_doses_count_as_one_serotonergic_agent() {
    // Four kratom doses in the window are ONE agent: dosing frequency is not
    // polypharmacy. The protocol must stay silent.
    let engine = SafetyEngine::new();
    let new_entry = make_log("Kratom", ItemType::Drug, 0);
    let existing = vec![
        make_log("Kratom", ItemType::Drug, 3),
        make_log("Kratom", ItemType::Drug, 8),
        make_log("Kratom", ItemType::Drug, 14),
    ];
    let alerts = engine.check_interactions(&new_entry, &existing);
    assert!(
        alerts.is_empty(),
        "repeated doses of one substance must not alert: {alerts:?}"
    );
}

#[test]
fn test_tagged_serotonergic_pair_alerts_when_protocol_silent() {
    // Kratom + rhodiola: both tagged Serotonergic, neither in the
    // name-substring list. The protocol counts 1 agent (kratom) and stays
    // silent, so the pair rule is the only signal that can reach this pair.
    let engine = SafetyEngine::new();
    let new_entry = make_log("Rhodiola Rosea", ItemType::Supplement, 0);
    let existing = vec![make_log("Kratom", ItemType::Drug, 2)];
    let alerts = engine.check_interactions(&new_entry, &existing);
    let sero = alerts
        .iter()
        .filter(|a| a.message.contains("serotonin-syndrome"))
        .count();
    assert_eq!(sero, 1, "kratom + rhodiola must fire the serotonergic pair rule: {alerts:?}");
    assert!(alerts.iter().any(|a| a.severity == AlertSeverity::Critical));
}

#[test]
fn test_serotonin_protocol_suppresses_duplicate_pair_rule() {
    // Two distinct serotonergics: the protocol fires with the richer message;
    // the pair rule must not add a second serotonin alert for the same pair.
    let engine = SafetyEngine::new();
    let new_entry = make_log("Fluoxetine", ItemType::Medication, 0);
    let existing = vec![make_log("Kratom", ItemType::Drug, 2)];
    let alerts = engine.check_interactions(&new_entry, &existing);
    let sero = alerts
        .iter()
        .filter(|a| a.message.contains("serotonergic agents"))
        .count();
    assert_eq!(sero, 1, "protocol fires exactly once: {alerts:?}");
    assert!(
        !alerts.iter().any(|a| a.message.contains("Two serotonergic agents")),
        "pair rule must not double-alert when the protocol fired: {alerts:?}"
    );
}

#[test]
fn test_pair_rule_deduplicates_per_pair_and_rule() {
    // CBD logged twice in the window: each CYP rule fires once, not twice.
    let engine = SafetyEngine::new();
    let new_entry = make_log("Kratom", ItemType::Drug, 0);
    let existing = vec![
        make_log("CBD", ItemType::Supplement, 2),
        make_log("CBD", ItemType::Supplement, 5),
    ];
    let alerts = engine.check_interactions(&new_entry, &existing);
    let cyp3a4 = alerts
        .iter()
        .filter(|a| a.message.contains("CYP3A4 inhibition"))
        .count();
    assert_eq!(cyp3a4, 1, "one CBD+kratom CYP3A4 alert per save: {alerts:?}");
}

#[test]
fn test_name_variants_deduplicate_as_one_substance() {
    // "CBD Oil" and "cbd oil " are the same substance: one dedup key.
    assert_eq!(
        engine::safety::normalize_substance("CBD Oil"),
        engine::safety::normalize_substance("cbd oil ")
    );
    assert_eq!(
        engine::safety::normalize_substance("St. John's Wort"),
        engine::safety::normalize_substance("st john's wort")
    );
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
