# Feature Specification: Dual Frontend — Native Web UI + Terminal Dashboard

**Feature Branch**: `005-dual-frontend-terminal-web`

**Created**: 2026-10-11

**Status**: Draft — proposed, not ratified

**Depends on**: [001-biohacker-tracking-platform](../001-biohacker-tracking-platform/spec.md) (Leptos web app, engine crate)

**Input**: Planning discussion 2026-10-11. Goal: biohack2 has a **legit web UI** (forms, mobile, PWA) *and* a **btop-style terminal dashboard**, with the dashboard looking and behaving the same in a terminal and in the browser, and with one engine, one theme, and one build pipeline.

## Problem Statement

The app today is a web app. That is correct for logging — forms, autocomplete over 800+ catalog items, file import, mobile touch. It is wrong for the thing the app most wants to be at a glance: a dense, always-on panel of active doses, vitals trends, and alerts.

A terminal UI is excellent at *reading* and poor at *writing*. A web UI is excellent at *writing* and, as currently built, mediocre at dense at-a-glance reading. Building both as separate applications means two palettes, two widget sets, two data-access paths, and two codebases drifting apart. The requirement is therefore: **one widget codebase, one palette, one bundle.**

## User Scenarios & Testing *(mandatory)*

### User Story C1 - Terminal Dashboard (Priority: P1) 🎯 MVP

A user runs `biohack2 dash` on a Linux box, a phone container, or Windows and sees a btop-style panel: active-dose gauges per substance, vitals trends, redose and safety alerts, and a recent-entries strip. Keyboard-driven, no mouse, works in a plain terminal.

**Why this priority**: It is the cheapest surface to build on top of the engine (no WASM, no browser, direct SQLite), and it is the surface the user reaches for when the app is "on" rather than being used.

**Independent Test**: With a populated database, `biohack2 dash` renders panels with live values, responds to keyboard navigation, and degrades correctly in a 256-color terminal.

**Acceptance Scenarios**:

1. **Given** a populated database, **When** the dashboard starts, **Then** panels render with active doses, vitals, and alerts within 2 s.
2. **Given** a terminal without truecolor, **When** panels render, **Then** colours degrade to the 256-color palette with thresholds and severity still visually distinguishable.
3. **Given** a 60×20 terminal, **When** panels render, **Then** the layout degrades to a readable subset with no overlap, truncation of critical values, or crash.
4. **Given** an alert is raised while the dashboard is open, **When** the next frame renders, **Then** the alert is visible without user action.

---

### User Story C2 - Same Dashboard Inside the Web App (Priority: P1)

The same panel set renders inside the existing web app, in a container on the dashboard page, sharing the app's data store. No second app, no second login, no second database.

**Why this priority**: This is the requirement that decides the architecture. If the web dashboard cannot read the app's own store, it is a second application and the feature has failed.

**Independent Test**: Log a dose in the web app → the embedded dashboard panel updates in the same page, same session, with no server round-trip.

**Acceptance Scenarios**:

1. **Given** the web app is open, **When** the user navigates to the dashboard, **Then** the terminal-style panels render inside the page.
2. **Given** a dose is logged in the app, **When** the dashboard is visible, **Then** the panel updates from the same in-memory store, with no network request.
3. **Given** the user changes the theme in Settings, **When** the dashboard renders, **Then** panel colours follow the same palette as the rest of the app.

---

### User Story C3 - Shared Palette Across All Surfaces (Priority: P1)

One theme file defines the palette. The terminal, the embedded web dashboard, and the Leptos forms all read it. A user's btop theme file can be loaded and applied to all three.

**Why this priority**: This is the mechanism that makes "looks the same" real rather than aspirational, and it is the piece with no existing library support.

**Independent Test**: Load a btop `.theme` file → the terminal dashboard, the embedded web dashboard, and the app's CSS custom properties all change consistently.

**Acceptance Scenarios**:

1. **Given** a theme file, **When** it is applied, **Then** all three surfaces change and no surface keeps a stale colour.
2. **Given** a btop-format theme file, **When** it is loaded, **Then** it applies without manual translation.
3. **Given** a theme file with missing keys, **When** it is loaded, **Then** defaults fill the gaps and the missing keys are reported.
4. **Given** a colour-only-capable terminal, **When** a truecolor theme is active, **Then** colours map to the nearest supported value and the panel remains legible.

