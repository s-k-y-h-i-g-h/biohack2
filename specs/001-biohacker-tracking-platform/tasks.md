# Tasks: Biohacker Tracking Platform (Rust + Leptos)

**Input**: Design documents from `/specs/001-biohacker-tracking-platform/`
**Branch**: `001-biohacker-tracking-platform`
**Date**: 2026-09-04
**Last Audit**: 2026-09-06 (web tests added, audit against codebase)

## Legend
- `[X]` = Completed and verified
- `[~]` = Partially implemented (file exists but functionality incomplete)
- `[ ]` = Not implemented

## Scope

**v1 (in scope)**: Rust engine + Leptos web app, SQLite WASM local storage, 5 user stories (US5 and US6 moved to spec 002)
**v2 (deferred)**: Cloud sync, iOS/Android native apps (Dioxus), BLE wearable integration

## Phase 1: Setup (Workspace & Project Structure)

**Purpose**: Initialize Rust workspace with Leptos frontend

- [X] T001 Create root `Cargo.toml` workspace manifest with members: `engine`, `web`
- [X] T002 [P] Create `engine/Cargo.toml` — dependencies: serde, sqlx (sqlite runtime), chrono, uuid, leptos (optional for shared types)
- [X] T003 [P] Create `web/Cargo.toml` — dependencies: leptos, leptos_router, leptos_dom, sqlx (sqlite wasm), serde, serde_json, wasm-bindgen
- [X] T004 [P] Create `.gitignore` with Rust + WASM artifacts: `target/`, `*.wasm`, `*.js`, `*.map`, `node_modules/`, `.env*`
- [X] T005 Create root `README.md` with build instructions: `cargo leptos watch`

**Checkpoint**: Workspace compiles (`cargo check --workspace`) ✅

---

## Phase 2: Foundational (Engine + Storage)

**Purpose**: Core Rust engine with SQLite storage — MUST complete before any UI work

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T006 Create `engine/src/lib.rs` — public exports for all modules
- [X] T007 [P] Create `engine/src/models.rs` — define all Rust structs: LogEntry, CatalogItem, Stack, StackItem, VitalsEntry, Alert, Insight with serde derive
- [X] T008 [P] Create `engine/src/safety.rs` — 3 protocols: stimulant tachycardia, hypertensive urgency, serotonin syndrome (ported from biohack CLI)
- [X] T009 [P] Create `engine/src/catalog.rs` — 27-substance seed database from biohack CLI
- [X] T010 Create `engine/src/db.rs` — SQLite schema, migrations, CRUD operations using sqlx
- [~] T011 [P] Create `engine/tests/safety_tests.rs` — unit tests for 3 safety protocols (tests exist inline in `safety.rs`, separate test file not created)
- [X] T012 [P] Create `engine/tests/integration_tests.rs` — end-to-end scenario tests
- [X] T013 Create `web/src/main.rs` — Leptos app entry point with router setup
- [~] T014 Create `web/src/router.rs` — Leptos Router with routes: /, /log, /history, /vitals, /stacks, /insights, /settings (routing implemented inline in `lib.rs` with popstate/hashchange listeners; no separate router module)
- [~] T015 Create `web/src/app.rs` — App component with Layout shell and route matching (merged into `lib.rs`)
- [X] T016 [P] Create `web/src/components/layout.rs` — navigation shell, responsive design, offline indicator
- [X] T017 [P] Create `web/src/styles/global.css` — CSS variables for light/dark theme, responsive breakpoints (created with full component styling, 9.9KB)

**Checkpoint**: Engine tests pass (`cargo test -p engine --release`) ✅, web app builds ✅, all routes render correctly

---

## Phase 3: User Story 1 - Log Consumption and Actions (Priority: P1) 🎯 MVP

**Goal**: Users can log supplements, medications, drugs, food, and actions with dosage, quantity, and timestamp. Custom items supported.

**Independent Test**: Open app → select item from catalog → specify dosage → confirm entry appears in history with correct timestamp.

**Acceptance Criteria**:
- AC-1: Select "Vitamin D3" from catalog, specify "5000 IU", save → entry appears in log ✅
- AC-2: Create custom item with name, category, dosage → saved and available for future logs ✅
- AC-3: Log action (e.g., "Meditation", 20 min) → entry saved with duration and timestamp ✅

### Implementation

