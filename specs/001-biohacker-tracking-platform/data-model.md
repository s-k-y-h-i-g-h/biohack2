# Data Model: Biohacker Tracking Platform

**Date**: 2026-09-04
**Feature**: 001-biohacker-tracking-platform
**Engine**: Rust (same types used by `biohack` CLI)

## Entity Definitions

All entities are defined in `engine/src/models.rs` using `serde` for serialization. The same types serve both the Rust engine and the Leptos frontend (via `leptos`'s reactive system — no separate type definitions to maintain).

### LogEntry

Represents a single logged event (supplement, medication, food, action).

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | String (UUID v4) | Yes | Unique identifier |
| user_id | String | Yes | Owner (local device ID) |
| item_type | Enum | Yes | `Supplement \| Medication \| Drug \| Food \| Action` |
| item_id | Option<String> | No | Reference to CatalogItem (null for custom) |
| name | String | Yes | Display name (from catalog or custom) |
| quantity | Option<f64> | No | Amount taken (null for actions) |
| unit | Option<String> | No | Unit of measure (IU, mg, min, etc.) |
| route | Option<RouteType> | No | `Oral \| Sublingual \| Topical \| Inhalation \| Injectable` |
| timestamp | chrono::DateTime<Utc> | Yes | When the event occurred |
| stack_id | Option<String> | No | Parent stack if logged as part of one |
| notes | Option<String> | No | Free-text user notes |
| acknowledged_interaction | bool | No | Whether user acknowledged an interaction warning |
| custom_fields | Option<serde_json::Value> | No | Additional context |

**Validation rules**:
- `quantity` must be > 0 for consumption items (supplement, medication, drug, food)
- `timestamp` must not be in the future
- If `stack_id` is present, parent stack must exist

### CatalogItem

Represents a supplement, medication, drug, food, or action in the system's database.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | String (UUID v4) | Yes | Unique identifier |
| name | String | Yes | Canonical name |
| category | Enum | Yes | `Supplement \| Medication \| Drug \| Food \| Action` |
| dosage_range | Option<DosageRange> | No | `{ min: f64, max: f64, unit: String }` |
| half_life | Option<String> | No | Duration of action (e.g., "4h", "24h") |
| contraindications | Vec<String> | No | Known interactions (item IDs or categories) |
| warnings | Vec<String> | No | General safety warnings |
| is_custom | bool | No | true if user-created |
| source | Option<String> | No | Original source |
| version | i32 | No | Version for tracking updates |

**Seed data**: 27 substances from `biohack` CLI, versioned as `seed-v1`.

### Stack

Represents a named collection of catalog items and actions (protocol).

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | String (UUID v4) | Yes | Unique identifier |
| user_id | String | Yes | Owner |
| name | String | Yes | Stack name (e.g., "Morning Protocol") |
| description | Option<String> | No | Human-readable description |
| created_at | chrono::DateTime<Utc> | Yes | When created |
| updated_at | chrono::DateTime<Utc> | Yes | Last modified |
| items | Vec<StackItem> | Yes | Ordered list of items |

### StackItem

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| item_id | String (FK → CatalogItem.id) | Yes | Reference to CatalogItem |
| quantity | Option<f64> | No | Override quantity (null = use catalog default) |
| unit | Option<String> | No | Override unit (null = use catalog default) |
| note | Option<String> | No | Stack-specific note |

### VitalsEntry

Represents a set of vital measurements logged at one point in time.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | String (UUID v4) | Yes | Unique identifier |
| user_id | String | Yes | Owner |
| timestamp | chrono::DateTime<Utc> | Yes | Measurement time |
| bp_systolic | Option<i32> | No | mmHg |
| bp_diastolic | Option<i32> | No | mmHg |
| heart_rate | Option<i32> | No | bpm (resting) |
| weight | Option<f64> | No | kg |
| blood_glucose | Option<f64> | No | mg/dL or mmol/L |
| temperature | Option<f64> | No | °C |
| spo2 | Option<i32> | No | % |
| hrv | Option<f64> | No | ms (heart rate variability) |
| sleep_quality | Option<SleepQuality> | No | `Poor \| Fair \| Good \| Excellent` |
| custom_metrics | Option<serde_json::Value> | No | Additional metrics |
| notes | Option<String> | No | Contextual notes |

**Validation rules**:
- BP values must be positive integers
- HR must be 20-300 bpm
- Temperature must be 30-45°C
- SpO2 must be 50-100%

### Alert

Represents a notification triggered by abnormal vitals or dangerous interactions.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | String (UUID v4) | Yes | Unique identifier |
| user_id | String | Yes | Owner |
| type | Enum | Yes | `Vital \| Interaction \| Warning` |
| severity | Enum | Yes | `Info \| Warning \| Critical` |
| message | String | Yes | Human-readable alert text |
| recommendation | Option<String> | No | Suggested action |
| is_acknowledged | bool | No | User dismissed? |
| linked_entry_id | Option<String> | No | Related LogEntry or VitalsEntry |
| generated_at | chrono::DateTime<Utc> | Yes | When alert was created |
| resolved_at | Option<chrono::DateTime<Utc>> | No | When resolved |

**Alert generation rules**:
- `vital` alerts: Triggered when vitals fall outside clinical thresholds
- `interaction` alerts: Triggered when new log entry conflicts with existing items
- `warning` alerts: Informational (e.g., "You haven't logged in 3 days")

### Insight

Represents a correlation or trend derived from logged data.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | String (UUID v4) | Yes | Unique identifier |
| user_id | String | Yes | Owner |
| type | Enum | Yes | `Correlation \| Trend \| Pattern` |
| title | String | Yes | Brief description |
| description | String | Yes | Detailed explanation |
| confidence | f64 | Yes | 0.0-1.0 statistical confidence |
| supporting_data_points | i32 | Yes | Number of data points used |
| generated_at | chrono::DateTime<Utc> | Yes | When insight was computed |
| related_entry_ids | Vec<String> | No | References to contributing LogEntries |

**Generation rules**:
- Minimum 7 overlapping data points required (per spec)
- Correlations require statistically significant p-value (< 0.05)
- Insights invalidated if underlying data is deleted

## Relationships

```
User ──────────────────────────────────────┐
    │                                      │
    ├── has many ──> LogEntry              │
    │                              ┌───────┘
    │                              │
    ├── has many ──> VitalsEntry   │
    │                              │
    ├── has many ──> Stack        ──┘
    │         │
    │         └── contains ──> StackItem ──> CatalogItem
    │
    ├── has many ──> Alert    ──> linked_entry_id (LogEntry/VitalsEntry)
    │
    └── has many ──> Insight  ──> related_entry_ids (LogEntry[])

CatalogItem ──< consumed_by >── LogEntry
CatalogItem ──< flagged_in >── Alert (interaction warnings)
LogEntry ──< noted_in >─────── Insight (contributing data)
```

## SQLite Schema

All queries are compile-time checked via `sqlx`. Schema lives in `engine/src/db.rs` with migration support.

```sql
-- log_entries
CREATE TABLE log_entries (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    item_type TEXT NOT NULL CHECK(item_type IN ('supplement','medication','drug','food','action')),
    item_id TEXT,
    name TEXT NOT NULL,
    quantity REAL,
    unit TEXT,
    route TEXT,
    timestamp TEXT NOT NULL,
    stack_id TEXT,
    notes TEXT,
    acknowledged_interaction INTEGER NOT NULL DEFAULT 0,
    custom_fields TEXT,
    FOREIGN KEY(stack_id) REFERENCES stacks(id)
);

CREATE INDEX idx_log_entries_user_timestamp ON log_entries(user_id, timestamp);
CREATE INDEX idx_log_entries_stack ON log_entries(stack_id);
CREATE INDEX idx_log_entries_type ON log_entries(item_type);

-- catalog_items
CREATE TABLE catalog_items (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL CHECK(category IN ('supplement','medication','drug','food','action')),
    dosage_range TEXT,
    half_life TEXT,
    contraindications TEXT,
    warnings TEXT,
    is_custom INTEGER NOT NULL DEFAULT 0,
    source TEXT,
    version INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX idx_catalog_category ON catalog_items(category);
CREATE INDEX idx_catalog_name ON catalog_items(name);

-- stacks
CREATE TABLE stacks (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- stack_items (joined table)
CREATE TABLE stack_items (
    stack_id TEXT NOT NULL,
    item_id TEXT NOT NULL,
    quantity REAL,
    unit TEXT,
    note TEXT,
    PRIMARY KEY (stack_id, item_id),
    FOREIGN KEY(stack_id) REFERENCES stacks(id) ON DELETE CASCADE,
    FOREIGN KEY(item_id) REFERENCES catalog_items(id)
);

-- vitals_entries
CREATE TABLE vitals_entries (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    bp_systolic INTEGER,
    bp_diastolic INTEGER,
    heart_rate INTEGER,
    weight REAL,
    blood_glucose REAL,
    temperature REAL,
    spo2 INTEGER,
    hrv REAL,
    sleep_quality TEXT,
    custom_metrics TEXT,
    notes TEXT
);

CREATE INDEX idx_vitals_user_timestamp ON vitals_entries(user_id, timestamp);

-- alerts
CREATE TABLE alerts (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    type TEXT NOT NULL CHECK(type IN ('vital','interaction','warning')),
    severity TEXT NOT NULL CHECK(severity IN ('info','warning','critical')),
    message TEXT NOT NULL,
    recommendation TEXT,
    is_acknowledged INTEGER NOT NULL DEFAULT 0,
    linked_entry_id TEXT,
    generated_at TEXT NOT NULL,
    resolved_at TEXT
);

CREATE INDEX idx_alerts_user_unack ON alerts(user_id, is_acknowledged);
CREATE INDEX idx_alerts_user_generated ON alerts(user_id, generated_at);

-- insights
CREATE TABLE insights (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    type TEXT NOT NULL CHECK(type IN ('correlation','trend','pattern')),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    confidence REAL NOT NULL,
    supporting_data_points INTEGER NOT NULL,
    generated_at TEXT NOT NULL,
    related_entry_ids TEXT
);

CREATE INDEX idx_insights_user ON insights(user_id);
CREATE INDEX idx_insights_generated ON insights(generated_at);
```

## State Transitions

### LogEntry
```
Created → Saved → (optional: Note added) → (optional: Acknowledged interaction)
```

### VitalsEntry
```
Created → Saved → (Alert generated if out of range) → (Alert acknowledged)
```

### Alert
```
Generated → Active → Acknowledged → Resolved (by new entry or user dismissal)
```

### Stack
```
Created → Modified → Logged (creates LogEntries) → Updated (future logs use new version)
```

## Migration Notes

- Seed database imports from `biohack` CLI's 27-substance list via `engine/src/catalog.rs`
- Custom catalog items persist across app updates
- Stack items reference catalog by ID; if catalog item deleted, reference becomes orphaned (graceful degradation)
- SQLite file is stored via OPFS (Origin Private File System) in the browser — no server required
- No schema migrations needed for v1; single `001_initial.sql` with `sqlx migrate`
