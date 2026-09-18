use chrono::{DateTime, Utc};
use engine::models::*;

/// Unified entry type for the history view, combining log entries, vitals, and notes
#[derive(Debug, Clone)]
pub enum HistoryEntry {
    Log(LogEntry),
    Vitals(VitalsEntry),
    Note(Note),
}

impl HistoryEntry {
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            HistoryEntry::Log(e) => e.timestamp,
            HistoryEntry::Vitals(e) => e.timestamp,
            HistoryEntry::Note(e) => e.timestamp,
        }
    }

    pub fn name(&self) -> String {
        match self {
            HistoryEntry::Log(e) => e.name.clone(),
            HistoryEntry::Vitals(_) => "Vitals Reading".to_string(),
            HistoryEntry::Note(_) => "Note".to_string(),
        }
    }

    /// Human-readable detail line shown under the entry name:
    /// dosage for log entries, measurements for vitals, None for notes
    /// (note content is displayed as the note text).
    pub fn details(&self) -> Option<String> {
        match self {
            HistoryEntry::Log(e) => e.quantity.map(|q| {
                let unit = e.unit.as_deref().unwrap_or("");
                let qty_str = if q == q.trunc() {
                    format!("{}", q as i64)
                } else {
                    format!("{}", q)
                };
                if unit.is_empty() {
                    qty_str
                } else {
                    format!("{} {}", qty_str, unit)
                }
            }),
            HistoryEntry::Vitals(e) => {
                let parts: Vec<String> = [
                    match (e.bp_systolic, e.bp_diastolic) {
                        (Some(s), Some(d)) => Some(format!("BP {}/{}", s, d)),
                        (Some(s), None) => Some(format!("BP {}", s)),
                        (None, Some(d)) => Some(format!("BP {}", d)),
                        (None, None) => None,
                    },
                    e.heart_rate.map(|v| format!("HR {} bpm", v)),
                    e.weight.map(|v| format!("{:.1} kg", v)),
                    e.spo2.map(|v| format!("SpO2 {}%", v)),
                    e.temperature.map(|v| format!("{:.1}°C", v)),
                    e.hrv.map(|v| format!("HRV {} ms", v)),
                ]
                .into_iter()
                .flatten()
                .collect();
                if parts.is_empty() {
                    None
                } else {
                    Some(parts.join(" · "))
                }
            }
            HistoryEntry::Note(_) => None,
        }
    }

    pub fn category(&self) -> Option<String> {
        match self {
            HistoryEntry::Log(e) => Some(e.item_type.as_str().to_string()),
            HistoryEntry::Vitals(_) => Some("vitals".to_string()),
            HistoryEntry::Note(_) => Some("note".to_string()),
        }
    }
}
