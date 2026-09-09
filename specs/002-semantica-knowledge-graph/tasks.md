# Tasks: Semantica Knowledge Graph Integration

**Input**: Design documents from `/specs/002-semantica-knowledge-graph/`
**Branch**: `main`
**Date**: 2026-09-08

## Legend
- `[X]` = Completed and verified
- `[~]` = Partially implemented
- `[ ]` = Not implemented

## Scope

**v1 (in scope)**: Semantica knowledge graph integration with Rust web backend, substance knowledge API, drug interaction detection, and insights/correlations.
**v2 (deferred)**: LLM-generated advice, decision logging, full ontology reasoning.

---

## Phase 1: Setup & Environment

**Purpose**: Initialize Python environment, install Semantica, and set up the sidecar process.

- [ ] T001 Create `backend/.venv-semantica/` Python virtual environment using `uv venv` (or `python -m venv`).
- [ ] T002 Install Semantica and its dependencies into the virtual environment via `uv pip install semantica` (or `pip install`). Ensure it matches the expected version.
- [ ] T003 Create `backend/semantica/` directory for configuration, seed data, and runtime scripts.
- [ ] T004 [P] Create a script `backend/semantica/start_server.py` that launches `semantica-server` with a persistent data directory.
- [ ] T005 [P] Create a script `backend/semantica/stop_server.py` to gracefully terminate the sidecar.
- [ ] T006 Add a systemd or user-level service definition (optional) for auto-start on boot (deferred to deployment).
- [ ] T007 [P] Create a Rust module `engine/src/semantica_client.rs` with HTTP client functions to call the sidecar's REST API (GET/POST to `/kg/...` endpoints).
- [ ] T008 [P] Add a health-check endpoint in the Rust backend to verify Semantica is running and responsive; return 503 if unavailable.

**Checkpoint**: Semantica sidecar can be started/stopped, and the Rust backend can communicate with it.

---

## Phase 2: Foundational Models & API Layer

**Purpose**: Define graph entity models and implement the Rust proxy API layer.