---

### User Story C4 - Threshold-Indexed Colour Ramps (Priority: P2)

A gauge shifts hue as its value rises through safe → elevated → high, using the theme's ramp, in both terminal and browser.

**Why this priority**: This is btop's signature mechanism and the exact mechanism biohack2 needs for "how close to the redose window am I" and "is this value in range". It is the highest-value shared widget.

**Independent Test**: A gauge driven from 0 → 100% passes through the theme's ramp stops in order, identically in terminal and browser.

**Acceptance Scenarios**:

1. **Given** a value and a theme ramp, **When** a gauge renders, **Then** its colour is interpolated from the ramp at that value.
2. **Given** severity bands (safe/elevated/high), **When** a value crosses a band, **Then** the band change is visible in colour and in a text label (not colour alone).

---

### User Story C5 - Dashboard as an Optional View, Not the Shell (Priority: P3)

The dashboard is one view in the app. Logging, history, notes, stacks, and settings remain native web UI. The dashboard never replaces a form.

**Why this priority**: Guardrail. It encodes the decision that the terminal aesthetic is for reading, and prevents the project from building a worse form UI in a grid.

**Independent Test**: Every logging, editing, and import flow is reachable and usable in the web UI without any terminal-style panel.

**Acceptance Scenarios**:

1. **Given** a user needs to log a dose, **When** they use the app, **Then** the flow is the existing web form, not a grid panel.
2. **Given** the dashboard is disabled by the user, **When** they use the app, **Then** every feature remains available.

---

### Edge Cases

