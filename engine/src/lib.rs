//! Engine library for biohacker tracking platform.
//!
//! Provides:
//! - Core data models (LogEntry, CatalogItem, Stack, VitalsEntry, Alert, Insight)
//! - Safety protocol engine (stimulant tachycardia, hypertensive urgency, serotonin syndrome)
//! - Catalog with 27 substance seed data
//! - SQLite-backed persistence via sqlx (when built natively)
//! - LocalStorage persistence via gloo_storage (when built for WASM)

pub mod models;
pub mod safety;
pub mod catalog;

#[cfg(feature = "db")]
pub mod db;

pub use models::*;
pub use safety::*;
pub use catalog::*;

#[cfg(feature = "db")]
pub use db::*;
