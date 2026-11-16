# Feature Specification: Insights Loop — Evidence From the Dose Log and Vitals

**Feature Branch**: `006-insights-loop`

**Created**: 2026-10-11

**Status**: Draft — proposed, not ratified. Successor to US6, which spec 001 deferred to v2.

**Depends on**: [001](../001-biohacker-tracking-platform/spec.md) (LogEntry, VitalsEntry, Note, Insight), [003](../003-substance-core-pharmacokinetics/spec.md) (simulated state as a joinable series), [004](../004-wearable-health-ingest/spec.md) (dense vitals)

**Input**: Planning discussion 2026-10-11. The app's stated purpose is "provide useful feedback and information and analysis and insights for biohackers" (spec 001 input). US6 was deferred with no owner. This spec gives it a shape that survives contact with n=1 data.

## Problem Statement

biohack2 accumulates exactly the raw material of self-experimentation: doses, timestamps, vitals, and notes saying what the user noticed. It produces none of the conclusions.

The obvious move — correlate substance X against metric Y and report the correlation — is the move that produces confident nonsense. Personal data is short (weeks, not years), noisy (wearables, lifestyle, sleep), and confounded (the user changes several things at once). An insights feature that hands the user a p-value from 11 data points is worse than no insights feature, because it moves doses.

So the design constraint is the point: **the system reports what the data can support and says plainly when it cannot support anything.**

## User Scenarios & Testing *(mandatory)*

### User Story D1 - Intervention–Outcome Review (Priority: P1) 🎯 MVP

A user starts a new intervention (magnesium, logged 2026-10-01) and the app offers a review: resting HR before vs after, with n, the effect size, and an explicit statement of what this comparison cannot establish.

**Why this priority**: It is the smallest honest unit of insight, it uses only data the app already has, and it maps to how biohackers actually work — change one thing, watch something.

**Independent Test**: Log a start date for an intervention with ≥7 days of resting HR on each side → a review appears with pre/post medians, n per side, and the limitations block.

**Acceptance Scenarios**:

1. **Given** an intervention with data on both sides, **When** the review renders, **Then** it shows pre/post summaries, n per side, and the observed difference.
2. **Given** fewer than the configured minimum overlapping data points, **When** the review renders, **Then** it reports "insufficient data" and shows what is missing — it does not produce a number.
3. **Given** a review is displayed, **When** the user opens it, **Then** the limitations are shown in the same view as the result, not behind a link.

---

### User Story D2 - Note-Grounded Correlation Candidates (Priority: P2)

A user writes "increased anxiety after Ashwagandha at night". The app surfaces this as a *candidate* to test — linking the note to the substance and to HR/HRV data around those times — and proposes a review, not a conclusion.

**Why this priority**: Notes are the hypothesis source. This is the loop: note → candidate → review → decision → new note. It is P2 because it depends on D1 existing.

**Independent Test**: Log a note naming a substance → the app lists a candidate pairing with the surrounding data availability and a "set up review" action.

**Acceptance Scenarios**:

1. **Given** a note naming a catalog substance, **When** the candidate list renders, **Then** the pairing appears with the data available for it.
2. **Given** a candidate, **When** the user sets up a review, **Then** it becomes a D1 review with the note as its stated origin.
3. **Given** a candidate with no measurable outcome available, **When** it renders, **Then** it is marked as untestable with the reason.

---

### User Story D3 - Trend and Drift Detection (Priority: P2)

The app flags sustained drift in a metric (resting HR rising across 6 weeks; weight trend; HRV declining) with the window it measured and the data density behind it.

**Why this priority**: Drift is the signal personal tracking is genuinely good at, and it needs no causal claim.

**Independent Test**: Six weeks of ingested resting HR with a steady rise → a drift insight appears with window, slope, and n.

**Acceptance Scenarios**:

1. **Given** ≥ the minimum window of dense data, **When** drift is detected, **Then** an insight reports window, direction, magnitude, and n.
2. **Given** sparse data (gaps in the window), **When** drift is evaluated, **Then** the gaps are reported and the result is qualified by density.
3. **Given** a metric with no ingested history, **When** drift is evaluated, **Then** no insight is produced.

---

### User Story D4 - Insight Ledger (Priority: P3)

Insights are durable records with status: proposed → tested → supported / unsupported / inconclusive. A user can see what they concluded and when, and what they have already tested.

**Why this priority**: Without a ledger, insights evaporate and the user re-tests the same thing for the fifth time. The ledger is what turns a feature into a practice.

**Independent Test**: Mark a review as "unsupported" → it appears in the ledger with that status and its evidence, and the candidate list stops proposing it as new.

**Acceptance Scenarios**:

1. **Given** a completed review, **When** the user records an outcome, **Then** the insight persists with status, evidence, and date.
2. **Given** a ledger entry, **When** the user opens it, **Then** the underlying data window is reachable from it.
3. **Given** a tested candidate, **When** the candidate list renders, **Then** it is shown as tested, not as new.

