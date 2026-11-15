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

    pub(crate) fn extract(&self, v: &VitalsEntry) -> Option<f64> {
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

// ── US D3: trend / drift detection ──────────────────────────────────────────

/// Sustained drift in one metric over a measured window (US D3). Drift needs
/// no causal claim, which is exactly why it is the signal personal tracking
/// is genuinely good at — and the result says so in those terms (FR-095).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriftInsight {
    pub metric: Metric,
    pub window: (DateTime<Utc>, DateTime<Utc>),
    pub n: usize,
    /// Points per day inside the window (reported, never assumed).
    pub density: f64,
    pub direction: DriftDirection,
    /// Theil–Sen median slope, units per day. Robust to the outlier readings
    /// wearables produce; no distributional assumptions.
    pub slope_per_day: f64,
    /// slope × window span: the total move the slope implies over the window.
    pub total_move: f64,
    /// Fraction of pairwise slopes agreeing with `direction` (midrank for
    /// exact zeros). Directional consistency of the raw data, not a test.
    pub consistency: f64,
    pub gaps: Vec<Gap>,
    pub limitations: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftDirection {
    Rising,
    Falling,
}

/// Drift configuration. Defaults reflect spec 006's example unit of evidence
/// ("resting HR rising across 6 weeks"): four weeks minimum span, fourteen
/// points minimum, half a point per day minimum density.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DriftConfig {
    pub min_span_days: i64,
    pub min_points: usize,
    /// Below this density the insight is produced but qualified (AC D3-2).
    pub sparse_below_per_day: f64,
    /// |total move| must clear this multiple of the series MAD (or MAD==0
    /// with a nonzero move) before drift is reported at all.
    pub mad_multiple: f64,
    /// Pairwise-slope agreement required with the chosen direction.
    pub min_consistency: f64,
}

impl Default for DriftConfig {
    fn default() -> Self {
        Self {
            min_span_days: 28,
            min_points: 14,
            sparse_below_per_day: 0.5,
            mad_multiple: 1.5,
            min_consistency: 0.75,
        }
    }
}

fn mad(xs: &[f64]) -> f64 {
    let m = median(xs);
    let dev: Vec<f64> = xs.iter().map(|x| (x - m).abs()).collect();
    median(&dev)
}

/// Detect sustained drift in `metric` across the vitals history (US D3).
///
/// Returns `None` — no insight, no number — when the data cannot support one
/// (no history, span or point count below the floor, magnitude inside the
/// noise band, or direction not consistent enough). Pure function (FR-099).
pub fn detect_drift(
    vitals: &[VitalsEntry],
    metric: Metric,
    now: DateTime<Utc>,
    cfg: &DriftConfig,
) -> Option<DriftInsight> {
    let s = samples(vitals, metric, DateTime::<Utc>::MIN_UTC, now);
    if s.len() < cfg.min_points {
        return None; // AC D3-3: no ingested history ⇒ no insight
    }
    let t0 = s.first()?.0;
    let t1 = s.last()?.0;
    let span_days = (t1 - t0).num_days();
    if span_days < cfg.min_span_days {
        return None;
    }

    // Theil–Sen: median of all pairwise slopes. O(n²), trivial for n=1 data.
    let mut slopes: Vec<f64> = Vec::new();
    for i in 0..s.len() {
        for j in (i + 1)..s.len() {
            let dt = (s[j].0 - s[i].0).num_seconds() as f64 / 86400.0;
            if dt > 0.0 {
                slopes.push((s[j].1 - s[i].1) / dt);
            }
        }
    }
    if slopes.is_empty() {
        return None;
    }
    let slope = median(&slopes);
    let total_move = slope * span_days as f64;

    // Directional consistency of the raw pairwise movement (midrank zeros).
    let zeros = slopes.iter().filter(|x| x.abs() < f64::EPSILON).count() as f64;
    let pos = slopes.iter().filter(|x| **x > f64::EPSILON).count() as f64;
    let consistency_up = (pos + 0.5 * zeros) / slopes.len() as f64;
    let (direction, consistency) = if consistency_up >= 0.5 {
        (DriftDirection::Rising, consistency_up)
    } else {
        (DriftDirection::Falling, 1.0 - consistency_up)
    };

    // Magnitude vs. the series' own spread: drift must clear its noise band.
    let ys: Vec<f64> = s.iter().map(|(_, y)| *y).collect();
    let spread = mad(&ys);
    let magnitude_ok = if spread > 0.0 {
        total_move.abs() >= cfg.mad_multiple * spread
    } else {
        total_move.abs() > 0.0
    };
    if !magnitude_ok || consistency < cfg.min_consistency {
        return None; // inside the noise band: no number, no claim
    }

    let gaps = find_gaps(&s, t0, t1 + Duration::seconds(1), Side::Post);
    let density = density(s.len(), span_days.max(1));

    let mut limitations = format!(
        "Descriptive only, over this window, in your data: {} → {} (UTC days), metric: {}. \
         {} points at {:.2}/day. This is a sustained movement in what you recorded, not a \
         population-level finding, not a clinical conclusion, and not medical advice; day \
         boundaries are UTC.",
        t0.date_naive(),
        t1.date_naive(),
        metric.definition(),
        s.len(),
        density,
    );
    if density < cfg.sparse_below_per_day {
        limitations.push_str(
            " Data are sparse in this window: the drift is qualified by that density — \
             gaps in recording can mimic a trend.",
        );
    }
    if !gaps.is_empty() {
        limitations.push_str(&format!(
            " {} gap(s) of ≥2 days without samples sit inside the window; missing-device time \
             is not improvement.",
            gaps.len()
        ));
    }

    Some(DriftInsight {
        metric,
        window: (t0, t1),
        n: s.len(),
        density,
        direction,
        slope_per_day: slope,
        total_move,
        consistency,
        gaps,
        limitations,
    })
}

