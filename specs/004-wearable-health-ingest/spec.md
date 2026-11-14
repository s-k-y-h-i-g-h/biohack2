# Feature Specification: Wearable & Health-Platform Data Ingestion

**Feature Branch**: `004-wearable-health-ingest`

**Created**: 2026-10-11

**Status**: Draft — proposed, not ratified

**Depends on**: [001-biohacker-tracking-platform](../001-biohacker-tracking-platform/spec.md) (VitalsEntry, safety engine)

**Input**: Planning discussion 2026-10-11. Hardware in scope: Samsung Galaxy Watch7, Oura Ring Gen 5, Whoop. Constraint from the platform owner: **local-first, LAN-only where possible; no dependency on a cloud service the app cannot operate without.**

## Problem Statement

`VitalsEntry` has fields for heart rate, HRV, SpO₂, sleep quality, weight, glucose — and today every one of them is typed by hand. Meanwhile the user's watch and ring already measure them continuously, at a density no manual log approaches.

Manual vitals logging has two failure modes that make the safety engine weaker than it looks: **sparse** (a resting HR once a week cannot support a tachycardia alert) and **retrospective** (the alert fires days after the event). Continuous ingestion turns the safety engine from a spot-check into a monitor.

## User Scenarios & Testing *(mandatory)*

### User Story B1 - Ingest a Wearable Export Without Any Account (Priority: P1) 🎯 MVP

A user exports a day of watch data as a file; the sync agent parses it and the vitals appear in biohack2 with correct timestamps. No OAuth, no cloud account, no vendor SDK.

**Why this priority**: It is the route that works on the hardware the user owns, offline, with no credentials, and it establishes the ingest pipeline every later source will use. It is also the route that requires no Kotlin, no Rust↔Java bridge, and no vendor partnership.

**Independent Test**: Place a FIT/CSV export in the agent's inbox → within one sync cycle the readings appear as `VitalsEntry` rows with source attribution and no duplicate on re-ingest.

**Acceptance Scenarios**:

1. **Given** a supported export file, **When** the agent ingests it, **Then** readings become `VitalsEntry` rows carrying `source` and `source_record_id`.
2. **Given** the same file ingested twice, **When** the second ingest completes, **Then** no duplicate rows are created.
3. **Given** a malformed or unsupported file, **When** ingest fails, **Then** the failure is reported with the file name and reason, and the file is retained for retry — not silently dropped.

---

### User Story B2 - Continuous Ingest From a Companion App (Priority: P2)

A companion app on the phone reads the platform health store and pushes readings to biohack2 on the LAN every few minutes. Resting HR, HRV, sleep stages, and SpO₂ arrive continuously.

**Why this priority**: Real-time data is what makes alerts useful, and it is the higher-fidelity version of B1. It is P2 because it requires a mobile component and a platform integration, both of which are heavier than file ingest.

**Independent Test**: Companion app pushes a reading → it appears in the vitals dashboard within one cycle; an out-of-range value triggers the existing safety alert path.

**Acceptance Scenarios**:

1. **Given** the companion app is running, **When** a new reading is available, **Then** it reaches biohack2 within the configured interval with no manual action.
2. **Given** biohack2 is unreachable (phone off LAN), **When** readings accumulate, **Then** the companion buffers them and delivers on reconnect, in order, without duplication.
3. **Given** a delivered reading is abnormal, **When** it lands, **Then** the existing FR-008 alert path fires — ingestion does not create a second alerting system.

---

### User Story B3 - Multi-Source Merging With Provenance (Priority: P2)

A user has both a watch and a ring. Both report resting HR for the same night. The user sees both values, knows which came from which device, and the app does not silently average them.

**Why this priority**: Multiple sources is the real situation. Silent merging is how a tracking app becomes untrustworthy.

**Independent Test**: Ingest the same night from two devices → both readings visible, each labelled, no averaged value presented as ground truth.

**Acceptance Scenarios**:

1. **Given** two sources reporting the same metric for overlapping time, **When** the dashboard renders, **Then** both are shown with source labels.
2. **Given** a conflict, **When** the user opens the detail, **Then** the app states that it does not merge and shows each source's value and timestamp.
3. **Given** a source is disconnected, **When** history is viewed, **Then** the gap is visible as a gap, not filled.

---

### User Story B4 - Sleep & Recovery Context for Alerts (Priority: P3)

The safety engine's contextual advice (FR-009) can reference measured sleep and HRV, not just logged supplements.

**Why this priority**: It multiplies the value of everything already built, and it depends on B2 landing.

**Independent Test**: Log a high resting HR on a night with 4 h measured sleep → the alert's contextual advice references the measured sleep deficit.

**Acceptance Scenarios**:

1. **Given** ingested sleep/HRV data, **When** an alert is generated, **Then** the recommendation may cite measured recovery context with its source and date.
2. **Given** no ingested recovery data, **When** an alert is generated, **Then** advice falls back to log-derived context only, exactly as today.

