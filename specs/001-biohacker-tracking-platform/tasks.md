# Tasks: Biohacker Tracking Platform (Rust + Leptos)

**Input**: Design documents from `/specs/001-biohacker-tracking-platform/`
**Branch**: `001-biohacker-tracking-platform`
**Date**: 2026-09-04

## Scope

**v1 (in scope)**: Rust engine + Leptos web app, SQLite WASM local storage, 7 user stories
**v2 (deferred)**: Cloud sync, iOS/Android native apps (Dioxus), BLE wearable integration

## Phase 1: Setup (Workspace & Project Structure)

**Purpose**: Initialize Rust workspace with Leptos frontend

- [X] T001 Create root `Cargo.toml` workspace manifest with members: `engine`, `web`
- [X] T002 [P] Create `engine/Cargo.toml` — dependencies: serde, sqlx (sqlite runtime), chrono, uuid, leptos (optional for shared types)
- [X] T003 [P] Create `web/Cargo.toml` — dependencies: leptos, leptos_router, leptos_dom, sqlx (sqlite wasm), serde, serde_json, wasm-bindgen
- [X] T004 [P] Create `.gitignore` with Rust + WASM artifacts: `target/`, `*.wasm`, `*.js`, `*.map`, `node_modules/`, `.env*`
- [X] T005 Create root `README.md` with build instructions: `cargo leptos watch`

**Checkpoint**: Workspace compiles (`cargo check --workspace`)

---

## Phase 2: Foundational (Engine + Storage)

**Purpose**: Core Rust engine with SQLite storage — MUST complete before any UI work

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T006 Create `engine/src/lib.rs` — public exports for all modules
- [X] T007 [P] Create `engine/src/models.rs` — define all Rust structs: LogEntry, CatalogItem, Stack, StackItem, VitalsEntry, Alert, Insight with serde derive
- [X] T008 [P] Create `engine/src/safety.rs` — 3 protocols: stimulant tachycardia, hypertensive urgency, serotonin syndrome (ported from biohack CLI)
- [X] T009 [P] Create `engine/src/catalog.rs` — 27-substance seed database from biohack CLI
- [X] T010 Create `engine/src/db.rs` — SQLite schema, migrations, CRUD operations using sqlx
- [X] T011 [P] Create `engine/tests/safety_tests.rs` — unit tests for 3 safety protocols
- [X] T012 [P] Create `engine/tests/integration_tests.rs` — end-to-end scenario tests
- [X] T013 Create `web/src/main.rs` — Leptos app entry point with router setup
- [X] T014 Create `web/src/router.rs` — Leptos Router with routes: /, /log, /history, /vitals, /stacks, /insights, /settings
- [X] T015 Create `web/src/app.rs` — App component with Layout shell and route matching
- [X] T016 [P] Create `web/src/components/layout.rs` — navigation shell, responsive design, offline indicator
- [X] T017 [P] Create `web/src/styles/global.css` — CSS variables for light/dark theme, responsive breakpoints

**Checkpoint**: Engine tests pass (`cargo test -p engine --release`), web app builds (`cargo leptos build`)

---

## Phase 3: User Story 1 - Log Consumption and Actions (Priority: P1) 🎯 MVP

**Goal**: Users can log supplements, medications, drugs, food, and actions with dosage, quantity, and timestamp. Custom items supported.

**Independent Test**: Open app → select item from catalog → specify dosage → confirm entry appears in history with correct timestamp.

**Acceptance Criteria**:
- AC-1: Select "Vitamin D3" from catalog, specify "5000 IU", save → entry appears in log
- AC-2: Create custom item with name, category, dosage → saved and available for future logs
- AC-3: Log action (e.g., "Meditation", 20 min) → entry saved with duration and timestamp

### Implementation

