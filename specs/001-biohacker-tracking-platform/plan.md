# Implementation Plan: Biohacker Tracking Platform

**Branch**: `001-biohacker-tracking-platform` | **Date**: 2026-09-04 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-biohacker-tracking-platform/spec.md`

**Note**: v1 is fully local-first with no cloud dependency. Cloud sync deferred to v2.

## Summary

A local-first, offline-capable web application for biohackers to log supplements, medications, drugs, food, and actions. The application tracks vitals, flags dangerous drug interactions, provides clinical alerts (tachycardia, hypertension), surfaces insights and correlations, and supports stack/protocol logging. Runs fully offline with no account required; cloud sync deferred to v2.

Built with **Rust + Leptos** — reusing the existing `biohack` CLI engine (27-substance database, 3 deterministic safety protocols) directly. SQLite via WASM + OPFS for local-first storage. End-to-end type safety: same Rust types flow from database through engine to UI.

## Technical Context

**Language/Version**: Rust 2024 edition + Leptos 0.7 (web); same Rust codebase as `biohack` CLI

**Primary Dependencies**:
- Frontend Web: Leptos (fine-grained reactivity, SSR-ready, ~60KB WASM)
- Backend Engine: Rust (shared safety protocols, catalog, logging logic — reuses `biohack`)
- Database: SQLite via `sqlx` (build-time) + `sqlite-wasm` or `op-sqlite` for runtime
- Local Storage: SQLite WASM with OPFS backend (proven 2026 pattern for local-first)
- Build: `cargo-leptos` for WASM compilation + HMR

**Storage**:
- Local: SQLite (single file) via WASM; persisted via OPFS or IndexedDB-backed SQLite
- Catalog: Embedded seed from `biohack` CLI (27 substances) + periodic updates via app releases
- Cloud: Encrypted PostgreSQL — deferred to v2

**Testing**: Rust test suite (safety engine + integration tests — reuses `biohack` tests); Leptos component tests with `wasm-bindgen-test`

**Target Platform**: Web application (primary, immediate); iOS/Android native via Dioxus (deferred to v2)

**Project Type**: Web application with Rust engine, Leptos frontend, SQLite local storage

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
- Single user per installation (no multi-user)
- Safety protocols are informational only; not medical advice
- Data at rest must be encrypted (cloud deferred to v2)

**Scale/Scope**:
- Single user, fully local-first (no cloud required for v1)
- 27-substance seed database (from biohack)
- Web application (Rust + Leptos) — v1 priority
- Local storage via SQLite WASM
- Cloud sync, mobile apps, BLE — deferred to v2

## Constitution Check

| Principle | Compliance | Notes |
|-----------|------------|-------|
| I. Open Source Foundation | ✅ | Reuses `biohack` CLI engine (Rust); Leptos is MIT-licensed |
| II. Comprehensive Test Coverage | ✅ | Rust safety engine tests (existing); Leptos component tests; SQLite integration tests |
| III. Smooth UX | ✅ | Leptos provides fine-grained reactivity, sub-second DOM updates, instant HMR |
| IV. Performance | ✅ | WASM engine directly accesses SQLite; no serialization bridge; <60KB total bundle |
| V. Modular Architecture | ✅ | Clear separation: engine (Rust), frontend (Leptos), storage (SQLite); mobile via Dioxus |

**Gates**: All gates pass. No violations requiring justification.

## Project Structure

```text
biohack2/
├── Cargo.toml                    # Workspace manifest
├── .gitignore
├── README.md
│
├── engine/                       # Rust: shared safety protocols, catalog, logging
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                # Re-exports all public API
│       ├── safety.rs             # 3 protocols: tachycardia, hypertensive urgency, serotonin
│       ├── catalog.rs            # 27-substance database + update mechanism
│       ├── models.rs             # LogEntry, CatalogItem, Stack, VitalsEntry, Alert, Insight
│       └── db.rs                 # SQLite schema + queries
│   └── tests/
│       ├── safety_tests.rs       # Safety protocol tests (ported from biohack)
│       └── integration_tests.rs  # End-to-end scenario tests
│
├── web/                          # Leptos web frontend
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs               # App entry point (SSR + WASM)
│       ├── router.rs             # Leptos router: /, /log, /history, /vitals, /stacks, /insights, /settings
│       ├── state/
│       │   ├── store.rs          # Leptos signals/stores for app state
│       │   └── db.rs             # SQLite WASM client (sqlx-compiled queries)
│       ├── components/
│       │   ├── layout.rs         # Navigation shell, offline indicator
│       │   ├── log_form.rs       # Supplement/action log form with catalog search
│       │   ├── history_view.rs   # Timeline/calendar history with filters
│       │   ├── vitals_form.rs    # Vitals entry form
│       │   ├── vitals_dashboard.rs # Recent vitals with trend indicators
│       │   ├── alert_banner.rs   # Prominent clinical alert display
│       │   ├── stack_builder.rs  # Stack creation/management UI
│       │   ├── interaction_warning.rs # Danger interaction warning modal
│       │   ├── insights_feed.rs  # Correlation/trend insights
│       │   ├── note_input.rs     # Inline note editor
│       │   └── theme_toggle.rs   # Dark/light mode toggle
│       ├── pages/
│       │   ├── log_page.rs
│       │   ├── history_page.rs
│       │   ├── vitals_page.rs
│       │   ├── stacks_page.rs
│       │   ├── insights_page.rs
│       │   └── settings_page.rs
│       └── styles/
│           └── global.css        # CSS variables for theming
│
├── sync/                         # Deferred to v2: cloud sync service
│   └── (OAuth, encryption, REST API)
│
└── specs/
    └── 001-biohacker-tracking-platform/
        ├── spec.md
        ├── plan.md
        ├── data-model.md
        ├── research.md
        ├── quickstart.md
        └── tasks.md
```

**Deferred to v2**:
- `sync/` — Cloud sync service (OAuth, encrypted PostgreSQL)
- iOS/Android native apps (via Dioxus)
- BLE wearable integration
- PowerSync integration for SQLite ↔ PostgreSQL sync

## Complexity Tracking

_N/A — No constitution violations._
