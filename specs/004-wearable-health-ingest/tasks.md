# Tasks: Wearable & Health-Platform Data Ingestion

**Input**: Design documents from `/specs/004-wearable-health-ingest/`
**Branch**: `004-wearable-health-ingest`
**Date**: 2026-10-11
**Status**: Proposed — not started. Task numbering starts at T300 (spec 001: T001–T118; spec 003: T200–T257).

## Legend
- `[X]` = Completed and verified
- `[~]` = Partially implemented
- `[ ]` = Not implemented

## Scope

**v1**: source-agnostic ingest layer, file-based ingest (FIT/CSV/JSON), provenance + idempotency, metric definitions, conflict surfacing, existing safety path.
**v2 (deferred)**: Android companion app (Health Connect via Kotlin + Rust bridge), vendor REST APIs (Oura/Whoop/Strava, OAuth), cloud.

**Gate before Phase 2**: per-vendor access route documented with its platform-terms basis (FR-060). Do not build against a route whose terms are unreviewed.

---

## Phase 1: Ingest Abstraction

**Purpose**: One pipeline. No vendor-specific code reaches the dashboard, the safety engine, or the data model.

- [ ] T300 Define the ingest contract in `engine/src/ingest.rs`: `RawReading` (source_id, source_record_id, metric, metric_definition, value, unit, observed_at, ingested_at, verbatim payload)
- [ ] T301 Define `IngestSource` entity (kind: `file` | `companion` | `api`; device label; platform; first/last seen)
- [ ] T302 Implement the metric registry: metric name + definition (`hrv: rmssd` vs `sdnn`; sleep score scale) — FR-055. Values with incompatible definitions must never be compared
- [ ] T303 Implement `RawReading` → `VitalsEntry` mapping, with unmappable metrics routed to `custom_metrics` carrying their definition
- [ ] T304 Persist `IngestSource` and the raw-reading ledger (SQLite + WASM storage paths)
- [ ] T305 Implement idempotency: unique constraint on (`source_id`, `source_record_id`); re-ingest is a no-op, verified by test (FR-052, SC-051)
- [ ] T306 Implement timestamp normalisation: explicit source-timezone → UTC conversion, with the source timezone recorded (edge case; this is the class of bug that broke all write flows 2026-09-09)
- [ ] T307 Implement implausible-timestamp quarantine (future, absurdly old) → quarantined + reported, never stored
- [ ] T308 Implement failure reporting: file/source + reason, input retained for retry, never silently dropped (FR-059)
- [ ] T309 Unit tests: mapping, idempotency, timezone conversion, quarantine, failure retention

**Checkpoint**: A reading can enter the system through one path, is attributed, is idempotent, and its timezone is explicit.

---

## Phase 2: File-Based Ingest (v1 MVP)

**Purpose**: The route that works with no account, no network, no SDK.

- [ ] T310 Record the parser library decision in `research.md` (candidate `fitparser`) with **licence verified** before code
- [ ] T311 Implement the inbox watcher: a directory the agent scans for new export files
- [ ] T312 Implement FIT parsing → `RawReading` set (HR, HRV, sleep stages, SpO₂, temperature, steps as available)
- [ ] T313 Implement CSV/JSON generic parsing with a documented column contract
- [ ] T314 Implement per-vendor field mapping tables as **data** (device → metric + definition), not code branches
- [ ] T315 Implement the CLI ingest command: `biohack2 ingest <file>` reporting counts, mapping, and failures
- [ ] T316 Implement the sync-agent mode (watch inbox, ingest, report) runnable on the LAN host
- [ ] T317 Fixture tests: real-format sample files → expected `VitalsEntry` rows (no network, no device)
- [ ] T318 Idempotency test on a real fixture re-ingested twice (SC-051)
- [ ] T319 Test: a day of export data visible in the vitals dashboard in under 30 s offline (SC-050)

**Checkpoint**: Export file → vitals in the app, offline, no account, idempotent.

---

## Phase 3: Provenance & Conflict Surfacing

**Purpose**: The user always knows which device said what, and nothing is silently averaged.

