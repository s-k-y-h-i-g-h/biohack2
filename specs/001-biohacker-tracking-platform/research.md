# Research Report: Biohacker Tracking Platform (Leptos Pivot)

**Date**: 2026-09-04
**Feature**: 001-biohacker-tracking-platform

## Unknowns Resolved

### 1. Frontend Framework: Leptos vs SolidJS

**Decision**: Leptos (Rust + WASM).

**Rationale**:
- Reuses `biohack` Rust engine directly — same types, no bridge code, no drift
- End-to-end type safety: DB schema change errors UI at compile time
- ~60KB total WASM bundle vs ~200KB TypeScript bundle
- Fine-grained reactivity (signals) — same performance characteristics as SolidJS
- Dioxus (Leptos cousin) compiles to iOS/Android natively — mobile path is trivial
- `cargo-leptos watch` gives CSS HMR; WASM rebuilds only on file save

**Alternatives considered**:
- **TypeScript + SolidJS**: Requires separate type definitions that drift from Rust engine; bridge layer for safety checks adds maintenance burden
- **TypeScript + React**: Larger bundle, no engine reuse
- **SvelteKit**: No Rust integration; separate type maintenance

### 2. Local Storage: SQLite WASM via OPFS

**Decision**: SQLite via `sqlite-wasm` with Origin Private File System (OPFS).

**Rationale**:
- Proven pattern in 2026 for local-first web apps
- ACID transactions, full SQL support, indexes — no ORM to learn
- OPFS provides persistent, large-capacity storage in the browser without user permission prompts
- Single file simplifies backup/export (vs IndexedDB's multi-store model)
- Leptos can query SQLite directly; no serialization bridge needed
- PowerSync free tier available for v2 cloud sync (SQLite ↔ PostgreSQL automatic sync)

**Alternatives considered**:
- **IndexedDB**: Native browser API, but no SQL; custom ORM needed; harder to query complex relationships
- **WASM SQLite without OPFS**: IndexedDB-backed SQLite (`sql.js`) — works but OPFS is more robust for persistence
- **Dexie.js (IndexedDB wrapper)**: Simpler API but still no native SQL

### 3. Build Toolchain: cargo-leptos

**Decision**: `cargo-leptos` for WASM compilation, dev server, and HMR.

**Rationale**:
- Standard toolchain for Leptos apps
- Fast WASM compilation (leveraging Rust's incremental builds)
- CSS HMR without full rebuild
- SSR support for SEO/prod builds
- Integrates with `cargo test --release` for Rust engine tests

### 4. Safety Protocol Reuse from biohack CLI

**Decision**: Direct reuse of `biohack` engine via workspace dependency.

**Rationale**:
- `biohack` already has 3 tested safety protocols (stimulant tachycardia, hypertensive urgency, serotonin syndrome)
- Adding `biohack` as a workspace member in `Cargo.toml` gives zero-cost reuse
- Same `CatalogItem` types flow from engine to DB to UI
- Tests in `biohack/tests/` run against the same engine code

**Alternatives considered**:
- **Reimplement in web crate**: Duplicates logic; risk of divergence from tested Rust code
- **Call `biohack` CLI**: Requires binary installation; breaks "no install" PWA goal

### 5. Cloud Sync Strategy (v2)

**Decision**: PowerSync free tier for in-browser SQLite ↔ PostgreSQL sync.

**Rationale**:
- Automatic conflict resolution
- Works with existing SQLite schema
- Free tier sufficient for single-user local-first apps
- Client-side encryption before upload (meets FR-019/020/021)

**Alternatives considered**:
- **Custom sync service**: More control but more maintenance
- **Firebase**: Vendor lock-in; overkill for single-user

## External Data Sources

### Drug Interaction Database

**Decision**: Embed known interactions in Rust engine (compile-time checked).

**Rationale**:
- Small dataset (interactions between 27 substances + common drugs)
- Compile-time checked means no runtime parsing errors
- Extensible: new interactions added as code changes, not data files
- Disclaimer: "interaction data may be incomplete; consult a healthcare professional"

**Known gaps**:
- Natural product interactions less documented than pharmaceuticals
- Some "folk knowledge" interactions not in academic literature
- Examine.com API (800+ supplements) available for future expansion

### Clinical Reference Ranges

**Decision**: Hardcoded thresholds in `engine/src/safety.rs` (same as `biohack` CLI).

**Rationale**:
- Well-established clinical thresholds (AHA, WHO)
- Same values as existing tested biohack CLI
- User-configurable baselines deferred to v2

**Key thresholds**:
- Tachycardia: HR > 100 bpm (adult, resting)
- Hypertensive urgency: SBP ≥ 180 or DBP ≥ 120
- Hypotension: SBP < 90 or DBP < 60
- SpO2 < 95% (respiratory concern)
- Temperature > 38°C or < 35°C (fever/hypothermia)
