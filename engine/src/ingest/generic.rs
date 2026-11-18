//! Generic CSV/JSON parsing with the documented column contract (T313).
//!
//! Contract of record: `specs/004-wearable-health-ingest/research.md` §R4.
//! CSV header row required (names case-insensitive); JSON accepts an array of
//! objects or `{"readings": [...]}`. Required: `timestamp`, `metric`, `value`.
//! Optional: `unit`, `timezone`, `definition`, `record_id`. Extra columns are
//! ignored for mapping but live on verbatim in `payload` (FR-059 audit trail).
//!
//! Discipline inherited from the rest of `ingest`:
//! - naive timestamps without an explicit `timezone` are **refused**, never
//!   guessed (the 2026-09-09 precedent);
//! - `definition` is never defaulted to a "likely" value — absent means
//!   `"unspecified"`, which routes to `custom_metrics`, never a native column
//!   (FR-055);
//! - a bad row never aborts the batch: it becomes a [`ParseFailure`] carrying
//!   the raw input so the caller can retain it for retry.

use chrono::{DateTime, Utc};

use super::{RawReading, normalise_timestamp};

/// One input record the parser could not turn into a reading. `raw` keeps the
/// verbatim source text (CSV line or JSON object) for retention + retry.
#[derive(Debug, Clone)]
pub struct ParseFailure {
    /// 1-based record ordinal (CSV: line number including the header).
    pub record: usize,
    pub reason: String,
    pub raw: String,
}

/// What a file parse produced: readings to ingest, failures to report.
/// Nothing in, nothing silently out.
#[derive(Debug, Default)]
pub struct ParseOutcome {
    pub readings: Vec<RawReading>,
    pub failures: Vec<ParseFailure>,
    /// Highest record ordinal this run actually examined (CSV: line number
    /// incl. header; JSON: 1-based item index). Lets callers prove that a
    /// previously-failed ordinal has been re-checked and now passes.
    pub records_seen: usize,
}

impl ParseOutcome {
    fn fail(&mut self, record: usize, reason: impl Into<String>, raw: impl Into<String>) {
        self.failures.push(ParseFailure {
            record,
            reason: reason.into(),
            raw: raw.into(),
        });
    }
}

/// Non-cryptographic FNV-1a/64 — a dedupe key for records the source never
/// gave an id (research.md §R3), not a security boundary.
fn fnv1a64(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x1000_0000_01b3);
    }
    h
}

/// Synthesised idempotency key for id-less records: stable for identical
/// content across re-ingests (SC-051), new when the content is corrected
/// (FR-059 retry semantics).
pub fn synth_record_id(raw: &str) -> String {
    format!("fnv1a64-{:016x}", fnv1a64(raw.trim()))
}

/// Timestamp text → UTC via the shared normaliser, with one addition for
/// file exports: a bare integer is interpreted as Unix epoch **seconds**.
/// Digit strings ≥ 1e12 that the plausibility filter rejects as seconds are
/// re-read as milliseconds — a deterministic disambiguation (as seconds
/// those values land in the far future), not a guess.
fn parse_timestamp(
    raw: &str,
    timezone: Option<&str>,
    now: DateTime<Utc>,
) -> Result<(DateTime<Utc>, Option<String>), String> {
    use super::TimestampRejection;
    let t = raw.trim();
    if !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit()) {
        let digits: u64 = t.parse().map_err(|_| format!("epoch overflow: {t}"))?;
        // Plausibility decides the unit *numerically* before conversion:
        // seconds must land in [2000-01-01, now + skew]. A ≥ 1e12 value can
        // never be valid seconds (that is year ~33000), so re-reading it as
        // milliseconds is a deterministic disambiguation, not a guess.
        const MIN_SECS: u64 = 946_684_800; // 2000-01-01T00:00:00Z
        let plausible_secs = |s: u64| s >= MIN_SECS && (s as i64) <= now.timestamp() + 300;
        let div = if plausible_secs(digits) {
            1
        } else if digits >= 1_000_000_000_000 && plausible_secs(digits / 1000) {
            1000
        } else {
            1 // let the normaliser produce the canonical rejection
        };
        let err = |e: TimestampRejection| format!("{e:?}");
        return match DateTime::from_timestamp((digits / div) as i64, 0)
            .map(|utc| utc.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
            .map(|s| normalise_timestamp(&s, None, now))
        {
            Some(Ok(v)) => Ok(v),
            Some(Err(e)) => Err(err(e)),
            None => Err(format!("epoch timestamp out of range: {t}")),
        };
    }
    normalise_timestamp(t, timezone, now).map_err(|e| format!("{e:?}"))
}

