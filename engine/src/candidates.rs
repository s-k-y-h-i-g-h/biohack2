//! Spec 006 D2 + D4 — note-grounded correlation candidates and the insight
//! ledger.
//!
//! Honesty contract (FR-097): a note is a *hypothesis source*, never a
//! finding. `Candidate` deliberately has **no verdict field** — nothing in
//! this module can express a conclusion, and the only way to a verdict is a
//! full D1 `run_review` whose floors the user cannot skip. The ledger
//! (D4/FR-098) is what stops the loop from re-proposing a hypothesis the
//! user already tested: once a `(substance, metric)` pair carries a
//! non-`Proposed` status, the candidate list shows it as tested, not as new.
//!
//! All functions here are pure and deterministic: same inputs, same outputs,
//! no clock, no RNG (entry ids are content-hashed, `now` is passed in).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::insights::Metric;
use crate::models::{CatalogItem, Note, VitalsEntry};

/// Status per FR-098. A pair leaves `Proposed` only through an actual review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LedgerStatus {
    Proposed,
    Tested,
    Supported,
    Unsupported,
    Inconclusive,
}

impl LedgerStatus {
    pub fn is_tested(self) -> bool {
        self != LedgerStatus::Proposed
    }

    pub fn label(self) -> &'static str {
        match self {
            LedgerStatus::Proposed => "proposed",
            LedgerStatus::Tested => "tested",
            LedgerStatus::Supported => "supported",
            LedgerStatus::Unsupported => "unsupported",
            LedgerStatus::Inconclusive => "inconclusive",
        }
    }
}

/// Durable ledger record (FR-098): status, evidence link, outcome date.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: Uuid,
    pub substance: String,
    pub metric: Metric,
    pub status: LedgerStatus,
    /// Link back to the data window / review that produced the outcome
    /// (AC D4-2: the underlying window must be reachable from the entry).
    pub evidence: String,
    pub outcome_at: Option<DateTime<Utc>>,
    pub user_note: Option<String>,
}

/// Deterministic id so `record_outcome` needs no RNG: content-hashed on the
/// unique `(substance, metric)` key, case-folded on substance.
fn entry_id(substance: &str, metric: Metric) -> Uuid {
    let key = format!("{}\u{1}{}", substance.to_lowercase(), metric.name());
    let mut h1: u128 = 0xcbf29ce484222325;
    let mut h2: u128 = 0x84222325cbf29ce4;
    for b in key.bytes() {
        h1 = (h1 ^ b as u128).wrapping_mul(0x100000001b3);
        h2 = h2.rotate_left(7) ^ (b as u128).wrapping_mul(0x9e3779b97f4a7c15);
    }
    Uuid::from_u128(h1 ^ h2.rotate_left(64))
}

/// Find or create the entry for `(substance, metric)`, set its status and
/// stamp, and return the full ledger. Updating an existing entry preserves
/// `user_note` unless a new one is given. Order: existing entries keep their
/// position; new entries are appended (input order is the persistence order).
pub fn record_outcome(
    entries: &[LedgerEntry],
    substance: &str,
    metric: Metric,
    status: LedgerStatus,
    evidence: &str,
    user_note: Option<String>,
    now: DateTime<Utc>,
) -> Vec<LedgerEntry> {
    let id = entry_id(substance, metric);
    let mut out = entries.to_vec();
    if let Some(existing) = out.iter_mut().find(|e| e.id == id) {
        existing.status = status;
        existing.evidence = evidence.to_string();
        existing.outcome_at = Some(now);
        if user_note.is_some() {
            existing.user_note = user_note;
        }
    } else {
        out.push(LedgerEntry {
            id,
            substance: substance.to_string(),
            metric,
            status,
            evidence: evidence.to_string(),
            outcome_at: Some(now),
            user_note,
        });
    }
    out
}

/// Has this pair already been taken through a review? (AC D4-3)
pub fn is_tested(entries: &[LedgerEntry], substance: &str, metric: Metric) -> bool {
    let id = entry_id(substance, metric);
    entries.iter().any(|e| e.id == id && e.status.is_tested())
}

