# Quickstart Validation Guide: Biohacker Tracking Platform

**Date**: 2026-09-04
**Feature**: 001-biohacker-tracking-platform

This guide provides runnable validation scenarios to prove the feature works end-to-end. Each scenario maps to a Success Criterion (SC) from the spec.

## Prerequisites

- Rust toolchain (for safety engine tests)
- Node.js 18+ (for frontend)
- PostgreSQL (for cloud sync tests, optional)
- 27-substance seed database available

## Validation Scenarios

### VS-001: Log Consumption (SC-001)

**Goal**: Verify users can log an item in under 15 seconds.

**Setup**:
1. Build frontend: `npm run build`
2. Open app in browser
3. Ensure at least one catalog item exists (seed database)

**Steps**:
1. Note current time
2. Click "Log Item"
3. Search for "Vitamin D3"
4. Select "5000 IU"
5. Confirm log

**Expected**:
- Entry appears in history within 15 seconds
- Entry shows correct name, dose, timestamp

**Verify**: `git log --oneline -1` shows commit with entry data

---

### VS-002: History View Performance (SC-002)

**Goal**: Verify filtered history loads in under 2 seconds.

**Setup**:
1. Create test database with 1,000 log entries spanning 30 days
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
1. Run Rust safety engine tests: `cargo test --release`
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
- Data persists in local storage
- UI indicates "Offline" status
- Reconnecting triggers sync (if cloud enabled)

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
- File is valid JSON/CSV and can be re-imported

---

### VS-009: Cloud Sync & Encryption (FR-019, FR-020, FR-021)

**Goal**: Verify cloud data is encrypted and deletable.

**Setup**:
1. Configure cloud sync with test OAuth provider
2. Enable cloud mode

**Steps**:
1. Log an item
2. Open cloud storage (debug mode or inspect headers)
3. Verify stored payload is encrypted
4. Delete cloud data via API
5. Verify local data remains intact

**Expected**:
- Cloud-stored data is unreadable without decryption key
- Deletion removes cloud copy only
- Local data persists after cloud deletion
- Export includes all cloud data in readable format

---

### VS-010: Safety Protocols from biohack CLI (FR-008)

**Goal**: Verify the 3 deterministic protocols from `biohack` CLI are implemented.

**Setup**:
1. Run `cargo test` in shared engine directory
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

## Running All Validations

```bash
# 1. Build and test Rust engine
cd engine
cargo test --release

# 2. Build frontend
cd ../frontend
npm run build
npm test

# 3. Run E2E validation scenarios
node scripts/run-validations.js --all

# 4. Generate coverage report
npm run coverage
```

## Pass Criteria

All VS-001 through VS-010 must pass before marking feature complete.