/// Shared field → `RawReading` mapping for both CSV and JSON paths.
/// `raw` is the verbatim input text used for id synthesis + payload.
#[allow(clippy::too_many_arguments)]
fn build_reading(
    source_id: &str,
    now: DateTime<Utc>,
    record_id: Option<String>,
    timestamp: &str,
    timezone: Option<&str>,
    metric: &str,
    value_raw: &str,
    unit: Option<&str>,
    definition: Option<&str>,
    raw: String,
) -> Result<RawReading, String> {
    let metric = metric.trim().to_ascii_lowercase();
    if metric.is_empty() {
        return Err("missing metric".to_string());
    }
    let value: f64 = value_raw
        .trim()
        .parse()
        .map_err(|_| format!("value is not a number: {:?}", value_raw.trim()))?;
    if !value.is_finite() {
        return Err(format!("value is not finite: {value}"));
    }
    let (observed_at, source_timezone) = parse_timestamp(timestamp, timezone, now)?;
    Ok(RawReading {
        source_id: source_id.to_string(),
        source_record_id: match record_id.map(|s| s.trim().to_string()) {
            Some(s) if !s.is_empty() => s,
            _ => synth_record_id(&raw),
        },
        metric,
        // Never guessed: absent definition stays "unspecified" and the
        // registry routes it to custom metrics (FR-055).
        metric_definition: definition
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "unspecified".to_string()),
        value,
        unit: unit.unwrap_or("").trim().to_string(),
        observed_at,
        source_timezone,
        ingested_at: now,
        payload: Some(raw),
    })
}

/// Parse a generic CSV file per the column contract (research.md §R4).
/// `source_id` labels where the file came from (file path or device label) —
/// attribution lives at the boundary, not in the file (FR-051).
pub fn parse_generic_csv(source_id: &str, text: &str, now: DateTime<Utc>) -> ParseOutcome {
    let mut out = ParseOutcome::default();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(text.as_bytes());

    let headers: Vec<String> = match rdr.headers() {
        Ok(h) => h.iter().map(|s| s.trim().to_ascii_lowercase()).collect(),
        Err(e) => {
            out.fail(0, format!("unreadable header row: {e}"), String::new());
            return out;
        }
    };
    let col = |name: &str| headers.iter().position(|h| h == name);
    let (i_ts, i_metric, i_value) = (col("timestamp"), col("metric"), col("value"));
    let (Some(i_ts), Some(i_metric), Some(i_value)) = (i_ts, i_metric, i_value) else {
        out.fail(
            0,
            "header lacks a required column (timestamp, metric, value)",
            headers.join(","),
        );
        return out;
    };
    let (i_unit, i_tz, i_def, i_id) = (
        col("unit"),
        col("timezone"),
        col("definition"),
        col("record_id"),
    );

    for (idx, rec) in rdr.records().enumerate() {
        let record = idx + 2; // header is line 1
        out.records_seen = out.records_seen.max(record);
        let rec = match rec {
            Ok(r) => r,
            Err(e) => {
                out.fail(record, format!("csv read error: {e}"), String::new());
                continue;
            }
        };
        let get = |i: Option<usize>| {
            i.and_then(|i| rec.get(i))
                .map(str::to_string)
                .filter(|s| !s.trim().is_empty())
        };
        let raw = rec.iter().collect::<Vec<_>>().join(",");
        let (Some(timestamp), Some(metric), Some(value)) =
            (get(Some(i_ts)), get(Some(i_metric)), get(Some(i_value)))
        else {
            out.fail(
                record,
                "row lacks a required field (timestamp, metric, value)",
                raw,
            );
            continue;
        };
        match build_reading(
            source_id,
            now,
            get(i_id),
            &timestamp,
            get(i_tz).as_deref(),
            &metric,
            &value,
            get(i_unit).as_deref(),
            get(i_def).as_deref(),
            raw,
        ) {
            Ok(r) => out.readings.push(r),
            Err(reason) => out.fail(record, reason, rec.iter().collect::<Vec<_>>().join(",")),
        }
    }
    out
}

