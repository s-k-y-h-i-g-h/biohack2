# Feature Specification: Substance Core — Active Dose & Redose Windows

**Feature Branch**: `003-substance-core-pharmacokinetics`

**Created**: 2026-10-11

**Status**: Draft — proposed, not ratified

**Depends on**: [001-biohacker-tracking-platform](../001-biohacker-tracking-platform/spec.md) (LogEntry, CatalogItem, Alert, safety engine)

**Input**: Planning discussion 2026-10-11. The catalog already carries `half_life` per substance and the log already carries every dose with a timestamp. Nothing in the app currently uses the first to interpret the second. The feature is: **turn the dose log into a time-resolved picture of what is on the body right now, and alert on the redose window.**

## Problem Statement

biohack2 records *events*. A biohacker's actual question is about *state*:

- "How much caffeine is still on board right now?"
- "When does my magnesium level drop below where I want it?"
- "I redosed creatine three times today — is that cumulative load or is it a plateau?"
- "My stack has been running for 6 weeks. Is anything accumulating?"

Every one of these is answerable from data the app already stores, and none of them are answered today. The dose log is a list; the useful object is a curve.

## User Scenarios & Testing *(mandatory)*

### User Story A1 - See Active Dose Over Time (Priority: P1) 🎯 MVP

A user opens the dashboard and sees, for each active substance, a band showing how much is estimated to be on board across the last 24 hours and the next 12, with "now" marked. Caffeine logged at 08:00 shows a decaying band; a 15:00 coffee visibly re-raises it.

**Why this priority**: This is the single highest-value transformation of data the app already has. It converts a log into a model the user can act on, and it is the panel the CLI dashboard and the web dashboard both need.

**Independent Test**: Log 200 mg caffeine at 08:00 with a catalog half-life of 5 h → the panel shows a decaying curve peaking near the absorption time and roughly halving every 5 h; at 18:00 the displayed amount is approximately a quarter of peak.

**Acceptance Scenarios**:

1. **Given** a dose log for one substance, **When** the panel renders, **Then** an estimated on-board amount is shown for each time point across the window, derived only from the log and catalog parameters.
2. **Given** two doses of the same substance, **When** the second falls inside the first's decay window, **Then** the curve shows the summed contribution, not the latest dose alone.
3. **Given** a substance with no numeric half-life in the catalog, **When** the panel renders, **Then** the substance is shown as "no model" rather than being silently omitted or drawn with an invented value.

---

### User Story A2 - Redose Window Alerts (Priority: P1)

A user targeting 300–500 mg of magnesium on board sees the band fall below target at 21:00 and the app raises a redose alert with the window in which redosing keeps the band in range.

**Why this priority**: This is the actionable half of A1. A curve the user must read is a chart; a curve that speaks is a feature.

**Independent Test**: Set a target band for a substance → let the simulated band cross below the lower edge → an `Alert` is generated with type redose, the crossing time, and the recommended window.

**Acceptance Scenarios**:

1. **Given** a target band configured for a substance, **When** the simulated on-board amount crosses below the band, **Then** an alert is generated naming the substance, the crossing time, and the redose window.
2. **Given** an alert is raised, **When** the user logs a dose that restores the band, **Then** the alert resolves on the next render.
3. **Given** the user has no target band configured, **When** the panel renders, **Then** no redose alert is raised for that substance.

---

### User Story A3 - Plateau and Accumulation Awareness (Priority: P2)

A user running creatine for 6 weeks sees a plateau curve (saturation), not a decay curve. A user running an SSRI sees a rising-to-steady-state curve. A user running a fat-soluble vitamin sees accumulation across days.

**Why this priority**: Applying a naive decay model to these substances produces confidently wrong numbers, which is worse than no number. The model-class distinction is what keeps the feature honest.

**Independent Test**: Assign model class `plateau` to creatine → the curve saturates at a ceiling and does not decay to zero between doses. Assign `accumulation` to vitamin D → the curve rises across days toward a steady state.