- [ ] T320 Surface `source` label on every ingested vitals row in the UI (SC-054)
- [ ] T321 Implement conflict detection: same metric, overlapping time, different sources → both shown, labelled
- [ ] T322 Implement the conflict detail view: per-source value + timestamp + explicit "not merged" statement (B3/AC-2)
- [ ] T323 Implement gap visibility: disconnected source renders as a gap, not a fill (B3/AC-3)
- [ ] T324 Test: two sources, same night, both visible, no averaged value presented as truth

**Checkpoint**: Multi-source data is legible as multi-source data.

---

## Phase 4: Safety Path Integration

**Purpose**: Ingested data drives the alerts that already exist. No second alerting system.

- [ ] T325 Route ingested readings through `SafetyEngine::check_vitals()` on landing (FR-057) — reuse spec 001's alert path
- [ ] T326 Test: an ingested out-of-range reading triggers the existing alert banner
- [ ] T327 Extend `contextual_advice()` to cite measured sleep/HRV context with source and date (B4, FR-009 extension)
- [ ] T328 Test: advice falls back to log-derived context when no ingest data exists (B4/AC-2)
- [ ] T329 Test: alert volume under continuous ingest is bounded — no alert storm from 288 readings/day (dedup/coalesce policy documented)

**Checkpoint**: Continuous data makes the existing safety engine a monitor, with bounded alert volume.

---

## Phase 5: Companion App (v2 — deferred, spec'd for shape)

**Purpose**: Continuous push. Heavier: needs a mobile component and a platform bridge.

- [ ] T330 Document the Android health-store access route and its terms basis (FR-060) — **gate**
- [ ] T331 Record the Rust↔Kotlin bridge decision in `research.md` (candidate `uniffi`) with licence verified
- [ ] T332 Define the push API on the server: batched `RawReading` POST, auth by LAN pairing token, no cloud
- [ ] T333 Implement sender-side buffering: readings accumulate while biohack2 is unreachable
- [ ] T334 Implement ordered, deduplicated delivery on reconnect (B2/AC-2, SC-053)
- [ ] T335 Implement the companion app read loop against the platform health store
- [ ] T336 Test: 24 h disconnect → buffered readings delivered in order, no loss, no duplication (SC-053)
- [ ] T337 Test: reading lands in dashboard in under 10 s on LAN (SC-052)
- [ ] T338 Test: OS background-kill mid-cycle resumes without duplication

**Checkpoint**: Continuous ingest with no manual action and no cloud dependency.

---

## Phase 6: Vendor APIs (v2 — deferred)

- [ ] T339 Oura REST API client (OAuth) — requires explicit user opt-in; breaks local-first, so it is opt-in only
- [ ] T340 Whoop API client (OAuth)
- [ ] T341 Strava API client (OAuth) — activity context for insights
- [ ] T342 All API clients land in the same `RawReading` pipeline — no new data model (SC-055)

---

## Sequencing & Dependencies

```
Phase 1 (abstraction) ──┬──> Phase 2 (file ingest, MVP) ──> Phase 3 (provenance) ──> Phase 4 (safety)
                        └──> Phase 5 (companion, v2) ──> Phase 6 (vendor APIs, v2)
```

- Phase 2 is the MVP. Ship it alone; it delivers value with no mobile work.
- Phase 4 is cheap and high-value — do not defer it behind Phase 5.
- Phase 5/6 are v2. Spec'd here so the abstraction in Phase 1 is built to accommodate them (SC-055).

## Critical Constraints (do not violate)

1. **No vendor-specific code above the ingest layer** (FR-050, SC-055). Adding a vendor must not touch dashboard, safety, or models.
2. **Local-first.** No v1 route requires a vendor cloud (FR-058). Health data does not leave the LAN in v1.
3. **Terms before code** (FR-060). Documented per vendor, before implementation.
4. **No merging of conflicting sources** (FR-056). Surfaced, never averaged.
5. **Idempotency is tested** (FR-052). Re-ingest is a no-op, proven by fixture test.
6. **Timezone conversion is explicit and tested** — the 2026-09-09 chrono incident is the precedent.