/// D2: a substance × metric pairing derived from a note, with the data
/// availability for it. No verdict is expressible here, by construction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Candidate {
    pub substance: String,
    /// `None` only for the untestable summary: the note named the substance
    /// but no measured outcome exists to pair it with (AC D2-3).
    pub metric: Option<Metric>,
    /// Stated origin of the hypothesis: the earliest note naming it (AC D2-2
    /// — the review keeps the note as its origin).
    pub origin_note_id: Uuid,
    pub origin_note_at: DateTime<Utc>,
    /// Readings of this metric present in the user's vitals history.
    pub data_points: usize,
    pub testable: bool,
    pub reason: String,
    /// Shown as tested, not as new (AC D4-3).
    pub tested: bool,
}

/// Whole-word, case-folded mention test: "ashwagandha" matches
/// "Increased anxiety after Ashwagandha at night" but "d3" does not match
/// "ld3x". Byte-level boundary checks are UTF-8 safe (non-ASCII bytes are
/// never alphanumeric, so they read as word boundaries).
fn mentions(haystack_lower: &str, needle_lower: &str) -> bool {
    if needle_lower.is_empty() {
        return false;
    }
    let bytes = haystack_lower.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = haystack_lower[from..].find(needle_lower) {
        let start = from + rel;
        let end = start + needle_lower.len();
        let before_ok = start == 0 || !bytes[start - 1].is_ascii_alphanumeric();
        let after_ok = end == bytes.len() || !bytes[end].is_ascii_alphanumeric();
        if before_ok && after_ok {
            return true;
        }
        from = end;
    }
    false
}

/// The most recent note naming `substance` — the anchor for re-testing a
/// ledger pair. Same whole-word matcher as candidate generation, so the
/// re-test measures the same substance the first test meant; picking the
/// newest mention is a fixed rule, not a knob the user can tune to shop
/// for a kinder window.
pub fn latest_mention<'a>(notes: &'a [Note], substance: &str) -> Option<&'a Note> {
    let name_lower = substance.to_lowercase();
    let mut hits: Vec<&Note> = notes
        .iter()
        .filter(|n| mentions(&n.content.to_lowercase(), &name_lower))
        .collect();
    hits.sort_by_key(|a| (a.timestamp, a.id));
    hits.last().copied()
}

const ALL_METRICS: [Metric; 7] = [
    Metric::HeartRate,
    Metric::Hrv,
    Metric::Weight,
    Metric::Spo2,
    Metric::BloodGlucose,
    Metric::BpSystolic,
    Metric::BpDiastolic,
];

