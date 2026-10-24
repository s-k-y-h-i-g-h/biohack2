//! Mechanism-class taxonomy for interaction detection.
//!
//! WHY THIS EXISTS
//! ---------------
//! The original safety engine caught interactions by NAME SUBSTRING only
//! (`is_stimulant_name`, `is_serotonergic_name` — 22 + 14 hardcoded names).
//! Name collisions cannot express: additive antiplatelet load, CYP inhibition
//! (inhibitor × substrate), additive CNS depression, additive thyroid
//! modulation. A 2026-10-02 sweep of all 496 catalog pairs by Kev-4B
//! (see `~/.pulse/kev-catalog-sweep-analysis.md`) proved a model is not the
//! answer: supplement-supplement interactions are weak signals and the model
//! approved 494/496 pairs, most confidently when they genuinely interact.
//! The answer is a deterministic CLASS taxonomy: tag each substance with
//! mechanism classes, then check CLASS-PAIR rules. Deterministic, unit-testable,
//! offline, auditable, and it scales to the 800+ Examine.com items.
//!
//! DESIGN
//! ------
//! - `MechanismClass` — the taxonomy. A substance carries zero or more classes.
//! - `PAIR_RULES` — symmetric (class, class) → severity + message + recommendation.
//! - `tag_for_name` — single source of truth for classification: canonical seed
//!   table first, keyword fallback (reusing the existing substring lists) so
//!   custom user entries are still classified.
//! - `pair_rules` — engine-facing: given two tag sets, return the rule hits.

use serde::{Deserialize, Serialize};

use crate::models::AlertSeverity;

// ── Taxonomy ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MechanismClass {
    /// Inhibits platelet aggregation / raises bleeding risk.
    Antiplatelet,
    /// Inhibits CYP3A4 — raises exposure of 3A4 substrates.
    Cyp3a4Inhibitor,
    /// Metabolized by CYP3A4 — exposure raised by 3A4 inhibitors.
    Cyp3a4Substrate,
    /// Inhibits CYP2C19.
    Cyp2c19Inhibitor,
    /// Metabolized by CYP2C19.
    Cyp2c19Substrate,
    /// Inhibits CYP1A2.
    Cyp1a2Inhibitor,
    /// Metabolized by CYP1A2.
    Cyp1a2Substrate,
    /// Serotonergic activity — serotonin-syndrome risk in combination.
    Serotonergic,
    /// CNS stimulant — additive HR/BP load.
    Stimulant,
    /// Raises blood pressure.
    Hypertensive,
    /// Lowers blood pressure.
    Hypotensive,
    /// CNS depressant / sedating — additive sedation.
    CnsDepressant,
    /// Modulates thyroid axis (T3/T4).
    ThyroidModulating,
    /// Modulates immune function.
    Immunomodulating,
    /// Vitamin-K–driven procoagulant activity (clotting-factor synthesis).
    Procoagulant,
    /// Modulates glutamate/NMDA signaling.
    GlutamatergicModulating,
    /// Lowers blood glucose.
    Hypoglycemic,
    /// Chelates/binds co-ingested substances in the gut — timing separation needed.
    AbsorptionChelator,
    /// Monoamine-oxidase inhibition — raises synaptic monoamines (serotonin,
    /// dopamine, tyramine); the classic serotonin-syndrome / hypertensive-crisis axis.
    MaoInhibitor,
}