---

### Edge Cases

- **Confounding.** The user changed three things in the same week. The review must list other changes inside the window, because a review that hides them is a lie.
- **Data density lies.** 7 points over 7 days and 7 points over 7 hours are not the same evidence. Density is reported, not assumed.
- **Multiple comparisons.** Scanning 40 substances against 8 metrics guarantees "significant" results by chance. The system MUST account for the scan it performed, and MUST NOT present a scan-derived result as a tested hypothesis.
- **Wearable gaps.** A disconnected device produces a gap that looks like improvement. Gaps are surfaced in every review whose window contains one.
- **Timezone and DST.** Reviews aggregate by local day. The 2026-09-09 chrono incident is the cautionary precedent; day-boundary logic needs WASM tests. *Resolved 2026-11-16:* the engine groups gap/density coverage by UTC calendar day (it takes no timezone input); the shared `check_day_boundary_vectors` fixture spanning the 2026-10-25 DST-end weekend is asserted by the native test and inside real wasm32 via `wasm_vectors_node_check.mjs` (SC-095).
- **N=1.** No claim of statistical significance to a population. Language is descriptive: "in your data, over this window".

## Requirements *(mandatory)*

### Functional Requirements

- **FR-090**: The system MUST compute insights **deterministically from stored data** — dose log, vitals, notes, simulated state — with no model inference in the result path.
- **FR-091**: Every insight MUST carry its **evidence window, n, data density, and the metric definition used**.
- **FR-092**: The system MUST refuse to produce a numeric result below a configured minimum data threshold, and MUST state what is missing.
- **FR-093**: Every review MUST list **other logged changes inside its window** (substances started/stopped, dose changes, actions) so confounders are visible.
- **FR-094**: Every insight MUST carry a **limitations statement** generated with it, in the same view as the result.
- **FR-095**: Language MUST be descriptive and self-limited ("over this window, in your data"). The system MUST NOT phrase personal results as population-level findings, clinical conclusions, or medical advice.
- **FR-096**: The system MUST account for **multiple comparisons** across a scan and MUST label scan-derived results as candidates, never as tested findings.
- **FR-097**: Notes naming a catalog substance MUST generate a **candidate** for review; the system MUST NOT generate a conclusion from a note alone.
- **FR-098**: Insights MUST be persisted as durable ledger records with status (`proposed`, `tested`, `supported`, `unsupported`, `inconclusive`), evidence links, and outcome date.
- **FR-099**: Reviews MUST be reproducible: the same data window and metric definitions MUST yield the same result, verified by test vectors.
- **FR-100**: Insights MUST work fully offline.
- **FR-101**: Gaps in ingested data inside a review window MUST be reported inside that review.

### Key Entities

- **Intervention**: `id`, `substance_id` or `action_id`, `started_at`, `ended_at: Option`, `note_id: Option` (origin), `status`.
- **ReviewSpec**: `intervention_id`, `metric`, `metric_definition`, `pre_window`, `post_window`, `min_points`, `created_at`.
- **ReviewResult**: `review_id`, `pre_summary`, `post_summary`, `n_pre`, `n_post`, `density`, `effect_size: Option`, `confounders: Vec<...>`, `gaps: Vec<...>`, `limitations: String`, `verdict` (`supported` | `unsupported` | `inconclusive` | `insufficient_data`).
- **InsightLedgerEntry**: existing `Insight` (spec 001) extended with `status`, `review_id`, `outcome_at`, `user_note`.

## Success Criteria *(mandatory)*

- **SC-090**: A user can go from a note to a set-up review in under 60 s.
- **SC-091**: 100% of displayed numeric insights show n, window, density, and limitations. Verified by test that no result path can render without them.
- **SC-092**: Below the data threshold, the system produces zero numeric results (verified by test).
- **SC-093**: A review's confounder list matches the log for its window in a fixture with known concurrent changes.
- **SC-094**: A user can list everything they have tested and its outcome, in under 5 s.
- **SC-095**: Review results are reproducible from stored data (test vectors, wasm32 and native identical).

## Assumptions

- A dataframe/join engine will be evaluated for the joins (candidate: `polars`, 15M downloads) and a statistics crate for the descriptive stats (candidates: `statrs`, `linregress`). **Adoption gated on licence verification — not yet performed.**
- v1 is descriptive statistics only: medians, deltas, rank comparisons, simple trend fits. No Bayesian modelling, no causal inference machinery.
- Minimum data thresholds are user-configurable with conservative defaults; defaults are documented in the UI so the user knows what "enough" meant.
- Insights are personal observations, not clinical findings.

## Out of Scope for v1

- Population-level claims, comparison against other users, cohort data.
- Automated stack recommendations ("you should take X"). The system reports what its data shows; the user decides.
- Fitting individual pharmacokinetics from vitals (see 003, out of scope).
- Any insight path that involves a model, an LLM, or a network call.
