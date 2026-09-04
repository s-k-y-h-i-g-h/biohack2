# Tasks: Biohacker Tracking Platform (v1)

**Input**: Design documents from `/specs/001-biohacker-tracking-platform/`
**Branch**: `001-biohacker-tracking-platform`
**Date**: 2026-09-04

## Scope

**v1 (in scope)**: Web application (TypeScript + SolidJS), local-first storage, 7 user stories
**v2 (deferred)**: Cloud sync, iOS/Android native apps, BLE wearable integration

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [ ] T001 Create monorepo root structure: package.json, tsconfig.json, vite.config.ts at repo root in `package.json`
- [ ] T002 [P] Initialize Rust workspace in `engine/Cargo.toml` with members: `engine`, `shared`
- [ ] T003 [P] Initialize web frontend in `web/package.json` with SolidJS, TypeScript, Vite dependencies
- [ ] T004 [P] Create shared types crate in `shared/src/lib.rs` — define LogEntry, CatalogItem, Stack, VitalsEntry, Alert, Insight structs using `serde`
- [ ] T005 Create `.gitignore` for Rust + Node.js artifacts in `.gitignore`
- [ ] T006 [P] Configure ESLint + Prettier for web frontend in `web/.eslintrc.cjs` and `web/.prettierrc`
- [ ] T007 [P] Configure Rust clippy + rustfmt in `engine/rustfmt.toml` and `.cargo/config.toml`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before any user story can be implemented

**Checkpoint**: No user story work can begin until this phase is complete.

- [ ] T008 Create database schema migration in `web/src/db/migrations/001_initial.sql` — tables: log_entries, catalog_items, stacks, stack_items, vitals_entries, alerts, insights
- [ ] T009 [P] Implement IndexedDB storage layer in `web/src/services/StorageService.ts` — wrap IndexedDB API with CRUD operations for all entities
- [ ] T010 [P] Implement SQLite WASM binding layer in `web/src/services/SqlJsService.ts` — optional fallback using `sql.js` for desktop/electron
- [ ] T011 Create database abstraction interface in `web/src/services/DatabaseService.ts` — unified API over IndexedDB/SQLite
- [ ] T012 [P] Build catalog seed data in `web/src/data/catalog-seed-v1.json` — embed 27 substances from biohack CLI with dose ranges, half-lives, contraindications
- [ ] T013 [P] Create safety protocol engine stub in `engine/src/safety.rs` — module declarations for stimulant tachycardia, hypertensive urgency, serotonin syndrome
- [ ] T014 Create routing structure in `web/src/App.tsx` — routes: /, /log, /history, /vitals, /stacks, /insights, /settings
- [ ] T015 Create layout shell component in `web/src/components/Layout.tsx` — navigation, responsive design, offline indicator
- [ ] T016 [P] Configure error boundaries in `web/src/components/ErrorBoundary.tsx` — global error handling with user-friendly messages
- [ ] T017 [P] Create logging utility in `web/src/utils/logger.ts` — structured logging with timestamps and context

**Checkpoint**: Foundation ready — user story implementation can now begin.

---

## Phase 3: User Story 1 - Log Consumption and Actions (Priority: P1) — MVP

**Goal**: Users can log supplements, medications, drugs, food, and actions with dosage, quantity, and timestamp. Custom items supported.

**Independent Test**: Open app → select item from catalog → specify dosage → confirm entry appears in history with correct timestamp.

**Acceptance Criteria**:
- AC-1: Select "Vitamin D3" from catalog, specify "5000 IU", save → entry appears in log
- AC-2: Create custom item with name, category, dosage → saved and available for future logs
- AC-3: Log action (e.g., "Meditation", 20 min) → entry saved with duration and timestamp

### Implementation

- [ ] T018 [US1] Create CatalogItem model in `web/src/models/CatalogItem.ts` — type definitions, validation schemas
- [ ] T019 [US1] Create LogEntry model in `web/src/models/LogEntry.ts` — type definitions, validation schemas
- [ ] T020 [US1] Implement catalog service in `web/src/services/CatalogService.ts` — search, filter by category, load seed data
- [ ] T021 [P] [US1] Create LogForm component in `web/src/components/LogForm.tsx` — search catalog, select item, input dosage/quantity/unit, submit
- [ ] T022 [US1] Implement log service in `web/src/services/LogService.ts` — create, read, update log entries via DatabaseService
- [ ] T023 [US1] Create LogSuccess component in `web/src/components/LogSuccess.tsx` — confirmation toast after successful log
- [ ] T024 [US1] Wire up LogPage in `web/src/pages/LogPage.tsx` — mount LogForm, handle submission, navigate to history
- [ ] T025 [P] [US1] Add loading states and error handling to LogForm
- [ ] T026 [P] [US1] Implement offline indicator in Layout showing "Offline" when network unavailable