impl MechanismClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Antiplatelet => "antiplatelet",
            Self::Cyp3a4Inhibitor => "cyp3a4_inhibitor",
            Self::Cyp3a4Substrate => "cyp3a4_substrate",
            Self::Cyp2c19Inhibitor => "cyp2c19_inhibitor",
            Self::Cyp2c19Substrate => "cyp2c19_substrate",
            Self::Cyp1a2Inhibitor => "cyp1a2_inhibitor",
            Self::Cyp1a2Substrate => "cyp1a2_substrate",
            Self::Serotonergic => "serotonergic",
            Self::Stimulant => "stimulant",
            Self::Hypertensive => "hypertensive",
            Self::Hypotensive => "hypotensive",
            Self::CnsDepressant => "cns_depressant",
            Self::ThyroidModulating => "thyroid_modulating",
            Self::Immunomodulating => "immunomodulating",
            Self::Procoagulant => "procoagulant",
            Self::GlutamatergicModulating => "glutamatergic_modulating",
            Self::Hypoglycemic => "hypoglycemic",
            Self::AbsorptionChelator => "absorption_chelator",
            Self::MaoInhibitor => "mao_inhibitor",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "antiplatelet" => Some(Self::Antiplatelet),
            "cyp3a4_inhibitor" => Some(Self::Cyp3a4Inhibitor),
            "cyp3a4_substrate" => Some(Self::Cyp3a4Substrate),
            "cyp2c19_inhibitor" => Some(Self::Cyp2c19Inhibitor),
            "cyp2c19_substrate" => Some(Self::Cyp2c19Substrate),
            "cyp1a2_inhibitor" => Some(Self::Cyp1a2Inhibitor),
            "cyp1a2_substrate" => Some(Self::Cyp1a2Substrate),
            "serotonergic" => Some(Self::Serotonergic),
            "stimulant" => Some(Self::Stimulant),
            "hypertensive" => Some(Self::Hypertensive),
            "hypotensive" => Some(Self::Hypotensive),
            "cns_depressant" => Some(Self::CnsDepressant),
            "thyroid_modulating" => Some(Self::ThyroidModulating),
            "immunomodulating" => Some(Self::Immunomodulating),
            "procoagulant" => Some(Self::Procoagulant),
            "glutamatergic_modulating" => Some(Self::GlutamatergicModulating),
            "hypoglycemic" => Some(Self::Hypoglycemic),
            "absorption_chelator" => Some(Self::AbsorptionChelator),
            "mao_inhibitor" => Some(Self::MaoInhibitor),
            _ => None,
        }
    }
}

// ── Pair rules ────────────────────────────────────────────────────────────────

/// A rule hit: one class-pair matched between two substances.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleHit {
    /// Stable rule id for audit/tests, e.g. `"additive_antiplatelet"`.
    pub code: &'static str,
    pub severity: AlertSeverity,
    pub message: &'static str,
    pub recommendation: &'static str,
}