---

### Edge Cases

- **Timestamp semantics.** Wearable exports carry local-time stamps and timezones. Storage is UTC. A wrong conversion shifts every reading by hours and silently corrupts trend and alert windows. Conversion MUST be explicit and tested.
- **Device clock drift.** Wearable clocks drift against phone and server clocks. Readings with implausible timestamps (future, absurdly old) MUST be quarantined and reported, not stored.
- **Unit variance.** Sleep quality scales differ per vendor; HRV is reported as RMSSD or SDNN depending on device. Ingest MUST record the metric definition, not just the number.
- **Vendor terms.** Automated access to vendor health data is governed by each platform's terms. Ingest MUST NOT be built in a way that violates them, and the chosen route per source MUST be documented with its terms basis.
- **Privacy boundary.** Health data crossing a network is the most sensitive data the app handles. LAN-only by default; any remote path requires explicit user opt-in and is out of v1 scope.
- **Battery and background limits.** A phone companion app is killed by the OS. Buffered delivery (B2/AC-2) is a requirement, not a nicety.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-050**: The system MUST ingest wearable data through a **source-agnostic ingest layer** that produces `VitalsEntry` rows. No frontend, dashboard, or safety code may know which vendor produced a reading.
- **FR-051**: Every ingested reading MUST carry `source` (device/platform identifier) and `source_record_id` enabling idempotent re-ingest.
- **FR-052**: Ingest MUST be idempotent: re-ingesting the same source records MUST NOT create duplicates.
- **FR-053**: The system MUST support **file-based ingest** (FIT, CSV, JSON exports) as a fully offline route requiring no account, no network, and no vendor SDK.
- **FR-054**: The system MUST support **push ingest** from a companion app over the local network, with buffering on the sender side and ordered, deduplicated delivery on reconnect.
- **FR-055**: Each ingested metric MUST record its **definition** (e.g. HRV as `rmssd` vs `sdnn`; sleep score scale) so values are never compared across incompatible definitions.
- **FR-056**: The system MUST NOT merge conflicting readings from different sources into a single value. Conflicts are surfaced.
- **FR-057**: Ingested readings MUST flow through the **existing** safety engine (FR-008) and MUST NOT introduce a parallel alerting path.
- **FR-058**: Ingest MUST be local-first: no v1 route may require a vendor cloud, and all v1 routes MUST function with no internet access.
- **FR-059**: The system MUST report ingest failures with file/source and reason, and MUST retain failed inputs for retry.
- **FR-060**: The chosen access route for each vendor MUST be documented together with the platform-terms basis for it, before implementation.

### Key Entities

- **IngestSource**: `id`, `kind` (`file` | `companion` | `api`), `device_label`, `platform`, `first_seen`, `last_seen`.
- **RawReading**: `source_id`, `source_record_id`, `metric`, `metric_definition`, `value`, `unit`, `observed_at`, `ingested_at`, `payload` (verbatim, retained for re-parse).
- **VitalsEntry**: existing entity (spec 001) — the merge target. Ingested readings map onto its fields; unmappable metrics go to `custom_metrics` with their definition.

## Success Criteria *(mandatory)*

- **SC-050**: A day of wearable export data is ingested and visible in the vitals dashboard in under 30 s, with no account and no internet.
- **SC-051**: Re-ingesting the same export produces zero duplicate rows (verified by test).
- **SC-052**: Companion push delivers a reading to the dashboard in under 10 s while on LAN.
- **SC-053**: After a 24 h disconnect, buffered readings deliver in order with no loss and no duplication.
- **SC-054**: 100% of ingested rows display their source label in the UI.
- **SC-055**: Adding a new wearable vendor requires no change to the dashboard, safety engine, or data model — verified by adding the second vendor as a test of the abstraction.

## Assumptions

- File-based ingest is v1. Companion-app ingest is v2. Vendor REST APIs (Oura, Whoop, Strava) are v2 and require OAuth — explicitly deferred, consistent with the local-first constraint.
- A FIT-file parser library will be evaluated for the file route (see research §Ecosystem survey 2026-10-11: `fitparser`). **Adoption gated on license verification** — not yet performed.
- A companion app on Android will use the platform health store; the engine is exposed to Kotlin via a Rust↔Kotlin bridge (candidate: `uniffi`). No Rust health-platform SDK exists (verified 2026-10-11: crates.io returns no Health Connect client; the only related crate is Tauri-specific with negligible adoption).
- Ingested data is personal data of the highest sensitivity class. It never leaves the LAN in v1.

## Out of Scope for v1

- Cloud sync of wearable data; vendor cloud APIs; OAuth flows.
- Real-time streaming from device radios (BLE/Galaxy Watch direct) — the companion-app route is the abstraction boundary.
- Derived wellness scores, readiness scores, or vendor scoring models. biohack2 stores measurements and its own interpretations.
- Any ingest route that requires the app to depend on a third-party server to function.
