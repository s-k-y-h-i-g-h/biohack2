//! Per-substance PK model parameters (spec 003 Phase 1, T202).
//!
//! A `SubstanceModel` is the machine-readable claim "this substance is
//! modelled THIS way with THESE parameters, and here is where the numbers
//! came from". Provenance is mandatory: a parameter without a source is a
//! guess wearing a lab coat.
//!
//! Persistence (T206/T207) is wired: SQLite via `db::upsert_substance_models`
//! / `db::get_substance_models`, WASM via `web::state::db::get_substance_models`.

use crate::models::ModelClass;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// One substance's model assignment and parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubstanceModel {
    /// Catalog item name (denormalised on purpose — models must survive
    /// catalog re-seeding with new UUIDs).
    pub substance: String,
    /// How it is modelled. `ModelClass::None` items still get a row so the
    /// audit is total: absence of a row means "never audited", which is not
    /// the same as "explicitly not modelled".
    pub model_class: ModelClass,
    /// Single-value half-life, hours. Set only for modelled substances.
    #[serde(default)]
    pub half_life_hours: Option<f64>,
    /// Range half-life, hours (low, high). Endpoints preserved (FR-033).
    #[serde(default)]
    pub half_life_range: Option<(f64, f64)>,
    /// Model-class-specific parameters (absorption rate, saturation ceiling,
    /// steady-state days…). Free-form JSON so Phase 2 can extend per class
    /// without schema churn; validated by each model at use time, not here.
    #[serde(default)]
    pub params: serde_json::Value,
    /// Where the numbers came from — e.g. `"catalog seed string '5h'"`,
    /// `"pubmed:xxxxxxxx"`. Required; empty means a violation, not a default.
    pub provenance: String,
    /// Catalog seed version this row was derived from, so stale models can
    /// be detected when the catalog changes.
    pub catalog_version: i32,
    /// Why `model_class` was chosen / why not modelled. Populated for every
    /// non-`decay` assignment and for every `none` (T204/T205 rationale).
    #[serde(default)]
    pub rationale: Option<String>,
}

impl SubstanceModel {
    /// An explicitly-unmodelled substance: `none` class with a logged reason.
    /// This is what parse failures become (T204) — visible, not silent.
    pub fn unmodelled(
        substance: impl Into<String>,
        reason: impl Into<String>,
        catalog_version: i32,
    ) -> Self {
        SubstanceModel {
            substance: substance.into(),
            model_class: ModelClass::None,
            half_life_hours: None,
            half_life_range: None,
            params: serde_json::Value::Null,
            provenance: "half_life parse failure (T204 audit)".to_string(),
            catalog_version,
            rationale: Some(reason.into()),
        }
    }

    /// True when this model carries at least one usable numeric half-life.
    pub fn has_numeric_half_life(&self) -> bool {
        self.half_life_hours.is_some() || self.half_life_range.is_some()
    }

    /// FR-036: a user override of the half-life, recorded WITH a timestamp.
    /// The prior provenance is chained, never replaced — an audit must be
    /// able to see both the catalog origin and the human edit. The override
    /// is a point value: the catalog range is cleared so it cannot silently
    /// win over the user's number in `band_half_lives`.
    pub fn with_user_half_life_override(
        &mut self,
        hours: f64,
        at: DateTime<Utc>,
    ) -> Result<(), String> {
        if !hours.is_finite() || hours <= 0.0 {
            return Err(format!(
                "half-life override {hours} is not a positive finite number of hours"
            ));
        }
        let prev = if self.provenance.trim().is_empty() {
            "no prior provenance recorded".to_string()
        } else {
            self.provenance.clone()
        };
        self.provenance = format!(
            "{prev} | user override at {}: half_life_hours -> {hours} h",
            at.to_rfc3339()
        );
        self.half_life_hours = Some(hours);
        self.half_life_range = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unmodelled_carries_reason_and_none_class() {
        let m = SubstanceModel::unmodelled("9-Me-BC", "'unknown': no recognisable number+unit", 1);
        assert_eq!(m.model_class, ModelClass::None);
        assert!(!m.has_numeric_half_life());
        assert!(m.rationale.unwrap().contains("unknown"));
    }

    #[test]
    fn serde_roundtrip_is_stable_and_accepts_missing_optionals() {
        let m = SubstanceModel {
            substance: "Caffeine".into(),
            model_class: ModelClass::Decay,
            half_life_hours: Some(5.0),
            half_life_range: None,
            params: serde_json::json!({"variability_factor": 0.25}),
            provenance: "catalog seed string '5h'".into(),
            catalog_version: 1,
            rationale: None,
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: SubstanceModel = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);

        // Old/partial rows: params, optionals absent entirely.
        let sparse =
            r#"{"substance":"X","model_class":"none","provenance":"test","catalog_version":1}"#;
        let back: SubstanceModel = serde_json::from_str(sparse).unwrap();
        assert_eq!(back.model_class, ModelClass::None);
        assert!(back.params.is_null());
    }
}

#[cfg(test)]
mod provenance_tests {
    use super::*;

    fn m() -> SubstanceModel {
        SubstanceModel {
            substance: "Testinum".into(),
            model_class: ModelClass::Decay,
            half_life_hours: Some(5.0),
            half_life_range: Some((4.0, 6.0)),
            params: serde_json::Value::Null,
            provenance: "catalog seed string '5h' + T205 assignment table".into(),
            catalog_version: 3,
            rationale: None,
        }
    }

    #[test]
    fn user_override_chains_provenance_with_timestamp() {
        let mut model = m();
        let at = DateTime::parse_from_rfc3339("2026-11-15T03:00:00+00:00")
            .unwrap()
            .with_timezone(&Utc);
        model
            .with_user_half_life_override(7.5, at)
            .expect("valid override");
        // Both origins visible: catalog source first, human edit with time.
        assert!(model.provenance.contains("catalog seed string '5h'"));
        assert!(
            model
                .provenance
                .contains("user override at 2026-11-15T03:00:00+00:00")
        );
        assert_eq!(model.half_life_hours, Some(7.5));
        // The range is cleared — it must not silently outrank the user value.
        assert_eq!(model.half_life_range, None);
        // FR-036 negative form: nonsense overrides are refused, model untouched.
        let before = model.clone();
        assert!(model.with_user_half_life_override(0.0, at).is_err());
        assert!(model.with_user_half_life_override(f64::NAN, at).is_err());
        assert_eq!(model, before);
    }

    #[test]
    fn empty_provenance_refuses_simulation() {
        // FR-036 runtime gate at the single simulation chokepoint.
        let mut model = m();
        model.provenance = "   ".into();
        let err = crate::pharmacokinetics::sample_series(
            &model,
            &[],
            Utc::now(),
            Utc::now() + chrono::Duration::hours(1),
            1.0,
        )
        .expect_err("blank provenance must refuse to simulate");
        assert!(err.contains("provenance"), "{err}");
        assert!(err.contains("FR-036"), "{err}");
    }
}