**Checkpoint**: User Story 1 fully functional — user can log items independently.

---

## Phase 4: User Story 2 - View and Inspect Logs (Priority: P2)

**Goal**: Users can browse their history with filtering by date range, category, and specific items. Timeline and calendar views available. Fully offline.

**Independent Test**: Log multiple items across days → open history → apply filters → verify entries display correctly.

**Acceptance Criteria**:
- AC-1: All entries displayed in reverse chronological order
- AC-2: Date-range filter and category filter work together
- AC-3: Summary view shows intake frequency and dosages over time

### Implementation

- [ ] T027 [US2] Create HistoryView component in `web/src/components/HistoryView.tsx` — list of log entries with date grouping
- [ ] T028 [US2] Implement history service in `web/src/services/HistoryService.ts` — query entries with filters (date range, category, item)
- [ ] T029 [P] [US2] Create TimelineView component in `web/src/components/TimelineView.tsx` — chronological display with visual timeline
- [ ] T030 [P] [US2] Create CalendarView component in `web/src/components/CalendarView.tsx` — monthly calendar with log count badges
- [ ] T031 [US2] Create FilterBar component in `web/src/components/FilterBar.tsx` — date range picker, category chips, search input
- [ ] T032 [US2] Implement history page in `web/src/pages/HistoryPage.tsx` — mount TimelineView/CalendarView, Wire FilterBar
- [ ] T033 [P] [US2] Add pagination/virtual scrolling for large datasets (>100 entries)
- [ ] T034 [P] [US2] Create summary statistics component in `web/src/components/SummaryStats.tsx` — intake frequency, total dosages over time range

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

- [ ] T035 [US3] Create VitalsEntry model in `web/src/models/VitalsEntry.ts` — type definitions, validation (BP 60-250, HR 20-300, etc.)
- [ ] T036 [US3] Create vitals service in `web/src/services/VitalsService.ts` — CRUD for vitals entries
- [ ] T037 [P] [US3] Implement clinical thresholds config in `web/src/config/clinical-thresholds.ts` — HR >100 tachycardia, SBP≥180/DBP≥120 hypertension, etc.
- [ ] T038 [US3] Create VitalsForm component in `web/src/components/VitalsForm.tsx` — inputs for BP, HR, weight, temp, SpO2, sleep quality
- [ ] T039 [US3] Create VitalsDashboard component in `web/src/components/VitalsDashboard.tsx` — display recent vitals with trend indicators
- [ ] T040 [US3] Implement vitals alert service in `web/src/services/VitalsAlertService.ts` — check vitals against thresholds, generate Alert entries
- [ ] T041 [P] [US3] Create AlertBanner component in `web/src/components/AlertBanner.tsx` — prominent warning display for abnormal vitals
- [ ] T042 [US3] Create VitalsPage in `web/src/pages/VitalsPage.tsx` — mount VitalsForm and VitalsDashboard
- [ ] T043 [P] [US3] Add contextual advice logic in `web/src/services/ContextualAdviceService.ts` — cross-reference recent supplements/medications
- [ ] T044 [P] [US3] Implement alert acknowledgment and dismissal in AlertBanner

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

- [ ] T045 [US4] Create Stack model in `web/src/models/Stack.ts` — type definitions
- [ ] T046 [US4] Create StackItem model in `web/src/models/StackItem.ts` — type definitions
- [ ] T047 [US4] Implement stack service in `web/src/services/StackService.ts` — CRUD for stacks, batch log entries
- [ ] T048 [P] [US4] Create StackBuilder component in `web/src/components/StackBuilder.tsx` — add/remove items, set quantities, save stack
- [ ] T049 [P] [US4] Create StackListView component in `web/src/components/StackListView.tsx` — display user's stacks with log button
- [ ] T050 [US4] Implement stack page in `web/src/pages/StacksPage.tsx` — mount StackBuilder and StackListView
- [ ] T051 [P] [US4] Add YAML import/export for stacks in `web/src/services/YamlService.ts` — parse/save stack definitions
- [ ] T052 [P] [US4] Create stack editing modal in `web/src/components/StackEditModal.tsx` — modify existing stacks

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