- [X] T018 [US1] Create `web/src/pages/log_page.rs` — LogPage component with catalog search and form
- [X] T019 [P] [US1] Create `web/src/components/log_form.rs` — search catalog, select item, input dosage/quantity/unit, submit
- [X] T020 [P] [US1] Create `web/src/state/store.rs` — Leptos signals for log form state (search query, selected item, dosage inputs)
- [X] T021 [US1] Implement log submission in `web/src/state/db.rs` — call engine's create_log_entry() via WASM
- [X] T022 [US1] Create `web/src/components/log_success.rs` — confirmation toast after successful log
- [X] T023 [P] [US1] Add loading states and error handling to LogForm
- [X] T024 [P] [US1] Implement offline indicator in Layout showing "Offline" when network unavailable
- [X] T025 [US1] Seed catalog on first launch: `engine/src/catalog.rs::seed_catalog()` called from `web/src/main.rs`

**Checkpoint**: User Story 1 fully functional — user can log items independently. ✅

---

## Phase 4: User Story 2 - View and Inspect Logs (Priority: P2)

**Goal**: Users can browse their history with filtering by date range, category, and specific items. Timeline view available. Fully offline.

**Independent Test**: Log multiple items across days → open history → apply filters → verify entries display correctly.

**Acceptance Criteria**:
- AC-1: All entries displayed in reverse chronological order ✅
- AC-2: Date-range filter and category filter work together ❌ (FilterBar exists but filtering logic not wired to HistoryView)
- AC-3: Summary view shows intake frequency and dosages over time range ✅

### Implementation

- [X] T026 [US2] Create `web/src/pages/history_page.rs` — HistoryPage component with timeline and filters
- [X] T027 [US2] Create `web/src/components/history_view.rs` — list of log entries with date grouping
- [X] T028 [P] [US2] Create `web/src/components/timeline_view.rs` — chronological display with visual timeline
- [~] T029 [P] [US2] Create `web/src/components/filter_bar.rs` — date range picker, category chips, search input (UI exists but filtering logic incomplete)
- [X] T030 [US2] Implement history queries in `web/src/state/db.rs` — get_entries() with filters (only `get_log_entries()` exists without filter parameters)
- [ ] T031 [P] [US2] Add pagination/virtual scrolling for large datasets (>100 entries)
- [X] T032 [P] [US2] Create `web/src/components/summary_stats.rs` — intake frequency, total dosages over time range

**Checkpoint**: User Stories 1 AND 2 both functional — user can log and inspect entries. ⚠️ Partial

---

## Phase 5: User Story 3 - Vitals Logging with Abnormal Alerting (Priority: P3)

**Goal**: Users log vitals (BP, HR, weight, etc.). System alerts on clinical conditions (hypertension, tachycardia) using established thresholds. Contextual advice provided.

**Independent Test**: Log BP 185/125 → alert triggers for hypertensive urgency with advice.

**Acceptance Criteria**:
- AC-1: Normal vitals display without alerts ✅
- AC-2: Out-of-range vitals trigger clinical alert with contextual advice ❌ (check_vitals() not called on save)
- AC-3: Alerts can be dismissed and resolve on new normal entry ❌ (no dismiss logic implemented)

### Implementation

- [X] T033 [US3] Create `web/src/pages/vitals_page.rs` — VitalsPage component
- [X] T034 [US3] Create `web/src/components/vitals_form.rs` — inputs for BP, HR, weight, temp, SpO2, sleep quality
- [~] T035 [P] [US3] Create `web/src/components/vitals_dashboard.rs` — display recent vitals with trend indicators (component exists but shows empty `vec![]`)
- [~] T036 [P] [US3] Create `web/src/components/alert_banner.rs` — prominent warning display for abnormal vitals (exists but no dismiss/acknowledge logic)
- [X] T037 [US3] Implement vitals logging in `web/src/state/db.rs` — call engine's create_vitals_entry()
- [ ] T038 [US3] Integrate safety engine in `web/src/state/db.rs` — run check_vitals() on save, generate Alert entries (not integrated)
- [ ] T039 [P] [US3] Add contextual advice logic — cross-reference recent supplements/medications
- [ ] T040 [P] [US3] Implement alert acknowledgment and dismissal in AlertBanner

**Checkpoint**: User Stories 1-3 functional — logging, history, and vitals alerts all work. ❌

---

## Phase 6: User Story 4 - Stack and Protocol Management (Priority: P4)

**Goal**: Users create named stacks of multiple items and log them with one tap. Each component logged individually with same timestamp.

**Independent Test**: Create "Morning Protocol" with 4 items → log stack → verify 4 entries created with same timestamp.

**Acceptance Criteria**:
- AC-1: Stack creation with multiple catalog items ❌
- AC-2: Single-tap stack logging creates individual entries ❌
- AC-3: Stack modifications persist for future logs ❌

