//! Spec-003 T233: regenerate pharmacokinetic alerts after every save.
//!
//! PK alerts (redose, cumulative load) describe the state of the *body*,
//! not the entry being saved — so unlike interaction/dose alerts they never
//! block a save. Instead the whole PK alert set is derived after each
//! successful write: previously-generated, still-open PK alerts are
//! resolved as superseded, and the current ones are recomputed from stored
//! data only (no network, no catalog fetch — the catalog is already in
//! localStorage). A dose that restores a band therefore resolves its redose
//! alert by construction (T232/A2: the recomputation finds no crossing).
//!
//! Resolution here is `resolve_alert` (resolved_at only), never
//! `acknowledge_alert`: a superseded system alert was not seen-and-accepted
//! by the user, and the banner only shows unresolved+unacknowledged alerts.

use crate::state::db::{
    create_alert, get_alerts, get_log_entries, get_target_bands, resolve_alert, search_catalog,
};
use chrono::{DateTime, Utc};
use engine::models::{AlertFilter, AlertType};
use engine::pharmacokinetics::DoseEvent;
use engine::pk_assignments::build_models;
use engine::redose::{cumulative_load_alerts, redose_alert};

const LOCAL_USER: &str = "local-device";

/// Supersede open PK alerts and write the current set. Returns the number of
/// new alerts created. Errors from storage propagate; an entry that cannot
/// become a DoseEvent (no quantity/unit) is skipped, matching
/// `DoseEvent::from_log_entry`'s reject-don't-default rule.
pub fn refresh_pharmacokinetic_alerts(now: DateTime<Utc>) -> Result<usize, String> {
    // 1. Retire the previous generation of system PK alerts.
    let open = get_alerts(&AlertFilter {
        user_id: Some(LOCAL_USER.to_string()),
        acknowledged: Some(false),
    })?;
    for a in open
        .into_iter()
        .filter(|a| matches!(a.alert_type, AlertType::Redose | AlertType::CumulativeLoad))
    {
        resolve_alert(&a.id)?;
    }

    // 2. Rebuild the dose history from stored log entries.
    let entries = get_log_entries()?;
    let doses: Vec<DoseEvent> = entries
        .iter()
        .filter_map(|e| DoseEvent::from_log_entry(e).ok())
        .collect();
    let catalog = search_catalog("")?;

    // 3. Redose alerts: one per target band whose substance is modelled.
    let models = build_models(&catalog);
    let mut fresh = Vec::new();
    for band in get_target_bands() {
        let Some(model) = models
            .iter()
            .find(|m| m.substance.eq_ignore_ascii_case(&band.substance))
        else {
            continue;
        };
        if !model.has_numeric_half_life() {
            continue; // nothing to project with — no alert, no guess
        }
        let doses_for: Vec<DoseEvent> = doses
            .iter()
            .filter(|d| d.substance.eq_ignore_ascii_case(&band.substance))
            .cloned()
            .collect();
        if doses_for.is_empty() {
            continue;
        }
        fresh.extend(redose_alert(
            LOCAL_USER, now, &doses_for, model, &band, None,
        ));
    }

    // 4. Cumulative-load alerts across the whole catalog.
    fresh.extend(cumulative_load_alerts(
        LOCAL_USER, now, &doses, &catalog, None,
    ));

    for a in &fresh {
        create_alert(a)?;
    }
    Ok(fresh.len())
}

/// T239 (A1/AC-3, FR-040): the "no model" gaps for substances the user
/// actually deals with — those with a target band or a log entry. The panel
/// renders these rows instead of a blank redose area: substance plus the
/// recorded reason. Silent omission would make alerting look broken;
/// explicit absence is the deliverable.
pub fn pk_coverage_gaps() -> Result<Vec<engine::explain::UnmodelledSubstance>, String> {
    let catalog = search_catalog("")?;
    let entries = get_log_entries()?;
    let key = |s: &str| s.trim().to_lowercase();
    let wanted: std::collections::HashSet<String> = get_target_bands()
        .iter()
        .map(|b| key(&b.substance))
        .chain(entries.iter().map(|e| key(&e.name)))
        .collect();
    Ok(engine::explain::unmodelled_report(&build_models(&catalog))
        .into_iter()
        .filter(|g| wanted.contains(&key(&g.substance)))
        .collect())
}
