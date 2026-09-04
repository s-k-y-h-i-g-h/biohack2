# Research Report: Biohacker Tracking Platform

**Date**: 2026-09-04
**Feature**: 001-biohacker-tracking-platform

## Unknowns Resolved

### 1. Local-first database for web (IndexedDB vs SQLite)

**Decision**: IndexedDB for pure web, with SQLite via Tauri/Electron as upgrade path for desktop.

**Rationale**:
- IndexedDB is built into browsers, requires no install, works fully offline
- Web-based local-first pattern proven by CRDT libraries (Automerge, Yjs)
- If desktop app needed later, SQLite (via `better-sqlite3` or Tauri) provides ACID guarantees
- Migration path: keep schema abstraction layer so swapping backends is low-cost

**Alternatives considered**:
- **SQLite via WASM** (`sql.js`): Works in browser but no native persistence between sessions without Service Worker + Cache API workaround — rejected
- **SQLite via Tauri**: Adds native dependency; viable if desktop-first strategy, but adds complexity for web-only MVP — deferred
- **PouchDB/CouchDB-compatible**: Overkill for single-user local-first; introduces eventual consistency complexity — rejected

### 2. Catalog update mechanism (app releases vs periodic refresh)

**Decision**: Manual seed database updates shipped with app releases. No background refresh.

**Rationale**:
- Simpler architecture: no background worker, no update server needed
- Aligns with "updates ship with app versions" clarification
- Prevents rate-limiting issues with external data sources
- Seed database (27 substances from biohack) is versioned alongside app

**Alternatives considered**:
- **Weekly background refresh**: Requires background service worker + error handling — added complexity without proportional benefit for v1
- **On-demand manual refresh button**: User must remember to update; worse UX than app release cadence

### 3. Auth model for cloud sync (OAuth vs custom)

**Decision**: OAuth 2.0 / OIDC for cloud sync; local-only mode requires no auth.

**Rationale**:
- OAuth is standard, battle-tested, supports PKCE for SPAs
- Local-only mode is primary use case; cloud is opt-in
- Enables potential future social features without re-architecting auth

**Alternatives considered**:
- **Magic link email auth**: More friction for single-user local-first app; email delivery adds failure mode
- **API key only**: No session management; less secure for cloud data
- **Custom username/password**: Reinvents wheel; OAuth is better practice

### 4. Safety protocol implementation (extend biohack vs reimplement)

**Decision**: Reuse `biohack` Rust engine directly; embed as shared library or CLI call.

**Rationale**:
- `biohack` already has 3 tested safety protocols (stimulant tachycardia, hypertensive urgency, serotonin syndrome)
- Reuse avoids duplication and ensures consistency
- Rust engine can be called from frontend via Wasm or from a local sync service

**Alternatives considered**:
- **Reimplement in TypeScript**: Duplicates logic; risk of divergence from tested Rust code
- **Call `biohack` CLI**: Adds dependency on binary installation; breaks "no install" PWA goal

### 5. Cloud encryption strategy

**Decision**: Client-side encryption before upload; server never sees plaintext.

**Rationale**:
- Meets FR-019 (encrypt at rest) and FR-020/021 (delete/export)
- User retains full control; even provider cannot read data
- Standard pattern for privacy-first apps (Signal, ProtonMail)

**Alternatives considered**:
- **Server-side encryption with user key**: More complex key management; server still touches encrypted blobs
- **End-to-end with shared keys**: Unnecessary for single-user; adds key distribution problem

## External Data Sources

### Drug interaction database

**Decision**: Source from DrugBank (public API) or build from FDA/EMA published data.

**Rationale**:
- DrugBank has structured interaction data with DOIs for verification
- FDA publications provide additional coverage for OTC supplements
- Community-maintained databases (e.g., OpenFDA) are free but may have coverage gaps

**Known gaps**:
- Natural product interactions less documented than pharmaceuticals
- Some "folk knowledge" interactions not in academic literature
- Always include disclaimer about data incompleteness

### Clinical reference ranges

**Decision**: Use established clinical thresholds (not user baselines) as primary alert source.

**Rationale**:
- Per clarification: alerts trigger on clinical conditions (tachycardia, hypertension), not just baseline deviation
- Reference ranges from WHO, AHA, and other medical bodies are well-established
- User-configured baselines are secondary (for tracking personal trends)

**Key thresholds**:
- Tachycardia: HR > 100 bpm (adult, resting)
- Hypertensive urgency: SBP ≥ 180 or DBP ≥ 120
- Hypotension: SBP < 90 or DBP < 60 (for completeness)
- SpO2 < 95% (respiratory concern)
- Temperature > 38°C or < 35°C (fever/hypothermia)