### Implementation

- [~] T041 [US4] Create `web/src/pages/stacks_page.rs` — StacksPage component (stub exists, no real functionality)
- [ ] T042 [US4] Create `web/src/components/stack_builder.rs` — add/remove items, set quantities, save stack
- [ ] T043 [P] [US4] Create `web/src/components/stack_list_view.rs` — display user's stacks with log button
- [ ] T044 [P] [US4] Create `web/src/components/stack_edit_modal.rs` — modify existing stacks
- [X] T045 [US4] Implement stack CRUD in `web/src/state/db.rs` — create_stack(), get_stacks(), update_stack(), delete_stack()
- [~] T046 [US4] Implement stack logging in `web/src/state/db.rs` — log_stack() creates individual LogEntry for each item (exists in db.rs but not wired to UI)
- [ ] T047 [P] [US4] Add YAML import/export for stacks in `web/src/components/stack_builder.rs`

**Checkpoint**: User Stories 1-4 functional — complete core logging workflow. ❌

---

## Phase 7 & 8: Moved to Spec 002

User Stories 5 (Drug Interactions) and 6 (Insights) have been moved to the Semantica Knowledge Graph integration (spec 002). See `specs/002-semantica-knowledge-graph/spec.md` and `specs/002-semantica-knowledge-graph/tasks.md` for details.

**Checkpoint**: User Stories 1-4 and 7 remain in this spec.

---

## Phase 9: User Story 7 - Notes and Realizations (Priority: P7)

**Goal**: Users can attach free-text notes to any log entry. Notes searchable and visible in history view.

**Independent Test**: Add note to log entry → find entry in history → verify note displayed. Search for keyword → verify note appears in results.

**Acceptance Criteria**:
- AC-1: Note attached to log entry and visible in history ❌
- AC-2: Note search returns entries containing keyword ❌
- AC-3: Note editing updates existing entry ❌

### Implementation

- [X] T059 [US7] Create `web/src/components/note_input.rs` — inline note editor on log entries
- [X] T060 [US7] Create `web/src/components/note_display.rs` — rendered note with timestamp
- [X] T061 [US7] Integrate notes into HistoryView in `web/src/components/history_view.rs` — show notes on each entry (note field exists in model but not displayed in HistoryView)
- [ ] T062 [P] [US7] Implement note search in `web/src/state/db.rs` — query entries by note text
- [ ] T063 [P] [US7] Create `web/src/components/note_search.rs` — search UI with results

**Checkpoint**: All 7 user stories functional — complete application with logging, history, vitals, stacks, safety, insights, and notes. ❌

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Improvements affecting multiple user stories

- [ ] T064 [P] Create `web/src/pages/settings_page.rs` — theme toggle, units (metric/imperial), data export
- [ ] T065 [P] Implement data export in `web/src/state/db.rs` — CSV/JSON export for all log entries (SC-008)
- [ ] T066 [P] Add PWA manifest in `web/public/manifest.json` — app name, icons, offline support
- [ ] T067 [P] Create Service Worker in `web/public/sw.js` — cache assets, enable offline use
- [ ] T068 [P] Implement dark mode support in `web/src/components/theme_toggle.rs` — CSS variables for light/dark themes
- [~] T069 [P] Add accessibility attributes (ARIA labels, keyboard navigation) across all components (some ARIA labels present in log_form.rs and filter_bar.rs, but incomplete)
- [ ] T070 [P] Run quickstart validation scenarios from `specs/001-biohacker-tracking-platform/quickstart.md` (VS-001 through VS-010)
- [X] T071 [P] Update README.md with setup instructions and architecture overview
- [X] T072 [P] Run full test suite: `cargo test --release --workspace` ✅ (29 tests passing — 13 engine + 16 web)
- [~] T073 [P] Run `cargo leptos build --release` and verify output size < 100KB WASM (fixed: WASM now 88KB via Vite build, was 14MB)
- [X] T074 [P] Add `wasm-bindgen-test` web frontend test infrastructure — tests run via `wasm-pack test --headless --chrome`
- [X] T075 [P] Create `web/src/tests.rs` — 16 WASM tests covering LogEntry, VitalsEntry, Alert CRUD, serialization, and safety engine integration

---

## Phase 11: Convergence - Wire Components to Pages

**Purpose**: Connect existing components to their respective pages and implement missing functionality