- [X] T018 [US1] Create `web/src/pages/log_page.rs` — LogPage component with catalog search and form
- [X] T019 [P] [US1] Create `web/src/components/log_form.rs` — search catalog, select item, input dosage/quantity/unit, submit
- [X] T020 [P] [US1] Create `web/src/state/store.rs` — Leptos signals for log form state (search query, selected item, dosage inputs)
- [X] T021 [US1] Implement log submission in `web/src/state/db.rs` — call engine's create_log_entry() via WASM
- [X] T022 [US1] Create `web/src/components/log_success.rs` — confirmation toast after successful log
- [X] T023 [P] [US1] Add loading states and error handling to LogForm
- [X] T024 [P] [US1] Implement offline indicator in Layout showing "Offline" when network unavailable
- [X] T025 [US1] Seed catalog on first launch: `engine/src/catalog.rs::seed_catalog()` called from `web/src/main.rs`

**Checkpoint**: User Story 1 fully functional — user can log items independently.

---

## Phase 4: User Story 2 - View and Inspect Logs (Priority: P2)

**Goal**: Users can browse their history with filtering by date range, category, and specific items. Timeline view available. Fully offline.

**Independent Test**: Log multiple items across days → open history → apply filters → verify entries display correctly.

**Acceptance Criteria**:
- AC-1: All entries displayed in reverse chronological order
- AC-2: Date-range filter and category filter work together
- AC-3: Summary view shows intake frequency and dosages over time

### Implementation

- [X] T026 [US2] Create `web/src/pages/history_page.rs` — HistoryPage component with timeline and filters
- [X] T027 [US2] Create `web/src/components/history_view.rs` — list of log entries with date grouping
- [X] T028 [P] [US2] Create `web/src/components/timeline_view.rs` — chronological display with visual timeline
- [X] T029 [P] [US2] Create `web/src/components/filter_bar.rs` — date range picker, category chips, search input
- [X] T030 [US2] Implement history queries in `web/src/state/db.rs` — get_entries() with filters
- [X] T031 [P] [US2] Add pagination/virtual scrolling for large datasets (>100 entries)
- [X] T032 [P] [US2] Create `web/src/components/summary_stats.rs` — intake frequency, total dosages over time range

**Checkpoint**: User Stories 1 AND 2 both functional — user can log and inspect entries.

---

## Phase 5: User Story 3 - Vitals Logging with Abnormal Alerting (Priority: P3)

**Goal**: Users log vitals (BP, HR, weight, etc.). System alerts on clinical conditions (hypertension, tachycardia) using established thresholds. Contextual advice provided.

**Independent Test**: Log BP 185/125 → alert triggers for hypertensive urgency with advice.

**Acceptance Criteria**:
- AC-1: Normal vitals display without alerts
- AC-2: Out-of-range vitals trigger clinical alert with contextual advice
- AC-3: Alerts can be dismissed and resolve on new normal entry

### Implementation

- [X] T033 [US3] Create `web/src/pages/vitals_page.rs` — VitalsPage component
- [X] T034 [US3] Create `web/src/components/vitals_form.rs` — inputs for BP, HR, weight, temp, SpO2, sleep quality
- [X] T035 [P] [US3] Create `web/src/components/vitals_dashboard.rs` — display recent vitals with trend indicators
- [X] T036 [P] [US3] Create `web/src/components/alert_banner.rs` — prominent warning display for abnormal vitals
- [X] T037 [US3] Implement vitals logging in `web/src/state/db.rs` — call engine's create_vitals_entry()
- [X] T038 [US3] Integrate safety engine in `web/src/state/db.rs` — run check_vitals() on save, generate Alert entries
- [X] T039 [P] [US3] Add contextual advice logic — cross-reference recent supplements/medications
- [X] T040 [P] [US3] Implement alert acknowledgment and dismissal in AlertBanner

**Checkpoint**: User Stories 1-3 functional — logging, history, and vitals alerts all work.

---

## Phase 6: User Story 4 - Stack and Protocol Management (Priority: P4)

**Goal**: Users create named stacks of multiple items and log them with one tap. Each component logged individually with same timestamp.

**Independent Test**: Create "Morning Protocol" with 4 items → log stack → verify 4 entries created with same timestamp.

**Acceptance Criteria**:
- AC-1: Stack creation with multiple catalog items
- AC-2: Single-tap stack logging creates individual entries
- AC-3: Stack modifications persist for future logs

### Implementation

