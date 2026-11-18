# Spec 004 — Research & Decisions

Wearable health ingest. One section per decision, each with its verification
basis. Gates in `tasks.md` reference this file; nothing here is a placeholder.

## R1 — Phase 2 route basis (gate FR-060) — 2026-11-18

**Decision**: Phase 2 touches no vendor service. The only route is the user
manually exporting their own data with the vendor's own export feature and
handing the file to `biohack2 ingest`.

| Vendor | Official user export | How biohack2 receives it |
|---|---|---|
| Garmin (Connect) | per-activity FIT/GPX download; account data request (ZIP) | file on disk |
| Withings | account data export (CSV ZIP) | file on disk |
| Oura | CSV/JSON daily metrics export from account settings | file on disk |
| Whoop | CSV/JSON account data export | file on disk |
| Apple Health | manual export → ZIP with XML | out of scope v1 (needs XML mapping, T314 territory) |
| Fitbit | user data export (ZIP of CSVs) | file on disk |

No scraping, no credentials, no programmatic vendor access, no reverse-
engineered companion-app protocols in Phase 2. The data subject initiates
everything, so there is no vendor-terms exposure to review — this document *is*
the FR-060 record for file ingest. Phases 5–6 (companion push, vendor APIs)
each need their own documented basis before their code.

## R2 — Parser library decision (T310), licences verified — 2026-11-18

Verified live against crates.io on 2026-11-18:

| Crate | Version | Licence | Status | Decision |
|---|---|---|---|---|
| `fitparser` | 0.12.0 | **MIT** | updated 2026-11-03, ~107k downloads, edition 2024 | **Adopt** for T312 (FIT → RawReading). Native-only dependency; FIT never needs to run in WASM. |
| `csv` | 1.4.0 | **Unlicense OR MIT** | maintained | **Adopt** for T313 generic CSV parsing. Pure Rust; parsing from in-memory `&[u8]`/`&str` is WASM-safe (no fs in that path). |
| — JSON | — | — | `serde_json` already in engine | Use existing dependency. |

MIT is compatible with this project's licence; dual Unlicense/MIT is public-
domain-equivalent. No copyleft enters the tree.

## R3 — `record_id` synthesis for files with no id column — 2026-11-18

Idempotency (FR-052) needs a stable per-record key. Exports often have none.

**Decision**: if the `record_id` column/field is absent, synthesise
`fnv1a64-<16 hex>` over the trimmed raw record text (the verbatim CSV line or
the JSON object's canonical form).

- **Why FNV-1a/64, not a crypto hash**: this is a dedupe key, not a security
  boundary; FNV is dependency-free and WASM-trivial. Birthday-collision risk
  is negligible at health-record volumes (~4·10⁹ records for 50%).
- **Why content-based, not row-index**: re-ingesting the same export must be a
  no-op (SC-051) even if the user edited unrelated rows above — but note the
  flip side: a *corrected* value produces a new id and re-ingests, which is
  the desired retry semantics (FR-059).
- Duplicate *content* in one file collapses by the (`source_id`,
  `source_record_id`) PK — acceptable: an identical measurement with no id
  carries no new information.

## R4 — Generic column contract (T313) — 2026-11-18

CSV header row required (names case-insensitive); JSON accepts an array of
objects or `{"readings": [...]}`. Extra columns are ignored for mapping but
kept verbatim in `payload`.

| Column | Required | Meaning |
|---|---|---|
| `timestamp` | yes | RFC 3339 **with explicit offset/Z**, or naive ISO **only** with `timezone` supplied. Naive-without-tz is refused (2026-09-09 precedent). |
| `metric` | yes | canonical lower-case name (`heart_rate`, `hrv`, `spo2`, …) |
| `value` | yes | number (JSON numeric or numeric string; dot decimal) |
| `unit` | for native mapping | e.g. `bpm`, `%`, `ms`; missing/unknown ⇒ routes to `custom_metrics`, never guessed |
| `timezone` | for naive timestamps | explicit offset (`+02:00`, `Z`) |
| `definition` | for gated metrics | e.g. `rmssd` vs `sdnn`; absent ⇒ `"unspecified"` ⇒ never lands on a native column |
| `record_id` | no | stable vendor id; absent ⇒ synthesised per R3 |

Design rule inherited from the registry: **definitions are never defaulted to
a "likely" value.** An HRV export without a `definition` column lands in
custom metrics flagged `unspecified` rather than silently polluting the RMSSD
column (FR-055).

## R4b — CLI ingest, IANA zones, failure retention (T315) — 2026-11-18

`cli/src/main.rs` gained `biohack2 ingest <file.csv|.json> --source <id> [--user]`.
Engine features `csv` and `chrono-tz` are both behind `file-ingest`, so the
WASM bundle stays lean (`cargo tree -p web --target wasm32-unknown-unknown`
contains neither).

Smoke-driven decisions:
- **IANA zone names** (`Europe/London`) are resolved via `chrono-tz` when the
  `timezone` column carries a name instead of an offset — real exports do
  this. The offset *applied* is recorded per the T306 contract; DST-ambiguous
  instants are rejected and retained, never guessed.
- **Retained failures** (`ingest_failures`, FR-059) clear in two ways:
  content-keyed rows clear when the same record ingests successfully;
  ordinal-keyed rows (`record-N` from the parser) clear only when a later
  ingest of the same source re-examined position N and it passed
  (`ParseOutcome::records_seen` proves coverage). Positions beyond a shorter
  re-export stay retained. Verified end-to-end: keep / keep-dedupe / clear /
  shorter-file-keeps.

## R5 — WASM/IndexedDB persistence path (T304 remainder) — open


Investigation pending: web crate currently persists via server SQLite over the
sync API; a browser-local ingest path would need the ledger in IndexedDB with
the same PK semantics. Deferred deliberately — file ingest ships through the
native CLI/agent first (Phases 2–3 are native), so the WASM path has no
consumer until companion push (Phase 5) needs browser-resident staging.
