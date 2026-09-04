# Data Model: Biohacker Tracking Platform

**Date**: 2026-09-04
**Feature**: 001-biohacker-tracking-platform

## Entity Definitions

### LogEntry

Represents a single logged event (supplement, medication, food, action).

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string (UUID) | Yes | Unique identifier |
| userId | string | Yes | Owner (local-only: device ID) |
| itemType | enum | Yes | `supplement` \| `medication` \| `drug` \| `food` \| `action` |
| itemId | string | Yes | Reference to CatalogItem (or null for custom) |
| name | string | Yes | Display name (from catalog or custom) |
| quantity | float | No | Amount taken (null for actions) |
| unit | string | No | Unit of measure (IU, mg, min, etc.) |
| route | enum | No | `oral` \| `sublingual` \| `topical` \| `inhalation` \| `injectable` |
| timestamp | datetime | Yes | When the event occurred |
| stackId | string (FK) | No | Parent stack if logged as part of one |
| notes | string | No | Free-text user notes |
| acknowledgedInteraction | boolean | No | Whether user acknowledged an interaction warning |
| customFields | json | No | Additional context (e.g., food allergies, action intensity) |

**Validation rules**:
- `quantity` must be > 0 for consumption items
- `timestamp` must not be in the future
- If `stackId` is present, parent stack must exist

### CatalogItem

Represents a supplement, medication, drug, food, or action in the system's database.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string (UUID) | Yes | Unique identifier |
| name | string | Yes | Canonical name |
| category | enum | Yes | `supplement` \| `medication` \| `drug` \| `food` \| `action` |
| dosageRange | object | No | `{ min: float, max: float, unit: string }` |
| halfLife | string | No | Duration of action (e.g., "4h", "24h") |
| contraindications | [string] | No | Known interactions (drug IDs) |
| warnings | [string] | No | General safety warnings |
| isCustom | boolean | No | true if user-created |
| source | string | No | Original source (e.g., "biohack CLI seed", "community contribution") |
| version | integer | No | Version for tracking updates |

**Seed data**: 27 substances from `biohack` CLI, versioned as `seed-v1`.

### Stack

Represents a named collection of catalog items and actions (protocol).

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string (UUID) | Yes | Unique identifier |
| userId | string | Yes | Owner |
| name | string | Yes | Stack name (e.g., "Morning Protocol") |
| description | string | No | Human-readable description |
| createdAt | datetime | Yes | When created |
| updatedAt | datetime | Yes | Last modified |
| items | [StackItem] | Yes | Ordered list of items |

### StackItem

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| itemId | string (FK) | Yes | Reference to CatalogItem |
| quantity | float | No | Override quantity (null = use catalog default) |
| unit | string | No | Override unit (null = use catalog default) |
| note | string | No | Stack-specific note |

### VitalsEntry

Represents a set of vital measurements logged at one point in time.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string (UUID) | Yes | Unique identifier |
| userId | string | Yes | Owner |
| timestamp | datetime | Yes | Measurement time |
| bloodPressureSystolic | float | No | mmHg |
| bloodPressureDiastolic | float | No | mmHg |
| heartRate | float | No | bpm (resting) |
| weight | float | No | kg |
| bloodGlucose | float | No | mg/dL or mmol/L |
| temperature | float | No | °C |
| spo2 | float | No | % |
| hrv | float | No | ms (heart rate variability) |
| sleepQuality | enum | No | `poor` \| `fair` \| `good` \| `excellent` |
| customMetrics | json | No | Additional metrics (e.g., RMSSD, cortisol) |
| notes | string | No | Contextual notes |

**Validation rules**:
- BP values must be positive integers
- HR must be 20-300 bpm
- Temperature must be 30-45°C
- SpO2 must be 50-100%

### Alert

Represents a notification triggered by abnormal vitals or dangerous interactions.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string (UUID) | Yes | Unique identifier |
| userId | string | Yes | Owner |
| type | enum | Yes | `vital` \| `interaction` \| `warning` |
| severity | enum | Yes | `info` \| `warning` \| `critical` |
| message | string | Yes | Human-readable alert text |
| recommendation | string | No | Suggested action |
| isAcknowledged | boolean | No | User dismissed? |
| linkedEntryId | string (FK) | No | Related LogEntry or VitalsEntry |
| generatedAt | datetime | Yes | When alert was created |
| resolvedAt | datetime | No | When resolved (new entry or dismissal) |

**Alert generation rules**:
- `vital` alerts: Triggered when vitals fall outside clinical thresholds
- `interaction` alerts: Triggered when new log entry conflicts with existing items
- `warning` alerts: Informational (e.g., "You haven't logged in 3 days")

### Insight

Represents a correlation or trend derived from logged data.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string (UUID) | Yes | Unique identifier |
| userId | string | Yes | Owner |
| type | enum | Yes | `correlation` \| `trend` \| `pattern` |
| title | string | Yes | Brief description |
| description | string | Yes | Detailed explanation |
| confidence | float | Yes | 0.0-1.0 statistical confidence |
| supportingDataPoints | integer | Yes | Number of data points used |
| generatedAt | datetime | Yes | When insight was computed |
| relatedEntryIds | [string] | No | References to contributing LogEntries |

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
    ├── has many ──> Alert    ──> linkedEntryId (LogEntry/VitalsEntry)
    │
    └── has many ──> Insight  ──> relatedEntryIds (LogEntry[])

CatalogItem ──< consumed_by >── LogEntry
CatalogItem ──< flagged_in >── Alert (interaction warnings)
LogEntry ──< noted_in >─────── Insight (contributing data)
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

## Indexes

- `LogEntry`: `(userId, timestamp)` — history queries
- `LogEntry`: `(userId, stackId)` — stack log lookups
- `VitalsEntry`: `(userId, timestamp)` — vitals timeline
- `Alert`: `(userId, isAcknowledged, generatedAt)` — unread alerts
- `CatalogItem`: `(category, name)` — search
- `Insight`: `(userId, generatedAt)` — insights feed

## Migration Notes

- Seed database imports from `biohack` CLI's 27-substance list
- Custom catalog items persist across app updates
- Stack items reference catalog by ID; if catalog item deleted, reference becomes orphaned (graceful degradation)
