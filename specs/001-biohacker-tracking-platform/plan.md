# Implementation Plan: Biohacker Tracking Platform

**Branch**: `001-biohacker-tracking-platform` | **Date**: 2026-09-04 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-biohacker-tracking-platform/spec.md`

## Summary

A local-first, offline-capable web application for biohackers to log supplements, medications, drugs, food, and actions. The application tracks vitals, flags dangerous drug interactions, provides clinical alerts (tachycardia, hypertension), surfaces insights and correlations, and supports stack/protocol logging. Cloud sync is optional with OAuth authentication; local-only mode requires no account. Data is encrypted at rest in the cloud, with full export/delete capabilities.

The existing `biohack` Rust CLI (with 27-substance database and 3 deterministic safety protocols) serves as the foundation — this application extends it with a richer UI, multi-platform support, and optional cloud features.

## Technical Context

**Language/Version**: TypeScript 5.x (web frontend); Rust 2024 (local backend engine, shared with biohack CLI)

**Primary Dependencies**: 
- Frontend: SolidJS or Svelte (local-first, reactive UI)
- Backend: Rust (shared engine with biohack CLI)
- Local storage: IndexedDB (via `idb` or `drizzle-orm` with SQLite/WASM)
- Cloud sync: Supabase or self-hosted (optional)

**Storage**: 
- Local-first: SQLite (via `better-sqlite3` for Electron/Tauri) or IndexedDB for web
- Cloud: Encrypted PostgreSQL (optional, on-device primary source of truth)
- Catalog: Embedded JSON seed (27 substances from biohack) + periodic updates via app releases

**Testing**: Vitest (frontend); Rust test suite (backend/safety engine — reuse from biohack)

**Target Platform**: Mobile and desktop web browsers (SC-009), progressive web app (PWA) capable of offline use

**Project Type**: Web application with local-first architecture (offline-capable, cloud-sync optional)

**Performance Goals**:
- Log item/action: <15 seconds from app open (SC-001)
- History load (filtered): <2 seconds for up to 1,000 entries (SC-002, SC-018)
- Drug interaction check: <3 seconds (SC-003)
- Vitals alert generation: <5 seconds (SC-004)
- Insights generation: ≥1 correlation with 7+ overlapping data points (SC-005)
- Stack logging: <30 seconds for 10+ items (SC-006)
- Export: <10 seconds for 5 years of daily entries (SC-008)

**Constraints**:
- All user stories MUST work fully offline (confirmed in clarification)
- Cloud sync is optional background feature
- OAuth authentication for cloud mode; local-only requires no account
- Single user per installation (no multi-user)
- Safety protocols are informational only; not medical advice
- Data at rest in cloud MUST be encrypted

**Scale/Scope**:
- Single user, local-first
- 27-substance seed database (from biohack CLI)
- Multiple frontend targets (web, potential desktop via Tauri)
- Optional cloud sync for cross-device access

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Compliance | Notes |
|-----------|------------|-------|
| I. Open Source Foundation | ✅ | Reuse biohack CLI engine (Rust); select permissively licensed frontend frameworks |
| II. Comprehensive Test Coverage | ✅ | Plan includes unit tests (Rust safety engine), integration tests (app flows), contract tests (API) |
| III. Smooth UX | ✅ | Reactivity-first frontend; offline-first reduces latency; PWA for smooth mobile experience |
| IV. Performance | ✅ | Performance targets defined in SCs; local-first ensures sub-second response times |
| V. Modular Architecture | ✅ | Clear separation: backend engine (Rust), frontend (TypeScript), sync service (optional) |

**Gates**: All gates pass. No violations requiring justification.

## Project Structure

```text
specs/001-biohacker-tracking-platform/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── contracts/           # Phase 1 output
    └── api.md           # Cloud sync API contract

src/
├── engine/              # Rust: shared safety protocols, catalog, logging logic
│   ├── src/
│   │   ├── safety.rs    # Stimulant tachycardia, hypertensive urgency, serotonin syndrome
│   │   ├── catalog.rs   # 27-substance seed database + update mechanism
│   │   ├── log.rs       # LogEntry, Stack, Alert entities
│   │   └── insights.rs  # Correlation engine
│   └── tests/
├── frontend/            # TypeScript: UI layer
│   ├── src/
│   │   ├── components/  # LogItem, HistoryView, VitalsDashboard, StackBuilder
│   │   ├── pages/       # Dashboard, History, Insights, Settings
│   │   ├── services/    # LocalStorageService, CloudSyncService (optional)
│   │   └── App.tsx
│   └── tests/
├── sync/                # Optional: Cloud sync service
│   ├── src/
│   │   ├── auth.rs      # OAuth flows
│   │   ├── encrypt.rs   # Client-side encryption before upload
│   │   └── server.rs    # REST API endpoints
│   └── tests/
└── shared/              # Shared types (Rust ↔ TypeScript bindings)
    └── schema.rs
```

**Structure Decision**: Hybrid architecture with Rust engine (reused from biohack CLI) for safety-critical logic, TypeScript frontend for UI. Cloud sync is a separate optional module. This aligns with Constitution V (modular architecture for multiple frontends).

## Complexity Tracking

_N/A — No constitution violations._