- [ ] T053 [US5] Implement safety engine in `engine/src/safety.rs` — 3 protocols: stimulant tachycardia, hypertensive urgency, serotonin syndrome
- [ ] T054 [US5] Build safety engine WASM bindings in `engine/src/lib.rs` — expose check_interactions() and check_vitals() to JavaScript
- [ ] T055 [P] [US5] Create SafetyEngineService in `web/src/services/SafetyEngineService.ts` — call WASM safety engine, handle results
- [ ] T056 [US5] Create InteractionWarning component in `web/src/components/InteractionWarning.tsx` — prominent warning UI with risk description
- [ ] T057 [US5] Integrate safety checks into LogForm in `web/src/components/LogForm.tsx` — run check before save, display warning if found
- [ ] T058 [P] [US5] Add interaction acknowledgment tracking in LogEntry model (`acknowledgedInteraction` field)
- [ ] T059 [P] [US5] Run Rust safety engine tests in `engine/tests/safety_tests.rs` — verify 3 protocols with test cases
- [ ] T060 [P] [US5] Add benchmark dataset for 90% interaction flagging accuracy in `engine/tests/benchmark.rs`

**Checkpoint**: Safety-critical interactions are detected and warnings displayed.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Improvements affecting multiple user stories

- [ ] T061 [P] Create Settings page in `web/src/pages/SettingsPage.tsx` — theme toggle, units (metric/imperial), data export
- [ ] T062 [P] Implement data export in `web/src/services/ExportService.ts` — CSV/JSON export for all log entries (SC-008)
- [ ] T063 [P] Add PWA manifest in `web/public/manifest.json` — app name, icons, offline support
- [ ] T064 [P] Create Service Worker in `web/src/service-worker.ts` — cache assets, enable offline use
- [ ] T065 [P] Add responsive design breakpoints in `web/src/styles/global.css` — mobile, tablet, desktop layouts
- [ ] T066 [P] Implement dark mode support in `web/src/components/ThemeToggle.tsx` — CSS variables for light/dark themes
- [ ] T067 [P] Add accessibility attributes (ARIA labels, keyboard navigation) across all components
- [ ] T068 [P] Run quickstart validation scenarios from `specs/001-biohacker-tracking-platform/quickstart.md` (VS-001 through VS-008)
- [ ] T069 [P] Update README.md with setup instructions and architecture overview
- [ ] T070 [P] Run full test suite and verify all tests pass: `npm test` and `cargo test --release`

---

## Dependencies & Execution Order

### Phase Dependencies
- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 — BLOCKS all user stories
- **Phase 3-7 (User Stories)**: All depend on Phase 2 completion
  - User stories can proceed in parallel (if team capacity allows)
  - Or sequentially in priority order (P1 → P2 → P3 → P4 → P5)
- **Phase 8 (Polish)**: Depends on all desired user stories being complete

### User Story Dependencies
- **US1 (P1)**: Can start after Phase 2 — no dependencies on other stories
- **US2 (P2)**: Can start after Phase 2 — integrates with US1 data models
- **US3 (P3)**: Can start after Phase 2 — uses LogEntry from US1, generates Alert
- **US4 (P4)**: Can start after Phase 2 — uses LogEntry from US1, creates Stack entries
- **US5 (P5)**: Can start after Phase 2 — integrates with US1 safety checks

### Parallel Opportunities
- All Phase 1 tasks marked [P] can run in parallel
- All Phase 2 tasks marked [P] can run in parallel (within phase)
- Once Phase 2 completes, all user stories can start in parallel
- All [P] tasks within each story can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all parallel tasks for US1 together:
Task: "Create CatalogItem model in web/src/models/CatalogItem.ts"
Task: "Create LogEntry model in web/src/models/LogEntry.ts"
Task: "Create LogForm component in web/src/components/LogForm.tsx"
Task: "Add loading states and error handling to LogForm"
Task: "Implement offline indicator in Layout showing Offline when network unavailable"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T007)
2. Complete Phase 2: Foundational (T008-T017)
3. Complete Phase 3: User Story 1 (T018-T026)
4. **STOP and VALIDATE**: Test US1 independently — log an item, verify it appears in history
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add US1 → Test independently → Deploy/Demo (MVP!)
3. Add US2 → Test independently → Deploy/Demo
4. Add US3 → Test independently → Deploy/Demo
5. Add US4 → Test independently → Deploy/Demo
6. Add US5 → Test independently → Deploy/Demo
7. Each story adds value without breaking previous stories

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

- **Phase 1 (Setup)**: 7 tasks
- **Phase 2 (Foundational)**: 10 tasks
- **Phase 3 (US1)**: 9 tasks
- **Phase 4 (US2)**: 8 tasks
- **Phase 5 (US3)**: 10 tasks
- **Phase 6 (US4)**: 8 tasks
- **Phase 7 (US5)**: 8 tasks
- **Phase 8 (Polish)**: 10 tasks
- **Total**: 70 tasks