**Acceptance Scenarios**:

1. **Given** a substance assigned model class `plateau`, **When** doses are logged daily, **Then** the curve saturates and does not return to baseline between doses.
2. **Given** a substance assigned model class `accumulation`, **When** doses are logged daily across a week, **Then** the curve trends upward toward a steady state.
3. **Given** a substance assigned `decay` (default), **When** doses are logged, **Then** the curve decays toward baseline.

---

### User Story A4 - Cumulative Load Guardrails (Priority: P2)

A user logs a third stimulant dose within 12 hours. The app raises a cumulative-load alert referencing the catalog's max daily dosage, distinct from the existing pairwise interaction alerts.

**Why this priority**: The safety engine currently reasons about *pairs* (FR-015) and about *vitals* (FR-008). It does not reason about *total load over time on one substance*. That is the missing axis, and it is the axis the dose curve makes visible.

**Independent Test**: Log doses totalling more than the catalog's max daily amount for one substance within 24 h → cumulative-load alert fires with the running total and the limit.

**Acceptance Scenarios**:

1. **Given** a substance with a catalog max daily amount, **When** logged doses within a rolling 24 h exceed it, **Then** a cumulative-load alert fires naming the total and the limit.
2. **Given** the same substance logged across two days, **When** the rolling window is evaluated, **Then** only doses inside the window count.

---

### User Story A5 - Honest Uncertainty (Priority: P3)

A user sees the caffeine estimate as a **band**, not a line, and can open a note explaining that the band comes from the catalog's half-life range and inter-individual variation, and that the true value is not knowable from a dose log.

**Why this priority**: This is the integrity constraint on the whole feature. A single line implies a precision the data cannot support, in an app whose purpose is informing health decisions.

**Independent Test**: A substance with catalog half-life "5–6 hours" renders a band whose width reflects that range; widening the half-life range widens the band.

**Acceptance Scenarios**:

1. **Given** a half-life range, **When** the curve renders, **Then** the band is computed from the range endpoints, not from a midpoint.
2. **Given** a single half-life value with no range, **When** the curve renders, **Then** the band is widened by a documented default variability factor and the panel states that it is an assumption.
3. **Given** the user opens the explanation, **When** they read it, **Then** it names the inputs (dose log, catalog half-life, model class) and states that this is not a measurement.

---

### Edge Cases

- **Missing numeric half-life.** Catalog `half_life` is currently `Option<String>` (e.g. `"5-6 hours"`). Parsing must be explicit and failure must be visible: "no model", never a silent default.
- **Dose unit mismatch.** A dose logged in a unit the model does not expect must not be silently converted; it must block the model and surface the mismatch.
- **Model class absent.** Default is `decay`. A substance with no class is modelled as decay and labelled as an assumption.
- **Very long half-lives.** A 5-day half-life over a 30-day window must not blow up the render window; the panel windows per substance.
- **Clock and timezone.** Dose timestamps are stored UTC. Curves render in local time. This is the same class of bug that broke all write flows in 2026-09-09 (chrono `wasmbind`/`now`/`clock`); the curve code must be covered by a WASM test, not just a native test.
- **WASM determinism.** The same log must produce the same curve in the browser and in the CLI. Floating-point drift between targets is a real risk and must be tested, not assumed.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-030**: The system MUST store a **numeric** half-life (hours) per catalog item, with an optional range, in addition to the existing human-readable `half_life` string. Existing string values MUST be migrated by explicit parse with a recorded failure list — never by silent coercion.
- **FR-031**: The system MUST assign each modelled substance a **model class** from a fixed set: `decay` (one-compartment, default), `oral_absorption` (decay with lag/absorption phase), `plateau` (saturation), `accumulation` (multi-day rise to steady state), `none` (explicitly not modelled). The class MUST be data, editable, and versioned with catalog releases.
- **FR-032**: The system MUST compute estimated on-board amount at arbitrary times **deterministically** from the dose log, the model class, and the parameters — offline, no network, no model inference. Rationale is the same as FR-015: a model is not a safe gate for health-relevant output.
- **FR-033**: The system MUST render estimated on-board amount as a **band** (low / expected / high) derived from the parameter range, and MUST NOT present a single line as if it were a measurement.
- **FR-034**: The system MUST support a user-configured **target band** per substance and MUST raise an `Alert` when the simulated band crosses below it, including the crossing time and a redose window.
- **FR-035**: The system MUST evaluate **rolling-window cumulative dose** per substance against the catalog maximum daily amount and raise a distinct `Alert` type for cumulative load, separate from pairwise interaction alerts.
- **FR-036**: Every parameter used in a simulation MUST carry **provenance** — catalog default, catalog release version, or user override with timestamp — and the panel MUST be able to display it.
- **FR-037**: The system MUST expose the simulation inputs (substance, model class, parameters, dose events) for any displayed curve, so a user can reproduce a number by hand.
- **FR-038**: Simulation MUST work fully offline and MUST produce identical output in the WASM frontend, the native CLI, and the server.
- **FR-039**: Simulation output MUST be labelled as an estimate derived from logged doses and reference parameters, and MUST NOT be presented as a measurement, diagnosis, or medical advice.
- **FR-040**: The system MUST NOT simulate substances for which it has no defensible parameter set. Explicit absence is a required state.