- [X] T041 [US4] Create `web/src/pages/stacks_page.rs` — StacksPage component
- [X] T042 [US4] Create `web/src/components/stack_builder.rs` — add/remove items, set quantities, save stack
- [X] T043 [P] [US4] Create `web/src/components/stack_list_view.rs` — display user's stacks with log button
- [X] T044 [P] [US4] Create `web/src/components/stack_edit_modal.rs` — modify existing stacks
- [X] T045 [US4] Implement stack CRUD in `web/src/state/db.rs` — create_stack(), get_stacks(), update_stack(), delete_stack()
- [X] T046 [US4] Implement stack logging in `web/src/state/db.rs` — log_stack() creates individual LogEntry for each item
- [X] T047 [P] [US4] Add YAML import/export for stacks in `web/src/components/stack_builder.rs`

**Checkpoint**: User Stories 1-4 functional — complete core logging workflow.

---

## Phase 7: User Story 5 - Drug Interaction Safety Alerts (Priority: P5)

**Goal**: System automatically checks for dangerous drug/supplement interactions when logging. Prominent warnings require acknowledgment.

**Independent Test**: Log Aspirin → attempt to log Ibuprofen → warning displays before save.

**Acceptance Criteria**:
- AC-1: Dangerous interaction detected and warning displayed
- AC-2: User must acknowledge warning before saving
- AC-3: Interaction check completes within 3 seconds

### Implementation

- [X] T048 [US5] Create `web/src/components/interaction_warning.rs` — prominent warning UI with risk description
- [X] T049 [US5] Integrate safety checks into LogForm in `web/src/components/log_form.rs` — run check_interactions() before save
- [X] T050 [P] [US5] Add interaction acknowledgment tracking in LogEntry model (already exists as `acknowledged_interaction`)
- [X] T051 [P] [US5] Run Rust safety engine tests: `cargo test --release -p engine -- safety_tests`
- [X] T052 [P] [US5] Add benchmark dataset for 90% interaction flagging accuracy in `engine/tests/`

**Checkpoint**: Safety-critical interactions are detected and warnings displayed.

---

## Phase 8: User Story 6 - Insights and Analysis (Priority: P6)

**Goal**: Users can view correlations between their supplements/actions and vital sign changes over time. Dashboard surfaces actionable insights.

**Independent Test**: Log 14+ days of correlated data → open insights dashboard → verify correlations displayed with supporting data points.

**Acceptance Criteria**:
- AC-1: Dashboard shows ≥1 correlation after 2+ weeks of data
- AC-2: Each insight includes confidence score and supporting data point count
- AC-3: Clicking insight shows contributing log entries

### Implementation

- [X] T053 [US6] Create `web/src/pages/insights_page.rs` — InsightsPage component
- [X] T054 [US6] Create `web/src/components/insights_feed.rs` — list of generated insights
- [X] T055 [P] [US6] Create `web/src/components/correlation_card.rs` — individual insight display with confidence meter
- [X] T056 [US6] Implement insights service in `engine/src/insights.rs` — correlation engine, trend analysis
- [X] T057 [P] [US6] Add "insufficient data" empty state in InsightsFeed when <7 overlapping points
- [X] T058 [P] [US6] Wire up click-to-detail in CorrelationCard showing contributing log entries

**Checkpoint**: User Stories 1-6 functional — user can log, inspect, view vitals, manage stacks, see safety alerts, and get insights.

---

## Phase 9: User Story 7 - Notes and Realizations (Priority: P7)

**Goal**: Users can attach free-text notes to any log entry. Notes searchable and visible in history view.

**Independent Test**: Add note to log entry → find entry in history → verify note displayed. Search for keyword → verify note appears in results.

**Acceptance Criteria**:
- AC-1: Note attached to log entry and visible in history
- AC-2: Note search returns entries containing keyword
- AC-3: Note editing updates existing entry

### Implementation

- [X] T059 [US7] Create `web/src/components/note_input.rs` — inline note editor on log entries
- [X] T060 [US7] Create `web/src/components/note_display.rs` — rendered note with timestamp
- [X] T061 [US7] Integrate notes into HistoryView in `web/src/components/history_view.rs` — show notes on each entry
- [X] T062 [P] [US7] Implement note search in `web/src/state/db.rs` — query entries by note text
- [X] T063 [P] [US7] Create `web/src/components/note_search.rs` — search UI with results

