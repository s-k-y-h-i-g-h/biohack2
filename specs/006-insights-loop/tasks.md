# Tasks: Insights Loop — Evidence From the Dose Log and Vitals

**Input**: Design documents from `/specs/006-insights-loop/`
**Branch**: `006-insights-loop`
**Date**: 2026-10-11
**Status**: Proposed — not started. Task numbering starts at T600 (001: T001–T118; 003: T200–T257; 004: T300–T342; 005: T400–T515).

## Legend
- `[X]` = Completed and verified
- `[~]` = Partially implemented
- `[ ]` = Not implemented

## Scope

**v1**: intervention tracking, pre/post reviews with hard data gates, note-derived candidates, drift detection, insight ledger.
**v2 (deferred)**: population comparison, automated recommendations, causal modelling.

**Gate**: no numeric insight renders without n, window, density, and limitations. Enforced in code, not in review.

---

## Phase 1: Intervention Tracking

**Purpose**: The unit of self-experiment. Without it there is nothing to review.

- [ ] T600 Add `Intervention` entity to `engine/src/models.rs` (substance/action ref, start, optional end, origin note, status)
- [ ] T601 Persist interventions (SQLite + WASM storage paths)
- [ ] T602 Implement intervention creation from a catalog item, with start date
- [ ] T603 Implement intervention end/resolution
- [ ] T604 Implement auto-detection of candidate interventions from the log: a substance newly appearing after a period of absence → proposed intervention (user confirms, does not auto-create)
- [ ] T605 Implement the confounder query: all logged changes (started/stopped substances, dose changes, new actions) inside a given window (FR-093)
- [ ] T606 Unit tests: creation, resolution, confounder query returns concurrent changes from a fixture

**Checkpoint**: An intervention is a tracked object with a window and a confounder list.

---

## Phase 2: Review Engine (Deterministic, Gated)

**Purpose**: The comparison, with the refusal built in.

- [ ] T610 Record the library decision in `research.md`: dataframe/join engine (candidate `polars`) and statistics (candidates `statrs`, `linregress`) — **licence verified before code**
- [ ] T611 Implement `ReviewSpec` (intervention, metric, metric definition, pre/post windows, min points)
- [ ] T612 Implement the metric series builder: vitals (+ ingested data from spec 004) and simulated state (from spec 003) joined on time
- [ ] T613 Implement the **data gate**: below `min_points` on either side → `insufficient_data`, with the missing amount stated (FR-092, D1/AC-2)
- [ ] T614 Implement descriptive summaries: median, IQR, n per side (no significance claims to a population)
- [ ] T615 Implement effect size as a descriptive delta with its definition stated
- [ ] T616 Implement density computation: points per day, gap count, coverage fraction (FR-091)
- [ ] T617 Implement gap surfacing inside the window (FR-101)
- [ ] T618 Implement the limitations generator: window, n, density, confounders present, "descriptive of your data only" (FR-094)
- [ ] T619 Implement the confounder block in the result (FR-093)
- [ ] T620 Implement `verdict` derivation: `supported` / `unsupported` / `inconclusive` / `insufficient_data` with the rule documented
- [ ] T621 **Structural test**: no code path can produce a numeric result without n, window, density, and limitations attached (SC-091)
- [ ] T622 **Gate test**: below threshold, zero numeric results produced (SC-092)
- [ ] T623 Fixture test: confounder list matches a fixture with known concurrent changes (SC-093)
- [ ] T624 Test vector: same window + definitions → same result, wasm32 and native identical (FR-099)
- [ ] T625 Test: day-boundary aggregation across a DST transition (the 2026-09-09 chrono precedent)

**Checkpoint**: Reviews are reproducible, gated, and self-describing. Refusal is a first-class output.

---

## Phase 3: Note-Derived Candidates

**Purpose**: Close the loop from qualitative observation to testable review.

- [ ] T630 Implement substance resolution from note text against the catalog (reuse the token-boundary matching approach that fixed the "Maori Green Mussel" false positive — do not use naive substring matching)
- [ ] T631 Implement candidate generation: note + resolved substance → pairing with available outcome metrics
- [ ] T632 Implement untestable marking: candidate with no measurable outcome → marked with the reason (D2/AC-3)
- [ ] T633 Implement "set up review" from a candidate → creates a `ReviewSpec` with the note as origin
- [ ] T634 Implement the candidate list view: pairing, data availability, origin note, tested status
- [ ] T635 Unit tests: resolution, candidate generation, untestable marking, note as review origin
- [ ] T636 Test: a note alone never produces a conclusion (FR-097)

