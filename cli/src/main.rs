//! biohack2 command line (spec-004 T315).
//!
//! `biohack2 ingest <file>` — read a wearable export (generic CSV/JSON column
//! contract, see `engine::ingest::generic`) into the local database, reporting
//! counts, quarantine, and per-record failures. Nothing is dropped silently:
//! every record that did not become a row is retained in `ingest_failures`
//! (FR-059) and visible again via `biohack2 failures`.

use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use engine::db;
use engine::ingest::{
    IngestSource, SourceKind,
    generic::{parse_generic_csv, parse_generic_json},
};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "biohack2", version, about = "Local-first biohacker tracking")]
struct Cli {
    /// Database file (env: BIOHACK2_DB)
    #[arg(
        long,
        global = true,
        env = "BIOHACK2_DB",
        default_value = "biohack2.db"
    )]
    db: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Ingest a wearable export file (generic CSV/JSON contract)
    Ingest {
        /// Export file (.csv / .json; contents sniffed as a fallback)
        file: PathBuf,
        /// Source id recorded on every reading (default: file stem)
        #[arg(long)]
        source: Option<String>,
        /// User the readings belong to
        #[arg(long, default_value = "default")]
        user: String,
    },
    /// List retained ingest failures — newest first (FR-059)
    Failures,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Ingest { file, source, user } => ingest(&cli.db, &file, source, &user).await,
        Command::Failures => failures(&cli.db).await,
    }
}

async fn connect(db_path: &Path) -> Result<db::DbPool> {
    if let Some(parent) = db_path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating db directory {}", parent.display()))?;
    }
    let url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());
    let pool = db::DbPool::connect(&url).await?;
    db::migrate(&pool).await?;
    Ok(pool)
}

async fn ingest(db_path: &Path, file: &Path, source: Option<String>, user: &str) -> Result<()> {
    let text =
        std::fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
    if text.trim().is_empty() {
        anyhow::bail!("{} is empty — nothing to ingest", file.display());
    }

    let source_id = source.unwrap_or_else(|| {
        file.file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "file".to_string())
    });

    // Sniff: extension first, then content (a JSON export named .txt still works).
    let looks_json = text.trim_start().starts_with('{') || text.trim_start().starts_with('[');
    let outcome = match (file.extension().and_then(|e| e.to_str()), looks_json) {
        (_, true) => parse_generic_json(&source_id, &text, Utc::now()),
        (Some("csv" | "CSV"), false) | (None, false) => {
            parse_generic_csv(&source_id, &text, Utc::now())
        }
        (Some(ext), false) => {
            // Unknown extension but not JSON: still try CSV — the generic
            // contract is CSV; erroring on the extension would just be pedantry.
            eprintln!("note: .{ext} not recognised, attempting CSV parse");
            parse_generic_csv(&source_id, &text, Utc::now())
        }
    };

    let pool = connect(db_path).await?;

    let now = Utc::now();
    db::upsert_ingest_sources(
        &pool,
        &[IngestSource {
            id: source_id.clone(),
            kind: SourceKind::File,
            device_label: file.file_name().map(|n| n.to_string_lossy().into_owned()),
            platform: None,
            first_seen: now,
            last_seen: now,
        }],
    )
    .await?;

    // Parser-level failures never reach ingest_readings — retain them here so
    // "no silent drops" holds for the whole file path (FR-059).
    for f in &outcome.failures {
        db::retain_ingest_failure(
            &pool,
            &source_id,
            &format!("record-{}", f.record),
            &f.reason,
            &f.raw,
        )
        .await?;
    }

    let report = db::ingest_readings(&pool, user, &outcome.readings).await?;

    // Clear stale ordinal-keyed failures this run re-examined and passed:
    // `record-N` failed before, position N was read again now, and it did
    // not fail again. Positions beyond this file or still failing are kept.
    let still_failing: std::collections::BTreeSet<usize> =
        outcome.failures.iter().map(|f| f.record).collect();
    let mut cleared = 0usize;
    for row in db::list_ingest_failures(&pool).await? {
        if row.source_id != source_id {
            continue;
        }
        let ordinal = row
            .source_record_id
            .strip_prefix("record-")
            .and_then(|n| n.parse::<usize>().ok());
        if let Some(n) = ordinal
            && n <= outcome.records_seen
            && !still_failing.contains(&n)
        {
            db::delete_ingest_failure(&pool, &source_id, &row.source_record_id).await?;
            cleared += 1;
        }
    }

    println!(
        "{}: {} readings parsed, {} parse failures",
        file.display(),
        outcome.readings.len(),
        outcome.failures.len()
    );
    println!(
        "ingested {} · duplicates skipped {} · quarantined {} · custom metrics {} · vitals rows {}",
        report.ingested,
        report.duplicates_skipped,
        report.quarantined.len(),
        report.custom_metrics,
        report.vitals_rows,
    );
    if report.safety_alerts > 0 {
        println!(
            "safety alerts raised {} — out-of-range readings reached the alert banner (FR-057)",
            report.safety_alerts
        );
    }
    if cleared > 0 {
        println!("cleared {cleared} previously retained failure(s) — fixed on re-ingest");
    }
    for (record, reason) in &report.quarantined {
        println!("  quarantined {record}: {reason}");
    }
    for f in &outcome.failures {
        println!(
            "  parse failure line {}: {} — {}",
            f.record,
            f.reason,
            clip(&f.raw)
        );
    }
    Ok(())
}

async fn failures(db_path: &Path) -> Result<()> {
    let pool = connect(db_path).await?;
    let rows = db::list_ingest_failures(&pool).await?;
    if rows.is_empty() {
        println!("no retained failures — nothing was dropped silently");
        return Ok(());
    }
    for r in &rows {
        println!(
            "{}  {}/{}  {}  {}",
            r.failed_at,
            r.source_id,
            r.source_record_id,
            r.reason,
            r.payload.as_deref().map(clip).unwrap_or_default(),
        );
    }
    println!(
        "{} retained failure(s) — fix the source file and re-ingest to clear",
        rows.len()
    );
    Ok(())
}

/// One-line, length-clipped preview for terminal output.
fn clip(s: &str) -> String {
    const MAX: usize = 120;
    let one_line: String = s.chars().take_while(|c| *c != '\n').collect();
    if one_line.chars().count() > MAX {
        let cut: String = one_line.chars().take(MAX - 1).collect();
        format!("{cut}…")
    } else {
        one_line
    }
}
