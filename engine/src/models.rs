//! Core data models for the biohacker tracking platform.
//!
//! All types are serializable with serde and map directly to SQLite tables.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Enums ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ItemType {
    #[default]
    Supplement,
    Medication,
    Drug,
    Food,
    Action,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteType {
    Oral,
    Sublingual,
    Topical,
    Inhalation,
    Injectable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SleepQuality {
    Poor,
    Fair,
    Good,
    Excellent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    #[default]
    Vital,
    Interaction,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    #[default]
    Warning,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InsightType {
    #[default]
    Correlation,
    Trend,
    Pattern,
}

// ── Value Objects ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DosageRange {
    pub min: f64,
    pub max: f64,
    pub unit: String,
}

// ── Entities ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: Uuid,
    pub user_id: String,
    pub item_type: ItemType,
    pub item_id: Option<Uuid>,
    pub name: String,
    pub quantity: Option<f64>,
    pub unit: Option<String>,
    pub route: Option<RouteType>,
    pub timestamp: DateTime<Utc>,
    pub stack_id: Option<Uuid>,
    pub notes: Option<String>,
    pub acknowledged_interaction: bool,
    pub custom_fields: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogItem {
    pub id: Uuid,
    pub name: String,
    pub category: ItemType,
    pub dosage_range: Option<DosageRange>,
    pub half_life: Option<String>,
    pub contraindications: Vec<String>,
    pub warnings: Vec<String>,
    pub is_custom: bool,
    pub source: Option<String>,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stack {
    pub id: Uuid,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub items: Vec<StackItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackItem {
    pub item_id: Uuid,
    pub quantity: Option<f64>,
    pub unit: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalsEntry {
    pub id: Uuid,
    pub user_id: String,
    pub timestamp: DateTime<Utc>,
    pub bp_systolic: Option<i32>,
    pub bp_diastolic: Option<i32>,
    pub heart_rate: Option<i32>,
    pub weight: Option<f64>,
    pub blood_glucose: Option<f64>,
    pub temperature: Option<f64>,
    pub spo2: Option<i32>,
    pub hrv: Option<f64>,
    pub sleep_quality: Option<SleepQuality>,
    pub custom_metrics: Option<serde_json::Value>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub user_id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub recommendation: Option<String>,
    pub is_acknowledged: bool,
    pub linked_entry_id: Option<Uuid>,
    pub generated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Insight {
    pub id: Uuid,
    pub user_id: String,
    pub insight_type: InsightType,
    pub title: String,
    pub description: String,
    pub confidence: f64,
    pub supporting_data_points: i32,
    pub generated_at: DateTime<Utc>,
    pub related_entry_ids: Vec<Uuid>,
}

/// Standalone free-text note (realization, observation) logged at a point in time.
/// A first-class entry like LogEntry and VitalsEntry — NOT an attachment to another entry.
/// The `notes` field on LogEntry is deprecated; new notes MUST be Note entities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub id: Uuid,
    pub user_id: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    /// Optional reference to a LogEntry for context
    pub linked_entry_id: Option<Uuid>,
}

// ── Query Types ───────────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone)]
pub struct LogEntryFilter {
    pub user_id: Option<String>,
    pub stack_id: Option<Uuid>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub category: Option<ItemType>,
}

#[derive(Debug, Default, Clone)]
pub struct VitalsEntryFilter {
    pub user_id: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Default, Clone)]
pub struct AlertFilter {
    pub user_id: Option<String>,
    pub acknowledged: Option<bool>,
}

// ── Sync merge support ────────────────────────────────────────────────────────

/// Uniform id/timestamp accessors so the web sync layer can merge any
/// entity collection by id without per-type closures.
pub trait SortableEntry {
    fn sort_id(&self) -> Uuid;
    fn sort_time(&self) -> chrono::DateTime<chrono::Utc>;
}

impl SortableEntry for LogEntry {
    fn sort_id(&self) -> Uuid {
        self.id
    }
    fn sort_time(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp
    }
}

impl SortableEntry for VitalsEntry {
    fn sort_id(&self) -> Uuid {
        self.id
    }
    fn sort_time(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp
    }
}

impl SortableEntry for Alert {
    fn sort_id(&self) -> Uuid {
        self.id
    }
    fn sort_time(&self) -> chrono::DateTime<chrono::Utc> {
        self.generated_at
    }
}

impl SortableEntry for Note {
    fn sort_id(&self) -> Uuid {
        self.id
    }
    fn sort_time(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp
    }
}
