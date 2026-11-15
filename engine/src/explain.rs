//! FR-037 / FR-039 — the audit surface for one simulated substance.
//!
//! `explain` answers, for a single substance, exactly what a simulation
//! used: model class, parameters, provenance (catalog origin plus any
//! timestamped user overrides, as chained by [`SubstanceModel`]), and the
//! dose events that entered the computation — separated from the events it
//! refused. The limitations text (T238) names the inputs that produced the
//! band and states in plain words that the result is an estimate, not a
//! measurement.

use crate::models::{LogEntry, ModelClass};
use crate::pharmacokinetics::DEFAULT_VARIABILITY;
use crate::pk_params::SubstanceModel;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Case-insensitive substance/name key — the same normalisation the alert
/// path uses, so "events used" here and "events used" there cannot drift.
fn name_key(s: &str) -> String {
    s.trim().to_lowercase()
}

/// The full explanation for one substance, as shown by the audit panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelExplanation {
    pub substance: String,
    pub model_class: ModelClass,
    /// Nominal half-life actually in effect (None for plateau/none).
    pub half_life_hours: Option<f64>,
    /// Catalog half-life range in effect, if the model carries one.
    pub half_life_range: Option<(f64, f64)>,
    /// Class-specific parameters exactly as the simulation sees them.
    pub params: serde_json::Value,
    /// Full provenance chain: catalog origin plus any `user override at …`
    /// links appended by `with_user_half_life_override`.
    pub provenance: String,
    pub catalog_version: i32,
    pub rationale: Option<String>,
    /// Dose events that would enter a simulation from `now` backwards:
    /// same substance, at or before `now`, with a positive finite quantity.
    pub events_used: Vec<Uuid>,
    /// Same-substance events the simulation refuses, with the reason per
    /// event — a silent drop would make the audit untrustworthy.
    pub events_refused: Vec<(Uuid, String)>,
    /// T238: the limitations text, generated from the inputs themselves.
    pub limitations: String,
}

/// Build the explanation for `model` against `events`, as of `now`.
/// Pure: no clock, no storage, no network.
pub fn explain(
    model: &SubstanceModel,
    events: &[LogEntry],
    now: DateTime<Utc>,
) -> ModelExplanation {
    let key = name_key(&model.substance);
    let mut events_used = Vec::new();
    let mut events_refused = Vec::new();
    for e in events {
        if name_key(&e.name) != key {
            continue;
        }
        if e.timestamp > now {
            events_refused.push((e.id, "timestamp is in the future".into()));
            continue;
        }
        match e.quantity {
            Some(q) if q.is_finite() && q > 0.0 => events_used.push(e.id),
            Some(q) => events_refused.push((
                e.id,
                format!("quantity {q} is not a positive finite number"),
            )),
            None => events_refused.push((e.id, "no quantity recorded".into())),
        }
    }
    ModelExplanation {
        substance: model.substance.clone(),
        model_class: model.model_class,
        half_life_hours: model.half_life_hours,
        half_life_range: model.half_life_range,
        params: model.params.clone(),
        provenance: model.provenance.clone(),
        catalog_version: model.catalog_version,
        rationale: model.rationale.clone(),
        events_used,
        events_refused,
        limitations: limitations_text(model),
    }
}

