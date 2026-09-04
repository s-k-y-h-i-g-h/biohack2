<!-- Sync Impact Report
Version change: 0.0.0 (template) → 1.0.0
Modified principles:
  - PRINCIPLE_1_NAME → I. Open Source Foundation & Code Quality
  - PRINCIPLE_2_NAME → II. Comprehensive Test Coverage
  - PRINCIPLE_3_NAME → III. Smooth and Consistent User Experience
  - PRINCIPLE_4_NAME → IV. Performance as a Design Goal
  - PRINCIPLE_5_NAME → V. Modular Architecture for Multiple Frontends
Added sections:
  - Technical Standards (Dependencies, Modularity)
  - Development Workflow (Code Review, CI Gates)
Removed sections: none
Deferred items: none
Ratification date assumed: 2025-09-04 (initial commit date)
Last amended: 2026-09-04
Version bump rationale: Initial constitution — MAJOR (0.0.0 → 1.0.0). No prior ratified version existed; this is the first substantive constitution for the project.
-->

# biohack2 Constitution

## Core Principles

### I. Open Source Foundation & Code Quality
Every component builds on existing open-source projects where a suitable, actively maintained option exists. Reinventing the wheel is a failure, not a feature. Code quality must be as high as possible: clear naming, comprehensive documentation, linting, and review are non-negotiable. Best-in-class solutions are pursued before custom implementations.

### II. Comprehensive Test Coverage
Tests must cover all features. The test type (unit, functional, or integration) is a secondary concern — what matters is that the application is provably working. Every new feature must ship with tests; tests are not optional.

### III. Smooth and Consistent User Experience
User experience is a first-class concern, not an afterthought. Interfaces must be smooth and consistent across all interaction surfaces. UX decisions require justification through user research, accessibility review, or explicit design rationale.

### IV. Performance as a Design Goal
Performance must be considered from the start of every feature cycle. However, correctness and maintainability take priority over micro-optimizations. Performance regressions that meaningfully degrade user experience are blockers.

### V. Modular Architecture for Multiple Frontends
The system is designed from the ground up to support multiple frontends sharing a common backend. Core logic lives in decoupled, independently deployable services or libraries. Frontend-specific code must not leak into shared layers.

## Technical Standards

### Dependencies
All external dependencies must be actively maintained, have a permissive license, and be reviewed for security implications before adoption. Dependency additions require review.

### Modularity
Components must expose clean, documented interfaces. Circular dependencies between modules are prohibited. Shared state between frontends must go through explicit contracts.

## Development Workflow

### Code Review
All changes require at least one review before merging. Reviews must cover correctness, test coverage, and alignment with this constitution.

### CI Gates
Automated pipelines must pass before any change merges: linting, type-checking, tests, and security scanning.

## Governance

This constitution is the source of truth for project decisions. It supersedes informal conventions.

**Amendment Procedure**: Any change to this document requires a pull request with a clear rationale, a review period of at least 48 hours, and approval from at least two maintainers. Backward-incompatible changes must include a migration plan.

**Compliance**: All pull requests and reviews must verify alignment with the principles in this document.

**Version**: 1.0.0 | **Ratified**: 2025-09-04 | **Last Amended**: 2026-09-04