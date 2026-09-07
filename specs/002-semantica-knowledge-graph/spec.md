# Feature Specification: Semantica Knowledge Graph Integration

**Feature Branch**: `main`

**Created**: 2026-09-08

**Status**: Draft

**Input**: User description: "Add Semantica integration. This will let us store a biohacking knowledge graph. We will also be able to make the application offer LLM-generated advice later and the decisions made when generating this advice could be logged in Semantica." (Note: LLM advice and decision logging are deferred to a separate feature; this spec covers only the knowledge graph foundation.)

## User Scenarios & Testing

### User Story 1 - View and Query Substance Knowledge (Priority: P1)

A user browsing their vitals or building a stack wants to see detailed information about a substance (e.g., creatine): its mechanisms, biomarkers affected, and recommended dosages. This information is stored in the Semantica knowledge graph and exposed via the web API.

**Why this priority**: This is the core value of the knowledge graph — enriching the user experience with structured domain knowledge.

**Independent Test**: Can be fully tested by querying the API for a substance ID and receiving a rich graph of related entities. Delivers immediate value by replacing hardcoded or static data.

**Acceptance Scenarios**:

1. **Given** the system has a substance "Creatine" in the knowledge graph, **When** a user requests `/knowledge/substance/creatine`, **Then** the API returns the substance node with its properties and a list of its mechanisms and affected biomarkers.
2. **Given** a user queries for biomarkers affected by "Creatine", **When** the API receives the request, **Then** it returns a list of biomarker entities with their relationships to creatine.
3. **Given** the knowledge graph is empty, **When** a user requests a substance, **Then** the API returns a 404 with a clear message.

---

### User Story 1a - List All Substances for the Log Page (Priority: P1)

A user needs to see a paginated list of all available substances to select one for logging. This list supports the existing log page functionality where all substances are displayed for selection.

**Why this priority**: The log page is a primary user interaction point; without a list endpoint, the frontend cannot render the substance selector. This is foundational for user workflow.

**Independent Test**: Can be fully tested by calling the list endpoint and verifying it returns a paginated list of substance names and IDs. This unblocks the frontend's log page.

**Acceptance Scenarios**:

1. **Given** the system has multiple substances in the knowledge graph, **When** a user requests `/knowledge/substances?page=1&limit=50`, **Then** the API returns a paginated list with substance names, IDs, and basic metadata.
2. **Given** the user provides a search query, **When** they request `/knowledge/substances?search=creatine`, **Then** the API returns only substances matching the search term (case-insensitive).
3. **Given** the knowledge graph contains 150 substances, **When** the user requests page 2 with limit 50, **Then** the API returns the next 50 substances with a `total_count` of 150.

---

### User Story 2 - Populate Knowledge Graph from Domain Sources (Priority: P2)

As a system administrator (or through an automated pipeline), I can ingest structured data (e.g., from scientific papers, databases) into the Semantica knowledge graph to keep the domain knowledge up-to-date.

**Why this priority**: This is a backend/ops concern; the immediate user value comes from having a populated graph. We can seed it manually first.

**Independent Test**: Can be tested by running an ingestion script and verifying that new nodes and relationships appear in the graph.

**Acceptance Scenarios**:

1. **Given** a CSV file with substance-mechanism-biomarker relationships, **When** an ingestion script is run, **Then** all entities and relationships are created in Semantica.
2. **Given** a duplicate ingestion, **When** the script is run again, **Then** existing nodes are updated (or deduplicated) without duplication.

---

### User Story 3 - Drug Interaction Safety via Knowledge Graph (Priority: P1)

A user logs a new substance and the system automatically checks for dangerous interactions with substances already in the user's log (within a relevant time window). The check uses the knowledge graph to query known interactions (e.g., "substance A inhibits enzyme CYP2D6, substance B is metabolized by CYP2D6") and returns a warning if a clinically significant interaction is found. The warning includes an explanation of the mechanism and a recommended action.

**Why this priority**: Safety is foundational. Moving interaction detection to the knowledge graph makes it scalable and auditable, replacing the hardcoded substring matcher.

**Independent Test**: Log a substance known to interact with another (e.g., fluoxetine and 5-HTP) and verify that the system returns an interaction warning with a mechanism explanation.

**Acceptance Scenarios**:

1. **Given** the user has logged fluoxetine in the last 24 hours, **When** they attempt to log 5-HTP, **Then** the system queries the knowledge graph for interactions and returns a warning: "Fluoxetine and 5-HTP both affect serotonin; risk of serotonin syndrome." with a recommendation to consult a healthcare provider.
2. **Given** the user logs a substance with no known interactions, **When** the check runs, **Then** no warning is displayed and the log entry is saved normally.
3. **Given** the knowledge graph contains interaction rules with confidence scores, **When** an interaction is detected, **Then** the warning includes the confidence level (e.g., "High confidence") and a reference to the underlying rule.
4. **Given** the knowledge graph is temporarily unavailable, **When** a user attempts to log a substance, **Then** the system falls back to a static interaction list (or logs a warning that interaction check is unavailable) and allows the save to proceed.

---

### User Story 4 - Insights and Correlations from Graph (Priority: P2)

A user opens the insights dashboard and sees correlations between their supplement intake, actions, and biomarker changes over time. The correlations are derived from the knowledge graph's mechanisms (e.g., "Creatine increases ATP production; higher ATP correlates with improved performance") and from statistical analysis of the user's own data.

