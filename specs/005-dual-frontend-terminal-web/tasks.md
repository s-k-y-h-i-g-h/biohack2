# Tasks: Dual Frontend — Native Web UI + Terminal Dashboard

**Input**: Design documents from `/specs/005-dual-frontend-terminal-web/`
**Branch**: `005-dual-frontend-terminal-web`
**Date**: 2026-10-11
**Status**: Proposed — not started. Task numbering starts at T400 (spec 001: T001–T118; 003: T200–T257; 004: T300–T342).

## Legend
- `[X]` = Completed and verified
- `[~]` = Partially implemented
- `[ ]` = Not implemented

## Scope

**v1**: shared theme layer, shared panel crate, terminal dashboard, in-browser dashboard inside the existing bundle.
**v2 (deferred)**: standalone browser terminal app, native mobile terminal, mouse-rich dashboard interaction.

**Gate before Phase 3**: the hinge spike (T420) must prove in-container rendering and draw-loop coexistence. Do not commit to FR-072 before it passes.

---

## Phase 0: The Hinge Spike (throwaway, time-boxed)

**Purpose**: Answer the one unknown that decides the architecture, in isolation, before any structure is built.

- [ ] T420 Spike: add `ratzilla` to `biohack2-web` behind a feature flag; render one bordered `Gauge` into a **container div** (not `document.body`) from a Leptos-driven call; confirm the app's own render loop and the panel's draw calls coexist
- [ ] T421 Spike: measure the WASM bundle delta (`dist/biohack2_web_bg.wasm`, wasm-opt'd) with the feature on vs off; record the number
- [ ] T422 Spike: confirm keyboard focus enters and exits the panel cleanly (no trap, visible focus)
- [ ] T423 Spike: confirm 256-color rendering path (force a 256-color terminal, verify legibility)
- [ ] T424 Record the spike result in `research.md`: backend choice confirmed or rejected, bundle delta, go/no-go on FR-072

**Checkpoint**: A number and a go/no-go decision. If T420 fails, the fallback is a server-rendered dashboard (documented in spec 005 Assumptions) and FR-072 is renegotiated.

---

## Phase 1: Theme Layer (the piece with no library support)

**Purpose**: One palette, three surfaces. This is the crate with genuine ecosystem absence — build it properly.

- [ ] T430 Create the `theme` crate in the workspace (`theme/`) — MIT OR Apache-2.0, matching the workspace
- [ ] T431 Define semantic colour roles as the palette schema (roles, not raw hex in widget code): `bg`, `fg`, `title`, `border`, `accent`, `success`, `warning`, `critical`, `meter_bg`, `inactive`, `graph0..graphN`, per-panel border roles
- [ ] T432 Implement severity band definitions (safe / elevated / high) as data with per-band colour ramps
- [ ] T433 Implement the btop theme-file parser: `theme[key]="#rrggbb"` → palette roles (FR-074). Report missing keys, fill defaults (C3/AC-3)
- [ ] T434 Ship the btop theme corpus mapping as a compatibility table (41 upstream themes → roles); check each upstream theme's licence before bundling any of them
- [ ] T435 Implement truecolor → 256-color mapping with a documented nearest-colour rule (FR-077, C1/AC-2)
- [ ] T436 Implement transparent-background handling (btop's empty `main_bg` = terminal default)
- [ ] T437 Implement the CSS export: palette → CSS custom properties consumable by `global.css` (FR-073)
- [ ] T438 Implement the ratatui export: palette → `Style`/`Color` values (FR-073)
- [ ] T439 Implement threshold-indexed ramp lookup: value → colour (FR-075)
- [ ] T440 Implement theme resolution order: built-in → btop file → user override; record provenance
- [ ] T441 Unit tests: btop file parses to roles; missing keys reported; 256-color mapping deterministic; CSS and ratatui exports agree on the same palette
- [ ] T442 Test: ramp lookup is monotonic across band boundaries (no colour discontinuity mid-band)

**Checkpoint**: One theme file drives terminal colours, web dashboard colours, and app CSS. btop themes load without manual translation.

---

## Phase 2: Shared Panel Crate

**Purpose**: Panel code written once, rendered twice. This is the crate that makes "looks the same" structural.

- [ ] T450 Create the `ui` crate in the workspace — depends on `ratatui` (via the renderer's re-export path to prevent duplicate-crate trait mismatches), `theme`, `engine`; **no web dependency** (FR-081)
- [ ] T451 Define the `Panel` trait: data contract + `render(area, frame)`; panels are target-agnostic
- [ ] T452 Implement `LayoutSpec` with size classes: `full` / `compact` / `minimal` (FR-076)
- [ ] T453 Define the documented minimum terminal size and the `minimal` degraded layout (FR-076, C1/AC-3)
- [ ] T454 Implement `GaugeRow` — threshold-ramp gauge with a non-colour severity label (FR-075, C4/AC-2)
- [ ] T455 Implement `BandChart` — low/expected/high band over time with a "now" marker
- [ ] T456 Implement `TrendSparkline` — metric over time with direction indicator
- [ ] T457 Implement `AlertPanel` — severity-ordered, with acknowledge affordance
- [ ] T458 Implement `RecentEntriesPanel` — last N entries, category-coloured
- [ ] T459 Implement `ActiveDosePanel` composing `GaugeRow` + `BandChart` + model-class badge (spec 003 T241)
- [ ] T460 Implement keyboard navigation model: panel focus cycling, panel-local actions, escape to exit
- [ ] T461 Implement the 256-color render path end-to-end in the crate (FR-077)
- [ ] T462 Unit tests with `ratatui`'s test backend: panel renders at minimum size with no overlap; severity conveyed without colour
- [ ] T463 Snapshot tests (`insta`) for panel output at three sizes — guards layout regressions

**Checkpoint**: Panels compile and render headlessly. No panel logic is web-aware.

---

## Phase 3: Terminal Dashboard (CLI)

**Purpose**: `biohack2 dash`. First consumer of the panel crate — the demanding app that shapes the API.

- [ ] T470 Add a `dash` subcommand to the CLI (`engine` binary or a new `cli` member) opening the local database
- [ ] T471 Implement the crossterm backend + `Terminal` setup with panic-safe teardown (restore on panic, not just on clean exit)
- [ ] T472 Implement the frame loop: fixed-interval redraw, event-driven on data change, CPU-bounded (no busy loop — this is a phone)
- [ ] T473 Implement the panel grid: active doses, vitals trends, alerts, recent entries
- [ ] T474 Implement live update: new log/vitals/alert appears on next frame with no user action (FR-078, C1/AC-4)
- [ ] T475 Implement theme loading at startup: built-in, `--theme <file>`, btop-format accepted
- [ ] T476 Implement terminal capability detection: truecolor vs 256-color, size, degraded mode selection
- [ ] T477 Implement resize handling with layout class switching
- [ ] T478 Test: renders on a populated DB within 2 s (SC-070)
- [ ] T479 Test: 60×20 terminal renders the readable subset, no overlap, no crash (C1/AC-3)
- [ ] T480 Test: 256-color terminal renders legibly with severity distinguishable (C1/AC-2)
- [ ] T481 Test: frame loop CPU cost measured and recorded on a phone-class device

**Checkpoint**: `biohack2 dash` is usable on a desktop terminal and in a phone container terminal.

---

## Phase 4: In-Browser Dashboard (same bundle)

**Purpose**: The same panels, inside the existing app, reading the app's own store.

- [ ] T490 Add `ratzilla` to `web/Cargo.toml` with default-features off; use `ratzilla::ratatui::…` as the ratatui path everywhere in `ui` and `web` (prevents two ratatui copies)
- [ ] T491 Bump workspace `thiserror` 1.0 → 2.0 (ratzilla pins `^2`)
- [ ] T492 Implement the `DomBackend`-backed render target in `web` implementing the `ui` crate's render-target contract
- [ ] T493 Implement the panel container component: a sized div in the dashboard page, with explicit focus entry/exit (FR-083, T422 result)
- [ ] T494 Wire the panel data to the app's existing in-memory store — **no server round-trip** (FR-072, C2/AC-2)
- [ ] T495 Implement live update from the app's existing reactive layer (`AppContext.data_version` pattern from spec 001) (FR-078)
- [ ] T496 Wire the theme layer to CSS custom properties so the panel and the app share one palette (FR-073, C3/AC-1)
- [ ] T497 Implement the dashboard as a route/view in the existing hash router (`#/dashboard`)
- [ ] T498 Implement the user-disableable setting (FR-080) and verify all flows work with it off (SC-076)
- [ ] T499 Implement lazy loading if the measured bundle delta exceeds budget (FR-082, SC-074)
- [ ] T500 Measure and record final bundle delta against the Phase 0 spike number (SC-074)
- [ ] T501 WASM tests: panel data derivation in-browser; dose logged → panel updates with no network (SC-075)
- [ ] T502 Accessibility: panel labelled as a visual region; its critical values also available as text (edge case)

**Checkpoint**: One bundle, one store, one palette, panels in both surfaces.

---

## Phase 5: Guardrails & Polish

- [ ] T510 Verify no data-entry flow is panel-only (FR-079, C5/AC-1) — enumerate every flow
- [ ] T511 Verify zero duplicated panel logic (SC-071) — structural check: one panel module, two render targets
- [ ] T512 Document the minimum terminal size, degraded layout, and 256-color behaviour in the README
- [ ] T513 Document the theme file format and the btop compatibility mapping
- [ ] T514 Record measured performance: dashboard launch, frame cost, bundle size, panel update latency
- [ ] T515 Accessibility pass on the embedded panel (keyboard, focus visibility, reduced-motion)

**Checkpoint**: Guardrails verified, numbers recorded, behaviour documented.

---

## Sequencing & Dependencies

```
Phase 0 (spike) ──> Phase 1 (theme) ──> Phase 2 (panels) ──┬──> Phase 3 (CLI dash)
                                                            └──> Phase 4 (web embed)
                                                          Phase 5 (guardrails, last)
```

**Ordering rule, non-negotiable**: Phase 3 (CLI) before Phase 4 (web embed). The CLI is the demanding consumer that shapes the panel API. Building the web embed first means designing an API from imagination — the failure mode of every abandoned crate in this genre.

Phase 1 and 2 are the extractable library (see `research.md` §Ecosystem survey). Publish only after Phase 3 ships, with the CLI as the reference app.

## Critical Constraints (do not violate)

1. **One bundle.** No second WASM artifact for the dashboard (FR-072). A separate app cannot share the SQLite/OPFS store.
2. **CLI first, web second.** API shaped by a real consumer.
3. **One ratatui.** Use the renderer's re-export path everywhere (edge case: two copies = trait failure).
4. **`ui` crate has no web dependency** (FR-081) so the CLI builds with web features off.
5. **Forms stay semantic HTML** (FR-079). The grid is for reading.
6. **Bundle delta is measured, not hoped** (SC-074). Record the number; lazy-load if over budget.
7. **btop is not ported.** Its renderer is a string painter; ratatui is a buffer painter. Take the theme schema, the threshold ramps, the title-in-border look, and the degraded-mode idea.