- **Two WASM bundles is a failure mode.** A separate browser app cannot share Rust memory with the app's SQLite/OPFS store, so it would need a server round-trip or a duplicated storage stack. The architecture must be one bundle.
- **Two ratatui copies is a compile-time trap.** If the dashboard crate and the renderer depend on different ratatui versions, trait bounds fail. The dashboard crate must re-export the ratatui it uses and consumers must use that path.
- **Small terminal on a phone.** The Fold3 outer screen and container terminals are small and often 256-color. The layout system needs a documented minimum size and a degraded mode.
- **Accessibility.** A terminal grid rendered as DOM is a wall of spans; screen readers get noise. Forms stay semantic HTML. The dashboard must be labelled as a visual panel and its critical information must also be available as text.
- **Bundle size.** The current WASM is ~1.49 MB (wasm-opt'd). A renderer plus a TUI framework is a real increase on a phone. Size is a measured acceptance criterion, not a hope.
- **Focus and input.** A grid panel inside a page competes with the page for keyboard focus. Focus must be explicit and escapable.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-070**: The system MUST provide a **terminal dashboard** rendering active doses, vitals trends, alerts, and recent entries from the local database, keyboard-driven, with no browser dependency.
- **FR-071**: The system MUST provide a **single widget codebase** for dashboard panels, compiled for the terminal renderer and for the in-browser renderer **without duplication of panel logic**.
- **FR-072**: The in-browser dashboard MUST render **inside the existing web application bundle**, sharing the app's data store in-process. A second WASM bundle for the dashboard is not permitted in v1.
- **FR-073**: The system MUST define **one palette** as the source of truth, consumed by the terminal renderer, the in-browser dashboard, and the app's CSS custom properties.
- **FR-074**: The system MUST accept **btop-format theme files** (`theme[key]="#rrggbb"`) and map them to the palette without manual translation.
- **FR-075**: The system MUST implement **threshold-indexed colour ramps** (value → colour) and severity bands, and MUST convey severity by a non-colour channel as well (label/glyph).
- **FR-076**: The dashboard MUST define a **minimum supported terminal size** and a documented degraded layout below it.
- **FR-077**: The dashboard MUST render correctly with **256-color** capability and MUST NOT depend on truecolor.
- **FR-078**: The dashboard MUST update on new data (log, vitals, alert) without user action, in both renderers.
- **FR-079**: The web application MUST retain **semantic HTML forms** for all data entry; the dashboard MUST NOT be the only path to any logging, editing, import, export, or settings flow.
- **FR-080**: The dashboard view MUST be **user-disableable** with no loss of functionality.
- **FR-081**: The dashboard crate MUST be usable **without** any web dependency, so the CLI builds with the web feature off.
- **FR-082**: The in-browser dashboard MUST be **lazy-loadable** (separately fetched code) if the measured bundle cost exceeds the agreed budget.
- **FR-083**: Keyboard focus in the embedded dashboard MUST be explicit, visible, and escapable, and MUST NOT trap the user out of the page's normal controls.

### Key Entities

- **Theme**: named palette; semantic colour roles (not raw hex in widget code); ramp definitions; severity band definitions; provenance (built-in | btop file | user file).
- **Panel**: a named widget with a data contract (e.g. `ActiveDosePanel`, `VitalsTrendPanel`, `AlertPanel`, `RecentEntriesPanel`), implemented once.
- **LayoutSpec**: panel grid with size classes (full / compact / minimal) for terminal and web layout.
- **RenderTarget**: `terminal` (crossterm) | `web-dom` (in-app renderer). Chosen at runtime; panel code is target-agnostic.

## Success Criteria *(mandatory)*

- **SC-070**: `biohack2 dash` shows live data within 2 s of launch on a populated database.
- **SC-071**: The same panel code renders in terminal and in browser with **zero duplicated panel logic** (verified by structure: one panel module, two render targets).
- **SC-072**: Applying one theme file changes all three surfaces; loading a btop theme requires no manual colour translation.
- **SC-073**: The dashboard is legible at the documented minimum terminal size and in 256-color mode.
- **SC-074**: The web app's WASM size increase from the embedded dashboard is **measured and recorded** before the feature is accepted, and stays within the agreed budget or ships lazy-loaded.
- **SC-075**: A dose logged in the web app appears in the embedded dashboard in under 1 s with no network request.
- **SC-076**: All existing web app flows remain usable with the dashboard disabled.

## Design Decisions (ratified 2026-10-11)

| Decision | Choice | Basis |
|---|---|---|
| Web UI role | Legit web UI is primary; dashboard is a view | Forms need touch, autocomplete, file import; a grid is poor at writing |
| Dashboard renderer | `ratatui` | Ecosystem standard; 58.7M downloads; MIT |
| In-browser rendering | `ratzilla` **as a `Backend` inside the existing bundle** | ratzilla's example is `Terminal::new(backend)`, so its backends implement ratatui's `Backend` trait → panel code is target-agnostic and one bundle is possible |
| ratzilla backend | `DomBackend` | Needs clicks, CSS integration, text selection, all-browser support; WebGL2 lacks `SingleClick` and wants the viewport |
| Build pipeline | Existing `wasm-pack` + `wasm-opt` (`build-web.sh`) | `trunk` as a second pipeline is not adopted; one artifact, one PWA, one service worker |
| Palette source | `web/src/styles/global.css` tokens → theme file → all surfaces | One palette, three renderers |
| btop code | **Not ported** | btop is a string painter (`createBox`/`Meter`/`Graph` return `String`); ratatui is a buffer painter. Porting means discarding btop's renderer. Its UI primitives already exist in `ratatui-widgets`. What is taken from btop is design: theme schema, threshold ramps, title-in-border, degraded mode |
| Shared library | Extract a dashboard/theme crate **after** the first working panel | Every abandoned crate in this genre was designed without a demanding consumer |

## Assumptions

- `ratzilla` is MIT OR Apache-2.0 (verified 2026-10-11) and the workspace is MIT OR Apache-2.0 (verified) — licence-compatible.
- `ratzilla` depends on `ratatui ^0.30.1`; the workspace uses 0.30.2 — caret-compatible.
- `ratzilla` re-exports its ratatui (`pub use ratatui`); dashboard code uses that path to prevent duplicate-crate trait mismatches.
- `ratzilla` pins `thiserror ^2` while the workspace pins `thiserror = "1.0"`; the workspace moves to 2.
- **Unverified, must be proven before commitment**: that the chosen backend can render into a specific container element (not `document.body`) and that manual `terminal.draw()` coexists cleanly with the app's own render loop. This is the hinge of FR-072 and is a ~30-line spike.

## Out of Scope for v1

- A standalone browser "terminal app" as a second bundle.
- Native mobile terminal UI.
- Mouse-driven dashboard interaction beyond click-to-open; the terminal is keyboard-first.
- Pixel-identical rendering between terminal and browser. The goal is a shared visual language, shared palette, shared panels — not identical pixels.
- Terminal-based data entry forms.
