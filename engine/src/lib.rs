//! Engine library for biohacker tracking platform.
//!
//! Provides:
//! - Core data models (LogEntry, CatalogItem, Stack, VitalsEntry, Alert, Insight)
//! - Safety protocol engine (stimulant tachycardia, hypertensive urgency, serotonin syndrome)
//! - Catalog with 27 substance seed data
//! - SQLite-backed persistence via sqlx

pub mod models;
pub mod safety;
pub mod catalog;
pub mod db;

pub use models::*;
pub use safety::*;
pub use catalog::*;
pub use db::*;