/// Symmetric class-pair rules. Order within the tuple is irrelevant —
/// `pair_rules` checks both directions.
const PAIR_RULES: &[(
    MechanismClass,
    MechanismClass,
    &str,
    AlertSeverity,
    &str,
    &str,
)] = &[
    (
        MechanismClass::Stimulant,
        MechanismClass::Stimulant,
        "stimulant_stimulant",
        AlertSeverity::Warning,
        "Additive stimulant load: two stimulants in the same window compound heart-rate and blood-pressure effects",
        "Count total stimulant load, not per-substance. Reduce dose of each, avoid stacking, monitor HR/BP.",
    ),
    (
        MechanismClass::Stimulant,
        MechanismClass::Hypertensive,
        "stimulant_hypertensive",
        AlertSeverity::Warning,
        "Stimulant + blood-pressure-raising agent: additive hypertensive effect",
        "Monitor BP; avoid stacking; recheck before adding further stimulants.",
    ),
    (
        MechanismClass::Antiplatelet,
        MechanismClass::Antiplatelet,
        "additive_antiplatelet",
        AlertSeverity::Warning,
        "Additive antiplatelet load: two agents that inhibit platelet aggregation compound bleeding risk",
        "Watch for bruising/bleeding; reduce combined dose; disclose to any dentist/surgeon before procedures.",
    ),
    (
        MechanismClass::Cyp3a4Inhibitor,
        MechanismClass::Cyp3a4Substrate,
        "cyp3a4_inhibition",
        AlertSeverity::Warning,
        "CYP3A4 inhibition: the inhibitor raises blood levels of the 3A4 substrate",
        "Expect higher-than-usual exposure to the substrate; reduce substrate dose or separate doses and monitor for side effects.",
    ),
    (
        MechanismClass::Cyp2c19Inhibitor,
        MechanismClass::Cyp2c19Substrate,
        "cyp2c19_inhibition",
        AlertSeverity::Warning,
        "CYP2C19 inhibition: the inhibitor raises blood levels of the 2C19 substrate",
        "Expect higher exposure to the substrate; reduce dose and monitor for side effects.",
    ),
    (
        MechanismClass::Cyp1a2Inhibitor,
        MechanismClass::Cyp1a2Substrate,
        "cyp1a2_inhibition",
        AlertSeverity::Warning,
        "CYP1A2 inhibition: the inhibitor raises blood levels of the 1A2 substrate (caffeine is the classic case)",
        "Expect markedly higher substrate exposure (caffeine AUC can rise several-fold); cut substrate dose.",
    ),
    (
        MechanismClass::Serotonergic,
        MechanismClass::Serotonergic,
        "serotonergic_pair",
        AlertSeverity::Critical,
        "Two serotonergic agents in the same window: serotonin-syndrome risk",
        "Monitor for clonus, hyperreflexia, hyperthermia, diaphoresis, agitation. Seek emergency care if symptoms develop.",
    ),
    (
        MechanismClass::CnsDepressant,
        MechanismClass::CnsDepressant,
        "additive_cns_depression",
        AlertSeverity::Warning,
        "Additive CNS depression: two sedating agents compound drowsiness and impairment",
        "Avoid driving/operating machinery; reduce combined dose; separate doses by several hours.",
    ),
    (
        MechanismClass::ThyroidModulating,
        MechanismClass::ThyroidModulating,
        "additive_thyroid_modulation",
        AlertSeverity::Warning,
        "Two thyroid-modulating agents in the same window: additive effect on T3/T4",
        "If on thyroid medication, separate by 4+ hours and monitor for symptom shifts; recheck labs if stacking regularly.",
    ),
    (
        MechanismClass::Hypoglycemic,
        MechanismClass::Hypoglycemic,
        "additive_hypoglycemia",
        AlertSeverity::Warning,
        "Two glucose-lowering agents in the same window: additive hypoglycemia risk",
        "Monitor glucose; take with food; reduce combined dose.",
    ),
    (
        MechanismClass::Hypotensive,
        MechanismClass::Hypotensive,
        "additive_hypotension",
        AlertSeverity::Info,
        "Two blood-pressure-lowering agents: additive hypotensive effect",
        "Rise slowly from sitting; monitor BP if stacking regularly.",
    ),
    (
        MechanismClass::AbsorptionChelator,
        MechanismClass::AbsorptionChelator,
        "chelation_timing",
        AlertSeverity::Info,
        "Two chelating minerals compete for absorption in the gut",
        "Separate doses by 2+ hours, or take at meals on opposite sides of the day.",
    ),
    (
        MechanismClass::Antiplatelet,
        MechanismClass::Procoagulant,
        "antiplatelet_procoagulant",
        AlertSeverity::Warning,
        "Antiplatelet + vitamin-K procoagulant: opposing clotting pathways (bleeding-risk agents vs clotting-factor synthesis)",
        "Effects can antagonize each other and shift unpredictably; if on warfarin/DOAC, disclose both and monitor INR/bleeding signs; disclose before procedures.",
    ),
    (
        MechanismClass::Immunomodulating,
        MechanismClass::Immunomodulating,
        "additive_immunomodulation",
        AlertSeverity::Info,
        "Two immunomodulating agents in the same window: combined immune effect is not simply additive",
        "Usually benign; if immunosuppressed, autoimmune, or pre-transplant, review the stack with a clinician.",
    ),
    (
        MechanismClass::GlutamatergicModulating,
        MechanismClass::GlutamatergicModulating,
        "additive_glutamatergic_modulation",
        AlertSeverity::Info,
        "Two glutamatergic/NMDA-modulating agents: combined effect on glutamate signaling",
        "Expect shifted NMDA tone (e.g. NAC + memantine); start low, note cognitive/emotional side effects, separate doses if jittery.",
    ),
    (
        MechanismClass::MaoInhibitor,
        MechanismClass::Serotonergic,
        "mao_serotonergic",
        AlertSeverity::Critical,
        "MAO inhibition + serotonergic agent: classic serotonin-syndrome pairing — MAO-A blockade raises synaptic serotonin, the co-agent adds to it",
        "Do not combine. Seek emergency care for clonus, hyperreflexia, hyperthermia, diaphoresis, agitation. Requires a washout period between the two (MAO inhibition outlasts the parent compound).",
    ),
    (
        MechanismClass::MaoInhibitor,
        MechanismClass::MaoInhibitor,
        "stacked_mao_inhibition",
        AlertSeverity::Critical,
        "Two MAO-inhibiting agents in the same window: additive monoamine blockade, supratherapeutic by definition",
        "Do not stack. Pick one MAO-active agent per window; monitor BP and for serotonin-syndrome symptoms.",
    ),
    (
        MechanismClass::MaoInhibitor,
        MechanismClass::Stimulant,
        "mao_stimulant",
        AlertSeverity::Warning,
        "MAO inhibition + stimulant: monoamine blockade compounds the pressor load (tyramine and catecholamines are not cleared)",
        "Expect a stronger-than-usual BP/HR response; start at a fraction of the usual stimulant dose, avoid tyramine-rich food, monitor BP.",
    ),
    (
        MechanismClass::MaoInhibitor,
        MechanismClass::Hypertensive,
        "mao_hypertensive",
        AlertSeverity::Warning,
        "MAO inhibition + blood-pressure-raising agent: additive pressor effect with reduced tyramine/catecholamine clearance",
        "Monitor BP; avoid aged/fermented (tyramine-rich) foods; reduce the pressor dose.",
    ),
];