/// Scan notes for catalog substance mentions; pair each mention with every
/// metric that has measurements, plus an untestable summary when none do.
/// One candidate per `(substance, metric)` — repeated notes strengthen the
/// origin trail but never manufacture data.
///
/// `catalog` is expected to be the (small) seed catalog; scan cost is
/// O(notes × items × metrics).
pub fn generate_candidates(
    notes: &[Note],
    catalog: &[CatalogItem],
    vitals: &[VitalsEntry],
    ledger: &[LedgerEntry],
) -> Vec<Candidate> {
    // Readings per metric, computed once.
    let points: Vec<(Metric, usize)> = ALL_METRICS
        .iter()
        .map(|m| (*m, vitals.iter().filter(|v| m.extract(v).is_some()).count()))
        .collect();

    let mut out: Vec<Candidate> = Vec::new();
    let mut seen: Vec<(String, Option<&'static str>)> = Vec::new();

    // Deterministic note order: oldest first, id breaks timestamp ties.
    let mut ordered: Vec<&Note> = notes.iter().collect();
    ordered.sort_by_key(|a| (a.timestamp, a.id));

    for item in catalog {
        let name_lower = item.name.to_lowercase();
        // Earliest note naming this substance is the stated origin.
        let origin = ordered
            .iter()
            .find(|n| mentions(&n.content.to_lowercase(), &name_lower));
        let Some(origin) = origin else { continue };

        let with_data: Vec<(Metric, usize)> = points
            .iter()
            .copied()
            .filter(|(_, count)| *count > 0)
            .collect();

        if with_data.is_empty() {
            // AC D2-3: no measurable outcome available → untestable, reason.
            let key = (item.name.to_lowercase(), None);
            if !seen.contains(&key) {
                seen.push(key);
                out.push(Candidate {
                    substance: item.name.clone(),
                    metric: None,
                    origin_note_id: origin.id,
                    origin_note_at: origin.timestamp,
                    data_points: 0,
                    testable: false,
                    reason: "no vitals readings recorded for any measurable outcome".to_string(),
                    tested: false,
                });
            }
            continue;
        }

        for (metric, count) in with_data {
            let key = (item.name.to_lowercase(), Some(metric.name()));
            if seen.contains(&key) {
                continue;
            }
            seen.push(key);
            let tested = is_tested(ledger, &item.name, metric);
            out.push(Candidate {
                substance: item.name.clone(),
                metric: Some(metric),
                origin_note_id: origin.id,
                origin_note_at: origin.timestamp,
                data_points: count,
                testable: true,
                reason: format!(
                    "note names {name}; {count} {metric} reading(s) available for a review",
                    name = item.name,
                    metric = metric.name()
                ),
                tested,
            });
        }
    }

    out.sort_by(|a, b| {
        (
            a.substance.to_lowercase(),
            a.metric.map(|m| m.name()).unwrap_or(""),
        )
            .cmp(&(
                b.substance.to_lowercase(),
                b.metric.map(|m| m.name()).unwrap_or(""),
            ))
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::seed_catalog;
    use chrono::Duration;

    fn note(content: &str, day: i64) -> Note {
        Note {
            id: Uuid::new_v4(),
            user_id: "u1".into(),
            content: content.into(),
            timestamp: Utc::now() - Duration::days(day),
            linked_entry_id: None,
        }
    }

    fn vitals_with_hr(n: usize) -> Vec<VitalsEntry> {
        (0..n)
            .map(|i| VitalsEntry {
                id: Uuid::new_v4(),
                user_id: "u1".into(),
                timestamp: Utc::now() - Duration::days(i as i64),
                bp_systolic: None,
                bp_diastolic: None,
                heart_rate: Some(60 + i as i32),
                weight: None,
                blood_glucose: None,
                temperature: None,
                spo2: None,
                hrv: None,
                sleep_quality: None,
                custom_metrics: None,
                notes: None,
            })
            .collect()
    }

    fn catalog_hit(name: &str) -> Vec<CatalogItem> {
        seed_catalog()
            .into_iter()
            .filter(|c| c.name.eq_ignore_ascii_case(name))
            .collect()
    }

    #[test]
    fn ac_d2_1_note_naming_substance_yields_candidate_with_data() {
        let catalog = catalog_hit("Ashwagandha");
        assert!(!catalog.is_empty(), "seed catalog must contain Ashwagandha");
        let notes = vec![note("Increased anxiety after Ashwagandha at night", 3)];
        let vitals = vitals_with_hr(9);
        let cands = generate_candidates(&notes, &catalog, &vitals, &[]);
        assert!(!cands.is_empty());
        let hr = cands
            .iter()
            .find(|c| c.metric == Some(Metric::HeartRate))
            .expect("heart-rate pairing");
        assert_eq!(hr.data_points, 9);
        assert!(hr.testable);
        assert!(!hr.tested);
        // AC D2-2: origin is the stated note.
        assert_eq!(hr.origin_note_id, notes[0].id);
    }

    #[test]
    fn ac_d2_3_no_measurements_is_untestable_with_reason() {
        let catalog = catalog_hit("Ashwagandha");
        let notes = vec![note("started ashwagandha", 1)];
        let cands = generate_candidates(&notes, &catalog, &[], &[]);
        assert_eq!(cands.len(), 1);
        assert!(!cands[0].testable);
        assert!(cands[0].metric.is_none());
        assert!(cands[0].reason.contains("no vitals readings"));
    }

    #[test]
    fn notes_naming_nothing_produce_nothing() {
        let catalog = catalog_hit("Ashwagandha");
        let notes = vec![note("slept fine, walked 8k steps", 1)];
        assert!(generate_candidates(&notes, &catalog, &vitals_with_hr(5), &[]).is_empty());
    }

    #[test]
    fn latest_mention_picks_newest_not_oldest() {
        // A re-test must anchor on the newest mention (more data after it),
        // never drift to the earliest one just because it sorts first.
        let notes = vec![
            note("started magnesium", 12),
            note("raised magnesium to 400mg", 5),
            note("slept fine", 1),
        ];
        let found = latest_mention(&notes, "Magnesium").expect("a mention");
        assert_eq!(found.content, "raised magnesium to 400mg");
        assert!(latest_mention(&notes, "CoQ10").is_none());
    }

    #[test]
    fn whole_word_matching_avoids_substring_false_positives() {
        // Contract: caller passes case-folded inputs (generate_candidates
        // lowercases both sides before matching).
        assert!(mentions("took ashwagandha tonight", "ashwagandha"));
        assert!(mentions("ashwagandha, capped", "ashwagandha"));
        assert!(!mentions("ld3x and k2", "d3"));
        assert!(mentions("vitamin d3 levels", "d3"));
    }

    #[test]
    fn ac_d4_record_then_show_as_tested_not_new() {
        let catalog = catalog_hit("Ashwagandha");
        let notes = vec![note("anxiety after ashwagandha", 2)];
        let vitals = vitals_with_hr(10);

        // Mark unsupported (AC D4-1), with evidence link and outcome date.
        let now = Utc::now();
        let ledger = record_outcome(
            &[],
            "Ashwagandha",
            Metric::HeartRate,
            LedgerStatus::Unsupported,
            "review:14d+14d,hr,rank-biserial",
            None,
            now,
        );
        assert_eq!(ledger.len(), 1);
        assert_eq!(ledger[0].outcome_at, Some(now));
        assert!(is_tested(&ledger, "ashwagandha", Metric::HeartRate));

        // The candidate list now shows it tested, not new (AC D4-3).
        let cands = generate_candidates(&notes, &catalog, &vitals, &ledger);
        let hr = cands
            .iter()
            .find(|c| c.metric == Some(Metric::HeartRate))
            .unwrap();
        assert!(hr.tested);
        // Untested pairs are still new.
        let other = cands.iter().find(|c| c.metric != Some(Metric::HeartRate));
        if let Some(o) = other {
            assert!(!o.tested);
        }

        // Re-testing updates in place; the ledger never grows duplicates.
        let later = record_outcome(
            &ledger,
            "ashwagandha",
            Metric::HeartRate,
            LedgerStatus::Inconclusive,
            "review:2",
            Some("re-ran with wider window".into()),
            now + Duration::days(5),
        );
        assert_eq!(later.len(), 1);
        assert_eq!(later[0].status, LedgerStatus::Inconclusive);
        assert!(later[0].user_note.is_some());
    }

    #[test]
    fn determinism_same_input_identical_output() {
        let catalog = catalog_hit("Ashwagandha");
        let notes = vec![note("a: ashwagandha", 1), note("b: ashwagandha", 2)];
        let vitals = vitals_with_hr(4);
        let a = generate_candidates(&notes, &catalog, &vitals, &[]);
        let b = generate_candidates(&notes, &catalog, &vitals, &[]);
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
        // Origin picks the earliest note deterministically regardless of order.
        let c = generate_candidates(
            &notes.iter().rev().cloned().collect::<Vec<_>>(),
            &catalog,
            &vitals,
            &[],
        );
        assert_eq!(a[0].origin_note_id, c[0].origin_note_id);
    }

    #[test]
    fn no_verdict_is_expressible_in_a_candidate() {
        // Structural guarantee (FR-097): Candidate has no verdict/field that
        // could carry a conclusion. If someone adds one, this test documents
        // the intent; the type-level guarantee is the absence of the field.
        let json = serde_json::to_value(Candidate {
            substance: "x".into(),
            metric: None,
            origin_note_id: Uuid::new_v4(),
            origin_note_at: Utc::now(),
            data_points: 0,
            testable: false,
            reason: "r".into(),
            tested: false,
        })
        .unwrap();
        let keys: Vec<&str> = json
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        for forbidden in ["verdict", "conclusion", "effect", "p_value", "significant"] {
            assert!(
                !keys.contains(&forbidden),
                "Candidate must not carry {forbidden}"
            );
        }
    }
}
