//! Engine library for biohacker tracking platform.
//!
//! Provides:
//! - Core data models (LogEntry, CatalogItem, Stack, VitalsEntry, Alert, Insight)
//! - Safety protocol engine (stimulant tachycardia, hypertensive urgency, serotonin syndrome)
//! - Catalog with 27 substance seed data
//! - SQLite-backed persistence via sqlx (when built natively)
//! - LocalStorage persistence via gloo_storage (when built for WASM)

pub mod catalog;
pub mod models;
pub mod safety;

#[cfg(feature = "db")]
pub mod db;

pub use catalog::*;
pub use models::*;
pub use safety::*;

#[cfg(feature = "db")]
pub use db::*;