/// Given two tag sets, return every rule that matches (both directions checked).
/// Hits are deduplicated by rule code — a pair of substances gets one alert
/// per rule, not one per class-combination.
pub fn pair_rules(a: &[MechanismClass], b: &[MechanismClass]) -> Vec<RuleHit> {
    let mut hits: Vec<RuleHit> = Vec::new();
    for (x, y, code, severity, message, recommendation) in PAIR_RULES {
        let matched = (a.contains(x) && b.contains(y)) || (a.contains(y) && b.contains(x));
        if matched && !hits.iter().any(|h| h.code == *code) {
            hits.push(RuleHit {
                code,
                severity: severity.clone(),
                message,
                recommendation,
            });
        }
    }
    hits
}

// ── Classification ────────────────────────────────────────────────────────────

/// Canonical tags for the 33-item seed catalog, keyed by normalized name token.
/// Each entry: (name-key, tags). Keys are matched with `contains` on the
/// normalized name, so "Omega-3 Fish Oil" matches "omega-3".
const SEED_TAGS: &[(&str, &[MechanismClass])] = &[
    ("apigenin", &[MechanismClass::Antiplatelet, MechanismClass::Cyp1a2Inhibitor, MechanismClass::Cyp3a4Substrate, MechanismClass::Cyp2c19Substrate]),
    ("ashwagandha", &[MechanismClass::ThyroidModulating, MechanismClass::Immunomodulating, MechanismClass::CnsDepressant]),
    ("berberine", &[MechanismClass::Cyp3a4Inhibitor, MechanismClass::Hypoglycemic]),
    ("cbd", &[MechanismClass::Cyp3a4Inhibitor, MechanismClass::Cyp2c19Inhibitor, MechanismClass::CnsDepressant]),
    ("caffeine", &[MechanismClass::Stimulant, MechanismClass::Hypertensive, MechanismClass::Cyp1a2Substrate]),
    ("coq10", &[MechanismClass::Hypotensive]),
    ("curcumin", &[MechanismClass::Antiplatelet, MechanismClass::Cyp3a4Inhibitor]),
    ("cyclazodone", &[MechanismClass::Stimulant, MechanismClass::Hypertensive]),
    ("kratom", &[MechanismClass::Serotonergic, MechanismClass::Stimulant, MechanismClass::CnsDepressant, MechanismClass::Cyp3a4Substrate, MechanismClass::Cyp2c19Substrate]),
    ("l-theanine", &[MechanismClass::CnsDepressant]),
    ("l-tyrosine", &[MechanismClass::ThyroidModulating]),
    ("magnesium", &[MechanismClass::AbsorptionChelator, MechanismClass::Hypotensive]),
    ("melatonin", &[MechanismClass::CnsDepressant]),
    ("memantine", &[MechanismClass::CnsDepressant, MechanismClass::GlutamatergicModulating]),
    ("n-methyl-cyclazodone", &[MechanismClass::Stimulant, MechanismClass::Hypertensive]),
    ("nac", &[MechanismClass::GlutamatergicModulating]),
    ("omega-3", &[MechanismClass::Antiplatelet, MechanismClass::Hypotensive]),
    ("piracetam", &[MechanismClass::GlutamatergicModulating]),
    ("quercetin", &[MechanismClass::Antiplatelet, MechanismClass::Cyp3a4Inhibitor, MechanismClass::Immunomodulating]),
    ("reishi", &[MechanismClass::Antiplatelet, MechanismClass::Immunomodulating]),
    ("resveratrol", &[MechanismClass::Antiplatelet, MechanismClass::Immunomodulating]),
    ("rhodiola", &[MechanismClass::Serotonergic, MechanismClass::Stimulant]),
    ("cordyceps", &[MechanismClass::Immunomodulating]),
    ("umcka", &[MechanismClass::Immunomodulating]),
    ("9-me-bc", &[MechanismClass::MaoInhibitor, MechanismClass::Serotonergic, MechanismClass::Stimulant, MechanismClass::Hypertensive]),
    ("vitamin d3", &[MechanismClass::Immunomodulating]),
    ("vitamin k2", &[MechanismClass::Procoagulant]),
    ("zinc", &[MechanismClass::AbsorptionChelator]),
];

