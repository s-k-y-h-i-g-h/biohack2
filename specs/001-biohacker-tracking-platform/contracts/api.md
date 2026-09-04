# API Contracts: Biohacker Tracking Platform

**Date**: 2026-09-04
**Feature**: 001-biohacker-tracking-platform

## Local API (WASM-based, no network required)

### POST /check-interactions

Check for dangerous drug interactions (local WASM execution).

**Request body**:
```json
{
  "existing_items": [
    { "item_id": "uuid", "name": "Aspirin", "taken_at": "ISO8601" }
  ],
  "new_item": {
    "item_id": "uuid",
    "name": "Ibuprofen",
    "dose": 400,
    "unit": "mg"
  }
}
```

**Response** `200 OK`:
```json
{
  "interactions": [
    {
      "severity": "moderate",
      "item1": "Aspirin",
      "item2": "Ibuprofen",
      "risk": "Increased risk of gastrointestinal bleeding",
      "recommendation": "Avoid concurrent use or consult physician"
    }
  ],
  "checked_at": "ISO8601"
}
```

---

### POST /check-vitals

Check vitals against clinical thresholds.

**Request body**:
```json
{
  "vitals": {
    "heart_rate": 110,
    "blood_pressure_systolic": 145,
    "blood_pressure_diastolic": 95,
    "spo2": 96,
    "temperature": 37.2
  },
  "contextual_log": [
    { "name": "Caffeine", "taken_at": "2026-09-04T08:30:00Z" }
  ]
}
```

**Response** `200 OK`:
```json
{
  "alerts": [
    {
      "type": "tachycardia",
      "severity": "warning",
      "message": "Heart rate 110 bpm exceeds normal resting range (60-100 bpm)",
      "recommendation": "Consider reducing stimulant intake; monitor for 15 minutes",
      "contextual_advice": "Last caffeine intake was 2 hours ago"
    }
  ]
}
```

---

## Cloud Sync API (OAuth-secured, deferred to v2)

### Authentication

All endpoints require OAuth 2.0 / OIDC Bearer token. Local-only mode bypasses all endpoints.

```
Authorization: Bearer ***
```

---

### Data Synchronization

#### POST /sync/upload

Upload local data changes to cloud.

**Request body**:
```json
{
  "operations": [
    {
      "type": "create" | "update" | "delete",
      "entity": "log_entry" | "vitals_entry" | "stack" | "alert" | "insight",
      "id": "uuid",
      "data": { ... }
    }
  ],
  "client_id": "device-uuid",
  "last_sync_timestamp": "ISO8601"
}
```

**Response** `200 OK`:
```json
{
  "synced": true,
  "remote_timestamp": "ISO8601",
  "conflicts": []
}
```

---

#### GET /sync/download

Download changes since last sync.

**Query params**:
- `last_sync_timestamp`: ISO8601 (required)
- `limit`: integer (default 100, max 500)

**Response** `200 OK`:
```json
{
  "last_sync_timestamp": "ISO8601",
  "changes": [
    {
      "type": "create" | "update" | "delete",
      "entity": "log_entry",
      "id": "uuid",
      "timestamp": "ISO8601",
      "data": { ... }
    }
  ],
  "has_more": false
}
```

---

#### POST /sync/encrypt

Client-side encryption request (optional; returns encrypted payload upload URL).

**Response** `200 OK`:
```json
{
  "upload_url": "https://...presigned-url",
  "encrypted_key_id": "key-id",
  "expires_at": "ISO8601"
}
```

---

### Data Deletion

#### DELETE /data/{entity_type}/{entity_id}

Delete a single entity.

**Response** `204 No Content` on success.

---

#### POST /data/delete-all

Initiate full account data deletion.

**Request body**:
```json
{
  "confirmation": "DELETE ALL MY DATA"
}
```

**Response** `204 No Content` on success.

---

### Data Export

#### GET /data/export

Export all user data in standard format.

**Query params**:
- `format`: `csv` | `json` (default `json`)
- `since`: ISO8601 (optional filter)
- `until`: ISO8601 (optional filter)

**Response** `200 OK` (streaming):
```
Content-Type: application/json
Content-Disposition: attachment; filename="biohack-export.json"

{
  "exported_at": "ISO8601",
  "version": "1.0",
  "entries": [...],
  "vitals": [...],
  "stacks": [...],
  "alerts": [...]
}
```

---

## Protocol Definition (YAML Schema)

Stacks can be defined via YAML for batch creation or programmatic use.

```yaml
version: "1.0"
name: "Morning Protocol"
description: "Daily morning stack"
items:
  - item_id: "vit-d3"
    quantity: 5000
    unit: "IU"
  - item_id: "mg-glycinate"
    quantity: 400
    unit: "mg"
  - item_id: "cold-exposure"
    duration: 10
    unit: "min"
notes: "Start after breakfast"
```

---

## Error Responses

All endpoints return standard HTTP status codes with consistent error format:

```json
{
  "error": {
    "code": "INVALID_REQUEST",
    "message": "Blood pressure systolic must be between 60-250",
    "details": { "field": "blood_pressure_systolic", "value": 300 }
  }
}
```

---

## Rate Limits

- Local API: No limits (runs on device)
- Cloud API: 100 requests/minute per authenticated user
- Export: 1 request/minute (generates large payloads)
