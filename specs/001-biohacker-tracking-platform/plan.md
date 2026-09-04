# Implementation Plan: Biohacker Tracking Platform

**Branch**: `001-biohacker-tracking-platform` | **Date**: 2026-09-04 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-biohacker-tracking-platform/spec.md`

## Summary

A local-first, offline-capable native mobile application for iOS and Android for biohackers to log supplements, medications, drugs, food, and actions. The application tracks vitals, flags dangerous drug interactions, provides clinical alerts (tachycardia, hypertension), surfaces insights and correlations, and supports stack/protocol logging. Cloud sync is optional with OAuth authentication; local-only mode requires no account. Data is encrypted at rest in the cloud, with full export/delete capabilities.

The existing `biohack` Rust CLI (with 27-substance database and 3 deterministic safety protocols) serves as the foundation — this application extends it with a richer UI, multi-platform support, and optional cloud features.

## Technical Context

**Language/Version**: Swift 5.9+ (iOS); Kotlin 1.9+ (Android); TypeScript 5.x + SolidJS (web); Rust 2024 (shared engine)

**Primary Dependencies**: 
- Frontend iOS: Swift + SwiftUI
- Frontend Android: Kotlin + Jetpack Compose
- Frontend Web: TypeScript + SolidJS (reactive, local-first patterns)
- Backend: Rust 2024 (shared engine with biohack CLI)
- Database: SQLite (native on mobile); PostgreSQL (cloud backend)
- Cloud sync: OAuth 2.0 / OIDC
- BLE: CoreBluetooth (iOS), BleManager (Android) for wearable integration

**Storage**: 
- Mobile local: SQLite (native, ACID compliant)
- Web local: IndexedDB (offline-capable via service worker)
- Cloud: Encrypted PostgreSQL (synced across all platforms)
- Catalog: Embedded JSON seed (27 substances from biohack) + periodic updates via app releases

**Testing**: TypeScript + Vitest (web); Rust test suite (backend/safety engine — reuse from biohack); Swift/Kotlin tests deferred to v2

**Target Platform**: Web application (primary, immediate); iOS and Android native apps (deferred to v2)

**Project Type**: Web application with mobile apps planned for future release

**Performance Goals**:
- Log item/action: <15 seconds from app open (SC-001)
- History load (filtered): <2 seconds for up to 1,000 entries (SC-002, SC-018)
- Drug interaction check: <3 seconds (SC-003)
- Vitals alert generation: <5 seconds (SC-004)
- Insights generation: ≥1 correlation with 7+ overlapping data points (SC-005)
- Stack logging: <30 seconds for 10+ items (SC-006)
- Export: <10 seconds for 5 years of daily entries (SC-008)
- Push notification latency: <30 seconds from server to device (v2 mobile)
- BLE sync with wearables: <5 seconds per device (v2 mobile)

**Constraints**:
- All user stories MUST work fully offline (confirmed in clarification)
- Cloud sync is optional background feature
- OAuth authentication for cloud mode; local-only requires no account
- Single user per installation (no multi-user)
- Safety protocols are informational only; not medical advice
- Data at rest in cloud MUST be encrypted

**Scale/Scope**:
- Single user, local-first on web; cloud-synced across devices
- 27-substance seed database (from biohack)
- Web application (TypeScript + SolidJS) — v1 priority
- iOS native app (SwiftUI) — deferred to v2
- Android native app (Jetpack Compose) — deferred to v2
- BLE integration for wearables — deferred to v2

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Compliance | Notes |
|-----------|------------|-------|
| I. Open Source Foundation | ✅ | Reuse biohack CLI engine (Rust); select permissively licensed frontend frameworks |
| II. Comprehensive Test Coverage | ✅ | Plan includes unit tests (Rust safety engine), integration tests (app flows), contract tests (API) |
|| III. Smooth UX | ✅ | Web-first with responsive design; mobile apps deferred to v2 |
|| IV. Performance | ✅ | Performance targets defined in SCs; local-first ensures sub-second response times |
|| V. Modular Architecture | ✅ | Clear separation: backend engine (Rust), web frontend (TypeScript), sync service (optional), mobile frontends (deferred) |

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
├── web/                 # Web application (v1 priority)
│   ├── src/
│   │   ├── components/  # LogForm, HistoryTable, VitalsChart, StackBuilder
│   │   ├── pages/       # Dashboard, History, Vitals, Stacks, Insights
│   │   ├── services/    # IndexedDBService, CloudSyncService, SafetyEngineService
│   │   ├── workers/     # Safety check Web Worker
│   │   └── App.tsx
│   ├── public/
│   └── tests/
├── sync/                # Optional: Cloud sync service (Rust backend)
│   ├── src/
│   │   ├── auth.rs      # OAuth flows
│   │   ├── encrypt.rs   # Client-side encryption
│   │   └── server.rs    # REST API endpoints
│   └── tests/
└── shared/              # Shared types (Rust ↔ TypeScript)
    └── schema.rs

**Deferred to v2**:
- `ios/` — iOS native app (SwiftUI)
- `android/` — Android native app (Jetpack Compose)
- BLE wearable integration

```

**Structure Decision**: Hybrid architecture with Rust engine (reused from biohack CLI) for safety-critical logic, TypeScript web app for v1, with mobile frontends planned for v2. Cloud sync is a separate optional module. This aligns with Constitution V (modular architecture for multiple frontends).

## Complexity Tracking

_N/A — No constitution violations._
