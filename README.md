# Biohack Tracker

Local-first, offline-capable biohacking tracker built with Rust + Leptos.

## Tech Stack

- **Frontend**: Leptos 0.7 (Rust → WASM, ~60KB bundle)
- **Engine**: Rust 2024 (reuses `biohack` CLI safety protocols)
- **Storage**: SQLite via WASM + OPFS (local-first, no cloud required)
- **Build**: `cargo-leptos` for WASM compilation + HMR

## Prerequisites

- Rust 2024 toolchain
- `cargo-leptos`: `cargo install cargo-leptos`
- SQLite3 (for build-time sqlx checks)
- Browser with OPFS support (Chrome 109+, Edge 109+, Firefox 118+)

## Quick Start

```bash
# Install cargo-leptos if needed
cargo install cargo-leptos

# Install SQLite dev headers (if not present)
# macOS: brew install sqlite
# Ubuntu: sudo apt install libsqlite3-dev
# Windows: Download SQLite DLL and set SQLX_OFFLINE=true

# Run development server with HMR
cargo leptos watch

# Build for production
cargo leptos build --release

# Run tests
cargo test --release --workspace
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

## Development

The application is local-first — all data is stored in SQLite via WASM in the browser. No account required, no cloud dependency for v1.

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
cargo test --release -p engine

# Full workspace tests
cargo test --release --workspace

# WASM component tests
cargo test --release -p biohack2-web --target wasm32-unknown-unknown
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
