//! Insights loop (spec 006) — intervention–outcome review (US D1, MVP).
//!
//! Design rule (spec 006): personal n=1 data is short, noisy and confounded.
//! The engine reports what the data can support and says plainly when it
//! cannot support anything. No model inference in the result path (FR-090);
//! deterministic and reproducible for identical inputs (FR-099); fully
//! offline (FR-100).
//!
//! Known limitation surfaced in every result's limitations text: day
//! boundaries are UTC, not user-local (the 2026-09-09 chrono incident made
//! day-boundary handling a named hazard; local-day aggregation needs a
//! user-timezone input this engine does not yet take).

use crate::models::{LogEntry, VitalsEntry};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// A numeric vital this engine can review. Each metric carries its own
/// definition string so a result names exactly what was measured (FR-091).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Metric {
    HeartRate,
    Hrv,
    Weight,
    Spo2,
    BloodGlucose,
    BpSystolic,
    BpDiastolic,
}

impl Metric {
    pub fn name(&self) -> &'static str {
        match self {
            Metric::HeartRate => "heart_rate",
            Metric::Hrv => "hrv",
            Metric::Weight => "weight",
            Metric::Spo2 => "spo2",
            Metric::BloodGlucose => "blood_glucose",
            Metric::BpSystolic => "bp_systolic",
            Metric::BpDiastolic => "bp_diastolic",
        }
    }

    pub fn definition(&self) -> &'static str {
        match self {
            Metric::HeartRate => "recorded heart rate (bpm), all readings in window",
            Metric::Hrv => "recorded heart-rate variability, all readings in window",
            Metric::Weight => "recorded body weight, all readings in window",
            Metric::Spo2 => "recorded blood oxygen saturation (%), all readings in window",
            Metric::BloodGlucose => "recorded blood glucose, all readings in window",
            Metric::BpSystolic => "recorded systolic blood pressure (mmHg), all readings in window",
            Metric::BpDiastolic => {
                "recorded diastolic blood pressure (mmHg), all readings in window"
            }
        }
    }

    fn extract(&self, v: &VitalsEntry) -> Option<f64> {
        match self {
            Metric::HeartRate => v.heart_rate.map(|x| x as f64),
            Metric::Hrv => v.hrv,
            Metric::Weight => v.weight,
            Metric::Spo2 => v.spo2.map(|x| x as f64),
            Metric::BloodGlucose => v.blood_glucose,
            Metric::BpSystolic => v.bp_systolic.map(|x| x as f64),
            Metric::BpDiastolic => v.bp_diastolic.map(|x| x as f64),
        }
    }
}

/// The change under review: a substance/action first logged at `started_at`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intervention {
    pub name: String,
    pub started_at: DateTime<Utc>,
}

/// Review parameters. `min_points` is the configured floor below which no
/// numeric result may be produced (FR-092).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSpec {
    pub metric: Metric,
    pub pre_days: i64,
    pub post_days: i64,
    pub min_points: usize,
}

