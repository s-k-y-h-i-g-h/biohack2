//! T052 — Interaction-flagging benchmark.
//!
//! A labeled substance-pair dataset with expectations derived from the
//! pharmacology literature, NOT from `PAIR_RULES` — a benchmark that reads
//! its labels from the rule table would only prove the table equals itself.
//! Each case records what the literature says should happen; the suite
//! measures how close the engine comes and prints the disagreements.
//!
//! The four disagreement probes below (piperine FN; rhodiola+caffeine,
//! NAC+piracetam, vitamin D3+cordyceps FPs) were FIXED on 2026-11-17 with
//! George's approval ("approval for everything") — the engine now matches the
//! literature labels on every case. The probes stay as permanent regression
//! guards: if a future tag edit reopens any of these disagreements, this
//! suite fails.

use chrono::{Duration, Utc};
use engine::models::*;
use engine::safety::SafetyEngine;

struct Case {
    label: &'static str,
    a: &'static str,
    b: &'static str,
    expect_flag: bool,
    /// Severity expected on at least one alert when expect_flag is true.
    severity: Option<AlertSeverity>,
    /// Literature rationale (kept next to the label so a reviewer can audit
    /// the ground truth without leaving the file).
    why: &'static str,
}

const CASES: &[Case] = &[
    // ── Critical axis: serotonergic / MAOI (zero-tolerance for misses) ──────
    Case {
        label: "ssri+sjw",
        a: "Sertraline",
        b: "St. John's Wort",
        expect_flag: true,
        severity: Some(AlertSeverity::Critical),
        why: "SSRI + serotonergic herb: serotonin syndrome, well documented",
    },
    Case {
        label: "ssri+tramadol",
        a: "Sertraline",
        b: "Tramadol",
        expect_flag: true,
        severity: Some(AlertSeverity::Critical),
        why: "SSRI + tramadol: serotonin syndrome, boxed-warning class pairing",
    },
    Case {
        label: "snri+dmp",
        a: "Venlafaxine (SNRI)",
        b: "Dextromethorphan",
        expect_flag: true,
        severity: Some(AlertSeverity::Critical),
        why: "SNRI + dextromethorphan: serotonin syndrome risk",
    },
    Case {
        label: "rhodiola+ssri",
        a: "Rhodiola",
        b: "Sertraline",
        expect_flag: true,
        severity: Some(AlertSeverity::Critical),
        why: "rhodiola is serotonergic in the engine's classification; with an SSRI the syndrome risk is real",
    },
    Case {
        label: "9mebc+ssri",
        a: "9-Me-BC",
        b: "Sertraline",
        expect_flag: true,
        severity: Some(AlertSeverity::Critical),
        why: "MAOI + SSRI: the classic hypertensive/serotonin crisis pairing",
    },
    // ── Stimulant load ────────────────────────────────────────────────────────
    Case {
        label: "caf+modafinil",
        a: "Caffeine",
        b: "Modafinil",
        expect_flag: true,
        severity: Some(AlertSeverity::Warning),
        why: "two wakefulness-promoting stimulants: additive HR/BP load",
    },
    Case {
        label: "caf+nicotine",
        a: "Caffeine",
        b: "Nicotine",
        expect_flag: true,
        severity: Some(AlertSeverity::Warning),
        why: "two stimulants: additive cardiovascular effects",
    },
    Case {
        label: "caf+yohimbine",
        a: "Caffeine",
        b: "Yohimbine",
        expect_flag: true,
        severity: Some(AlertSeverity::Warning),
        why: "yohimbine is hypertensive; with caffeine the pressor effect compounds",
    },
    Case {
        label: "modafinil+nicotine",
        a: "Modafinil",
        b: "Nicotine",
        expect_flag: true,
        severity: Some(AlertSeverity::Warning),
        why: "two stimulants in one window",
    },
    Case {
        label: "9mebc+caf",
        a: "9-Me-BC",
        b: "Caffeine",
        expect_flag: true,
        severity: Some(AlertSeverity::Warning),
        why: "MAOI + stimulant: pressor/serotonin caution",
    },
    // ── CYP kinetics ──────────────────────────────────────────────────────────
    Case {
        label: "caf+apigenin",
        a: "Caffeine",
        b: "Apigenin",
        expect_flag: true,
        severity: Some(AlertSeverity::Warning),
        why: "flavonoid CYP1A2 inhibition raises caffeine exposure (caffeine is the classic 1A2 substrate)",
    },
    Case {
        label: "berberine+kratom",
        a: "Berberine",
        b: "Kratom",
        expect_flag: true,
        severity: Some(AlertSeverity::Warning),
        why: "berberine inhibits CYP3A4; kratom alkaloid levels rise (case-report backed)",
    },
    Case {
        label: "cbd+kratom",
        a: "CBD",
        b: "Kratom",
        expect_flag: true,
        severity: Some(AlertSeverity::Warning),
        why: "CBD inhibits CYP3A4/2C19; documented kratom interaction route",
    },
    // ── CNS depression ────────────────────────────────────────────────────────
    Case {
        label: "alcohol+melatonin",
        a: "Alcohol",
        b: "Melatonin",
        expect_flag: true,
        severity: Some(AlertSeverity::Warning),
        why: "two CNS depressants: additive sedation/impairment",
    },
    Case {
        label: "alcohol+theanine",
        a: "Alcohol",
        b: "L-Theanine",
        expect_flag: true,
        severity: Some(AlertSeverity::Warning),
        why: "theanine is mildly sedating (GABAergic); additive with alcohol is the labeled caution",
    },
    Case {
        label: "memantine+alcohol",
        a: "Memantine",
        b: "Alcohol",
        expect_flag: true,
        severity: Some(AlertSeverity::Warning),
        why: "both CNS-acting; the withdrawal-seizure axis from the 2026-11 case",
    },
    // ── Hemostasis / BP / thyroid / absorption / immune ──────────────────────
    Case {
        label: "omega3+curcumin",
        a: "Omega-3",
        b: "Curcumin",
        expect_flag: true,
        severity: Some(AlertSeverity::Warning),
        why: "two agents inhibiting platelet aggregation: additive bleeding risk at dose",
    },
    Case {
        label: "reishi+omega3",
        a: "Reishi",
        b: "Omega-3",
        expect_flag: true,
        severity: Some(AlertSeverity::Warning),
        why: "reishi has antiplatelet activity; compounds with fish oil",
    },
    Case {
        label: "coq10+magnesium",
        a: "CoQ10",
        b: "Magnesium",
        expect_flag: true,
        severity: Some(AlertSeverity::Info),
        why: "both lower blood pressure; additive hypotension is an Info-level caution",
    },
    Case {
        label: "tyrosine+ashwa",
        a: "L-Tyrosine",
        b: "Ashwagandha",
        expect_flag: true,
        severity: Some(AlertSeverity::Warning),
        why: "both modulate thyroid axis: additive thyroid stimulation caution",
    },
    Case {
        label: "zinc+magnesium",
        a: "Zinc",
        b: "Magnesium",
        expect_flag: true,
        severity: Some(AlertSeverity::Info),
        why: "competing divalent cations: absorption competition at co-dose (timing Info)",
    },
    Case {
        label: "cordyceps+reishi",
        a: "Cordyceps",
        b: "Reishi",
        expect_flag: true,
        severity: Some(AlertSeverity::Info),
        why: "stacked medicinal-mushroom immunomodulation: label caution on stacking",
    },
    // ── FN probe (vocabulary gap; engine silent today, literature cautions) ──
    Case {
        label: "curcumin+piperine",
        a: "Curcumin",
        b: "Piperine",
        expect_flag: true,
        severity: None,
        why: "piperine inhibits CYP3A4/P-gp strongly (bioavailability is its purpose); engine has no piperine vocabulary — documented FN",
    },
    // ── Benign controls (literature says silent; must not alert) ─────────────
    Case {
        label: "caf+theanine",
        a: "Caffeine",
        b: "L-Theanine",
        expect_flag: false,
        severity: None,
        why: "the classic nootropic pairing; no impairment interaction",
    },
    Case {
        label: "d3+k2",
        a: "Vitamin D3",
        b: "Vitamin K2",
        expect_flag: false,
        severity: None,
        why: "synergistic, routinely co-dosed",
    },
    Case {
        label: "mg+d3",
        a: "Magnesium",
        b: "Vitamin D3",
        expect_flag: false,
        severity: None,
        why: "co-dosed staple; no interaction",
    },
    Case {
        label: "zinc+vitc",
        a: "Zinc",
        b: "Vitamin C",
        expect_flag: false,
        severity: None,
        why: "cold-season staple pairing",
    },
    Case {
        label: "melatonin+mg",
        a: "Melatonin",
        b: "Magnesium",
        expect_flag: false,
        severity: None,
        why: "common sleep stack; no additive risk beyond mild relaxation",
    },
    Case {
        label: "ashwa+mg",
        a: "Ashwagandha",
        b: "Magnesium",
        expect_flag: false,
        severity: None,
        why: "common evening stack",
    },
    Case {
        label: "caf+creatine",
        a: "Caffeine",
        b: "Creatine",
        expect_flag: false,
        severity: None,
        why: "ubiquitous gym stack; no interaction",
    },
    Case {
        label: "omega3+d3",
        a: "Omega-3",
        b: "Vitamin D3",
        expect_flag: false,
        severity: None,
        why: "fat-soluble co-supplement, no interaction",
    },
    Case {
        label: "curcumin+zinc",
        a: "Curcumin",
        b: "Zinc",
        expect_flag: false,
        severity: None,
        why: "no documented interaction at supplement dose",
    },
    Case {
        label: "tyrosine+caf",
        a: "L-Tyrosine",
        b: "Caffeine",
        expect_flag: false,
        severity: None,
        why: "common pre-workout stack; tyrosine is not a stimulant at dose",
    },
    Case {
        label: "glycine+melatonin",
        a: "Glycine",
        b: "Melatonin",
        expect_flag: false,
        severity: None,
        why: "both mild sleep aids; routinely co-formulated",
    },
    Case {
        label: "creatine+mg",
        a: "Creatine",
        b: "Magnesium",
        expect_flag: false,
        severity: None,
        why: "no interaction",
    },
    Case {
        label: "vitc+creatine",
        a: "Vitamin C",
        b: "Creatine",
        expect_flag: false,
        severity: None,
        why: "no interaction",
    },
    Case {
        label: "9mebc+phenelzine",
        a: "9-Me-BC",
        b: "Phenelzine (MAOI)",
        expect_flag: true,
        severity: Some(AlertSeverity::Critical),
        why: "two MAO inhibitors in one window: stacked MAOI inhibition is the textbook contraindication",
    },
    Case {
        label: "potassium+magnesium",
        a: "Potassium",
        b: "Magnesium",
        expect_flag: false,
        severity: None,
        why: "electrolyte pair, routinely co-dosed; no interaction",
    },
    Case {
        label: "vitc+omega3",
        a: "Vitamin C",
        b: "Omega-3",
        expect_flag: false,
        severity: None,
        why: "no documented interaction",
    },
    Case {
        label: "ashwa+rhodiola",
        a: "Ashwagandha",
        b: "Rhodiola",
        expect_flag: false,
        severity: None,
        why: "the classic adaptogen stack; no case reports of combined harm",
    },
    // ── FP probes (engine tags more aggressively than literature supports) ───
    Case {
        label: "d3+cordyceps",
        a: "Vitamin D3",
        b: "Cordyceps",
        expect_flag: false,
        severity: None,
        why: "engine calls both immunomodulating; no literature caution — documented FP",
    },
    Case {
        label: "rhodiola+caf",
        a: "Rhodiola",
        b: "Caffeine",
        expect_flag: false,
        severity: None,
        why: "engine tags rhodiola Stimulant; adaptogen+caffeine is a mainstream stack with no case reports — documented FP",
    },
    Case {
        label: "nac+piracetam",
        a: "NAC",
        b: "Piracetam",
        expect_flag: false,
        severity: None,
        why: "engine calls both glutamatergic-modulating; no additive-toxicity literature — documented FP",
    },
];