**Checkpoint**: All 7 user stories functional — complete application with logging, history, vitals, stacks, safety, insights, and notes.

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Improvements affecting multiple user stories

- [X] T064 [P] Create `web/src/pages/settings_page.rs` — theme toggle, units (metric/imperial), data export
- [X] T065 [P] Implement data export in `web/src/state/db.rs` — CSV/JSON export for all log entries (SC-008)
- [X] T066 [P] Add PWA manifest in `web/public/manifest.json` — app name, icons, offline support
- [X] T067 [P] Create Service Worker in `web/public/sw.js` — cache assets, enable offline use
- [X] T068 [P] Implement dark mode support in `web/src/components/theme_toggle.rs` — CSS variables for light/dark themes
- [X] T069 [P] Add accessibility attributes (ARIA labels, keyboard navigation) across all components
- [X] T070 [P] Run quickstart validation scenarios from `specs/001-biohacker-tracking-platform/quickstart.md` (VS-001 through VS-010)
- [X] T071 [P] Update README.md with setup instructions and architecture overview
- [X] T072 [P] Run full test suite: `cargo test --release --workspace`
- [X] T073 [P] Run `cargo leptos build --release` and verify output size < 100KB WASM

---

## Dependencies & Execution Order

### Phase Dependencies
- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 — BLOCKS all user stories
- **Phase 3-9 (User Stories)**: All depend on Phase 2 completion
  - User stories can proceed in parallel (if team capacity allows)
  - Or sequentially in priority order (P1 → P2 → P3 → P4 → P5 → P6 → P7)
- **Phase 10 (Polish)**: Depends on all desired user stories being complete

### User Story Dependencies
- **US1 (P1)**: Can start after Phase 2 — no dependencies on other stories
- **US2 (P2)**: Can start after Phase 2 — integrates with US1 data models
- **US3 (P3)**: Can start after Phase 2 — uses LogEntry from US1, generates Alert
- **US4 (P4)**: Can start after Phase 2 — uses LogEntry from US1, creates Stack entries
- **US5 (P5)**: Can start after Phase 2 — integrates with US1 safety checks
- **US6 (P6)**: Can start after Phase 2 — uses LogEntry and VitalsEntry from US1/US3
- **US7 (P7)**: Can start after Phase 2 — extends LogEntry with notes

### Parallel Opportunities
- All Phase 1 tasks marked [P] can run in parallel
- All Phase 2 tasks marked [P] can run in parallel (within phase)
- Once Phase 2 completes, all user stories can start in parallel
- All [P] tasks within each story can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all parallel tasks for US1 together:
Task: "Create LogForm component in web/src/components/log_form.rs"
Task: "Create LogSuccess component in web/src/components/log_success.rs"
Task: "Add loading states and error handling to LogForm"
Task: "Implement offline indicator in Layout"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T005)
2. Complete Phase 2: Foundational (T006-T017)
3. Complete Phase 3: User Story 1 (T018-T025)
4. **STOP and VALIDATE**: Test US1 independently — log an item, verify it appears in history
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add US1 → Test independently → Deploy/Demo (MVP!)
3. Add US2 → Test independently → Deploy/Demo
4. Add US3 → Test independently → Deploy/Demo
5. Add US4 → Test independently → Deploy/Demo
6. Add US5 → Test independently → Deploy/Demo
7. Add US6 → Test independently → Deploy/Demo
8. Add US7 → Test independently → Deploy/Demo
9. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:
1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1
   - Developer B: User Story 2
   - Developer C: User Story 3
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence

---

## Total Task Count

- **Phase 1 (Setup)**: 5 tasks
- **Phase 2 (Foundational)**: 12 tasks
- **Phase 3 (US1)**: 8 tasks
- **Phase 4 (US2)**: 7 tasks
- **Phase 5 (US3)**: 8 tasks
- **Phase 6 (US4)**: 7 tasks
- **Phase 7 (US5)**: 5 tasks
- **Phase 8 (US6)**: 6 tasks
- **Phase 9 (US7)**: 5 tasks
- **Phase 10 (Polish)**: 10 tasks
- **Total**: 70 tasks
