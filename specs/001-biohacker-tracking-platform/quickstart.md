# Quickstart Validation Guide: Biohacker Tracking Platform

**Date**: 2026-09-04
**Feature**: 001-biohacker-tracking-platform

This guide provides runnable validation scenarios to prove the feature works end-to-end. Each scenario maps to a Success Criterion (SC) from the spec.

## Prerequisites

- Rust toolchain (2024 edition) with `cargo-leptos` and `sqlx-cli`
- SQLite3 for running migrations
- Browser with OPFS support (Chrome 109+, Edge 109+, Firefox 118+)
- 27-substance seed database available (from `biohack` CLI)

## Validation Scenarios

### VS-001: Log Consumption (SC-001)

**Goal**: Verify users can log an item in under 15 seconds.

**Setup**:
1. Build engine: `cargo test --release -p engine`
2. Build web: `cargo leptos build --release`
3. Start dev server: `cargo leptos watch`
4. Open app in browser at http://localhost:3000
5. Ensure catalog is seeded (27 substances)

**Steps**:
1. Note current time
2. Click "Log Item"
3. Search for "Vitamin D3"
4. Select "5000 IU"
5. Confirm log

**Expected**:
- Entry appears in history within 15 seconds
- Entry shows correct name, dose, timestamp
- SQLite file contains the entry (verify: `sqlite3 biohack.db "SELECT * FROM log_entries;"`)

---

### VS-002: History View Performance (SC-002)

**Goal**: Verify filtered history loads in under 2 seconds.

**Setup**:
1. Populate test database with 1,000 log entries spanning 30 days
2. Build and start app

**Steps**:
1. Navigate to History view
2. Apply filter: date range = last 7 days, category = "Supplements"
3. Measure load time (browser DevTools Network tab)

**Expected**:
- View loads in < 2 seconds
- Only matching entries displayed
- Pagination or virtual scrolling if > 100 results

---

### VS-003: Drug Interaction Check (SC-003, SC-007)

**Goal**: Verify interaction warnings display within 3 seconds and achieve 90%+ accuracy.

**Setup**:
1. Run Rust safety engine tests: `cargo test --release -p engine -- safety_tests`
2. Ensure 27-substance seed is loaded

**Steps**:
1. Log "Aspirin" 325mg
2. Immediately attempt to log "Ibuprofen" 400mg
3. Measure time from second log to warning display
4. Verify warning content matches known interaction

**Expected**:
- Warning displays within 3 seconds
- Warning describes "increased bleeding risk" or similar
- User must acknowledge before saving

**Test dataset coverage**: Run against benchmark dataset of known interactions; verify ≥90% flagged.

---

### VS-004: Vitals Alerting (SC-004)

**Goal**: Verify abnormal vital alerts trigger within 5 seconds.

**Setup**:
1. Build frontend with vitals dashboard
2. Ensure clinical thresholds are configured

**Steps**:
1. Navigate to Vitals section
2. Enter blood pressure: 185/125
3. Save entry
4. Measure time to alert appearance

**Expected**:
- Alert appears within 5 seconds
- Alert severity = "critical"
- Alert message references hypertensive urgency
- Contextual advice references recent supplements/medications

**Additional test**: Log normal vitals (120/80, HR 72) — verify no alert generated.

---

### VS-005: Insights Generation (SC-005)

**Goal**: Verify insights dashboard surfaces correlations with sufficient data.

**Setup**:
1. Populate test database with 14 days of correlated supplement + vitals data
2. Ensure minimum 7 overlapping data points exist

**Steps**:
1. Navigate to Insights dashboard
2. Wait for analysis to complete
3. Verify at least one correlation is displayed

**Expected**:
- Dashboard shows ≥1 insight after 2+ weeks of data
- Each insight includes confidence score and supporting data point count
- Clicking insight shows contributing log entries

**Edge case**: With < 7 overlapping points, dashboard shows "Insufficient data" message.

---

### VS-006: Stack Logging (SC-006)

**Goal**: Verify stack logging completes in under 30 seconds for 10+ items.

**Setup**:
1. Create test stack with 12 items
2. Ensure all catalog items exist

**Steps**:
1. Note current time
2. Navigate to Stacks
3. Select test stack
4. Click "Log Stack"
5. Confirm

**Expected**:
- All 12 items logged individually with same timestamp
- Total flow completes in < 30 seconds
- Each item appears in history view

---

### VS-007: Offline Functionality

**Goal**: Verify all P1-P4 user stories work without network.

**Setup**:
1. Disconnect network (airplane mode)
2. Open app

**Steps**:
1. Log a supplement
2. View history
3. Log vitals
4. Create and log a stack

**Expected**:
- All operations succeed
- Data persists in SQLite file (OPFS)
- UI indicates "Offline" status

---

### VS-008: Data Export (SC-008)

**Goal**: Verify export completes within 10 seconds for 5 years of daily entries.

**Setup**:
1. Populate test database with 1,825 entries (5 years × ~1 entry/day)
2. Ensure export endpoint is available

**Steps**:
1. Navigate to Settings → Export
2. Select JSON format
3. Trigger export
4. Measure download time

**Expected**:
- Export file downloads within 10 seconds
- File contains all entries with complete metadata
- File is valid JSON and can be re-imported

---

### VS-009: Safety Protocols from biohack CLI (FR-008)

**Goal**: Verify the 3 deterministic protocols from `biohack` CLI are implemented.

**Setup**:
1. Run tests: `cargo test --release -p engine`
2. Ensure tests pass for all 3 protocols

**Steps**:
1. Log "Caffeine" 200mg at 08:00
2. At 09:00, log vitals with HR 110
3. Verify stimulant tachycardia alert triggers

**Expected**:
- Protocol 1 (stimulant tachycardia): HR > 100 + stimulant within 4h → alert
- Protocol 2 (hypertensive urgency): SBP ≥ 180 or DBP ≥ 120 → alert
- Protocol 3 (serotonin syndrome): Multiple serotonergic agents → alert

**Additional**: Log items from different categories to verify false positive rate < 5%.

---

### VS-010: Type Safety Validation

**Goal**: Verify that DB schema changes are caught at compile time.

**Setup**:
1. Modify a column type in `engine/src/db.rs`
2. Attempt to build: `cargo build --release`

**Steps**:
1. Make a schema change (e.g., rename a column)
2. Run `cargo build --release`
3. Verify compile errors reference the changed type

**Expected**:
- Build fails with clear error messages pointing to affected code
- No runtime type mismatches possible

---

## Running All Validations

```bash
# 1. Run Rust engine tests
cargo test --release -p engine

# 2. Build web frontend
cargo leptos build --release

# 3. Start dev server
cargo leptos watch

# 4. Run validation scenarios in browser
# (open http://localhost:3000 and follow VS-001 through VS-010)

# 5. Verify SQLite data
sqlite3 biohack.db ".tables"
sqlite3 biohack.db "SELECT COUNT(*) FROM log_entries;"
```

## Pass Criteria

All VS-001 through VS-010 must pass before marking feature complete.