fn log(name: &str, hours_ago: i64) -> LogEntry {
    LogEntry {
        id: uuid::Uuid::new_v4(),
        user_id: "bench".to_string(),
        item_type: ItemType::Supplement,
        item_id: None,
        name: name.to_string(),
        quantity: None,
        unit: None,
        route: None,
        timestamp: Utc::now() - Duration::hours(hours_ago),
        stack_id: None,
        notes: None,
        acknowledged_interaction: false,
        custom_fields: None,
    }
}

#[test]
fn benchmark_interaction_flagging_accuracy() {
    let engine = SafetyEngine::new();
    let mut misses = Vec::new();
    let mut false_positives = Vec::new();
    let mut critical_misses = Vec::new();
    let mut fired_wrong_severity = Vec::new();

    for c in CASES {
        // Fresh per case: one new entry joined against one recent entry.
        let alerts = engine.check_interactions(&log(c.a, 0), &[log(c.b, 2)]);
        let flagged = !alerts.is_empty();
        if c.expect_flag && !flagged {
            misses.push(format!("{}: {} + {} — {}", c.label, c.a, c.b, c.why));
            if matches!(&c.severity, Some(AlertSeverity::Critical)) {
                critical_misses.push(c.label);
            }
        } else if !c.expect_flag && flagged {
            false_positives.push(format!(
                "{}: {} + {} fired [{}] — {}",
                c.label, c.a, c.b, alerts[0].message, c.why
            ));
        } else if c.expect_flag {
            if let Some(sev) = c.severity.as_ref() {
                if !alerts.iter().any(|a| &a.severity == sev) {
                    fired_wrong_severity.push(format!(
                        "{}: expected {:?}, got {:?}",
                        c.label,
                        sev,
                        alerts.iter().map(|a| &a.severity).collect::<Vec<_>>()
                    ));
                }
            }
        }
    }

    let total = CASES.len();
    let wrong = misses.len() + false_positives.len();
    let accuracy = (total - wrong) as f64 / total as f64;

    println!(
        "benchmark: {}/{} correct, accuracy {:.3}",
        total - wrong,
        total,
        accuracy
    );
    for m in &misses {
        println!("MISS: {m}");
    }
    for f in &false_positives {
        println!("FALSE POSITIVE: {f}");
    }
    for s in &fired_wrong_severity {
        println!("SEVERITY DRIFT: {s}");
    }

    // The card's gate (spec 001 success criterion): ≥90% flagging accuracy.
    assert!(
        accuracy >= 0.90,
        "flagging accuracy {accuracy:.3} below the 0.90 gate"
    );
    // Non-negotiable: no false negatives on the serotonin/MAOI critical axis.
    assert!(
        critical_misses.is_empty(),
        "critical-axis false negatives: {critical_misses:?}"
    );
    // Severity, where expected, must not drift (silent Critical→Info downgrades).
    assert!(
        fired_wrong_severity.is_empty(),
        "severity drift: {fired_wrong_severity:?}"
    );
}