### Key Entities

- **SubstanceModel**: `id`, `catalog_item_id`, `model_class`, `half_life_hours: f64`, `half_life_range: Option<(f64, f64)>`, `params: serde_json::Value` (absorption rate, lag, ceiling, etc.), `provenance`, `catalog_version`.
- **DoseEvent**: derived view over `LogEntry` — `substance_id`, `amount`, `unit`, `timestamp`, `entry_id`. The simulation input.
- **ActiveDosePoint**: `t`, `low`, `expected`, `high`, `unit` — one rendered sample.
- **TargetBand**: `substance_id`, `low`, `high`, `window_hours`, `updated_at`.
- **RedoseAlert**: an `Alert` with `alert_type = redose`, carrying crossing time, window, and the parameter provenance.

## Success Criteria *(mandatory)*

- **SC-030**: A 30-day active-dose series for 10 substances renders in under 500 ms on desktop and under 2 s on a mid-tier phone browser.
- **SC-031**: Redose alerts appear within 5 s of opening the dashboard, computed from stored data with no network call.
- **SC-032**: 100% of displayed curve points are reproducible from the dose log plus recorded parameters, verified by test — no hidden state, no smoothing, no fitted values.
- **SC-033**: Identical input produces identical output on wasm32 and x86_64 (bit-exact within a documented tolerance), verified by a shared test vector.
- **SC-034**: Every modelled substance displays its model class and parameter provenance in one interaction.
- **SC-035**: A user can go from "I logged a dose" to "I see the curve move" in under 5 seconds.

## Assumptions

- Reference parameters ship with catalog releases (consistent with FR-006); the app does not fetch pharmacokinetic data at runtime.
- Simulation is a **population-estimate** tool for personal protocol tuning, not a clinical tool. Individual pharmacokinetics vary beyond what a dose log can reveal.
- A pharmacometric library will be evaluated for the compartment math (see [research](../001-biohacker-tracking-platform/research.md) §Ecosystem survey 2026-10-11: `pharmsol`). **Adoption is gated on license verification** — not yet performed.
- v1 models oral and IV routes. Dermal, inhaled, and food-matrix absorption are out of scope.
- v1 does not attempt drug–drug PK interactions (CYP-mediated exposure changes). That is a v2 candidate; FR-015 covers pharmacodynamic pairs only.

## Out of Scope for v1

- Therapeutic drug monitoring, blood-concentration claims, clinical dosing.
- Model fitting to user vitals (inferring an individual's half-life from their data) — tempting, statistically treacherous at personal sample sizes, and a false-precision generator.
- Drug–drug pharmacokinetic interaction modelling.
