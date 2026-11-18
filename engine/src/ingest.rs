//! Source-agnostic wearable ingest contract (spec-004, FR-050..FR-059).
//!
//! Core types and pure functions live here (WASM-safe: no sqlx). The SQLite
//! persistence half is in `db.rs` behind the `db` feature.
//!
//! Guarantees implemented here:
//! - **Definition-gated metrics** (FR-055): a metric only lands in a native
//!   `VitalsEntry` column when its *definition* matches the column's meaning.
//!   Anything else becomes a `custom_metrics` entry carrying its definition,
//!   so incompatible values are never compared.
//! - **Unit normalisation to canonical units**: kg, mmHg, bpm, ms, °C, %,
//!   mg/dL. Unknown units are custom, never guessed.
//! - **Explicit timezone** (T306): timestamps arrive either as RFC 3339 with
//!   an explicit offset, or naive + a supplied source timezone. Naive without
//!   a zone is refused — the 2026-09-09 write-flow bug class.
//! - **Implausible-timestamp quarantine** (T307): more than 5 min into the
//!   future (clock-skew tolerance) or before 2000-01-01 → quarantined, never
//!   stored.

use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

// ── Sources ──────────────────────────────────────────────────────────────────

/// How a batch of readings entered the system (FR-053/FR-054/FR-058).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    /// Offline file import (FIT/CSV/JSON export) — no account, no network.
    File,
    /// Push from a companion app over the local network.
    Companion,
    /// Pull from a vendor/cloud API (never required in v1).
    Api,
}

// ── File parsers (T313) ────────────────────────────────────────────────────────
#[cfg(feature = "file-ingest")]
pub mod generic;

/// A device/platform we ingest from. `id` is stable per device+platform,
/// decided at the boundary that creates the source record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestSource {
    pub id: String,
    pub kind: SourceKind,
    pub device_label: Option<String>,
    pub platform: Option<String>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

// ── Raw readings ─────────────────────────────────────────────────────────────

/// One measurement as produced by a source, before mapping. This is the
/// ingest contract every reader (file/companion/api) must produce (T300).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawReading {
    pub source_id: String,
    /// Vendor's own stable id for this record — the idempotency key (FR-051).
    pub source_record_id: String,
    pub metric: String,
    /// What the number *means* — `rmssd` vs `sdnn`, `score_0_100`, etc.
    pub metric_definition: String,
    pub value: f64,
    pub unit: String,
    pub observed_at: DateTime<Utc>,
    /// Offset recorded at ingest time (T306). `None` only when the source
    /// gave an explicit UTC/Z timestamp.
    pub source_timezone: Option<String>,
    pub ingested_at: DateTime<Utc>,
    /// Verbatim source fragment (JSON/CSV line) for audit and retry (FR-059).
    pub payload: Option<String>,
}

// ── Timestamp normalisation (T306/T307) ──────────────────────────────────────

/// Clock-skew tolerance for future timestamps.
pub const FUTURE_SKEW_TOLERANCE_MINUTES: i64 = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimestampRejection {
    /// Unparseable timestamp.
    Unparseable(String),
    /// Naive timestamp without a declared source timezone — refused, never
    /// guessed (this caused the 2026-09-09 write-flow breakage).
    MissingTimezone,
    /// More than [`FUTURE_SKEW_TOLERANCE_MINUTES`] after `now`.
    ImplausibleFuture,
    /// Before 2000-01-01 (epoch-0 misencodes, 1601 Windows defaults, …).
    ImplausiblePast,
}