#[cfg(test)]
mod drift_tests {
    use super::*;
    use crate::models::VitalsEntry;
    use chrono::TimeZone;
    use uuid::Uuid;

    fn t(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, mo, d, h, mi, 0).unwrap()
    }

    fn hr(ts: DateTime<Utc>, hr: i32) -> VitalsEntry {
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

    /// 6 weeks of resting HR: 60 bpm rising 0.4/day ⇒ steady drift.
    fn rising_six_weeks() -> Vec<VitalsEntry> {
        let start = t(2026, 8, 1, 8, 0);
        (0..42)
            .map(|d| hr(start + Duration::days(d), (60.0 + 0.4 * d as f64) as i32))
            .collect()
    }

    #[test]
    fn d3_detects_six_week_drift_with_window_slope_n() {
        let v = rising_six_weeks();
        let d = detect_drift(
            &v,
            Metric::HeartRate,
            t(2026, 9, 12, 8, 0),
            &DriftConfig::default(),
        )
        .expect("six-week steady rise must produce a drift insight");
        assert_eq!(d.direction, DriftDirection::Rising);
        assert_eq!(d.n, 42);
        assert_eq!(d.window, (t(2026, 8, 1, 8, 0), t(2026, 9, 11, 8, 0)));
        assert!(d.slope_per_day > 0.3, "slope {}", d.slope_per_day);
        assert!(d.consistency >= 0.75);
        assert!(d.limitations.contains("in your data"));
        assert!(d.limitations.contains("not a clinical conclusion"));
    }

    #[test]
    fn d3_flat_series_produces_no_insight() {
        let start = t(2026, 8, 1, 8, 0);
        let v: Vec<_> = (0..42)
            .map(|d| {
                // tiny alternating wobble: zero slope, real noise
                hr(start + Duration::days(d), 60 + (d % 2) as i32)
            })
            .collect();
        assert!(
            detect_drift(
                &v,
                Metric::HeartRate,
                t(2026, 9, 12, 8, 0),
                &DriftConfig::default()
            )
            .is_none()
        );
    }

    #[test]
    fn d3_short_history_produces_no_insight() {
        let v = rising_six_weeks();
        // Only 10 days of it ingested ⇒ below both floors.
        let short: Vec<_> = v.into_iter().take(10).collect();
        assert!(
            detect_drift(
                &short,
                Metric::HeartRate,
                t(2026, 8, 11, 8, 0),
                &DriftConfig::default()
            )
            .is_none()
        );
    }

    #[test]
    fn d3_sparse_data_gets_qualified_by_density() {
        // Same rise, but every other day missing ⇒ density < 0.5/day.
        let v: Vec<_> = rising_six_weeks()
            .into_iter()
            .enumerate()
            .filter(|(i, _)| i % 3 == 0)
            .map(|(_, x)| x)
            .collect();
        let d = detect_drift(
            &v,
            Metric::HeartRate,
            t(2026, 9, 12, 8, 0),
            &DriftConfig::default(),
        )
        .expect("14 points over 41 days still clears the floors");
        assert!(d.density < 0.5);
        assert!(d.limitations.contains("sparse"));
        assert!(!d.gaps.is_empty(), "every-other-day gaps must be listed");
    }

    #[test]
    fn d3_determinism() {
        let v = rising_six_weeks();
        let now = t(2026, 9, 12, 8, 0);
        let a = detect_drift(&v, Metric::HeartRate, now, &DriftConfig::default());
        let b = detect_drift(&v, Metric::HeartRate, now, &DriftConfig::default());
        assert_eq!(a, b);
    }
}
