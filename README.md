# Biohack Tracker

Local-first, offline-capable biohacking tracker built with Rust + Leptos.

## Tech Stack

- **Frontend**: Leptos 0.7 (Rust → WASM; ~1.3MB raw / ~375KB gzipped — see T073 note in tasks.md)
- **Engine**: Rust 2024 (reuses `biohack` CLI safety protocols)
- **Storage**: localStorage via gloo-storage (local-first, no cloud required; OPFS/SQLite deferred)
- **Build**: `wasm-pack` + `build-web.sh` (includes wasm-opt + PWA asset deployment)

## Prerequisites

- Rust stable toolchain (`rustup`, with `wasm32-unknown-unknown` target)
- `wasm-pack` for building the WASM frontend
- `binaryen` (wasm-opt) — optional, for a smaller bundle
- Python 3 for `server.py` (static SPA server)

## Quick Start

```bash
# Install wasm-pack if needed
cargo install wasm-pack
# wasm-opt (optional, shrinks the bundle):
npm install -g binaryen

# Build the frontend into dist/ (WASM + PWA assets)
bash build-web.sh

# Serve at http://localhost:8082
python server.py

# Run tests
cargo test --workspace          # engine (27 tests)
cd web && wasm-pack test --headless --chrome   # web (16 tests)
```

## Architecture

```
biohack2/
├── engine/          # Rust: safety protocols, catalog, logging logic
│   ├── src/
│   │   ├── lib.rs
│   │   ├── models.rs
│   │   ├── safety.rs
│   │   ├── catalog.rs
│   │   └── db.rs
│   └── tests/
├── web/             # Leptos web frontend
│   ├── src/
│   │   ├── main.rs
│   │   ├── router.rs
│   │   ├── app.rs
│   │   ├── state/
│   │   ├── components/
│   │   ├── pages/
│   │   └── styles/
│   └── public/
└── specs/
    └── 001-biohacker-tracking-platform/
```

## Ingesting wearable exports

The native CLI imports CSV/JSON exports into the SQLite database:

```bash
cargo build -p biohack2-cli
biohack2 --db biohack2.db ingest whoop_export.csv --source whoop --user george
biohack2 failures          # retained bad rows, newest first
```

`--db` also reads `BIOHACK2_DB`; `--source` defaults to the file stem.

**Column contract (CSV long format):** required `timestamp`, `metric`,
`value`; optional `unit`, `definition` (e.g. `score_0_100`), `record_id`,
`timezone`. JSON expects an array of objects with the same field names.

- Timestamps: RFC 3339 (any offset or `Z`), naive + `timezone` column
  (fixed offset `+00:00`/`+0000`, IANA name `Europe/London`, or `Z`/`UTC`),
  or epoch seconds/milliseconds. The stored instant is always UTC; the
  source offset is kept as provenance. DST-ambiguous instants are rejected,
  never guessed.
- Metrics matching known definitions map to native columns; everything else
  lands as custom metrics. Rows whose scale cannot be determined are
  quarantined rather than silently converted.
- Re-ingesting the same file is safe: identical records dedupe on a stable
  id (source `record_id`, else a content hash).
- Nothing is dropped silently. Unparseable rows are retained verbatim
  (`biohack2 failures`) and clear once a re-ingest of the fixed file proves
  that position passes.
- Readings that land go through the existing safety engine: an out-of-range
  value raises the same alerts the app shows (one per protocol per day, so a
  day of continuous readings cannot storm the alert banner).

See `specs/004-wearable-health-ingest/research.md` (§R3, §R4, §R4b) for the
decision record behind this contract.

## Development

The application is local-first. The WASM frontend persists to **localStorage via `gloo-storage`**; SQLite via sqlx is the native/server path (`engine/src/db.rs`, `server/`). OPFS + SQLite-WASM is **not implemented** — see the drift table in `specs/README.md`. No account required, no cloud dependency for v1.

See `specs/README.md` for the full feature map across specs 001 and 003–006.

### Safety Protocols

Three deterministic protocols from the `biohack` CLI:
1. **Stimulant tachycardia**: HR > 100 bpm + stimulant within 4h
2. **Hypertensive urgency**: SBP ≥ 180 or DBP ≥ 120
3. **Serotonin syndrome risk**: Multiple serotonergic agents

### Data Model

See `specs/001-biohacker-tracking-platform/data-model.md` for full schema.

## Testing

```bash
# Engine tests (safety protocols, catalog, DB)
cargo test -p engine

# Full workspace tests
cargo test --workspace

# WASM component tests (requires Chrome + matching chromedriver)
cd web && wasm-pack test --headless --chrome
```

## Validation

Run the quickstart validation scenarios from `specs/001-biohacker-tracking-platform/quickstart.md`:
- VS-001: Log consumption (< 15s)
- VS-002: History view performance (< 2s)
- VS-003: Drug interaction check (< 3s)
- VS-004: Vitals alerting (< 5s)
- VS-005: Insights generation
- VS-006: Stack logging (< 30s)
- VS-007: Offline functionality
- VS-008: Data export (< 10s)
- VS-009: Safety protocols from biohack CLI
- VS-010: Type safety validation

## License

MIT OR Apache-2.0
