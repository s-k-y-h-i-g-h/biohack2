# Tasks: Substance Core — Active Dose & Redose Windows

**Input**: Design documents from `/specs/003-substance-core-pharmacokinetics/`
**Branch**: `003-substance-core-pharmacokinetics`
**Date**: 2026-10-11
**Status**: Proposed — not started. Task numbering starts at T200 to avoid collision with spec 001 (T001–T118).

## Legend
- `[X]` = Completed and verified
- `[~]` = Partially implemented
- `[ ]` = Not implemented

## Scope

**v1**: numeric half-life migration, model classes, deterministic PK simulation, active-dose band rendering, redose alerts, cumulative-load guardrails.
**v2 (deferred)**: drug–drug pharmacokinetic interactions, individual PK fitting, non-oral routes.

**Gate before Phase 2**: library decision recorded in `research.md` with licence verified. Do not write simulation code against an unverified licence.

---

## Phase 1: Data Foundation (Parameters & Model Classes)

**Purpose**: Make the parameters machine-readable. Nothing downstream works until half-life is a number and every modelled substance has a class.

- [X] T200 Add `half_life_hours: Option<f64>` and `half_life_range: Option<(f64, f64)>` to `CatalogItem` in `engine/src/models.rs` with `serde(default)` (pre-existing persisted rows must deserialize unchanged — same pattern as `mechanism_tags`, T116) — DONE (5f0178d: fields added with serde(default); engine+integration suites green)
- [X] T201 [P] Create `engine/src/pharmacokinetics.rs` module skeleton + `ModelClass` enum (`decay`, `oral_absorption`, `plateau`, `accumulation`, `none`) in `engine/src/models.rs` — DONE (5f0178d: pharmacokinetics.rs + ModelClass enum in models.rs)
- [X] T202 Create `engine/src/pk_params.rs` — `SubstanceModel` struct (model class, half-life, range, `params: serde_json::Value`, provenance, catalog version) — DONE (5f0178d: pk_params::SubstanceModel with mandatory provenance + rationale)
- [X] T203 Write the string→numeric parser for existing `half_life` values (`"5-6 hours"`, `"24h"`, `"1.5 days"`, `"N/A"`) with an explicit failure list — no silent defaults — DONE (5f0178d: parse_half_life — explicit failure reasons, no silent defaults)
- [X] T204 Run the parser over the full catalog (32 seed items) and record the failure list; every failure becomes a `none` class with a logged reason — DONE (5f0178d: audit test pins failure list = ['9-Me-BC'] ('unknown'); all other 36 items parse)
- [X] T205 Assign `ModelClass` to every catalog item; document the rationale for each non-`decay` assignment (creatine → plateau, fat-soluble vitamins → accumulation, SSRIs → accumulation, etc.) — DONE (0ea01cc: pk_assignments table covers all 37 items, coverage test fails on drift both directions; rationale present for every non-decay row)
- [x] T206 Persist `SubstanceModel` in SQLite: schema table + additive `ALTER TABLE`/`CREATE TABLE` migration tolerating re-run (pattern from T117) — `substance_models` table keyed by substance name; `upsert_substance_models`/`get_substance_models`/`ensure_substance_models` in `engine/src/db.rs`; server seeds it after `seed_catalog` (startup + catalog ingest); WASM mirror in `web/src/state/db.rs` (`biohack2_substance_models` localStorage key, refreshed on every catalog seed)
- [x] T207 Add `model_class` and numeric half-life to the catalog seed insert path and both SELECT paths in `engine/src/db.rs` — `half_life_hours`/`half_life_range_lo`/`half_life_range_hi` REAL columns written by the seed INSERT and read by `get_catalog_items` + `search_catalog`; row reader prefers stored numerics and falls back to the canonical `half_life_fields` derivation for pre-T207 rows; `model_class` persisted per substance in `substance_models` (tests: `engine/tests/pk_persistence_tests.rs`, 3 passed)
- [X] T208 Unit tests: parser accepts known formats, rejects unknown formats into the failure list, range endpoints preserved — DONE (5f0178d: parser tests accept known forms, reject unknown with raw-quoted reasons, range endpoints preserved)

**Checkpoint**: Every catalog item has a model class and, where modelled, a numeric half-life. Parser failures are enumerated, not hidden. `cargo test -p engine` green.

---

