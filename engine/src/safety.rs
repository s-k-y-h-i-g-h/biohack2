//! Safety protocol engine.
//!
//! Implements 3 deterministic protocols ported from the biohack CLI:
//! 1. Stimulant tachycardia (HR > 100 + stimulant within 4h)
//! 2. Hypertensive urgency (SBP >= 180 or DBP >= 120)
//! 3. Serotonin syndrome risk (multiple serotonergic agents)

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::models::{Alert, AlertSeverity, AlertType, ItemType, LogEntry, VitalsEntry};

/// Represents a recent substance log for context-aware alerting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentSubstance {
    pub name: String,
    pub category: ItemType,
    pub taken_at: chrono::DateTime<Utc>,
    pub is_stimulant: bool,
    pub is_serotonergic: bool,
}

/// Results from running safety protocols against vitals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyResult {
    pub alerts: Vec<Alert>,
    pub protocols_triggered: Vec<String>,
}

/// The safety protocol engine.
pub struct SafetyEngine {
    tachycardia_threshold: i32,
    hypertensive_sbp_threshold: i32,
    hypertensive_dbp_threshold: i32,
    stimulant_window_hours: u64,
    serotonergic_count_threshold: u32,
}

impl Default for SafetyEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SafetyEngine {
    /// Creates a new safety engine with default thresholds.
    pub fn new() -> Self {
        Self {
            tachycardia_threshold: 100,
            hypertensive_sbp_threshold: 180,
            hypertensive_dbp_threshold: 120,
            stimulant_window_hours: 4,
            serotonergic_count_threshold: 2,
        }
    }

    /// Creates a new safety engine with custom thresholds (for testing).
    #[cfg(test)]
    pub fn with_thresholds(tachycardia: i32, hypertensive_sbp: i32, hypertensive_dbp: i32) -> Self {
        Self {
            tachycardia_threshold: tachycardia,
            hypertensive_sbp_threshold: hypertensive_sbp,
            hypertensive_dbp_threshold: hypertensive_dbp,
            stimulant_window_hours: 4,
            serotonergic_count_threshold: 2,
        }
    }

    /// Checks vitals against all safety protocols.
    ///
    /// Returns alerts for any triggered protocols.
    pub fn check_vitals(
        &self,
        entry: &VitalsEntry,
        recent_substances: &[RecentSubstance],
    ) -> SafetyResult {
        let mut alerts = Vec::new();
        let mut triggered = Vec::new();

        // Protocol 1: Stimulant tachycardia
        if let Some(alert) = self.check_stimulant_tachycardia(entry, recent_substances) {
            alerts.push(alert);
            triggered.push("stimulant_tachycardia".to_string());
        }

        // Protocol 2: Hypertensive urgency
        if let Some(alert) = self.check_hypertensive_urgency(entry) {
            alerts.push(alert);
            triggered.push("hypertensive_urgency".to_string());
        }

        SafetyResult {
            alerts,
            protocols_triggered: triggered,
        }
    }

    /// Checks for dangerous drug/supplement interactions.
    ///
    /// This handles Protocol 3: serotonin syndrome risk from multiple serotonergic agents.
    pub fn check_interactions(
        &self,
        new_entry: &LogEntry,
        existing_entries: &[LogEntry],
    ) -> Vec<Alert> {
        let mut alerts = Vec::new();

        // Check for serotonin syndrome risk
        let serotonergic_count = self.count_serotonergic_agents(new_entry, existing_entries);
        if serotonergic_count >= self.serotonergic_count_threshold {
            alerts.push(Alert {
                id: uuid::Uuid::new_v4(),
                user_id: new_entry.user_id.clone(),
                alert_type: AlertType::Interaction,
                severity: AlertSeverity::Critical,
                message: format!(
                    "Serotonin syndrome risk: {} serotonergic agents detected",
                    serotonergic_count
                ),
                recommendation: Some(
                    "Monitor for: clonus, hyperreflexia, hyperthermia, diaphoresis, agitation. \
                     Seek emergency care if symptoms develop."
                        .to_string(),
                ),
                is_acknowledged: false,
                linked_entry_id: Some(new_entry.id),
                generated_at: Utc::now(),
                resolved_at: None,
            });
        }

        alerts
    }

    // ── Protocol Implementations ──────────────────────────────────────────────

    fn check_stimulant_tachycardia(
        &self,
        entry: &VitalsEntry,
        recent_substances: &[RecentSubstance],
    ) -> Option<Alert> {
        let hr = entry.heart_rate?;
        if hr <= self.tachycardia_threshold {
            return None;
        }

        // Check if any stimulant was taken within the window
        let window_start = Utc::now() - Duration::hours(self.stimulant_window_hours as i64);
        let has_recent_stimulant = recent_substances.iter().any(|s| s.is_stimulant && s.taken_at >= window_start);

        if !has_recent_stimulant {
            return None;
        }

        Some(Alert {
            id: uuid::Uuid::new_v4(),
            user_id: entry.user_id.clone(),
            alert_type: AlertType::Vital,
            severity: AlertSeverity::Critical,
            message: format!("Stimulant-associated tachycardia: HR {} bpm with recent stimulant use", hr),
            recommendation: Some(
                "Consider: cold face immersion (30s), hydrate with electrolytes, \
                 magnesium glycinate 400mg, L-theanine 200-400mg. \
                 No further stimulants for 6 hours."
                    .to_string(),
            ),
            is_acknowledged: false,
            linked_entry_id: Some(entry.id),
            generated_at: entry.timestamp,
            resolved_at: None,
        })
    }

