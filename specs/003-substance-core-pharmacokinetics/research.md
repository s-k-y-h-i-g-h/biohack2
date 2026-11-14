# Spec 003 — Research & Decisions

## T209 — PK library decision: **hand-roll** (2026-11-14)

**Decision:** implement the Phase 2 simulation in-repo (`engine/src/pharmacokinetics.rs`),
no external PK library. No third-party code is adopted, so constraint 4
("licence before code for any adopted PK library") has nothing to gate.

**Why:**
- Every Phase 2 model is a closed-form analytic: one-compartment first-order
  decay (`0.5^(t/t½)`), superposition by summation, lag/absorption as a
  Bateman function, plateau as a saturating exponential, accumulation as a
  multi-day geometric rise. The spec itself scopes decay as "analytic, no ODE
  solver needed" (T212). A library buys an ODE solver we have decided not to
  need.
- Determinism across native/WASM (FR-038) is easier to defend with plain
  `f64` ops in our own code than inside a dependency we do not control.
- The engine crate deliberately carries no logging stack to keep the WASM
  bundle lean (see the tracing removal comment in `engine/Cargo.toml`); a
  full PK framework cuts against that.

**Alternative considered — `pharmsol`:** the most complete open PK framework.
Its licence **was not verifiable at decision time** (this machine has had no
internet since 2026-11-14 ~19:50; verification requires fetching the upstream
repo). The hand-roll decision is independent of that licence outcome: even
permissive, pharmsol is C++/Python with heavy machinery for exactly the models
we implement analytically. If a future need arises for PBPK beyond the four
model classes, re-open this decision **with the licence verified first**.

**Verified at decision time:** our own dependency set only — the decision adds
zero new dependencies, which is the only licence surface it touches.

## T218 — Default variability factor for single-value half-lives (2026-11-14)

Documented default: **±25 % on the half-life** (`DEFAULT_VARIABILITY = 0.25`).
Band edges are computed by evaluating the curve at `t½ × (1 − 0.25)` (fast
clearance → `low`) and `t½ × (1 + 0.25)` (slow clearance → `high`); the
expected line uses the nominal `t½`. Rationale: inter-individual variability
of elimination half-life for common supplements/drugs commonly lands in the
20–30 % band, so 25 % neither hides uncertainty nor fabricates a wider one.
When a catalog range exists it is used instead and no assumption is applied.
Series points carry `band_is_assumption` so the panel can state the
assumption (FR-033 / A5 / AC-2).
