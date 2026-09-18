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

impl RecentSubstance {
    /// Classify a [`LogEntry`] for context-aware safety alerting.
    ///
    /// The single source of this mapping: every caller (server handler and
    /// the WASM vitals page) must go through here so the stimulant/
    /// serotonergic classification cannot drift between the two.
    pub fn from_log_entry(log: &LogEntry) -> Self {
        Self {
            name: log.name.clone(),
            category: log.item_type,
            taken_at: log.timestamp,
            is_stimulant: is_stimulant_name(&log.name),
            is_serotonergic: is_serotonergic_name(&log.name),
        }
    }

    /// Classify a slice of log entries, e.g. recent history for a vitals check.
    pub fn from_log_entries(logs: &[LogEntry]) -> Vec<Self> {
        logs.iter().map(Self::from_log_entry).collect()
    }
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
        let has_recent_stimulant = recent_substances
            .iter()
            .any(|s| s.is_stimulant && s.taken_at >= window_start);

        if !has_recent_stimulant {
            return None;
        }

        Some(Alert {
            id: uuid::Uuid::new_v4(),
            user_id: entry.user_id.clone(),
            alert_type: AlertType::Vital,
            severity: AlertSeverity::Critical,
            message: format!(
                "Stimulant-associated tachycardia: HR {} bpm with recent stimulant use",
                hr
            ),
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

        let is_hypertensive =
            sbp >= self.hypertensive_sbp_threshold || dbp >= self.hypertensive_dbp_threshold;

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
    is_serotonergic_name(&entry.name)
}

/// Returns true if the name matches a serotonergic substance.
pub fn is_serotonergic_name(name: &str) -> bool {
    // Normalize: lowercase, punctuation → space, collapse runs of spaces
    // so "St. John's Wort" → "st john s wort" (matches "st john")
    let cleaned: String = name
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '-' {
                c
            } else {
                ' '
            }
        })
        .collect();
    let n = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    n.contains("ssri")
        || n.contains("snri")
        || n.contains("mao")
        || n.contains("tryptophan")
        || n.contains("5-htp")
        || n.contains("tramadol")
        || n.contains("dextromethorphan")
        || n.contains("st john")
        || n.contains("fluoxetine")
        || n.contains("sertraline")
        || n.contains("paroxetine")
        || n.contains("kratom")
        || n.contains("mitragynine")
}

/// Returns true if the name matches a stimulant substance.
pub fn is_stimulant_name(name: &str) -> bool {
    let n = name.to_lowercase();
    n.contains("caffeine")
        || n.contains("coffee")
        || n.contains("espresso")
        || n.contains("tea")
        || n.contains("energy drink")
        || n.contains("ephedrine")
        || n.contains("amphetamine")
        || n.contains("adderall")
        || n.contains("ritalin")
        || n.contains("methylphenidate")
        || n.contains("modafinil")
        || n.contains("armodafinil")
        || n.contains("nicotine")
        || n.contains("yohimbine")
        || n.contains("synephrine")
        || n.contains("cyclazodone")
        || n.contains("pemoline")
        || n.contains("focalin")
        || n.contains("vyvanse")
        || n.contains("lisdexamfetamine")
        || n.contains("kratom")
}

/// Runs all safety checks for a vitals entry.
pub fn run_safety_check(
    entry: &VitalsEntry,
    recent_substances: &[RecentSubstance],
) -> SafetyResult {
    let engine = SafetyEngine::new();
    engine.check_vitals(entry, recent_substances)
}

/// Contextual advice derived from the user's recent consumption log (FR-009).
///
/// Cross-references recent supplements/medications/actions to enrich an
/// alert's recommendation with log-derived context (e.g. "no magnesium this
/// week" for hypertension, "recent stimulant use" for tachycardia).
pub fn contextual_advice(alert: &Alert, recent_substances: &[RecentSubstance]) -> Option<String> {
    let now = Utc::now();
    let week_ago = now - Duration::days(7);

    let taken_last_week: Vec<&RecentSubstance> = recent_substances
        .iter()
        .filter(|s| s.taken_at >= week_ago)
        .collect();

    let msg = alert.message.to_lowercase();

    // Hypertension → check magnesium (relaxation of vascular smooth muscle)
    if msg.contains("hypertens") {
        let has_magnesium = taken_last_week.iter().any(|s| {
            let n = s.name.to_lowercase();
            n.contains("magnesium") || n.contains("mg glycinate") || n.contains("citrate")
        });
        let has_stimulant_use = taken_last_week.iter().any(|s| s.is_stimulant);

        let mut parts = Vec::new();
        if !has_magnesium {
            parts.push(
                "Your magnesium intake has been low this week — consider magnesium glycinate 200-400mg"
                    .to_string(),
            );
        }
        if has_stimulant_use {
            parts.push(
                "You've used stimulants recently — avoid caffeine and other stimulants until BP normalizes"
                    .to_string(),
            );
        }
        if parts.is_empty() {
            return None;
        }
        return Some(parts.join(". "));
    }

    // Tachycardia → stimulant context
    if msg.contains("tachycardia") {
        let recent_stimulants: Vec<&str> = taken_last_week
            .iter()
            .filter(|s| s.is_stimulant)
            .map(|s| s.name.as_str())
            .collect();
        if !recent_stimulants.is_empty() {
            return Some(format!(
                "Recent stimulants in your log: {}. Allow 6+ hours before any further stimulant use.",
                recent_stimulants.join(", ")
            ));
        }
        return None;
    }

    // Interaction alerts → list co-ingested agents
    if msg.contains("serotonin") {
        let serotonergic: Vec<&str> = taken_last_week
            .iter()
            .filter(|s| s.is_serotonergic)
            .map(|s| s.name.as_str())
            .collect();
        if !serotonergic.is_empty() {
            return Some(format!(
                "Serotonergic agents in your recent log: {}.",
                serotonergic.join(", ")
            ));
        }
    }

    None
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