    fn check_hypertensive_urgency(&self, entry: &VitalsEntry) -> Option<Alert> {
        let sbp = entry.bp_systolic?;
        let dbp = entry.bp_diastolic?;

        let is_hypertensive = sbp >= self.hypertensive_sbp_threshold || dbp >= self.hypertensive_dbp_threshold;

        if !is_hypertensive {
            return None;
        }

        Some(Alert {
            id: uuid::Uuid::new_v4(),
            user_id: entry.user_id.clone(),
            alert_type: AlertType::Vital,
            severity: AlertSeverity::Critical,
            message: format!(
                "Hypertensive urgency: BP {}/{} mmHg (threshold: {}/{})",
                sbp, dbp, self.hypertensive_sbp_threshold, self.hypertensive_dbp_threshold
            ),
            recommendation: Some(
                "Seek medical attention if symptoms present (chest pain, dyspnea, \
                 neuro symptoms, vision changes). Otherwise: slow breathing (6/min for 5min), \
                 hydrate, avoid caffeine/stimulants/NSAIDs, recheck in 30 minutes."
                    .to_string(),
            ),
            is_acknowledged: false,
            linked_entry_id: Some(entry.id),
            generated_at: entry.timestamp,
            resolved_at: None,
        })
    }

    fn count_serotonergic_agents(&self, new_entry: &LogEntry, existing: &[LogEntry]) -> u32 {
        let window_start = Utc::now() - Duration::hours(24);
        let mut count = 0u32;

        if is_serotonergic_item(new_entry) {
            count += 1;
        }

        for entry in existing {
            if entry.timestamp >= window_start && is_serotonergic_item(entry) {
                count += 1;
            }
        }

        count
    }
}

/// Returns true if the log entry represents a serotonergic substance.
fn is_serotonergic_item(entry: &LogEntry) -> bool {
    (matches!(entry.item_type, ItemType::Medication | ItemType::Drug)
        || matches!(entry.item_type, ItemType::Supplement))
        && (entry
            .name
            .to_lowercase()
            .contains("ssri")
            || entry.name.to_lowercase().contains("snao")
            || entry.name.to_lowercase().contains("mao")
            || entry.name.to_lowercase().contains("tryptophan")
            || entry.name.to_lowercase().contains("5-htp")
            || entry.name.to_lowercase().contains("tramadol")
            || entry.name.to_lowercase().contains("dextromethorphan")
            || entry.name.to_lowercase().contains("st john")
            || entry.name.to_lowercase().contains("fluoxetine")
            || entry.name.to_lowercase().contains("sertraline")
            || entry.name.to_lowercase().contains("paroxetine"))
}

/// Runs all safety checks for a vitals entry.
pub fn run_safety_check(entry: &VitalsEntry, recent_substances: &[RecentSubstance]) -> SafetyResult {
    let engine = SafetyEngine::new();
    engine.check_vitals(entry, recent_substances)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

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

    fn make_substance(name: &str, is_stimulant: bool, _is_serotonergic: bool) -> RecentSubstance {
        RecentSubstance {
            name: name.to_string(),
            category: ItemType::Supplement,
            taken_at: Utc::now() - Duration::hours(1),
            is_stimulant,
            is_serotonergic: false,
        }
    }

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

    #[test]
    fn test_serotonin_syndrome_detection() {
        let engine = SafetyEngine::new();
        let new_entry = LogEntry {
            id: uuid::Uuid::new_v4(),
            user_id: "test".to_string(),
            item_type: ItemType::Medication,
            item_id: None,
            name: "Fluoxetine".to_string(),
            quantity: None,
            unit: None,
            route: None,
            timestamp: Utc::now(),
            stack_id: None,
            notes: None,
            acknowledged_interaction: false,
            custom_fields: None,
        };
        let existing = vec![
            LogEntry {
                id: uuid::Uuid::new_v4(),
                user_id: "test".to_string(),
                item_type: ItemType::Supplement,
                item_id: None,
                name: "5-HTP".to_string(),
                quantity: None,
                unit: None,
                route: None,
                timestamp: Utc::now() - Duration::hours(1),
                stack_id: None,
                notes: None,
                acknowledged_interaction: false,
                custom_fields: None,
            },
            LogEntry {
                id: uuid::Uuid::new_v4(),
                user_id: "test".to_string(),
                item_type: ItemType::Supplement,
                item_id: None,
                name: "St. John's Wort".to_string(),
                quantity: None,
                unit: None,
                route: None,
                timestamp: Utc::now() - Duration::hours(2),
                stack_id: None,
                notes: None,
                acknowledged_interaction: false,
                custom_fields: None,
            },
        ];
        let alerts = engine.check_interactions(&new_entry, &existing);
        assert!(!alerts.is_empty());
        assert_eq!(alerts[0].alert_type, AlertType::Interaction);
    }

    #[test]
    fn test_no_serotonin_alert_for_non_serotonergic() {
        let engine = SafetyEngine::new();
        let new_entry = LogEntry {
            id: uuid::Uuid::new_v4(),
            user_id: "test".to_string(),
            item_type: ItemType::Supplement,
            item_id: None,
            name: "Vitamin D3".to_string(),
            quantity: None,
            unit: None,
            route: None,
            timestamp: Utc::now(),
            stack_id: None,
            notes: None,
            acknowledged_interaction: false,
            custom_fields: None,
        };
        let existing = vec![LogEntry {
            id: uuid::Uuid::new_v4(),
            user_id: "test".to_string(),
            item_type: ItemType::Supplement,
            item_id: None,
            name: "Magnesium".to_string(),
            quantity: None,
            unit: None,
            route: None,
            timestamp: Utc::now() - Duration::hours(1),
            stack_id: None,
            notes: None,
            acknowledged_interaction: false,
            custom_fields: None,
        }];
        let alerts = engine.check_interactions(&new_entry, &existing);
        assert!(alerts.is_empty());
    }
}
