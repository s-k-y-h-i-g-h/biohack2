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

## Ecosystem Survey — 2026-10-11

Swept ~40 queries against the crates.io API plus GitHub search, to decide what biohack2 should adopt, write itself, and skip. Raw data: `crates_survey.json` (scratch). **Licences below are NOT all verified — adoption is gated on verification, recorded per task (003 T209, 004 T310, 005 T490, 006 T610).**

### Adopt (real, maintained, fills a real gap)

| Crate | v | Downloads | Updated | Licence | For |
|---|---|---|---|---|---|
| `ratatui` | 0.30.2 | 58.7M | 2026-06-19 | MIT | Terminal UI (spec 005) |
| `ratatui-widgets` | 0.3.2 | 23.2M | 2026-06-19 | MIT | `Block`/`Gauge`/`Sparkline`/`Chart`/`Table` — btop's UI primitives already exist |
| `ratzilla` | 0.3.1 | 507k | 2026-08-31 | **MIT OR Apache-2.0** (verified; matches workspace) | Render ratatui in-browser (spec 005) |
| `pharmsol` | 0.29.3 | 53k | 2026-09-15 | **unverified** | PK/PD simulation, NCA, compartment models (spec 003) |
| `polars` | — | 15M | active | **unverified** | Insight joins (spec 006) |
| `statrs` | — | 41M | active | **unverified** | Descriptive stats (spec 006) |
| `colorgrad` | 0.9.0 | 1.6M | 2026-09-28 | **unverified** | Threshold-indexed colour ramps (spec 005) |
| `tui-textarea` | 0.7.0 | 2.8M | 2024-10-22 | **unverified** | Text editing in TUI |
| `fitparser` | 0.11.0 | 80k | 2026-05 | **unverified** | FIT file ingest (spec 004) |
| `uniffi` | — | 14M | active | **unverified** | Rust↔Kotlin bridge for Health Connect (spec 004 v2) |
| `fhir` | 4.2.3 | — | 2026-10-04 | **unverified** | Clinical interchange, future |

`ratzilla` verified: owned by the ratatui core team (`orhun`, `junkdog`, `github:ratatui:core`); depends on `ratatui ^0.30.1` (workspace uses 0.30.2 — caret-compatible); re-exports its ratatui (`pub use ratatui`); pins `thiserror ^2` while the workspace pins `1.0` (workspace moves to 2).

### Write ourselves (confirmed voids — searched, nothing exists)

| Area | Finding |
|---|---|
| **Health Connect (Rust)** | No SDK. Only `tauri-plugin-healthkit` v0.1.4, **94 downloads**, Tauri-specific — wrong architecture. |
| **OpenFoodFacts** | **Zero crates.** Public REST API + MongoDB dump. Thin client, ~200 lines. |
| **Oura / Whoop / Samsung Health** | Zero crates. REST APIs exist (OAuth) — deferred to v2 per local-first constraint. |
| **btop theme → ratatui** | Nothing. `ratatui-themekit` v0.6.1 (2,989 dl) has 15 slots and TOML config; does not speak btop's `theme[key]="#hex"` vocabulary, cannot ingest the 41-theme corpus, no threshold ramps, no 256-color fallback. **This is the gap spec 005 Phase 1 fills.** |
| **Nutrition in Rust** | Graveyard: `diet-calc` (2.4k dl, 2025-04), `nosh` (1.3k dl, 2024), `macro-factor-api` (253 dl). The asset is the data, not the code. |

### Skip

| Crate | Why |
|---|---|
| `nca` | v0.1.0, one release 2024, dead. `pharmsol` has NCA. |
| `tauri-plugin-healthkit` | 94 downloads, wrong architecture. |
| `tui-framework-experiment` | "Proper widget framework" by a ratatui maintainer; 84k dl, last release 2025-11-29, still named *experiment*. The widget-framework idea has been tried and stalled. |
| `ratatui-theme` | v0.0.0, 22 downloads, name reserved, never built. |
| `webatui` | v0.1.1, 3.4k dl, last 2024-02. `ratzilla` supersedes it. |
| `trunk` as a second pipeline | Second artifact, second PWA, second service worker. Existing `wasm-pack` + `wasm-opt` pipeline is kept (spec 005 decision). |

### btop: not ported (decision, 2026-10-11)

Read `src/btop_draw.hpp` directly. btop is a **string painter**: `createBox(...) -> string`, `Meter::operator()(int) -> string` with an `array<string,101>` cache, `Graph::operator()() -> string&`, colours as `string`, a `Symbols` table of border glyphs. Ratatui is a **buffer painter**: `Widget::render(self, area: Rect, buf: &mut Buffer)` with a diff renderer. Porting btop means discarding btop's renderer and rebuilding on ratatui's — the translation buys nothing, because every UI primitive it would produce already ships in `ratatui-widgets`.

The non-UI code is worse: `src/linux/btop_collect.cpp` is 131 KB of `/proc`, sysfs, hwmon and i915 DRM plumbing — btop's largest file, and biohack2 has no use for any of it.

Rust btop ports are not a shortcut: `jw/brt` (15 ★, README: "This is going to be btop in rust"), `toppy` (8 ★), `rtop` (6 ★), `octomon` (12 ★) — 47 repos match `btop rust`, all abandoned or toy-scale. The mature answer is `ClementTsang/bottom` (14,093 ★, **MIT**, v0.14.9, 178 `.rs` files): read `src/app/layout_manager.rs` (47 KB), `src/options/config/style/`, `src/canvas/components/time_series/` as reference. Do not fork.

**Taken from btop (design, not code)**: theme schema (`theme[cpu_start|mid|end]` gradient triples, ~40 semantic keys, 41 official themes); threshold-indexed colour ramps; title-in-border (native in ratatui); `tty_mode` degraded rendering; transparent-background semantics (empty `main_bg` = terminal default).

**Licensing note**: btop is Apache-2.0. Its theme files carry no per-file header; the community themes (Dracula, Catppuccin, Nord, Solarized, Everforest) each have upstream licences. Bundling a theme pack requires checking each. Ingesting a user's own file at runtime is a compatibility feature and clean. btop's CONTRIBUTING (2026-03-22) requires AI-generated contributions be marked `[AI generated]`; non-disclosure closes the PR — upstreaming a port is a hostile path.

### Adoption gate (applies to every "adopt" row above)

Licence verification is **not yet performed** for `pharmsol`, `polars`, `statrs`, `colorgrad`, `tui-textarea`, `fitparser`, `uniffi`, `fhir`. Each spec's first implementation phase carries a task that records the licence before code is written against it. biohack2 is MIT OR Apache-2.0 (verified in `Cargo.toml`); copyleft or attribution-heavy terms must be checked against that.