/// T238 (FR-039, A5): limitations text generated from the model's actual
/// inputs — never a canned string. Every named number must exist on the
/// model; if a number is not on the model it is not named.
pub fn limitations_text(model: &SubstanceModel) -> String {
    let mut parts: Vec<String> = Vec::new();

    if model.model_class == ModelClass::None || !model.has_numeric_half_life() {
        let reason = model
            .rationale
            .as_deref()
            .unwrap_or("no numeric half-life is recorded for this substance");
        parts.push(format!(
            "{}: not modelled — {}. No curve or level is produced; only time-since-dose facts apply.",
            model.substance, reason
        ));
        return parts.join(" ");
    }

    parts.push(format!(
        "{}: these levels are an estimate, not a measurement.",
        model.substance
    ));

    // Name the inputs that produced the band.
    let mut inputs: Vec<String> = Vec::new();
    if let Some(h) = model.half_life_hours {
        inputs.push(format!("nominal half-life {h} h"));
    }
    if let Some((lo, hi)) = model.half_life_range {
        inputs.push(format!("catalog half-life range {lo}–{hi} h"));
    } else if model.half_life_hours.is_some() {
        inputs.push(format!(
            "no catalog range, so the band is widened by the documented ±{:.0}% variability assumption",
            DEFAULT_VARIABILITY * 100.0
        ));
    }
    if let Some(obj) = model.params.as_object().filter(|o| !o.is_empty()) {
        let mut keys: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
        keys.sort();
        inputs.push(format!("parameters {}", keys.join(", ")));
    }
    parts.push(format!("Inputs used: {}.", inputs.join("; ")));

    parts.push(format!(
        "Model class: {:?}. Provenance: {} (catalog version {}).",
        model.model_class, model.provenance, model.catalog_version
    ));

    parts.push(
        "The band reflects inter-individual variation in the inputs above; \
         the true level in your body right now is not knowable from a dose \
         log — only a blood draw could answer that."
            .into(),
    );
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ItemType;
    use chrono::Duration;

    fn model() -> SubstanceModel {
        SubstanceModel {
            substance: "Caffeine".into(),
            model_class: ModelClass::Decay,
            half_life_hours: Some(5.0),
            half_life_range: Some((4.0, 6.0)),
            params: serde_json::Value::Null,
            provenance: "catalog seed '5h'".into(),
            catalog_version: 3,
            rationale: None,
        }
    }

    fn entry(name: &str, qty: Option<f64>, age_hours: i64, now: DateTime<Utc>) -> LogEntry {
        LogEntry {
            id: Uuid::new_v4(),
            user_id: "local-device".into(),
            item_type: ItemType::Supplement,
            item_id: None,
            name: name.into(),
            quantity: qty,
            unit: Some("mg".into()),
            route: None,
            timestamp: now - Duration::hours(age_hours),
            stack_id: None,
            notes: None,
            acknowledged_interaction: false,
            custom_fields: None,
        }
    }

    #[test]
    fn explain_returns_class_parameters_provenance_and_events_used() {
        let now = Utc::now();
        let m = model();
        let used = entry("caffeine", Some(100.0), 3, now); // case-insensitive
        let future = entry("Caffeine", Some(50.0), -2, now); // in the future
        let noqty = entry("CAFFEINE", None, 5, now);
        let zero = entry("caffeine", Some(0.0), 6, now);
        let other = entry("Magnesium", Some(200.0), 4, now); // different substance
        let e = explain(&m, &[used.clone(), future, noqty, zero, other], now);
        assert_eq!(e.model_class, ModelClass::Decay);
        assert_eq!(e.events_used, vec![used.id]);
        assert_eq!(e.events_refused.len(), 3);
        assert!(e.events_refused.iter().any(|(_, r)| r.contains("future")));
        assert!(
            e.events_refused
                .iter()
                .any(|(_, r)| r.contains("no quantity"))
        );
        assert!(e.provenance.contains("catalog seed '5h'"));
        assert_eq!(e.catalog_version, 3);
    }

    #[test]
    fn limitations_text_names_actual_inputs_and_says_estimate() {
        let m = model();
        let t = limitations_text(&m);
        assert!(t.contains("estimate, not a measurement"), "{t}");
        assert!(t.contains("nominal half-life 5 h"), "{t}");
        assert!(t.contains("catalog half-life range 4–6 h"), "{t}");
        assert!(!t.contains("variability assumption"), "{t}"); // range exists → no assumption wording
    }

    #[test]
    fn limitations_text_flags_assumed_band_and_unmodelled_class() {
        let mut m = model();
        m.half_life_range = None;
        let t = limitations_text(&m);
        assert!(t.contains("±25% variability assumption"), "{t}");

        let mut u = SubstanceModel::unmodelled("Weird Stuff", "no published PK", 3);
        let t2 = limitations_text(&u);
        assert!(t2.contains("not modelled"), "{t2}");
        assert!(t2.contains("No curve"), "{t2}");
        // Unmodelled substances must not claim to be a level estimate.
        assert!(!t2.contains("estimate, not a measurement"), "{t2}");
        u.half_life_hours = None;
        assert!(!u.has_numeric_half_life());
    }
}