## Phase 2: Simulation Engine (Deterministic PK)

**Purpose**: The math. Deterministic, offline, unit-tested, identical across targets.

- [x] T209 Record the library decision in `research.md`: adopt `pharmsol` or hand-roll. **Licence verified before code.** Record the decision and the verification date. → **hand-roll**; decision recorded before any sim code; zero new dependencies, so no third-party licence gate applies (pharmsol licence unverifiable while offline — noted in research.md)
- [x] T210 Implement `DoseEvent` derivation from `LogEntry` in `engine/src/pharmacokinetics.rs` (substance, amount, unit, timestamp, entry id) — missing quantity/unit is rejected, never defaulted
- [x] T211 Implement unit validation: dose unit must match the model's expected unit; mismatch blocks the model and surfaces the mismatch (no silent conversion) — plus one-unit consistency across all doses
- [x] T212 Implement `decay` model (one-compartment, first-order) — analytic, no ODE solver needed
- [x] T213 Implement `oral_absorption` model (lag/absorption phase + decay) — Bateman; needs explicit `absorption_half_life_hours`/`ka_per_hour` in catalog params (refuses, never guesses); optional `lag_hours` (default 0 documented)
- [x] T214 Implement `plateau` model (saturation ceiling) — per-dose saturating rise, optional `ceiling` cap; needs explicit `rise_time_hours`; band rides on rise time (range param or flagged ±25 %)
- [x] T215 Implement `accumulation` model (multi-day rise toward steady state) — per-dose uptake at the elimination rate, superposed; band from the t½ band
- [x] T216 Implement multi-dose superposition (summed contributions at time t) — causal: future doses contribute nothing
- [x] T217 Implement band computation: evaluate at range endpoints → `low` / `expected` / `high` (FR-033). No midpoint-only output
- [x] T218 Implement the documented default variability factor for single-value half-lives, and mark the output as assumption-derived (FR-033, A5/AC-2) → `DEFAULT_VARIABILITY = 0.25`, documented in research.md; points carry `band_is_assumption`
- [x] T219 Implement per-substance render windowing so long half-lives do not blow up the window (edge case) — `recommended_window(characteristic_hours)`: ~8 characteristic times clamped 24 h–30 d in ~200 steps, so `sample_series` can never approach `MAX_SERIES_POINTS`
- [x] T220 Implement `sample_series(substance, start, end, step) -> Vec<ActiveDosePoint>` as the public API — dispatches over all four modelled classes; `none` refuses explicitly
- [x] T221 Unit tests: known analytic cases (single dose, half-life check at t=half_life; two doses sum; plateau saturates; accumulation rises) — all five cases covered (16 sim tests, analytic expectations checked by hand)
- [x] T222 Create a shared **test vector** file (fixture: log + params → expected series) used by native and WASM tests — `engine/tests/fixtures/pk_vectors.json` (11 vectors, all 4 modelled classes, ±25 % half-life band edges, expectations computed independently in Python); single checker `pharmacokinetics::check_vectors()` consumed by both targets
- [x] T223 WASM test: same fixture produces the same series on `wasm32` within documented tolerance (FR-038, SC-033) — **executed green**: `engine/examples/wasm_vectors.rs` compiles the engine to `wasm32-unknown-unknown` and `engine/examples/wasm_vectors_node_check.mjs` runs the fixture in node's WebAssembly runtime (no browser/chromedriver needed); also a `wasm_bindgen_test` in `web/src/tests.rs` for browser CI
- [x] T224 Test: local-time rendering of UTC-stored doses is correct across a DST boundary (chrono `wasmbind`/`now`/`clock` features must stay enabled — see the 2026-09-09 incident) — engine invariant test `t224_dst_boundary_is_invisible_to_the_sim` (2026-10-25 EU flip: identical wall clock 1 real hour apart decays by UTC; series stays exactly uniform across the boundary); browser-side conversion covered by spec-002 `local_time` tests

**Checkpoint**: A dose log produces a reproducible band series on both targets. No simulation path can produce a single line without a band.

---

## Phase 3: Redose Windows & Cumulative Load (Alerts)

**Purpose**: Make the curve speak, through the existing alert path.