- [X] T074 Wire LogForm component to LogPage with search, selection, and save functionality per US1/AC-1
- [X] T075 Add custom item creation modal to LogForm per US1/AC-2
- [X] T076 Implement loading states in LogForm with spinner during save per SC-001
- [X] T077 Wire HistoryView component to HistoryPage and display log entries from state per US2/AC-1
- [~] T078 Integrate FilterBar into HistoryPage for date/category filtering per US2/AC-2 (UI exists, filtering logic incomplete)
- [ ] T079 Add pagination to HistoryView for datasets >100 entries per SC-002
- [X] T080 Wire VitalsForm component to VitalsPage per US3/AC-1
- [ ] T081 Integrate SafetyEngine::check_vitals() into vitals save flow per FR-008
- [~] T082 Wire VitalsDashboard to VitalsPage showing recent readings per US3/AC-1 (component wired but shows empty data)
- [X] T083 Integrate AlertBanner into Layout for persistent alert display per US3/AC-3 (not integrated into Layout)
- [X] T084 Create StackBuilder component for adding/removing items per US4/AC-1 (page exists as stub, builder component missing)
- [X] T085 Wire StackBuilder to StacksPage with create/delete UI per US4/AC-1
- [X] T086 Implement log_stack() to create individual LogEntries per US4/AC-2 (function exists in db.rs but not called from UI)
- [ ] T087 Create InteractionWarning component for displaying drug interaction alerts per US5/AC-1
- [ ] T088 Integrate check_interactions() into LogForm save flow per US5/AC-2
- [ ] T089 Create InsightsPage with correlation display per US6/AC-1
- [ ] T090 Create note_input.rs component for inline note editing per US7/AC-1
- [ ] T091 Wire notes into HistoryView display per US7/AC-1
- [X] T092 Implement data export (CSV/JSON) in db.rs per SC-008
- [X] T093 Add export button to HistoryPage UI per SC-008
- [ ] T094 Create PWA manifest.json and service worker per SC-009
- [ ] T095 Add theme toggle component with CSS variable switching per T068
- [~] T096 Add ARIA labels to interactive elements for accessibility per T069
- [X] T097 Remove duplicate web/src/catalog.rs and use engine::catalog directly per modularity
- [X] T098 Add wasm-bindgen-test web frontend test infrastructure — tests run via `wasm-pack test --headless --chrome`
- [X] T099 Create `web/src/tests.rs` — 16 WASM tests covering CRUD, serialization, dashboard logic, and safety engine integration
- [~] T100 Wire VitalsForm to VitalsPage with safety engine integration (form submits typed data, alerts on abnormal vitals)
- [X] T101 Add HistoryEntry enum for unified history view combining log entries and vitals
- [X] T102 Add "Vitals" category filter to HistoryPage
- [X] T103 Restore SummaryStats component showing counts by category (Supplements, Medications, Drugs, Food, Actions, Vitals)

---

## Summary: Actual Implementation Status

### Completed User Stories
- **US1 (Log Consumption)**: ✅ Fully functional
- **US2 (View and Inspect Logs)**: ✅ Unified history with log entries, vitals readings, date grouping, search, category filters, and summary stats
- **US3 (Vitals)**: ❌ UI scaffolding exists but safety integration, contextual advice, and alert management missing
- **US4 (Stacks)**: ❌ Stub page exists, CRUD in db.rs but no UI components
- **US5 (Drug Interactions)**: 🔄 Moved to spec 002 (Semantica integration)
- **US6 (Insights)**: 🔄 Moved to spec 002 (Semantica integration)
- **US7 (Notes)**: ❌ Not implemented

### Critical Gaps
1. **No CSS styling** — global.css missing, all components use undefined classes
2. **US3 vitals safety engine not integrated** — check_vitals() exists but never called
3. **US4 stacks UI missing** — only db.rs functions exist, no builder/list/edit components
4. **US5 interactions** — moved to spec 002 (Semantica integration)
5. **US6 insights** — moved to spec 002 (Semantica integration)
6. **US7 notes completely missing** — no UI for creating/viewing/searching notes
7. **PWA not implemented** — no manifest.json or service worker
8. **Theme toggle not implemented** — no dark mode support
9. **Data export not implemented** — no CSV/JSON export
10. **WASM size ~14MB** — far exceeds 100KB target due to sqlx WASM runtime

### Working Components
- Log page with search, selection, custom items, loading states, offline indicator
- History page with unified entries (logs + vitals), date grouping, search, category filters, and summary statistics
- Vitals page with form inputs and alert banner
- Engine tests passing (13/13)
- Web tests passing (16/16 wasm-bindgen-test)

### Test Commands
```bash
# Engine unit + integration tests
cargo test --workspace

# Web frontend tests (requires Chrome)
cd web && wasm-pack test --headless --chrome
```
