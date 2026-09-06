use engine::models::*;
use chrono::{DateTime, Utc};

/// Unified entry type for the history view, combining log entries and vitals
#[derive(Debug, Clone)]
pub enum HistoryEntry {
    Log(LogEntry),
    Vitals(VitalsEntry),
}

impl HistoryEntry {
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            HistoryEntry::Log(e) => e.timestamp,
            HistoryEntry::Vitals(e) => e.timestamp,
        }
    }

    pub fn name(&self) -> String {
        match self {
            HistoryEntry::Log(e) => e.name.clone(),
            HistoryEntry::Vitals(_) => "Vitals Reading".to_string(),
        }
    }

    pub fn category(&self) -> Option<String> {
        match self {
            HistoryEntry::Log(e) => Some(match e.item_type {
                ItemType::Supplement => "supplement",
                ItemType::Medication => "medication",
                ItemType::Drug => "drug",
                ItemType::Food => "food",
                ItemType::Action => "action",
            }.to_string()),
            HistoryEntry::Vitals(_) => Some("vitals".to_string()),
        }
    }
}