- [ ] T009 Create `engine/src/models/kg.rs` with Rust structs for Substance, Mechanism, Biomarker, Relationship (matching Semantica's entity schema).
- [ ] T010 Extend `engine/src/lib.rs` to re-export kg models.
- [ ] T011 Create `web/src/api/knowledge.rs` with Leptos server functions or fetch wrappers for querying the knowledge graph via the Rust backend.
- [ ] T012 Implement a REST endpoint in the Rust backend: `GET /api/knowledge/substance/{id}` that proxies to Semantica's `kg/neighbors` or similar, returning substance details with mechanisms and biomarkers.
- [ ] T013 Implement `GET /api/knowledge/substances` with pagination (page, limit) and search query (search param), proxying to Semantica's search/list endpoint.
- [ ] T014 [P] Implement error handling: 404 for missing entity, 503 for Semantica unavailable, 400 for invalid parameters.
- [ ] T015 [P] Add logging for all graph API calls (request/response summaries) to facilitate debugging.

**Checkpoint**: Rust backend can serve substance details and list substances from the graph.

---

## Phase 3: User Story 1 - View and Query Substance Knowledge (Priority: P1)

**Goal**: Users can retrieve detailed substance information (mechanisms, biomarkers) via the API.

**Independent Test**: Query `/api/knowledge/substance/creatine` and receive a JSON response with properties, mechanisms, and affected biomarkers.

**Acceptance Criteria**:
- AC-1: Substance request returns properties and a list of mechanisms and biomarkers.
- AC-2: Biomarker query returns list of substances affecting it.
- AC-3: 404 on missing substance.

### Implementation

- [ ] T016 [US1] Implement `GET /api/knowledge/substance/{id}` with full entity expansion (include related mechanisms and biomarkers) in the Rust backend.
- [ ] T017 [US1] Implement `GET /api/knowledge/biomarker/{id}` to return biomarker details and substances that affect it.
- [ ] T018 [P] [US1] Add caching of substance details in Rust (in-memory with TTL) to reduce Semantica load.
- [ ] T019 [P] [US1] Write integration tests in `engine/tests/knowledge_tests.rs` mocking Semantica responses to verify API behavior.

**Checkpoint**: Substance and biomarker queries work.

---

## Phase 4: User Story 1a - List All Substances for Log Page (Priority: P1)

**Goal**: Users can get a paginated list of substances for the log page selector.

**Independent Test**: Call `/api/knowledge/substances?page=1&limit=50` and get a paginated response.

**Acceptance Criteria**:
- AC-1: Returns paginated list with name, ID, basic metadata.
- AC-2: Search query filters results (case-insensitive).
- AC-3: Pagination includes total_count.

### Implementation

- [ ] T020 [US1a] Implement `GET /api/knowledge/substances` with page, limit, search query parameters in Rust backend.
- [ ] T021 [US1a] Proxy the request to Semantica's search endpoint and transform response to paginated format.
- [ ] T022 [P] [US1a] Add optional sorting by name or relevance.
- [ ] T023 [P] [US1a] Write tests for pagination and search.

**Checkpoint**: Substance list endpoint works.

---

## Phase 5: User Story 2 - Populate Knowledge Graph from Domain Sources (Priority: P2)

**Goal**: Administrators can seed/update the graph from structured data (CSV, JSON).

**Independent Test**: Run an ingestion script and verify new entities appear in queries.

**Acceptance Criteria**:
- AC-1: CSV/JSON ingestion creates entities and relationships.
- AC-2: Duplicate runs update existing entities without duplication.

### Implementation

- [ ] T024 [US2] Design a schema for seed data: substances, mechanisms, biomarkers, relationships (e.g., CSV with columns: source, target, relation_type, confidence).
- [ ] T025 [US2] Create a Python script `backend/semantica/ingest.py` that reads seed data and uses Semantica's Python client to create/update entities.
- [ ] T026 [US2] Include an initial seed dataset for 100+ substances, 200+ mechanisms, 500+ relationships (curated from existing catalog and known interactions).
- [ ] T027 [P] [US2] Add a validation step to check for missing required fields before ingestion.
- [ ] T028 [P] [US2] Implement idempotency: use upsert logic to avoid duplicates.
- [ ] T029 [P] [US2] Document the seed data format and ingestion process in `README.md` or a separate guide.

**Checkpoint**: Seed data can be loaded into Semantica.

---

## Phase 6: User Story 3 - Drug Interaction Safety via Knowledge Graph (Priority: P1)

**Goal**: When logging a substance, the system queries the graph for interactions with substances taken in the last 24h and returns warnings.

**Independent Test**: Log fluoxetine then 5-HTP; see interaction warning with mechanism explanation.

**Acceptance Criteria**:
- AC-1: Warning displayed when interaction found.
- AC-2: No warning for safe combinations.
- AC-3: Warning includes confidence and rule reference.
- AC-4: Fallback to static list when graph unavailable.

### Implementation

- [ ] T030 [US3] Extend the Rust backend to support an endpoint `POST /api/knowledge/interactions` that receives a list of substance IDs (current + recent) and returns interaction warnings from the graph.
- [ ] T031 [US3] Query Semantica for interactions using rules: e.g., "if substance A inhibits enzyme X and substance B is metabolized by X, then risk".
- [ ] T032 [US3] Integrate interaction check into the log submission flow: before saving, call the interactions endpoint and display warnings to the user.
- [ ] T033 [P] [US3] Implement a static fallback list of known interactions (from existing catalog contraindications) to use when Semantica is unavailable.
- [ ] T034 [P] [US3] Add a UI component `InteractionWarning` (already exists in spec 001) to display warnings with accept/dismiss buttons.
- [ ] T035 [P] [US3] Write tests: mock graph responses and verify warning logic.

**Checkpoint**: Drug interactions are detected using the knowledge graph.

---

## Phase 7: User Story 4 - Insights and Correlations from Graph (Priority: P2)

**Goal**: Users see correlations between their intake and biomarker changes, with mechanism explanations from the graph.

**Independent Test**: After 7+ overlapping days of data, dashboard displays correlation card with confidence and contributing entries.

**Acceptance Criteria**:
- AC-1: At least one correlation shown after 2+ weeks of data.
- AC-2: Clicking insight shows mechanism and contributing logs.
- AC-3: Insufficient data message when <7 overlapping points.
- AC-4: Hypothesis displayed when statistical correlation weak but mechanism suggests link.

### Implementation

- [ ] T036 [US4] Create a new Rust module `engine/src/insights.rs` with functions for statistical correlation (e.g., Pearson correlation) between substance intake and biomarker measurements.
- [ ] T037 [US4] Implement an endpoint `GET /api/insights` that:
    - Fetches user's log entries and vitals from SQLite.
    - For each substance-biomarker pair with >=7 overlapping days, computes correlation.
    - Queries the knowledge graph for mechanisms that connect the substance to the biomarker.
    - Returns a list of insights with confidence, mechanism explanation, and contributing entry IDs.
- [ ] T038 [US4] Create a UI component `InsightsPage` (stub in spec 001) that displays the insights as cards with confidence meters.
- [ ] T039 [US4] Implement click-to-detail: show mechanism and list of contributing log entries.
- [ ] T040 [P] [US4] Add an "Insufficient Data" empty state.
- [ ] T041 [P] [US4] Cache insights in local storage to reduce recomputation.
- [ ] T042 [P] [US4] Write tests for correlation logic and graph integration.

**Checkpoint**: Insights dashboard displays correlations with graph-backed explanations.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Improve robustness, performance, and developer experience.

- [ ] T043 [P] Add comprehensive logging and metrics for all Semantica API calls (latency, errors).
- [ ] T044 [P] Implement a timeout/retry policy for Semantica requests (e.g., 5s timeout, 2 retries) to maintain responsiveness.
- [ ] T045 [P] Document API endpoints and usage in `README.md` and/or OpenAPI spec.
- [ ] T046 [P] Create a `docker-compose.yml` (optional) to run Semantica, Rust backend, and frontend together for development.
- [ ] T047 [P] Run end-to-end tests: start Semantica, seed data, call APIs, verify responses.
- [ ] T048 [P] Review performance: ensure p95 query times <500ms for substance, <300ms for list.
- [ ] T049 [P] Add health check to backend that also verifies Semantica connectivity; serve `/health` endpoint.

**Checkpoint**: Production-ready knowledge graph integration.

---

## Summary

| User Story | Status | Priority |
|------------|--------|----------|
| US1: View substance knowledge | Not started | P1 |
| US1a: List substances | Not started | P1 |
| US2: Populate graph from sources | Not started | P2 |
| US3: Drug interaction safety | Not started | P1 |
| US4: Insights and correlations | Not started | P2 |

**Dependencies**: Phase 1 and 2 must be complete before any user story. US3 depends on US1a for substance lookup. US4 depends on US1 for mechanism details.

**Test Commands**:
```bash
# Run Rust backend tests
cargo test -p engine -- knowledge_tests

# Run Python ingestion script (will require Semantica running)
python backend/semantica/ingest.py --seed seeds/substances.csv
```

**Next Steps**: Begin with Phase 1 (setup) and Phase 2 (models & API).