impl ReviewSpec {
    /// Spec default: 14 days each side, floor of 7 points per side.
    pub fn default_for(metric: Metric) -> Self {
        Self {
            metric,
            pre_days: 14,
            post_days: 14,
            min_points: 7,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    pub n: usize,
    pub median: f64,
    pub min: f64,
    pub max: f64,
}

/// Another logged change visible inside the review window (FR-093).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Confounder {
    pub name: String,
    pub first_seen: DateTime<Utc>,
    /// Substance never logged before the review window (started), vs. one
    /// whose log volume changed inside it.
    pub kind: ConfounderKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfounderKind {
    StartedInWindow,
    LogVolumeChanged,
}

/// A run of ≥2 consecutive days inside a window with zero samples of the
/// metric (FR-101). Wearable gaps look like improvement; they must not.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gap {
    pub window: Side,
    pub start_day: chrono::NaiveDate,
    pub end_day: chrono::NaiveDate,
    pub days: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Pre,
    Post,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Verdict {
    /// An effect beyond the documented noise threshold, with enough data.
    ChangeObserved,
    /// Effect inside the noise band, with enough data.
    NoChangeObserved,
    /// Enough data but effect straddles the threshold.
    Inconclusive,
    /// Floor not met (FR-092): no numbers are reported at all.
    InsufficientData,
}

/// Deterministic review output. Carries evidence window, n, density, metric
/// definition, confounders, gaps and limitations in the same object as the
/// result (FR-091/093/094/095).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewResult {
    pub verdict: Verdict,
    pub metric: Metric,
    pub pre_window: (DateTime<Utc>, DateTime<Utc>),
    pub post_window: (DateTime<Utc>, DateTime<Utc>),
    pub pre: Option<Summary>,
    pub post: Option<Summary>,
    pub n_pre: usize,
    pub n_post: usize,
    /// Points per day per window (density is reported, not assumed).
    pub density_pre: f64,
    pub density_post: f64,
    /// Rank-biserial correlation between pre and post samples, in [-1, 1].
    /// None when the comparison was refused.
    pub effect_size: Option<f64>,
    pub median_shift: Option<f64>,
    pub confounders: Vec<Confounder>,
    pub gaps: Vec<Gap>,
    pub limitations: String,
    /// What is missing when verdict is InsufficientData (FR-092).
    pub missing: Vec<String>,
}

fn median(xs: &[f64]) -> f64 {
    let mut v = xs.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

/// Rank-biserial correlation from Mann-Whitney U. Deterministic, no
/// distributional assumptions, O(n·m) which is trivial for n=1 data.
fn rank_biserial(pre: &[f64], post: &[f64]) -> f64 {
    let (mut wins, mut losses, mut ties) = (0.0f64, 0.0f64, 0.0f64);
    for p in post {
        for q in pre {
            match p.partial_cmp(q) {
                Some(std::cmp::Ordering::Greater) => wins += 1.0,
                Some(std::cmp::Ordering::Less) => losses += 1.0,
                _ => ties += 1.0,
            }
        }
    }
    let total = pre.len() as f64 * post.len() as f64;
    if total == 0.0 {
        return 0.0;
    }
    // Midrank convention: ties split evenly.
    (wins + 0.5 * ties - (losses + 0.5 * ties)) / total
}

/// Collect (timestamp, value) samples of `metric` inside [from, to).
fn samples(
    vitals: &[VitalsEntry],
    metric: Metric,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Vec<(DateTime<Utc>, f64)> {
    let mut s: Vec<_> = vitals
        .iter()
        .filter(|v| v.timestamp >= from && v.timestamp < to)
        .filter_map(|v| metric.extract(v).map(|x| (v.timestamp, x)))
        .collect();
    s.sort_by_key(|(t, _)| *t);
    s
}

/// Runs of ≥2 consecutive whole UTC days inside [from, to) with no sample.
fn find_gaps(
    samples: &[(DateTime<Utc>, f64)],
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    window: Side,
) -> Vec<Gap> {
    let mut gaps = Vec::new();
    let first_day = from.date_naive();
    let last_day = (to - Duration::seconds(1)).date_naive();
    let days: Vec<_> = (0..=(last_day.signed_duration_since(first_day).num_days()))
        .map(|d| first_day + Duration::days(d))
        .collect();
    let has_sample: Vec<bool> = days
        .iter()
        .map(|d| samples.iter().any(|(t, _)| &t.date_naive() == d))
        .collect();
    let mut run_start: Option<usize> = None;
    for (i, present) in has_sample
        .iter()
        .enumerate()
        .chain(std::iter::once((days.len(), &true)))
    {
        if *present {
            if let Some(s) = run_start.take()
                && i - s >= 2
            {
                gaps.push(Gap {
                    window,
                    start_day: days[s],
                    end_day: days[i - 1],
                    days: (i - s) as i64,
                });
            }
        } else if run_start.is_none() {
            run_start = Some(i);
        }
    }
    gaps
}

fn density(n: usize, days: i64) -> f64 {
    if days <= 0 {
        0.0
    } else {
        n as f64 / days as f64
    }
}

/// Find logged changes inside the review window other than the intervention
/// itself (FR-093). A name first appearing inside the window is `Started`;
/// a name whose daily log volume more than doubles vs. the pre-window period
/// is `LogVolumeChanged`.
fn find_confounders(
    logs: &[LogEntry],
    intervention: &str,
    win_start: DateTime<Utc>,
    win_end: DateTime<Utc>,
) -> Vec<Confounder> {
    let norm = |s: &str| s.to_lowercase().replace([' ', '-', '_'], "");
    let target = norm(intervention);

    let mut by_name: std::collections::BTreeMap<String, Vec<DateTime<Utc>>> = Default::default();
    for l in logs {
        by_name.entry(norm(&l.name)).or_default().push(l.timestamp);
    }

    let mut out = Vec::new();
    for (name, mut stamps) in by_name {
        if name == target || name.is_empty() {
            continue;
        }
        stamps.sort();
        let in_window: Vec<_> = stamps
            .iter()
            .filter(|t| **t >= win_start && **t < win_end)
            .collect();
        if in_window.is_empty() {
            continue;
        }
        let before = stamps.iter().any(|t| *t < win_start);
        if !before {
            out.push(Confounder {
                name,
                first_seen: *in_window[0],
                kind: ConfounderKind::StartedInWindow,
            });
        } else {
            // crude volume test: more logs inside window than in the equal
            // stretch before it ⇒ volume changed (conservative, documented).
            let span = (win_end - win_start).num_seconds().max(1) as u64;
            let after = in_window.len();
            let before_count = stamps
                .iter()
                .filter(|t| {
                    **t >= win_start - chrono::Duration::seconds(span as i64) && **t < win_start
                })
                .count();
            if before_count == 0 || after > 2 * before_count {
                out.push(Confounder {
                    name,
                    first_seen: *in_window[0],
                    kind: ConfounderKind::LogVolumeChanged,
                });
            }
        }
    }
    out.sort_by_key(|c| c.first_seen);
    out
}

/// Effect threshold, documented so FR-099 reproducibility covers the rule
/// itself: |rank-biserial| ≥ 0.5 (medium effect, Cohen-equivalent) counts as
/// observed change; |median shift| ≤ 2 % of pre median counts as no change;
/// anything between is inconclusive. These are description thresholds for
/// one person's own data, not significance tests.
const RB_OBSERVED: f64 = 0.5;
const MEDIAN_NULL_PCT: f64 = 0.02;

/// Run an intervention–outcome review (US D1). Pure function of its inputs:
/// identical data + spec ⇒ identical result (FR-099).
pub fn run_review(
    vitals: &[VitalsEntry],
    logs: &[LogEntry],
    intervention: &Intervention,
    spec: &ReviewSpec,
    now: DateTime<Utc>,
) -> ReviewResult {
    let pre_start = intervention.started_at - Duration::days(spec.pre_days);
    let post_end = (intervention.started_at + Duration::days(spec.post_days)).min(now);
    let pre_window = (pre_start, intervention.started_at);
    let post_window = (intervention.started_at, post_end);

    let pre_s = samples(vitals, spec.metric, pre_window.0, pre_window.1);
    let post_s = samples(vitals, spec.metric, post_window.0, post_window.1);

    let mut gaps = find_gaps(&pre_s, pre_window.0, pre_window.1, Side::Pre);
    gaps.extend(find_gaps(&post_s, post_window.0, post_window.1, Side::Post));
    let confounders = find_confounders(logs, &intervention.name, pre_window.0, post_window.1);

    let mut missing = Vec::new();
    if (intervention.started_at - pre_start).num_days() < spec.pre_days {
        missing.push(format!(
            "pre-window shorter than requested ({} of {} days exist before the intervention)",
            (intervention.started_at - pre_start).num_days(),
            spec.pre_days
        ));
    }
    if pre_s.len() < spec.min_points {
        missing.push(format!(
            "pre-side: {} of {} required {} samples of {}",
            pre_s.len(),
            spec.min_points,
            spec.metric.name(),
            spec.metric.definition()
        ));
    }
    if post_s.len() < spec.min_points {
        missing.push(format!(
            "post-side: {} of {} required {} samples of {}",
            post_s.len(),
            spec.min_points,
            spec.metric.name(),
            spec.metric.definition()
        ));
    }

    let limitations = limitations_text(spec, &pre_window, &post_window, &gaps, &confounders);

    // FR-092: below the floor we refuse the number entirely.
    if !missing.is_empty() {
        return ReviewResult {
            verdict: Verdict::InsufficientData,
            metric: spec.metric,
            pre_window,
            post_window,
            pre: None,
            post: None,
            n_pre: pre_s.len(),
            n_post: post_s.len(),
            density_pre: density(pre_s.len(), spec.pre_days),
            density_post: density(post_s.len(), spec.post_days),
            effect_size: None,
            median_shift: None,
            confounders,
            gaps,
            limitations,
            missing,
        };
    }

    let pre_v: Vec<f64> = pre_s.iter().map(|(_, v)| *v).collect();
    let post_v: Vec<f64> = post_s.iter().map(|(_, v)| *v).collect();
    let (pre_med, post_med) = (median(&pre_v), median(&post_v));
    let shift = post_med - pre_med;
    let rb = rank_biserial(&pre_v, &post_v);

    let verdict = if rb.abs() >= RB_OBSERVED {
        Verdict::ChangeObserved
    } else if pre_med != 0.0 && (shift / pre_med).abs() <= MEDIAN_NULL_PCT {
        Verdict::NoChangeObserved
    } else {
        Verdict::Inconclusive
    };

    ReviewResult {
        verdict,
        metric: spec.metric,
        pre_window,
        post_window,
        pre: Some(Summary {
            n: pre_v.len(),
            median: pre_med,
            min: pre_v.iter().cloned().fold(f64::INFINITY, f64::min),
            max: pre_v.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        }),
        post: Some(Summary {
            n: post_v.len(),
            median: post_med,
            min: post_v.iter().cloned().fold(f64::INFINITY, f64::min),
            max: post_v.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        }),
        n_pre: pre_v.len(),
        n_post: post_v.len(),
        density_pre: density(pre_v.len(), spec.pre_days),
        density_post: density(post_v.len(), spec.post_days),
        effect_size: Some(rb),
        median_shift: Some(shift),
        confounders,
        gaps,
        limitations,
        missing,
    }
}

/// Limitations generated with the result, shown in the same view (FR-094),
/// in descriptive self-limited language (FR-095).
fn limitations_text(
    spec: &ReviewSpec,
    pre: &(DateTime<Utc>, DateTime<Utc>),
    post: &(DateTime<Utc>, DateTime<Utc>),
    gaps: &[Gap],
    confounders: &[Confounder],
) -> String {
    let mut t = format!(
        "Descriptive only, over this window, in your data: pre {} → {}, post {} → {} (UTC days), \
         metric: {}. This is not a population-level finding, not a clinical conclusion, and not \
         medical advice. n per side is small; a review of one person's data cannot establish \
         causation. Day boundaries are UTC, not your local timezone.",
        pre.0.date_naive(),
        pre.1.date_naive(),
        post.0.date_naive(),
        post.1.date_naive(),
        spec.metric.definition(),
    );
    if !gaps.is_empty() {
        t.push_str(&format!(
            " {} gap(s) of ≥2 days without samples sit inside these windows; missing-device time is not improvement.",
            gaps.len()
        ));
    }
    if !confounders.is_empty() {
        let names: Vec<&str> = confounders.iter().map(|c| c.name.as_str()).collect();
        t.push_str(&format!(
            " {} other logged change(s) overlap this window ({}): treat any observed shift as possibly theirs, not the intervention's.",
            confounders.len(),
            names.join(", ")
        ));
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ItemType, VitalsEntry};
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    fn base_ts() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 1, 8, 0, 0).unwrap()
    }

    fn vitals_at(ts: DateTime<Utc>, hr: i32) -> VitalsEntry {
        VitalsEntry {
            id: Uuid::new_v4(),
            user_id: "u".into(),
            timestamp: ts,
            bp_systolic: None,
            bp_diastolic: None,
            heart_rate: Some(hr),
            weight: None,
            blood_glucose: None,
            temperature: None,
            spo2: None,
            hrv: None,
            sleep_quality: None,
            custom_metrics: None,
            notes: None,
        }
    }

    fn log_at(ts: DateTime<Utc>, name: &str) -> LogEntry {
        LogEntry {
            id: Uuid::new_v4(),
            user_id: "u".into(),
            item_type: ItemType::Supplement,
            item_id: None,
            name: name.into(),
            quantity: Some(1.0),
            unit: Some("mg".into()),
            route: None,
            timestamp: ts,
            stack_id: None,
            notes: None,
            acknowledged_interaction: false,
            custom_fields: None,
        }
    }

    /// 14 days at 72 bpm before, 66 bpm after, 1/day ⇒ medium effect.
    fn shift_dataset(vitals: &mut Vec<VitalsEntry>) {
        for d in 0..14 {
            vitals.push(vitals_at(base_ts() - Duration::days(14 - d), 72));
        }
        for d in 0..14 {
            vitals.push(vitals_at(base_ts() + Duration::days(d), 66));
        }
    }

    #[test]
    fn d1_detects_shift_with_limitations() {
        let mut v = Vec::new();
        shift_dataset(&mut v);
        let r = run_review(
            &v,
            &[],
            &Intervention {
                name: "magnesium".into(),
                started_at: base_ts(),
            },
            &ReviewSpec::default_for(Metric::HeartRate),
            base_ts() + Duration::days(14),
        );
        assert_eq!(r.verdict, Verdict::ChangeObserved);
        assert_eq!(r.n_pre, 14);
        assert_eq!(r.n_post, 14);
        assert_eq!(r.pre.as_ref().unwrap().median, 72.0);
        assert_eq!(r.post.as_ref().unwrap().median, 66.0);
        assert_eq!(r.median_shift, Some(-6.0));
        // FR-094: limitations ride inside the result, not behind a link.
        assert!(r.limitations.contains("in your data"));
        assert!(r.limitations.contains("not medical advice"));
    }

    #[test]
    fn fr092_refuses_number_below_floor() {
        let mut v = Vec::new();
        for d in 0..5 {
            v.push(vitals_at(base_ts() - Duration::days(5 - d), 72));
        }
        let r = run_review(
            &v,
            &[],
            &Intervention {
                name: "magnesium".into(),
                started_at: base_ts(),
            },
            &ReviewSpec::default_for(Metric::HeartRate),
            base_ts() + Duration::days(2),
        );
        assert_eq!(r.verdict, Verdict::InsufficientData);
        assert!(r.pre.is_none() && r.post.is_none());
        assert!(r.effect_size.is_none() && r.median_shift.is_none());
        assert!(!r.missing.is_empty());
    }

    #[test]
    fn fr093_lists_confounders_in_window() {
        let mut v = Vec::new();
        shift_dataset(&mut v);
        let logs = vec![
            log_at(base_ts() - Duration::days(20), "magnesium"),
            log_at(base_ts() - Duration::days(20), "caffeine"),
            // Ashwagandha first appears inside the review window ⇒ confounder.
            log_at(base_ts() + Duration::days(2), "ashwagandha"),
        ];
        let r = run_review(
            &v,
            &logs,
            &Intervention {
                name: "magnesium".into(),
                started_at: base_ts(),
            },
            &ReviewSpec::default_for(Metric::HeartRate),
            base_ts() + Duration::days(14),
        );
        assert!(
            r.confounders
                .iter()
                .any(|c| c.name == "ashwagandha" && c.kind == ConfounderKind::StartedInWindow)
        );
        // caffeine only logged before the window ⇒ not a confounder.
        assert!(!r.confounders.iter().any(|c| c.name == "caffeine"));
        assert!(r.limitations.contains("ashwagandha"));
    }

    #[test]
    fn fr101_reports_device_gaps() {
        let mut v: Vec<VitalsEntry> = Vec::new();
        // pre-side dense; post-side has a 4-day hole (wearable disconnected).
        for d in 0..14 {
            v.push(vitals_at(base_ts() - Duration::days(14 - d), 72));
        }
        for d in 0..3 {
            v.push(vitals_at(base_ts() + Duration::days(d), 68));
        }
        for d in 7..11 {
            v.push(vitals_at(base_ts() + Duration::days(d), 68));
        }
        let spec = ReviewSpec {
            min_points: 5,
            ..ReviewSpec::default_for(Metric::HeartRate)
        };
        let r = run_review(
            &v,
            &[],
            &Intervention {
                name: "magnesium".into(),
                started_at: base_ts(),
            },
            &spec,
            base_ts() + Duration::days(14),
        );
        assert!(
            r.gaps.iter().any(|g| g.window == Side::Post && g.days >= 4),
            "expected a ≥4-day post gap, got {:?}",
            r.gaps
        );
        assert!(
            r.limitations
                .contains("missing-device time is not improvement")
        );
    }

    #[test]
    fn fr099_determinism_same_input_same_result() {
        let mut v = Vec::new();
        shift_dataset(&mut v);
        let iv = Intervention {
            name: "magnesium".into(),
            started_at: base_ts(),
        };
        let spec = ReviewSpec::default_for(Metric::HeartRate);
        let now = base_ts() + Duration::days(14);
        let a = run_review(&v, &[], &iv, &spec, now);
        let b = run_review(&v, &[], &iv, &spec, now);
        // Full structural equality incl. floats — pure function, no RNG.
        assert_eq!(a, b);
    }

    #[test]
    fn no_change_band_within_noise() {
        let mut v = Vec::new();
        for d in 0..28 {
            v.push(vitals_at(
                base_ts() - Duration::days(14) + Duration::days(d),
                72,
            ));
        }
        let r = run_review(
            &v,
            &[],
            &Intervention {
                name: "placebo".into(),
                started_at: base_ts(),
            },
            &ReviewSpec::default_for(Metric::HeartRate),
            base_ts() + Duration::days(14),
        );
        assert_eq!(r.verdict, Verdict::NoChangeObserved);
    }
}