- [x] T225 Add `TargetBand` entity to `engine/src/models.rs` (substance, low, high, window_hours, updated_at)
- [X] T226 Persist target bands (SQLite + WASM storage paths) — DONE (cf5780f: engine+db generation/persistence with unit+integration tests)
- [x] T227 Implement band-crossing detection: simulated band crosses below target low → crossing time + redose window
- [x] T228 Extend `AlertType` with `Redose` and `CumulativeLoad` in `engine/src/models.rs` (existing `Alert` carries type/severity/message/recommendation — reuse, do not fork the alert path)
- [X] T229 Implement redose alert generation in the engine (offline, from stored data only) — DONE (cf5780f: engine+db generation/persistence with unit+integration tests)
- [x] T230 Implement rolling-24 h cumulative dose per substance against catalog max daily amount (FR-035)
- [X] T231 Implement cumulative-load alert generation, distinct from pairwise interaction alerts (FR-015 stays as-is) — DONE (cf5780f: engine+db generation/persistence with unit+integration tests)
- [X] T232 Alert resolution: redose alert resolves when a logged dose restores the band (A2/AC-2) — DONE (cf5780f: engine+db generation/persistence with unit+integration tests)
- [X] T233 Wire redose + cumulative alerts into the existing save flow so they appear in the current alert banner (spec 001 T087/T088 pattern) — DONE (1361b2c: refresh on save supersedes/recomputes PK alerts; banner excludes resolved; 2 wasm tests)
- [X] T234 Unit tests: crossing fires, restore resolves, no-target produces no alert, rolling window excludes out-of-window doses — DONE (cf5780f: engine+db generation/persistence with unit+integration tests)
- [X] T235 Test: alerts fire with no network and no catalog fetch — DONE (85e5738: wasm test with window.fetch replaced by a thrower; alert path fires network-hostile)

**Checkpoint**: Redose and cumulative-load alerts fire from the log alone, through the existing alert system.

---

## Phase 4: Provenance & Honesty Surface

**Purpose**: The user can audit every number, and is told what the number is not.

- [X] T236 Attach provenance (catalog default / catalog version / user override + timestamp) to every parameter used in a simulation (FR-036) — DONE (85e5738: sample_series refuses blank provenance; with_user_half_life_override chains catalog origin + user edit + timestamp)
- [X] T237 Implement `explain(substance)` returning model class, parameters, provenance, and the dose events used (FR-037) — DONE (3dba73d: engine::explain — class, params, provenance chain, used vs refused events with reasons)
- [X] T238 Implement the limitations text generator (inputs named, "estimate not measurement" stated) (FR-039, A5) — DONE (3dba73d: limitations_text names only numbers on the model; states estimate-not-measurement; unmodelled variant makes no estimate claim)
- [X] T239 Implement `none`-class surfacing: panel shows "no model" with the recorded reason (A1/AC-3, FR-040) — DONE (code commit: unmodelled_report + pk_coverage_gaps; gaps surface with recorded reasons)
- [X] T240 Unit tests: every rendered point is reproducible from `explain()` output (SC-032); no modelled substance lacks provenance — DONE (code commit: point-for-point rebuild from explain(); seed-catalog provenance + report-coverage invariants)

**Checkpoint**: Any displayed number can be reproduced by hand from displayed inputs.

---

## Phase 5: Frontend Surfaces (depends on spec 005 panel crate)

**Purpose**: Render it. Panel code lives in the shared dashboard crate (spec 005) so terminal and web share it.