**Checkpoint**: Notes become testable hypotheses, never verdicts.

---

## Phase 4: Drift Detection

**Purpose**: The signal personal tracking is genuinely good at, with no causal claim.

- [ ] T640 Implement the drift window: rolling window over a metric with a density requirement
- [ ] T641 Implement trend estimation (simple, documented method — e.g. least-squares slope on medians per day)
- [ ] T642 Implement the density qualifier: gaps in the window reported and the result qualified (D3/AC-2)
- [ ] T643 Implement the no-data case: no insight produced (D3/AC-3)
- [ ] T644 Implement drift insight generation with window, direction, magnitude, n
- [ ] T645 Unit tests: fixture with a known rise produces the expected drift report; sparse fixture is qualified; empty fixture produces nothing

**Checkpoint**: Sustained drift is detected and reported with its density caveats attached.

---

## Phase 5: Insight Ledger

**Purpose**: Make conclusions durable so the user stops re-testing the same thing.

- [ ] T650 Extend `Insight` in `engine/src/models.rs` with `status` (`proposed`/`tested`/`supported`/`unsupported`/`inconclusive`), `review_id`, `outcome_at`, `user_note`
- [ ] T651 Persist ledger entries (SQLite + WASM storage paths)
- [ ] T652 Implement outcome recording from a completed review
- [ ] T653 Implement ledger listing: everything tested, status, date, evidence link (SC-094)
- [ ] T654 Implement evidence drill-down: from ledger entry to its data window (D4/AC-2)
- [ ] T655 Implement suppression: tested candidates are not re-proposed as new (D4/AC-3)
- [ ] T656 Unit tests: outcome recording, suppression, evidence link integrity

**Checkpoint**: Conclusions persist and shape what the app proposes next.

---

## Phase 6: Multiple-Comparison Honesty

**Purpose**: The scan is a false-positive generator. Account for it.

- [ ] T660 Implement scan accounting: when the system evaluates many pairings, record the scan size
- [ ] T661 Implement the labelling rule: scan-derived results are `proposed` candidates, never `tested` findings (FR-096)
- [ ] T662 Implement the UI distinction: candidates visually separated from tested results
- [ ] T663 Test: a scan result cannot enter the ledger as `supported` without a `ReviewSpec` (FR-096)

**Checkpoint**: The system does not launder a scan into a conclusion.

---

## Phase 7: UI & Presentation

- [ ] T670 Implement the Insights page: candidates, reviews, ledger
- [ ] T671 Implement the review detail view: result + n + window + density + confounders + limitations, all in one view (FR-094, SC-091)
- [ ] T672 Implement the insufficient-data view: what is missing, what would be enough
- [ ] T673 Implement the intervention setup flow (form, not panel — spec 005 FR-079)
- [ ] T674 Implement the candidate → review flow (SC-090: note to set-up review in under 60 s)
- [ ] T675 Implement the dashboard insight panel in the shared `ui` crate (spec 005 Phase 2) — terminal and web
- [ ] T676 Accessibility: results readable as text, not colour-coded severity alone

**Checkpoint**: Insights are visible in both surfaces, with limitations always adjacent to results.

---

## Sequencing & Dependencies

```
Phase 1 (interventions) ──> Phase 2 (review engine) ──┬──> Phase 3 (notes→candidates)
                                                       ├──> Phase 4 (drift, parallel)
                                                       ├──> Phase 5 (ledger)
                                                       └──> Phase 6 (scan honesty) → Phase 7 (UI)
```

- Phase 2 is the core and the hard part. Ship it with the gate enforced before any UI exists.
- Phase 4 (drift) is independent of Phase 3 and can run in parallel.
- Phase 7 depends on spec 005 Phase 2 for the shared panel.
- Spec 003 (simulated state) and spec 004 (dense vitals) both feed Phase 2's series builder. **Phase 2 is testable on hand-logged vitals alone** — do not block it on 003/004.

## Critical Constraints (do not violate)

1. **Deterministic only.** No model, no LLM, no network in the result path (FR-090).
2. **Refusal is a feature.** Below threshold → no number, stated plainly (FR-092).
3. **Limitations are inseparable from results** (FR-094, SC-091) — enforced structurally, not by convention.
4. **Descriptive language only.** "Over this window, in your data." Never population, clinical, or advice framing (FR-095).
5. **Confounders are shown, never hidden** (FR-093).
6. **A scan is not a finding** (FR-096).
7. **Reproducibility is tested** on both targets (FR-099).