/// Normalize a substance name for matching: lowercase, punctuation → space,
/// collapse whitespace. Same normalization as `safety::is_serotonergic_name`.
fn normalize(name: &str) -> String {
    let cleaned: String = name
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '-' {
                c
            } else {
                ' '
            }
        })
        .collect();
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// True if the normalized name contains a standalone MAO token: `mao`, `maoi`,
/// `mao-a`, `mao-b`, `mao-inhibitor`. Token-boundary matching is deliberate: a
/// plain substring match fires on **"Maori Green Mussel"** (`mao`-ri), a false
/// positive that raises a Critical serotonin-syndrome alert on a supplement.
pub fn has_mao_token(name: &str) -> bool {
    normalize(name).split_whitespace().any(|tok| {
        matches!(tok, "mao" | "maoi" | "mao-a" | "mao-b" | "maoab" | "maoc")
            || tok.starts_with("mao-")
    })
}

/// Single source of truth for substance classification.
///
/// 1. Canonical seed table (the 33 catalog items) — exact mechanism tags.
/// 2. Keyword fallback via the existing substring lists, so CUSTOM user entries
///    (e.g. "Adderall", "Fluoxetine") are still classified.
///
/// Returns a deduplicated tag list. Unknown names return an empty vec —
/// an unclassified substance simply participates in no class-pair rules.
pub fn tag_for_name(name: &str) -> Vec<MechanismClass> {
    let n = normalize(name);
    let mut tags: Vec<MechanismClass> = Vec::new();
    for (key, classes) in SEED_TAGS {
        if n.contains(key) {
            for c in *classes {
                if !tags.contains(c) {
                    tags.push(*c);
                }
            }
        }
    }
    // Keyword fallback for custom entries (reuses the engine's existing lists).
    if crate::safety::is_serotonergic_name(name) && !tags.contains(&MechanismClass::Serotonergic) {
        tags.push(MechanismClass::Serotonergic);
    }
    // Explicit MAO inhibitors named by a custom entry ("Phenelzine (MAOI)") join
    // the MAO class, so the mao_* pair rules reach them too. Token-boundary
    // match — see has_mao_token for the false-positive this avoids.
    if has_mao_token(name) && !tags.contains(&MechanismClass::MaoInhibitor) {
        tags.push(MechanismClass::MaoInhibitor);
    }
    if crate::safety::is_stimulant_name(name) {
        if !tags.contains(&MechanismClass::Stimulant) {
            tags.push(MechanismClass::Stimulant);
        }
        if !tags.contains(&MechanismClass::Hypertensive) {
            tags.push(MechanismClass::Hypertensive);
        }
    }
    tags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_tags_roundtrip_through_strings() {
        for (key, classes) in SEED_TAGS {
            for c in *classes {
                assert_eq!(MechanismClass::from_str(c.as_str()), Some(*c), "{key}");
            }
        }
    }

    #[test]
    fn caffeine_is_classified_with_cyp1a2_substrate() {
        let t = tag_for_name("Caffeine");
        assert!(t.contains(&MechanismClass::Stimulant));
        assert!(t.contains(&MechanismClass::Hypertensive));
        assert!(t.contains(&MechanismClass::Cyp1a2Substrate));
    }

    #[test]
    fn omega3_matches_by_token() {
        let t = tag_for_name("Omega-3 Fish Oil");
        assert!(t.contains(&MechanismClass::Antiplatelet));
        assert!(t.contains(&MechanismClass::Hypotensive));
    }

    #[test]
    fn unknown_name_gets_no_tags() {
        assert!(tag_for_name("Agaric Acid K2").is_empty());
    }

    #[test]
    fn vitamin_k2_is_procoagulant_and_flags_with_antiplatelets() {
        let t = tag_for_name("Vitamin K2");
        assert!(t.contains(&MechanismClass::Procoagulant));
        let a = tag_for_name("Curcumin");
        let hits = pair_rules(&a, &t);
        assert!(hits.iter().any(|h| h.code == "antiplatelet_procoagulant"));
    }

    #[test]
    fn nac_memantine_get_glutamatergic_rule() {
        let a = tag_for_name("NAC (N-Acetyl Cysteine)");
        let b = tag_for_name("Memantine");
        assert!(a.contains(&MechanismClass::GlutamatergicModulating));
        let hits = pair_rules(&a, &b);
        assert!(hits.iter().any(|h| h.code == "additive_glutamatergic_modulation"));
    }

    #[test]
    fn ashwagandha_tyrosine_get_thyroid_rule() {
        let a = tag_for_name("Ashwagandha");
        let b = tag_for_name("L-Tyrosine");
        let hits = pair_rules(&a, &b);
        assert!(hits.iter().any(|h| h.code == "additive_thyroid_modulation"));
    }

    #[test]
    fn cbd_apigenin_gets_cyp2c19_rule() {
        // CBD inhibits 2C19; apigenin is now tagged as a 2C19/3A4 substrate.
        let a = tag_for_name("CBD");
        let b = tag_for_name("Apigenin");
        let hits = pair_rules(&a, &b);
        assert!(hits.iter().any(|h| h.code == "cyp2c19_inhibition"));
        assert!(hits.iter().any(|h| h.code == "cyp3a4_inhibition"));
    }

    #[test]
    fn nine_me_bc_is_mao_inhibitor_and_flags_serotonergic_pair() {
        let a = tag_for_name("9-Me-BC");
        assert!(a.contains(&MechanismClass::MaoInhibitor));
        let b = tag_for_name("Rhodiola Rosea");
        let hits = pair_rules(&a, &b);
        let hit = hits
            .iter()
            .find(|h| h.code == "mao_serotonergic")
            .expect("9-Me-BC + serotonergic must fire mao_serotonergic");
        assert_eq!(hit.severity, AlertSeverity::Critical);
    }

    #[test]
    fn nine_me_bc_caffeine_gets_mao_stimulant_and_pressor_rules() {
        let a = tag_for_name("9-Me-BC");
        let b = tag_for_name("Caffeine");
        let hits = pair_rules(&a, &b);
        assert!(hits.iter().any(|h| h.code == "mao_stimulant"));
        assert!(hits.iter().any(|h| h.code == "mao_hypertensive"));
    }

    #[test]
    fn custom_maoi_name_joins_mao_class() {
        let t = tag_for_name("Phenelzine (MAOI)");
        assert!(t.contains(&MechanismClass::MaoInhibitor));
        let hits = pair_rules(&t, &tag_for_name("5-HTP"));
        assert!(hits.iter().any(|h| h.code == "mao_serotonergic"));
    }

    #[test]
    fn benign_control_nine_me_bc_zinc_gets_no_mao_alert() {
        let a = tag_for_name("9-Me-BC");
        let b = tag_for_name("Zinc");
        let hits = pair_rules(&a, &b);
        assert!(
            !hits.iter().any(|h| h.code.starts_with("mao_")),
            "benign pair must stay silent: {hits:?}"
        );
    }

    #[test]
    fn mao_token_match_rejects_maori_mussel() {
        // Regression: a plain substring match fired on "Maori" (mao-ri), which
        // raised a Critical serotonin-syndrome alert on a green-lipped mussel.
        assert!(!has_mao_token("Maori Green Mussel"));
        assert!(!tag_for_name("Maori Green Mussel").contains(&MechanismClass::MaoInhibitor));
        let hits = pair_rules(
            &tag_for_name("Maori Green Mussel"),
            &tag_for_name("Rhodiola Rosea"),
        );
        assert!(
            !hits.iter().any(|h| h.code == "mao_serotonergic"),
            "mussel must not inherit MAO class: {hits:?}"
        );
        // The real thing still fires.
        assert!(has_mao_token("Phenelzine (MAOI)"));
        assert!(has_mao_token("MAO-A inhibitor"));
        assert!(has_mao_token("maoi"));
    }

    #[test]
    fn vitamin_d3_reishi_get_immunomodulation_rule() {
        let a = tag_for_name("Vitamin D3");
        let b = tag_for_name("Reishi Mushroom");
        let hits = pair_rules(&a, &b);
        assert!(hits.iter().any(|h| h.code == "additive_immunomodulation"));
    }

    #[test]
    fn benign_control_zinc_probiotic_gets_no_alert() {
        let a = tag_for_name("Zinc");
        let b = tag_for_name("Probiotic");
        assert!(pair_rules(&a, &b).is_empty());
    }

    #[test]
    fn custom_entry_falls_back_to_substring_lists() {
        // Not in the seed table; caught by is_serotonergic_name.
        let t = tag_for_name("Fluoxetine");
        assert!(t.contains(&MechanismClass::Serotonergic));
        let t = tag_for_name("Adderall");
        assert!(t.contains(&MechanismClass::Stimulant));
    }

    #[test]
    fn antiplatelet_pair_hits() {
        let a = tag_for_name("Omega-3 Fish Oil");
        let b = tag_for_name("Curcumin");
        let hits = pair_rules(&a, &b);
        assert!(hits.iter().any(|h| h.code == "additive_antiplatelet"));
    }

    #[test]
    fn stimulant_pair_hits() {
        let a = tag_for_name("Caffeine");
        let b = tag_for_name("Cyclazodone");
        let hits = pair_rules(&a, &b);
        assert!(hits.iter().any(|h| h.code == "stimulant_stimulant"));
    }

    #[test]
    fn cyp1a2_inhibition_caught() {
        let a = tag_for_name("Apigenin");
        let b = tag_for_name("Caffeine");
        let hits = pair_rules(&a, &b);
        assert!(hits.iter().any(|h| h.code == "cyp1a2_inhibition"));
    }

    #[test]
    fn serotonergic_pair_is_critical() {
        let a = tag_for_name("Rhodiola Rosea");
        let b = tag_for_name("Kratom");
        let hits = pair_rules(&a, &b);
        let hit = hits.iter().find(|h| h.code == "serotonergic_pair").unwrap();
        assert_eq!(hit.severity, AlertSeverity::Critical);
    }

    #[test]
    fn benign_pair_hits_nothing() {
        let a = tag_for_name("Vitamin D3");
        let b = tag_for_name("Boron");
        assert!(pair_rules(&a, &b).is_empty());
    }

    #[test]
    fn hits_dedup_by_code() {
        // CBD inhibits 3A4+2C19+CNS; Kratom is substrate of both + CNS depressant.
        // (CBD is NOT serotonergic — serotonergic_pair must NOT fire here.)
        let a = tag_for_name("CBD");
        let b = tag_for_name("Kratom");
        let hits = pair_rules(&a, &b);
        assert!(hits.iter().any(|h| h.code == "cyp3a4_inhibition"));
        assert!(hits.iter().any(|h| h.code == "cyp2c19_inhibition"));
        assert!(hits.iter().any(|h| h.code == "additive_cns_depression"));
        assert!(!hits.iter().any(|h| h.code == "serotonergic_pair"));
        let mut codes: Vec<&str> = hits.iter().map(|h| h.code).collect();
        codes.sort_unstable();
        let n = codes.len();
        codes.dedup();
        assert_eq!(codes.len(), n, "duplicate rule codes");
    }
}