/// Parse a generic JSON file per the column contract (research.md §R4):
/// an array of objects, or an object with a `readings` array. Numbers may be
/// JSON numbers or numeric strings.
pub fn parse_generic_json(source_id: &str, text: &str, now: DateTime<Utc>) -> ParseOutcome {
    let mut out = ParseOutcome::default();
    let root: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => {
            out.fail(0, format!("invalid json: {e}"), String::new());
            return out;
        }
    };
    let items: Vec<serde_json::Value> = match &root {
        serde_json::Value::Array(a) => a.clone(),
        serde_json::Value::Object(o) => match o.get("readings").and_then(|v| v.as_array()) {
            Some(a) => a.clone(),
            None => {
                out.fail(
                    0,
                    "expected an array or {\"readings\": [...]}",
                    String::new(),
                );
                return out;
            }
        },
        _ => {
            out.fail(
                0,
                "expected an array or {\"readings\": [...]}",
                String::new(),
            );
            return out;
        }
    };

    for (idx, item) in items.iter().enumerate() {
        let record = idx + 1;
        out.records_seen = out.records_seen.max(record);
        let raw = item.to_string();
        let get = |k: &str| {
            item.get(k).and_then(|v| match v {
                serde_json::Value::String(s) => {
                    (!s.trim().is_empty()).then(|| s.trim().to_string())
                }
                serde_json::Value::Number(n) => Some(n.to_string()),
                _ => None,
            })
        };
        let (Some(timestamp), Some(metric), Some(value)) =
            (get("timestamp"), get("metric"), get("value"))
        else {
            out.fail(
                record,
                "record lacks a required field (timestamp, metric, value)",
                raw,
            );
            continue;
        };
        match build_reading(
            source_id,
            now,
            get("record_id"),
            &timestamp,
            get("timezone").as_deref(),
            &metric,
            &value,
            get("unit").as_deref(),
            get("definition").as_deref(),
            raw,
        ) {
            Ok(r) => out.readings.push(r),
            Err(reason) => out.fail(record, reason, item.to_string()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-11-18T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn csv_happy_path_and_id_synthesis() {
        let csv = "record_id,timestamp,metric,value,unit\n\
                   a1,2026-11-18T09:30:00Z,heart_rate,61,bpm\n\
                   ,2026-11-18T09:31:00Z,spo2,97,%\n";
        let out = parse_generic_csv("test.csv", csv, now());
        assert_eq!(out.readings.len(), 2, "{:?}", out.failures);
        assert_eq!(out.readings[0].source_record_id, "a1");
        assert!(out.readings[1].source_record_id.starts_with("fnv1a64-"));
        assert_eq!(out.readings[0].source_id, "test.csv");
        assert_eq!(out.readings[0].metric_definition, "unspecified");
        // Stable across parses: same content ⇒ same id (SC-051).
        let again = parse_generic_csv("test.csv", csv, now());
        assert_eq!(
            out.readings[1].source_record_id,
            again.readings[1].source_record_id
        );
    }

    #[test]
    fn csv_bad_rows_are_failures_not_losses() {
        let csv = "timestamp,metric,value\n\
                   not-a-date,heart_rate,61\n\
                   2026-11-18T09:30:00Z,,61\n\
                   2026-11-18T09:30:00Z,heart_rate,lol\n\
                   2026-11-18T09:32:00Z,heart_rate,62,bpm-ignored-extra\n";
        let out = parse_generic_csv("t.csv", csv, now());
        assert_eq!(out.readings.len(), 1);
        assert_eq!(out.failures.len(), 3);
        // Verbatim input retained for retry (FR-059).
        assert!(out.failures[0].raw.contains("not-a-date"));
        assert!(out.failures[2].reason.contains("not a number"));
    }

    #[test]
    fn csv_naive_timestamp_requires_timezone_column() {
        // The 2026-09-09 bug class: a naive local timestamp with a declared
        // offset must land at the right UTC instant.
        let csv = "timestamp,metric,value,unit,timezone\n\
                   2026-11-18 09:30:00,heart_rate,61,bpm,+02:00\n\
                   2026-11-18 09:30:00,heart_rate,62,bpm,\n";
        let out = parse_generic_csv("t.csv", csv, now());
        assert_eq!(out.readings.len(), 1);
        assert_eq!(
            out.readings[0]
                .observed_at
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "2026-11-18T07:30:00Z"
        );
        assert_eq!(out.readings[0].source_timezone.as_deref(), Some("+02:00"));
        assert_eq!(out.failures.len(), 1);
        assert!(
            out.failures[0].reason.contains("MissingTimezone"),
            "{:?}",
            out.failures[0].reason
        );
    }

    #[test]
    fn csv_iana_timezone_names_resolve() {
        // Real wearable exports ship IANA zone names, not numeric offsets.
        // Nov 2026: London is GMT (+00:00), New York is EST (-05:00).
        let csv = "timestamp,metric,value,unit,timezone\n\
                   2026-11-17 07:30:00,hrv,42,ms,Europe/London\n\
                   2026-11-17 07:30:00,heart_rate,58,bpm,America/New_York\n\
                   2026-11-17 07:30:00,heart_rate,59,bpm,Mars/Olympus_Mons\n";
        let out = parse_generic_csv("t.iana", csv, now());
        assert_eq!(out.readings.len(), 2, "{:?}", out.failures);
        let utc = |i: usize| {
            out.readings[i]
                .observed_at
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        };
        assert_eq!(utc(0), "2026-11-17T07:30:00Z");
        assert_eq!(out.readings[0].source_timezone.as_deref(), Some("+00:00"));
        assert_eq!(utc(1), "2026-11-17T12:30:00Z");
        assert_eq!(out.readings[1].source_timezone.as_deref(), Some("-05:00"));
        // Unknown zone: refused with the zone as the unparseable value.
        assert_eq!(out.failures.len(), 1);
        assert!(out.failures[0].reason.contains("Mars/Olympus_Mons"));
    }

    #[test]
    fn csv_utc_markers_and_fractional_seconds() {
        // `Z`/`UTC` in the timezone column are explicit-UTC markers (recorded
        // with offset None); fractional seconds ride on naive timestamps.
        let csv = "timestamp,metric,value,unit,timezone\n\
                   2026-11-17T07:30:00,hrv,42,ms,Z\n\
                   2026-11-17 07:30:00,heart_rate,58,bpm,UTC\n\
                   2026-11-17T07:30:00.4821,respiratory_rate,14,bpm,Europe/London\n";
        let out = parse_generic_csv("t.utc", csv, now());
        assert_eq!(out.readings.len(), 3, "{:?}", out.failures);
        for r in &out.readings {
            assert_eq!(
                r.observed_at
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                "2026-11-17T07:30:00Z"
            );
        }
        assert_eq!(out.readings[0].source_timezone, None);
        assert_eq!(out.readings[1].source_timezone, None);
        assert_eq!(out.readings[2].source_timezone.as_deref(), Some("+00:00"));
        // Fractional part preserved, not truncated away.
        assert_eq!(
            out.readings[2]
                .observed_at
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            "2026-11-17T07:30:00.482Z"
        );
    }

    #[test]
    fn csv_epoch_seconds_and_millis() {
        let csv = "timestamp,metric,value,unit\n\
                   1763466600,heart_rate,61,bpm\n\
                   1763466600000,heart_rate,62,bpm\n";
        let out = parse_generic_csv("t.csv", csv, now());
        assert_eq!(out.readings.len(), 2, "{:?}", out.failures);
        // Same instant expressed in s and ms.
        assert_eq!(out.readings[0].observed_at, out.readings[1].observed_at);
    }

    #[test]
    fn definition_never_guessed() {
        // HRV without a definition must not become RMSSD — it carries
        // "unspecified" so the registry routes it away from the native column.
        let csv = "timestamp,metric,value,unit,definition\n\
                   2026-11-18T09:30:00Z,hrv,55,ms,\n\
                   2026-11-18T09:31:00Z,hrv,56,ms,rmssd\n";
        let out = parse_generic_csv("t.csv", csv, now());
        assert_eq!(out.readings[0].metric_definition, "unspecified");
        assert_eq!(out.readings[1].metric_definition, "rmssd");
    }

    #[test]
    fn json_array_and_wrapped_object() {
        let arr = r#"[{"record_id":"j1","timestamp":"2026-11-18T09:30:00Z","metric":"Heart_Rate","value":61.5,"unit":"bpm"}]"#;
        let wrapped =
            r#"{"readings":[{"timestamp":1763466600,"metric":"spo2","value":"97","unit":"%"}]}"#;
        let a = parse_generic_json("fit.json", arr, now());
        let w = parse_generic_json("fit.json", wrapped, now());
        assert_eq!(a.readings.len(), 1, "{:?}", a.failures);
        assert_eq!(a.readings[0].metric, "heart_rate"); // normalised case
        assert_eq!(a.readings[0].value, 61.5);
        assert_eq!(w.readings.len(), 1, "{:?}", w.failures);
        assert!(w.readings[0].source_record_id.starts_with("fnv1a64-"));
    }

    #[test]
    fn json_bad_records_reported_per_item() {
        let json = r#"[{"timestamp":"2026-11-18T09:30:00Z","metric":"heart_rate","value":61},
                       {"metric":"heart_rate","value":62},
                       {"timestamp":"2026-11-18T09:31:00Z","metric":"heart_rate","value":"nan-text"}]"#;
        let out = parse_generic_json("t.json", json, now());
        assert_eq!(out.readings.len(), 1);
        assert_eq!(out.failures.len(), 2);
        assert_eq!(out.failures[0].record, 2);
        assert!(out.failures[1].reason.contains("not a number"));
    }
}
