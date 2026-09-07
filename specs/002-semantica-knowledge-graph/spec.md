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

### Edge Cases

- What happens when a substance or mechanism does not exist in the graph?
  - The API returns a 404 with a clear error; the engine may fall back to a default rule or prompt the user to add it.
- How does the system handle concurrent writes to the same graph from multiple users?
  - The backend's Semantica instance is single-threaded; we serialize writes via the API, and user_id scoping (if used) prevents cross-user contamination.
- What happens if the Semantica MCP server becomes unavailable?
  - The web backend should return a 503 and log the error; deterministic advice (which does not depend on Semantica) should still work.
- How are large graphs handled with respect to performance?
  - We'll use indexed queries and limit traversal depth; caching may be added later.

## Requirements

### Functional Requirements

- **FR-001**: The web backend MUST start a Semantica MCP server as a sidecar process.
- **FR-002**: The web backend MUST provide a RESTful (or gRPC) API that maps to Semantica MCP tools (e.g., `add_entity`, `add_relationship`, `query_graph`, `extract_relations`).
- **FR-003**: The API MUST support querying substances by name, mechanism, or biomarker.
- **FR-004**: The system MUST store substance domain entities (substance, mechanism, biomarker, relationship) in Semantica, not in SQLite.
- **FR-005**: The SQLite database MUST store a foreign key reference (the Semantica entity ID) for any log or note that references a substance.
- **FR-006**: The system MUST have a seed script to populate an initial set of substances, mechanisms, and biomarkers from a curated data source.
- **FR-007**: The API MUST expose endpoints for CRUD operations on entities (create, read, update, delete) with appropriate authorization.
- **FR-008**: The system MUST log all graph operations for audit purposes.
- **FR-009**: The API MUST support listing all substances with pagination (page, limit) and optional search by name, returning a paginated response with total count.
- **FR-010**: The web backend's installation process MUST create a dedicated Python virtual environment and install Semantica (with the `llm-openai` extra) into a subdirectory of the backend (e.g., `backend/.venv-semantica/`), separate from any system‑wide or user‑level Semantica installations. The runtime MUST use this environment exclusively.

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
- Network latency between the backend and Semantica MCP is negligible (local process).