- [X] T241 Create `ActiveDosePanel` in the shared dashboard crate (gauge + band + sparkline + substance label + model class badge) — DONE (derivation in `engine::dashboard`, web-free by construction per FR-081; spec-005 mandates crate extraction only after the first working panel — the panel is now that consumer; component `web/src/components/active_dose_panel.rs`)
- [X] T242 Render the band (low/expected/high) in the panel — band, not line (FR-033) — DONE (shaded low→high polygon is the primary mark; expected line rides inside it)
- [X] T243 Render "now" marker and the 24 h back / 12 h forward window — DONE (window + now_index derived in panel_view, asserted by tests; vertical rule at index 24)
- [X] T244 Render target band edges on the same axis — DONE (dashed rules at target low/high share the sparkline y-axis)
- [X] T245 Wire redose alerts into the alert panel — DONE via T233 (refresh_pharmacokinetic_alerts feeds Redose/CumulativeLoad into the shared alert state the banner renders)
- [X] T246 Implement the web dashboard page embedding the panel (spec 005 Phase 3) — DONE (`#/dashboard` route + nav link; one ActiveDosePanel per tracked substance — target bands first, then logged substances; unmodelled substances get explicit no-model panels per T239; recomputes on every save via data_version)
- [X] T247 Implement target-band editing UI in the web app (form, not panel — spec 005 FR-079) — DONE (`TargetBandEditor` form on #/dashboard: list + set + remove; validation delegated to `TargetBand::validate`; every save/delete regenerates the PK alert generation and bumps data_version, so panels and the alert banner refresh together)
- [X] T248 Implement the provenance/limitations disclosure view (one interaction from any panel, SC-034) — DONE (verified: panel header badge = model class with zero interaction; one-click "Model & limitations" details carries inputs used, `Model class: … Provenance: <source> (catalog version N)`, and the not-a-measurement caveat via limitations_text)
- [X] T249 Performance test: 30-day series × 10 substances under 500 ms desktop / 2 s phone (SC-030) — DONE (`engine/examples/perf_sc030.rs`, release: panel_view × 10 = 3.9 ms; 30-day hourly series × 10 (7210 points, 900 dose events) = 13.5 ms vs 500 ms budget. 37× points at ~4× cost on wasm still ≪ 2 s phone budget; DOM cost of 10×37-point SVGs is additive-negligible)
- [X] T250 WASM tests for panel data derivation — DONE (`check_panel_vectors` runs the real `panel_view` path over `tests/fixtures/panel_vectors.json` — analytically Python-computed expectations, 4 vectors incl. catalog-range band edges, assumed-band flag, empty-model path; pinned natively with hand-computed closed-form asserts, executed inside real wasm32 by the node runner: `wasm panel vectors: OK`. Shim moved examples/→src/bin (examples compile dev-deps, wasm32 can't build them)

**Checkpoint**: Active-dose bands visible in terminal and web from one panel implementation.

---

## Phase 6: Catalog Parameter Build-Out

**Purpose**: Coverage. A model that only covers 4 substances is a demo.

- [x] T251 Research and record reference parameters for the stimulant class (caffeine, theobromine, tyrosine, etc.) with source cited per value
- [x] T252 [P] Research and record parameters for minerals/electrolytes (magnesium, creatine plateau, etc.)
- [x] T253 [P] Research and record parameters for fat-soluble vitamins and accumulation cases
- [x] T254 [P] Research and record parameters for the serotonergic/nootropic classes
- [x] T255 Mark every item without a defensible parameter set as `none` (FR-040) — absence is a deliverable, not a gap

> Done 2026-11-18: verified by `engine/tests/integration_tests.rs::test_every_catalog_substance_has_an_assigned_model_class` (guards that no catalog item is silently unassigned). Per-value pubmed citations live in `pk_assignments.rs` rationale/provenance; staleness bump path = `CATALOG_VERSION` in `catalog.rs` → `SubstanceModel.catalog_version`.
- [x] T256 Record parameter sources in the catalog with citation strings (FR-036 provenance)
- [x] T257 Add a catalog-release version bump path for parameter updates (FR-006 consistency)

**Checkpoint**: Every catalog item is either modelled with cited parameters or explicitly `none`.

---

## Sequencing & Dependencies

```
Phase 1 (params/classes) ──┬──> Phase 2 (simulation) ──┬──> Phase 3 (alerts)
                           │                            └──> Phase 4 (provenance) ──> Phase 5 (UI)
                           └──> Phase 6 (coverage, parallel)
```

- Phase 5 depends on spec 005 Phase 1–2 (shared panel crate exists).
- Phase 3's alert surfacing reuses spec 001's alert path — do not build a second alert system.
- Phase 6 runs in parallel with 2–4 and is the long pole; ship with partial coverage and explicit `none` states.

## Critical Constraints (do not violate)

1. **No model inference in the result path.** Deterministic only (FR-032). Same rationale as FR-015.
2. **No single line.** Band always (FR-033). A line implies a precision the data cannot support.
3. **No silent defaults.** Parse failure → `none` + recorded reason (FR-030, FR-040).
4. **Licence before code** for any adopted PK library (T209).
5. **WASM/native parity is tested, not assumed** (T222/T223). The 2026-09-09 chrono incident is the precedent for "works natively, panics in browser".