/// Normalise a source timestamp to UTC and quarantine implausible values.
///
/// Accepted inputs:
/// - RFC 3339 **with** an explicit offset (`2026-11-18T09:30:00+02:00`, `…Z`)
/// - naive ISO (`2026-11-18T09:30:00` / `2026-11-18 09:30:00`) **only** with
///   `source_tz` supplied (e.g. `+02:00`).
///
/// Returns the UTC instant plus the offset string recorded with it
/// (`Some("+02:00")`; `None` when the source said `Z`).
pub fn normalise_timestamp(
    raw: &str,
    source_tz: Option<&str>,
    now: DateTime<Utc>,
) -> Result<(DateTime<Utc>, Option<String>), TimestampRejection> {
    let naive_upper_limit = now + chrono::Duration::minutes(FUTURE_SKEW_TOLERANCE_MINUTES);

    let (utc, offset_label) = if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        let label = if dt.offset().local_minus_utc() == 0 {
            None
        } else {
            Some(dt.offset().to_string())
        };
        (dt.with_timezone(&Utc), label)
    } else {
        let naive = NaiveDateTime::parse_from_str(raw, "%Y-%m-%dT%H:%M:%S%.f")
            .or_else(|_| NaiveDateTime::parse_from_str(raw, "%Y-%m-%dT%H:%M"))
            .or_else(|_| NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S%.f"))
            .or_else(|_| NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M"))
            .map_err(|_| TimestampRejection::Unparseable(raw.to_string()))?;
        let label = source_tz.ok_or(TimestampRejection::MissingTimezone)?;
        // Explicit UTC markers: unambiguous in any feature config, and the
        // contract records `None` for sources that plainly say UTC/Z.
        if label.eq_ignore_ascii_case("z") || label.eq_ignore_ascii_case("utc") {
            return Ok((naive.and_utc(), None));
        }
        if let Ok(off) = parse_offset(label) {
            let local = off
                .from_local_datetime(&naive)
                .single()
                .ok_or(TimestampRejection::Unparseable(raw.to_string()))?;
            (local.with_timezone(&Utc), Some(off.to_string()))
        } else {
            // Not a fixed offset — try an IANA name (`Europe/London`), the
            // form real wearable exports actually use. Only linked where
            // file parsing runs (native), never in the WASM bundle.
            #[cfg(feature = "file-ingest")]
            {
                let tz: chrono_tz::Tz = label
                    .trim()
                    .parse()
                    .map_err(|_| TimestampRejection::Unparseable(label.to_string()))?;
                let local = tz.from_local_datetime(&naive).single().ok_or_else(|| {
                    TimestampRejection::Unparseable(format!(
                        "{raw} ambiguous in {label} (DST transition)"
                    ))
                })?;
                let utc = local.with_timezone(&Utc);
                // Record the offset actually applied (contract T306); the
                // IANA name itself lives in the row's timezone provenance.
                let off_secs = naive.and_utc().signed_duration_since(utc).num_seconds();
                (utc, Some(format_offset_secs(off_secs)))
            }
            #[cfg(not(feature = "file-ingest"))]
            {
                return Err(TimestampRejection::Unparseable(label.to_string()));
            }
        }
    };

    if utc > naive_upper_limit {
        return Err(TimestampRejection::ImplausibleFuture);
    }
    let floor = chrono::NaiveDate::from_ymd_opt(2000, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    if utc < floor {
        return Err(TimestampRejection::ImplausiblePast);
    }
    Ok((utc, offset_label))
}

/// Render an offset in seconds as `+HH:MM` (or `-HH:MM`).
#[cfg(feature = "file-ingest")]
fn format_offset_secs(secs: i64) -> String {
    let sign = if secs < 0 { '-' } else { '+' };
    let a = secs.abs();
    format!("{sign}{:02}:{:02}", a / 3600, (a % 3600) / 60)
}

/// Parse `+HH:MM`, `-HHMM`, or `+HH` timezone offsets.
fn parse_offset(s: &str) -> Result<FixedOffset, TimestampRejection> {
    let bad = || TimestampRejection::Unparseable(s.to_string());
    let t = s.trim();
    let (sign, rest) = match t.as_bytes().first() {
        Some(b'+') => (1i32, &t[1..]),
        Some(b'-') => (-1i32, &t[1..]),
        _ => return Err(bad()),
    };
    let (h, m) = match rest.find(':') {
        Some(i) => (
            rest[..i].parse::<i32>().map_err(|_| bad())?,
            rest[i + 1..].parse::<i32>().map_err(|_| bad())?,
        ),
        None if rest.len() == 4 => (
            rest[..2].parse::<i32>().map_err(|_| bad())?,
            rest[2..].parse::<i32>().map_err(|_| bad())?,
        ),
        None if rest.len() == 2 => (rest.parse::<i32>().map_err(|_| bad())?, 0),
        _ => return Err(bad()),
    };
    FixedOffset::east_opt(sign * (h * 3600 + m * 60)).ok_or_else(bad)
}

// ── Metric registry (FR-055) ─────────────────────────────────────────────────

/// Canonical target of a metric on `VitalsEntry`. Anything not `Native`
/// becomes a custom metric carrying its definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeField {
    BpSystolic,
    BpDiastolic,
    HeartRate,
    WeightKg,
    GlucoseMgDl,
    TemperatureC,
    Spo2,
    /// `hrv` column means RMSSD — sdnn and friends are NOT comparable.
    HrvRmssdMs,
    SleepQuality,
}

/// Registry lookup: does this (metric, definition, unit) triple have a
/// native home, and what canonical value does it map to? `None` ⇒ custom.
///
/// Unit conversion is explicit and conservative: unknown units or unknown
/// definitions never guess, they fall through to custom.
pub fn resolve_native(
    metric: &str,
    definition: &str,
    unit: &str,
    value: f64,
) -> Option<(NativeField, f64)> {
    let lowered = metric.to_ascii_lowercase();
    let m = lowered.as_str();
    let u = unit.trim().to_ascii_lowercase();
    let d = definition.to_ascii_lowercase();
    match m {
        "bp_systolic" if u == "mmhg" => Some((NativeField::BpSystolic, value)),
        "bp_diastolic" if u == "mmhg" => Some((NativeField::BpDiastolic, value)),
        "heart_rate" if u == "bpm" => Some((NativeField::HeartRate, value)),
        "weight" => match u.as_str() {
            "kg" => Some((NativeField::WeightKg, value)),
            "lb" | "lbs" => Some((NativeField::WeightKg, value * 0.45359237)),
            _ => None,
        },
        "blood_glucose" => match u.as_str() {
            "mg/dl" => Some((NativeField::GlucoseMgDl, value)),
            "mmol/l" => Some((NativeField::GlucoseMgDl, value * 18.0182)),
            _ => None,
        },
        "temperature" => match u.as_str() {
            "c" | "°c" => Some((NativeField::TemperatureC, value)),
            "f" | "°f" => Some((NativeField::TemperatureC, (value - 32.0) * 5.0 / 9.0)),
            _ => None,
        },
        "spo2" if u == "%" => Some((NativeField::Spo2, value)),
        // Definition gate: the hrv column is RMSSD only. sdnn keeps its
        // definition in custom_metrics and is never compared to rmssd.
        "hrv" if d == "rmssd" && u == "ms" => Some((NativeField::HrvRmssdMs, value)),
        "sleep_quality" | "sleep_score" if d == "ordinal_0_3" && (0.0..=3.0).contains(&value) => {
            Some((NativeField::SleepQuality, value))
        }
        _ => None,
    }
}

/// Custom-metric key: metric + definition, so `hrv:sleep-sdnn` never
/// collides with a future `hrv:rmssd` custom entry.
pub fn custom_key(metric: &str, definition: &str) -> String {
    format!("{metric}:{definition}")
}

/// Build the custom_metrics JSON entry for a non-native reading.
pub fn custom_metric_json(reading: &RawReading) -> serde_json::Value {
    serde_json::json!({
        "value": reading.value,
        "unit": reading.unit,
        "definition": reading.metric_definition,
    })
}

// ── Ingest report ────────────────────────────────────────────────────────────

/// Outcome of an ingest batch. Failures are reported per record and never
/// silently dropped (FR-059); quarantined inputs keep their verbatim payload
/// so the caller can retain them for retry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IngestReport {
    /// New readings accepted and stored.
    pub ingested: usize,
    /// Duplicate (`source_id`, `source_record_id`) records — no-op (FR-052).
    pub duplicates_skipped: usize,
    /// Records quarantined, with the offending record id and reason.
    pub quarantined: Vec<(String, String)>,
    /// Readings that landed in `custom_metrics` instead of a native column.
    pub custom_metrics: usize,
    /// `VitalsEntry` rows created or updated by this batch.
    pub vitals_rows: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 18, 12, 0, 0).unwrap()
    }

    #[test]
    fn rfc3339_offset_is_converted_and_recorded() {
        let (utc, off) = normalise_timestamp("2026-11-18T09:30:00+02:00", None, now()).unwrap();
        assert_eq!(utc, Utc.with_ymd_and_hms(2026, 11, 18, 7, 30, 0).unwrap());
        assert_eq!(off.as_deref(), Some("+02:00"));
    }

    #[test]
    fn zulu_records_no_offset() {
        let (utc, off) = normalise_timestamp("2026-11-18T07:30:00Z", None, now()).unwrap();
        assert_eq!(utc, Utc.with_ymd_and_hms(2026, 11, 18, 7, 30, 0).unwrap());
        assert_eq!(off, None);
    }

    #[test]
    fn naive_without_zone_is_refused() {
        assert_eq!(
            normalise_timestamp("2026-11-18 09:30:00", None, now()),
            Err(TimestampRejection::MissingTimezone)
        );
    }

    #[test]
    fn naive_with_zone_is_converted() {
        let (utc, off) = normalise_timestamp("2026-11-18 09:30:00", Some("+02:00"), now()).unwrap();
        assert_eq!(utc, Utc.with_ymd_and_hms(2026, 11, 18, 7, 30, 0).unwrap());
        assert_eq!(off.as_deref(), Some("+02:00"));
    }

    #[test]
    fn future_and_ancient_are_quarantined() {
        assert_eq!(
            normalise_timestamp("2026-11-18T12:10:00Z", None, now()),
            Err(TimestampRejection::ImplausibleFuture)
        );
        assert_eq!(
            normalise_timestamp("1999-12-31T23:59:00Z", None, now()),
            Err(TimestampRejection::ImplausiblePast)
        );
        // 4 minutes ahead is inside the clock-skew tolerance.
        assert!(normalise_timestamp("2026-11-18T12:04:00Z", None, now()).is_ok());
    }

    #[test]
    fn unit_conversions_to_canonical() {
        assert_eq!(
            resolve_native("weight", "body_weight", "lb", 150.0)
                .unwrap()
                .1,
            68.0388555
        );
        assert!(
            (resolve_native("blood_glucose", "fasting", "mmol/L", 5.0)
                .unwrap()
                .1
                - 90.091)
                .abs()
                < 1e-9,
            "mmol/L -> mg/dL uses the 18.0182 molecular weight"
        );
        let (_, c) = resolve_native("temperature", "body", "°F", 98.6).unwrap();
        assert!((c - 37.0).abs() < 1e-9);
    }

    #[test]
    fn sdnn_is_never_mapped_to_the_rmssd_column() {
        assert_eq!(
            resolve_native("hrv", "rmssd", "ms", 42.0).map(|(f, _)| f),
            Some(NativeField::HrvRmssdMs)
        );
        assert_eq!(resolve_native("hrv", "sdnn", "ms", 60.0), None);
    }

    #[test]
    fn unknown_units_fall_to_custom() {
        assert_eq!(resolve_native("weight", "body_weight", "stone", 15.0), None);
        assert_eq!(resolve_native("hrv", "rmssd", "bpm", 42.0), None);
    }
}

/// A retained ingest failure (FR-059): the verbatim payload of a rejected
/// record with its reason, kept until a successful retry clears it. The
/// caller (CLI or UI) surfaces these so nothing received is silently lost.
#[derive(Debug, Clone)]
pub struct IngestFailure {
    pub source_id: String,
    pub source_record_id: String,
    pub reason: String,
    pub payload: Option<String>,
    pub failed_at: String,
}