**Why this priority**: Insights transform raw data into actionable knowledge, driving user engagement and long-term value. Leveraging the knowledge graph makes insights more explainable and grounded.

**Independent Test**: After logging 7+ overlapping days of data (e.g., creatine intake and heart rate), the dashboard displays a correlation card with a confidence score and a list of contributing log entries.

**Acceptance Scenarios**:

1. **Given** the user has logged consistent data over 2+ weeks, **When** they open the insights dashboard, **Then** at least one correlation or trend is displayed with supporting data points and a confidence score.
2. **Given** a correlation is displayed, **When** the user clicks on it, **Then** a detailed view shows the mechanism from the knowledge graph (e.g., "Creatine increases cellular hydration, which may reduce resting heart rate") and a list of the log entries that contributed to the correlation.
3. **Given** the user has fewer than 7 overlapping data points for a given pair, **When** they view the dashboard, **Then** a message "Insufficient data for correlations" is shown for that pair.
4. **Given** the knowledge graph contains a mechanism that connects two entities, **When** the statistical correlation is weak, **Then** the insight card displays the mechanism-based hypothesis separately from the statistical evidence, with a note "Mechanism suggests possible link, but data is inconclusive."

---

### Edge Cases

- What happens when a substance or mechanism does not exist in the graph?
  - The API returns a 404 with a clear error; the engine may fall back to a default rule or prompt the user to add it.
- How does the system handle concurrent writes to the same graph from multiple users?
  - The backend's Semantica instance is single-threaded; we serialize writes via the API, and user_id scoping (if used) prevents cross-user contamination.
- What happens if the Semantica backend or its sidecar process becomes unavailable?
  - The web backend should return a 503 and log the error; deterministic advice (which does not depend on Semantica) should still work.
- How are large graphs handled with respect to performance?
  - We'll use indexed queries and limit traversal depth; caching may be added later.

## Requirements

### Functional Requirements

- **FR-001**: The web backend MUST start a Semantica REST API server (`semantica-server`) as a sidecar process.
- **FR-002**: The web backend MUST provide a RESTful API layer in Rust that proxies, sanitizes, and aggregates calls to the sidecar `semantica-server` endpoints (e.g. mapping to `kg/neighbors`, `search`, etc.).
- **FR-003**: The API MUST support querying substances by name, mechanism, or biomarker.
- **FR-004**: The system MUST store substance domain entities (substance, mechanism, biomarker, relationship) in Semantica, not in SQLite.
- **FR-005**: The SQLite database MUST store a foreign key reference (the Semantica entity ID) for any log or note that references a substance.
- **FR-006**: The system MUST have a seed script to populate an initial set of substances, mechanisms, and biomarkers from a curated data source.
- **FR-007**: The API MUST expose endpoints for CRUD operations on entities (create, read, update, delete) with appropriate authorization, proxying the operations securely to `semantica-server`.
- **FR-008**: The system MUST log all graph operations for audit purposes.
- **FR-009**: The API MUST support listing all substances with pagination (page, limit) and optional search by name, returning a paginated response with total count.
- **FR-010**: The web backend's installation process MUST create a dedicated Python virtual environment and install Semantica into a subdirectory of the backend (e.g., `backend/.venv-semantica/`), separate from any system‑wide or user‑level Semantica installations. The runtime MUST use this environment exclusively.

### Key Entities

- **Substance**: A chemical compound or supplement (e.g., Creatine). Attributes: name, description, properties.
- **Mechanism**: A biological process or action (e.g., Phosphocreatine → ATP regeneration). Attributes: name, description, type.
- **Biomarker**: A measurable indicator (e.g., ATP level). Attributes: name, unit, normal range.
- **Relationship**: A directed edge between two entities (e.g., Substance increases Biomarker). Attributes: type (e.g., "increases", "supports"), confidence (optional).

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can retrieve substance details in under 500ms (p95) for a typical query.
- **SC-002**: The system can support at least 100 concurrent users querying the knowledge graph without degradation.
- **SC-003**: The initial seed script populates at least 100 substances, 200 mechanisms, and 500 relationships in under 5 minutes.
- **SC-004**: Users can retrieve a paginated list of 50 substances in under 300ms (p95) for typical page loads.

## Assumptions

- The web backend runs on a server with sufficient resources to run Semantica (Python + libraries) alongside the Rust backend.
- The initial domain knowledge seed data is available in a structured format (e.g., CSV or JSON).
- Authentication and user management already exist in the system; we will reuse them for user_id tagging if needed.
- Desktop frontends are not in scope for v1; the web API will serve web and mobile clients.
- The web backend will maintain its own dedicated Semantica installation and data directory, separate from any other Semantica instances on the system (as required by FR-010). This instance is not shared with other applications.
- Network latency between the backend and Semantica sidecar is negligible (local process).

## Clarifications

### Session 2026-09-07

- **Q**: Should the Semantica knowledge graph API be built as a RESTful JSON API, a gRPC service, or both?
  - **A**: RESTful JSON proxying a `semantica-server` sidecar. The Rust backend will run `semantica-server` as a local sidecar on port 8000 and expose a secured REST API layer that proxies, sanitizes, and aggregates these endpoints for client frontends.
