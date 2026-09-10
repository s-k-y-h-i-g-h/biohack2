//! Engine library for biohacker tracking platform.
//!
//! Provides:
//! - Core data models (LogEntry, CatalogItem, Stack, VitalsEntry, Alert, Insight)
//! - Safety protocol engine (stimulant tachycardia, hypertensive urgency, serotonin syndrome)
//! - Catalog with 32 substance seed data
//! - SQLite-backed persistence via sqlx (when built natively)
//! - LocalStorage persistence via gloo_storage (when built for WASM)
//!
//! `db` re-exports are gated behind the same `db` feature as the module so the
//! glob never collides with `catalog::seed_catalog` (fn) vs `db::seed_catalog`
//! (async fn) in WASM builds where `db` is absent.

pub mod catalog;
pub mod models;
pub mod safety;

#[cfg(feature = "db")]
pub mod db;

pub use catalog::*;
pub use models::*;
pub use safety::*;

// Note: `db` is deliberately NOT glob re-exported — `db::seed_catalog` (async,
// DB insert) would collide with `catalog::seed_catalog` (sync, seed data).
// Consumers use `engine::db::…` paths explicitly.
#[cfg(feature = "db")]
pub use db::DbPool;
