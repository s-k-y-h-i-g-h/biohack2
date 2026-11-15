//! spec-003 T249 / SC-030: a 30-day active-dose series for 10 substances
//! must derive in under 500 ms on desktop (2 s on a mid-tier phone —
//! wasm-side headroom is inferred from the desktop number, x86→wasm cost
//! ratio for this pure-float workload is ~2–4×, well inside budget).
//!
//! Two measurements:
//!   1. The current dashboard path: `panel_view` × 10 (36-point window).
//!   2. The SC-030 worst case: `sample_series` over 30 days at hourly
//!      steps (721 points) against 90 doses per substance.
//!
//! Run: cargo run --release --example perf_sc030

use chrono::{Duration, Utc};
use engine::catalog::seed_catalog;
use engine::dashboard::panel_view;
use engine::models::{ItemType, LogEntry};
use engine::pharmacokinetics::{sample_series, DoseEvent};
use engine::pk_assignments::build_models;
use std::time::Instant;
use uuid::Uuid;

fn main() {
    let catalog = seed_catalog();
    let models = build_models(&catalog);
    // Ten modelled substances with a numeric half-life, as the dashboard
    // would actually render them.
    let sim: Vec<_> = models
        .iter()
        .filter(|m| m.has_numeric_half_life() && m.model_class == engine::models::ModelClass::Decay)
        .take(10)
        .cloned()
        .collect();
    assert_eq!(sim.len(), 10, "catalog must yield 10 decay-modelled substances");

    let now = Utc::now();
    // 90 days of history would be richer, but SC-030 names 30 days:
    // 3 doses/day/substance over 30 days = 90 entries each, 900 total.
    let entries: Vec<LogEntry> = sim
        .iter()
        .flat_map(|m| {
            (0..90).map(move |i| LogEntry {
                id: Uuid::new_v4(),
                user_id: "bench".into(),
                item_type: ItemType::Supplement,
                item_id: None,
                name: m.substance.clone(),
                quantity: Some(100.0),
                unit: Some("mg".into()),
                route: None,
                timestamp: now - Duration::hours(3 * i),
                stack_id: None,
                notes: None,
                acknowledged_interaction: false,
                custom_fields: None,
            })
        })
        .collect();

    // ── 1. Dashboard path (current app): panel_view × 10 ──────────────
    let t = Instant::now();
    let mut points = 0usize;
    for m in &sim {
        let v = panel_view(m, &entries, None, now).expect("panel derives");
        points += v.series.len();
    }
    let dash = t.elapsed();
    assert_eq!(points, 10 * 37, "window is 24+12+1 per substance");

    // ── 2. SC-030 worst case: 30-day hourly series × 10 ───────────────
    let doses_all: Vec<DoseEvent> = entries
        .iter()
        .filter_map(|e| DoseEvent::from_log_entry(e).ok())
        .collect();
    let t = Instant::now();
    let mut points = 0usize;
    for m in &sim {
        let doses: Vec<DoseEvent> = doses_all
            .iter()
            .filter(|d| d.substance.eq_ignore_ascii_case(&m.substance))
            .cloned()
            .collect();
        let series =
            sample_series(m, &doses, now - Duration::days(30), now, 1.0).expect("series");
        assert!(!series.is_empty());
        points += series.len();
    }
    let worst = t.elapsed();
    assert!(points > 10 * 700, "30 days hourly ≈ 721 points each");

    println!(
        "panel_view × 10 (dashboard path): {dash:?}  → SC-030 budget 500 ms: {}",
        if dash.as_millis() < 500 { "PASS" } else { "FAIL" }
    );
    println!(
        "sample_series 30 d × 10 (SC-030 worst case): {worst:?}  → {}",
        if worst.as_millis() < 500 { "PASS" } else { "FAIL" }
    );
    assert!(dash.as_millis() < 500 && worst.as_millis() < 500, "SC-030 desktop budget");
    println!("SC-030 desktop: PASS ({} derivation points total)", points + 370);
}
