# API Contracts: Biohacker Tracking Platform

**Date**: 2026-09-04
**Feature**: 001-biohacker-tracking-platform

## Cloud Sync API (OAuth-secured)

### Authentication

All endpoints require OAuth 2.0 / OIDC Bearer token. Local-only mode bypasses all endpoints.

```
Authorization: Bearer <access_token>
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
  "clientId": "device-uuid",
  "lastSyncTimestamp": "ISO8601"
}
```

**Response** `200 OK`:
```json
{
  "synced": true,
  "remoteTimestamp": "ISO8601",
  "conflicts": []
}
```

**Response** `207 Multi-Status` (if conflicts):
```json
{
  "synced": false,
  "conflicts": [
    {
      "entityId": "uuid",
      "type": "log_entry",
      "reason": "server_has_newer_version",
      "serverVersion": "ISO8601"
    }
  ]
}
```

---

#### GET /sync/download

Download changes since last sync.

**Query params**:
- `lastSyncTimestamp`: ISO8601 (required)
- `limit`: integer (default 100, max 500)

**Response** `200 OK`:
```json
{
  "lastSyncTimestamp": "ISO8601",
  "changes": [
    {
      "type": "create" | "update" | "delete",
      "entity": "log_entry",
      "id": "uuid",
      "timestamp": "ISO8601",
      "data": { ... }
    }
  ],
  "hasMore": false
}
```

---

#### POST /sync/encrypt

Client-side encryption request (optional; returns encrypted payload upload URL).

**Response** `200 OK`:
```json
{
  "uploadUrl": "https://...presigned-url",
  "encryptedKeyId": "key-id",
  "expiresAt": "ISO8601"
}
```

---

### Data Deletion

#### DELETE /data/{entityType}/{entityId}

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
  "exportedAt": "ISO8601",
  "version": "1.0",
  "entries": [...],
  "vitals": [...],
  "stacks": [...],
  "alerts": [...]
}
```

---

## Interaction Check API (Local-Only)

### POST /check-interactions

Check for dangerous drug interactions (no auth required for local mode).

**Request body**:
```json
{
  "existingItems": [
    { "itemId": "uuid", "name": "Aspirin", "takenAt": "ISO8601" }
  ],
  "newItem": {
    "itemId": "uuid",
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
  "checkedAt": "ISO8601"
}
```

---

## Vitals Alert API (Local-Only)

### POST /check-vitals

Check vitals against clinical thresholds.

**Request body**:
```json
{
  "vitals": {
    "heartRate": 110,
    "bloodPressureSystolic": 145,
    "bloodPressureDiastolic": 95,
    "spo2": 96,
    "temperature": 37.2
  },
  "contextualLog": [
    { "name": "Caffeine", "takenAt": "2026-09-04T08:30:00Z" }
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
      "contextualAdvice": "Last caffeine intake was 2 hours ago"
    }
  ]
}
```

---

## Protocol Definition (YAML Schema)

Stacks can be defined via YAML for batch creation or programmatic use.

```yaml
version: 1.0
name: "Morning Protocol"
description: "Daily morning stack"
items:
  - itemId: "vit-d3"
    quantity: 5000
    unit: "IU"
  - itemId: "magnesium-glycinate"
    quantity: 400
    unit: "mg"
  - itemId: "cold-exposure"
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
    "details": { "field": "bloodPressureSystolic", "value": 300 }
  }
}
```

---

## Rate Limits

- Local API: No limits (runs on device)
- Cloud API: 100 requests/minute per authenticated user
- Export: 1 request/minute (generates large payloads)
