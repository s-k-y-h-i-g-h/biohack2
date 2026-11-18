# Spec 007 — Food & Nutrition Tracking (biohack1 diet port)

## Why

biohack1 shipped a diet feature (USDA FoodData Central + OpenFoodFacts
lookup, per-entry cached nutrition, daily status vs RDI/UL) that was never
ported to v2. v2 accepts `item_type='food'` log entries but they carry no
nutrition — the Food category is a dead end in the dashboard. The original
spec (001, FR set) lists food logging as a first-class capability; this
closes that gap on the web architecture.

## What moves / what changes

| biohack1 | v2 | change |
|---|---|---|
| `src/nutrient_ref.rs` (RDI/DRI table, status levels) | `engine/src/nutrient_ref.rs` | ported verbatim; `status_label` emoji markers dropped for UI tokens |
| `src/food_db.rs` (reqwest clients) | `server/src/food_lookup.rs` | server-side proxy (API key must not ship in WASM); TTL cache; OFF+USDA queried concurrently |
| `FoodLog` row w/ `nutrients` column | `LogEntry.custom_fields["nutrition"]` | rides existing SQLite **and** LocalStorage persistence with zero schema change; JSON shape (`Vec<{name,amount,unit}>`) is biohack1-compatible for data import |
| per-unit conversion table (cups→g, guessed factors) | dropped | grams-only at log time; never invent conversion constants |
| USDA raw nutrient names (`Total lipid (fat)`) | canonical names via USDA nutrient id | fixes biohack1 bug where raw names silently missed the reference table, so %DV/status matched only OFF foods |
| completeness heuristics (`NutrientCompleteness`) | deferred | UI shows the nutrient list directly; re-add only when a concrete need appears |

## Data-flow invariants

- Nutrients are scaled per-100g → consumed **at log time** and cached on the
  entry; day totals are pure sums over cached payloads → history and daily
  summaries work fully offline (spec 001 local-first guarantee).
- `zero` from an upstream source means *not reported* → dropped, never
  stored as 0.
- Malformed `custom_fields.nutrition` → entry contributes to
  `entries_without_nutrition`, never to totals; no panics.
- Same nutrient name in different units is never summed blindly — totals key
  on `(name, unit)`.

## Known limitations (documented, not hidden)

- USDA `foods/search` per-100g amounts are approximate for some records;
  OFF `*_100g` fields are exact. OFF is the default (keyless) path.
- Gram quantities only; no spoon/cup conversion.
- Reference values are adults 19-50 (FDA DV / NIH DRI) — one-size thresholds,
  same as biohack1. Not medical advice; same stance as the safety engine.

## Tasks

- [X] T001 `engine/src/nutrient_ref.rs` — ported reference table + status logic + tests
- [X] T002 `engine/src/food.rs` — shared types, `custom_fields` codec, `scale_per_100g`, `aggregate_day`, status mapping + tests
- [X] T003 engine wasm gate (`--no-default-features` wasm_vectors build) green
- [X] T004 `server/src/food_lookup.rs` — OFF/USDA proxy, TTL cache, fixture-parsing tests (no network in CI)
- [X] T005 routes `GET /api/food/search`, `GET /api/food/details/{id}` + `USDA_API_KEY` env gate
- [X] T006 web: food page — search → pick → grams → log (nutrition cached via `set_nutrition`), day summary card (macros + %DV chips, "incomplete day" badge when `entries_without_nutrition > 0`)
- [X] T007 dashboard: today's kcal/protein strip on the existing dashboard
- [ ] T008 e2e: live search smoke test against OFF (keyless) + full-stack build
  - 2026-11-18 live smoke: proxy returns real OFF results (canonical nutrient
    names, per-100g) end-to-end; OFF still 503s intermittently (~2 of 3 calls
    at ~1s spacing) — retries live inside one call, UI degrades to a toast.
    TODO: widen retry backoff (currently tight) and add a browser click-through.
