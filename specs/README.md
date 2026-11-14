# biohack2 — Specification & Feature Index

**Updated**: 2026-10-11
**Read this first.** It maps every planned feature to its spec, its requirement IDs, and its task range.

## Numbering convention (do not break it)

| Spec | FR range | Task range | Status |
|---|---|---|---|
| 001-biohacker-tracking-platform | FR-001…021 | T001–T118 | **Implemented** (v1 core) |
| 002-history-relative-time | FR-001…006 | — | **Removed 2026-09-13** (see note below) |
| 003-substance-core-pharmacokinetics | FR-030…040 | T200–T257 | Proposed |
| 004-wearable-health-ingest | FR-050…060 | T300–T342 | Proposed |
| 005-dual-frontend-terminal-web | FR-070…083 | T420–T515 | Proposed |
| 006-insights-loop | FR-090…101 | T600–T676 | Proposed |

Task IDs are **unique per spec** and never reused: 001 owns T001–T118, 003 owns T200+, 004 owns T300+, 005 owns T420+, 006 owns T600+. Cross-spec references (e.g. spec 003 citing spec 001's T087 alert path) are prose, never a second definition of the same ID.

FR ranges are allocated in blocks of 10 per spec so cross-references stay unambiguous.

> **Spec 002 note**: removed 2026-09-13. biohack2 has **no integration with Semantica** — the app owns its own safety features. Its FR-001…006 numbering is local to that file and predates the convention above.

---

## Feature map

Every planned feature, where it lives, and what it depends on.

### A. Active dose & redose windows — [spec 003](003-substance-core-pharmacokinetics/spec.md)

The app already knows *what you took*. It does not know *what is in you now*.

| Feature | FR | Task |
|---|---|---|
| Numeric half-life + range in the catalog (replaces the free-text field) | FR-030 | T200–T204 |
| Model class per substance (`decay` / `oral_absorption` / `plateau` / `accumulation` / `none`) | FR-031 | T201, T205 |
| Deterministic PK simulation from the existing dose log | FR-032 | T210–T221 |
| **Band, never a line** (low/expected/high from the half-life range) | FR-033 | T217–T218 |
| Target bands + redose window alerts | FR-034 | T225–T233 |
| Rolling-24 h cumulative load vs catalog max daily | FR-035 | T230–T231 |
| Provenance on every parameter used | FR-036 | T236–T237 |
| Native/WASM result parity | FR-038 | T222–T223 |
| Explicit "no model" for unmodelled substances | FR-040 | T239, T255 |

**Depends on**: spec 001's `LogEntry`, `CatalogItem`, `Alert` path. Spec 005 for rendering.
**Gate**: PK library licence verified (T209) before simulation code.

### B. Wearable & health-platform ingest — [spec 004](004-wearable-health-ingest/spec.md)

Vitals are typed by hand today. Wearables produce 288 HR points a night and none of them reach the app.

| Feature | FR | Task |
|---|---|---|
| Source-agnostic ingest contract (`RawReading`) | FR-050 | T300–T304 |
| Metric **definitions** (RMSSD vs SDNN never silently compared) | FR-055 | T302 |
| Idempotent ingest (`source_id` + `source_record_id`) | FR-052 | T305 |
| Explicit source-timezone → UTC | FR-053 | T306 |
| File ingest: FIT / CSV / JSON from an inbox directory | FR-054 | T310–T319 |
| Per-source provenance in the UI | FR-051 | T320 |
| Conflicts surfaced, never averaged | FR-056 | T321–T322 |
| Ingested data drives the **existing** safety engine | FR-057 | T325–T329 |
| Companion app (Health Connect) — v2 | FR-050 | T330–T338 |
| Vendor APIs (Oura/Whoop/Strava) — v2, opt-in | FR-060 | T339–T342 |

**Depends on**: spec 001's `VitalsEntry`, `SafetyEngine::check_vitals()`, `custom_metrics`.
**Gate**: per-vendor access route documented with its platform-terms basis (T330) before v2 code.

### C. Dual frontend — terminal dashboard + in-browser dashboard — [spec 005](005-dual-frontend-terminal-web/spec.md)

Two surfaces, one widget codebase, one palette. The web app is a form app; there is no glanceable view anywhere.

| Feature | FR | Task |
|---|---|---|
| Shared theme layer: semantic roles, one palette → ratatui + CSS | FR-073 | T430–T442 |
| btop theme-file compatibility (`theme[key]="#rrggbb"`, 41 upstream themes) | FR-074 | T433–T434 |
| Threshold-indexed colour ramps (btop's severity gradient) | FR-075 | T439 |
| 256-color + degraded-mode fallback | FR-077 | T435, T461, T476 |
| Shared panel crate (`ui`) — target-agnostic widgets | FR-081 | T450–T463 |
| Panels: gauge, band chart, sparkline, alerts, recent entries, active dose | FR-076 | T454–T459 |
| `biohack2 dash` — terminal dashboard | FR-078 | T470–T481 |
| **In-browser dashboard in the existing bundle** (ratzilla DomBackend) | FR-072 | T490–T502 |
| Forms stay semantic HTML (panels are for reading) | FR-079 | T510 |
| Live update from the app's own store, no server round-trip | FR-078 | T494–T495 |
| Bundle-size budget + lazy load | FR-082 | T499–T500 |

**Depends on**: spec 001's `web` crate, `AppContext.data_version`, `global.css`, hash router.
**Gate**: the hinge spike (T420–T424) proves in-container rendering + measures the WASM delta **before** FR-072 is committed.
**Ordering rule**: CLI (Phase 3) before web embed (Phase 4). The API is shaped by a real consumer.

### D. Insights loop — [spec 006](006-insights-loop/spec.md)

Successor to spec 001's deferred US6. The app collects evidence and draws no conclusions.

| Feature | FR | Task |
|---|---|---|
| Intervention tracking (the unit of self-experiment) | FR-093 | T600–T606 |
| Pre/post review on vitals + simulated state | FR-091 | T611–T617 |
| **Data gate**: below threshold → refusal, with the missing amount stated | FR-092 | T613, T622 |
| Limitations inseparable from results (n, window, density, confounders) | FR-094 | T618–T619, T621 |
| Descriptive language only — never population/clinical framing | FR-095 | T662 |
| Note-derived candidates → testable review | FR-097 | T630–T636 |
| Drift detection with density qualifiers | FR-098 | T640–T645 |
| Insight ledger: proposed/tested/supported/unsupported | FR-096 | T650–T656 |
| Multiple-comparison honesty: a scan is not a finding | FR-096 | T660–T663 |
| Reproducible on native and WASM | FR-099 | T624 |

**Depends on**: spec 001's `Insight`, `LogEntry`, `VitalsEntry`, note resolution. Spec 003 (simulated state) and spec 004 (dense vitals) enrich it — **but Phase 2 is testable on hand-logged vitals alone**, so it is not blocked on them.
**Gate**: no numeric insight renders without n, window, density, limitations. Enforced in code (T621), not in review.

---

## Build order across specs

```
001 (done)
  ├──> 003 Phase 1–2   numeric params → PK simulation        [no UI dependency]
  ├──> 004 Phase 1–2   ingest contract → file ingest (MVP)   [no UI dependency]
  ├──> 005 Phase 0–2   spike → theme crate → panel crate     [unblocks 003/004/006 UI]
  └──> 006 Phase 1–2   interventions → gated review engine   [no UI dependency]

005 Phase 3 (CLI dash) ──> 005 Phase 4 (web embed)
       │
       └── 003 Phase 5, 004 Phase 3, 006 Phase 7 all render through 005's `ui` crate
```

**The critical path is 005 Phase 0–2.** Everything visual in 003, 004, and 006 renders through the shared panel crate, so the theme layer and panel crate gate the whole v2 surface. Everything non-visual (003 P1–2, 004 P1–2, 006 P1–2) is independent and can proceed in parallel.

**Ship order that delivers value earliest**: 004 Phase 1–2 (file ingest, no mobile work) and 003 Phase 1–2 (PK math, no UI) are both self-contained and both useful headlessly. 005 Phase 0 is a day-scale spike that de-risks the largest architectural bet.

---

## Cross-cutting constraints

1. **Deterministic only in the result path.** No model, no LLM, no network in safety, PK, ingest mapping, or insight output. Same rule as spec 001 FR-015.
2. **Local-first.** No v1 feature requires a vendor cloud. Health data does not leave the LAN.
3. **Licence before code.** Every adopted crate has a task that records its licence *before* implementation. Verified so far: `ratzilla` (MIT OR Apache-2.0), `ratatui` (MIT), `bottom` (MIT, reference only). **Unverified**: `pharmsol`, `polars`, `statrs`, `colorgrad`, `tui-textarea`, `fitparser`, `uniffi`, `fhir`. Workspace licence is MIT OR Apache-2.0.
4. **WASM parity is tested, never assumed.** The 2026-09-09 chrono incident (`wasmbind`/`now`/`clock` features missing → `Utc::now()` compiled to `unreachable` → every save panicked in the browser, silently) is the standing precedent. Any time, timezone, or time-series code gets a wasm32 test.
5. **No second alert system.** 003 and 004 extend spec 001's `Alert`/`AlertType` path.
6. **No second data model.** 004's ingest lands in `VitalsEntry` + `custom_metrics`. 003's output derives from `LogEntry`. Nothing new reaches the dashboard.
7. **btop is not ported.** Its renderer is a string painter (`createBox -> string`, `Meter::operator()(int) -> string`); ratatui is a buffer painter (`Widget::render(area, &mut Buffer)`). Every primitive a port would produce already ships in `ratatui-widgets`. Take the theme schema, threshold ramps, title-in-border, and degraded-mode idea. Reference implementation to read: `ClementTsang/bottom` (MIT, 14k★). Full rationale: [spec 001 research.md §Ecosystem Survey](001-biohacker-tracking-platform/research.md).

## Known doc/code drift

| Doc says | Code does | Task |
|---|---|---|
| README: "all data is stored in SQLite via WASM in the browser" | WASM path uses **gloo_storage (localStorage)**; SQLite/sqlx is native-only (`engine/src/db.rs`, `server/`) | Fix README, or implement OPFS/SQLite-WASM |
| spec 001 plan.md: SQLite WASM + OPFS | Not implemented; `opfs`/`sqlite_wasm` absent from all Cargo.toml | Decide: implement, or mark deferred |
| spec 001: `thiserror = "1.0"` | `ratzilla` pins `thiserror ^2` | T491 (workspace moves to 2.0) |

## Ecosystem survey

Full adopt/write-ourselves/skip table, download counts, licence status, and the btop analysis: **[spec 001 research.md §Ecosystem Survey — 2026-10-11](001-biohacker-tracking-platform/research.md)**.

Headline findings:
- **Adopt**: `ratatui` 0.30.2 (58.7M dl), `ratatui-widgets` 0.3.2, `ratzilla` 0.3.1, `pharmsol` 0.29.3, `polars`, `statrs`, `colorgrad` 0.9.0, `fitparser` 0.11.0, `uniffi`.
- **Write ourselves — confirmed voids**: Health Connect (Rust) has no SDK (only a 94-download Tauri plugin); OpenFoodFacts has **zero** crates; Oura/Whoop/Samsung Health zero; **btop theme → ratatui has nothing** (`ratatui-themekit` v0.6.1, 2,989 dl, 15 slots, TOML-only — does not speak btop's vocabulary).
- **Skip**: `nca` (dead, 1 release), `tauri-plugin-healthkit` (94 dl, wrong architecture), `tui-framework-experiment` (ratatui maintainer's widget framework, stalled at "experiment"), `ratatui-theme` (v0.0.0, 22 dl, never built), `webatui` (superseded by ratzilla), `trunk` as a second pipeline.
- **Rust btop ports are all dead**: 47 repos match `btop rust`; `jw/brt` 15★, `toppy` 8★, `rtop` 6★, `octomon` 12★. The mature one is `bottom` (14,093★, MIT, 178 .rs files).
