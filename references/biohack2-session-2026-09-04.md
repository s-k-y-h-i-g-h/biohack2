# Session Reference: Biohacker Tracking Platform

**Date**: 2026-09-04
**Repository**: `github.com/s-k-y-h-i-g-h/biohack2`
**Feature**: 001-biohacker-tracking-platform

## GitHub Repository Discovery

User's GitHub username: `s-k-y-h-i-g-h`

### Relevant Repositories
| Repo | Language | Description | Relevance |
|------|----------|-------------|-----------|
| **biohack** | Rust | "Software which makes biohacking easier" | **Direct predecessor** — 27-substance database, 3 safety protocols |
| **personal-homeostasis-tracker** | — | No description | Related domain work |
| **Biohacker** | — | "Social biohacking web application" | Previous attempt at web app |
| **SelfPlusPlus** | C# | "You++" | Self-improvement tracking |
| **codel** | — | Autonomous AI agent | AI infrastructure context |
| **honcho** | Python | Memory library for stateful agents | Agent memory patterns |

### Non-Relevant (False Positives)
- `skyhighblockchain/skyhigh` — Unrelated blockchain project (same "SkyHigh" string in search results)

**Lesson**: Always verify GitHub username format when searching (`s-k-y-h-i-g-h` vs `SkyHigh`). Use API directly: `https://api.github.com/users/s-k-y-h-i-g-h/repos`.

## Existing Codebase: biohack CLI

### What It Provides
- **27 curated substances** with dose ranges, half-lives, contraindications
- **3 deterministic safety protocols**:
  1. Stimulant tachycardia (HR > 100 bpm + stimulant within 4h)
  2. Hypertensive urgency (SBP ≥ 180 or DBP ≥ 120)
  3. Serotonin syndrome risk (multiple serotonergic agents)
- Local-first storage via `sled` (Rust embedded database)
- CLI interface with `biohack log`, `biohack show`, `biohack check` commands
- Stack logging from YAML files
- Timeline view and report generation (markdown/csv)

### Integration Decision
The new app reuses the `biohack` Rust engine as a shared library. This ensures:
- Consistent safety logic across CLI and GUI
- No duplication of tested code
- Easy extension with additional protocols

## Platform Evolution

### Initial Assumption → Correction

**Assumed**: Native mobile (iOS + Android) for v1
**Actually wanted**: Web frontend for v1, native mobile deferred to v2

**User's explicit direction**:
1. "We probably want native from the start"
2. "We probably want desktop frontends too"
3. "We probably want to create the web frontend first and defer the mobile apps"
4. "The cloud backend can probably be deferred too"

**Final v1 scope**:
- Web application (TypeScript + SolidJS)
- Local-first storage (IndexedDB/WASM SQLite)
- Fully offline, no account required
- Cloud sync deferred to v2

**v2 deferred**:
- iOS native app (SwiftUI)
- Android native app (Jetpack Compose)
- BLE wearable integration
- Cloud sync with OAuth

## Key Design Decisions

### Local-First Architecture
- All user stories work fully offline
- Data stored locally (SQLite via WASM)
- Cloud sync optional background feature
- User can export data for migration

### Safety Protocols
Carried forward from `biohack` CLI:
- Stimulant tachycardia: HR > 100 bpm + stimulant in last 4h
- Hypertensive urgency: SBP ≥ 180 or DBP ≥ 120
- Serotonin syndrome risk: multiple serotonergic agents

### Catalog Management
- Seed database of 27 substances from `biohack`
- Updates shipped with app releases (no background refresh)
- User can create custom catalog entries

### Vitals Alerting
- Clinical thresholds (not just personal baseline)
- Tachycardia: HR > 100 bpm
- Hypertension: SBP ≥ 180 or DBP ≥ 120
- Contextual advice derived from user's log history

## Technical Stack (v1)

### Frontend
- **Language**: TypeScript 5.x
- **Framework**: SolidJS (reactive, local-first patterns)
- **Storage**: IndexedDB (fallback) / SQLite via WASM
- **Build**: Vite or similar

### Backend
- **Language**: Rust 2024
- **Purpose**: Shared safety engine, catalog management
- **Reuse**: Direct import from `biohack` crate

### Cloud (v2)
- **Auth**: OAuth 2.0 / OIDC
- **Storage**: Encrypted PostgreSQL
- **Sync**: Client-side encryption, server stores ciphertext only

## Session Artifacts

Generated files in `specs/001-biohacker-tracking-platform/`:
- `spec.md` — Feature specification (21 requirements, 10 success criteria)
- `plan.md` — Implementation plan (technical context, project structure)
- `research.md` — Research decisions (local-first DB, catalog updates, auth)
- `data-model.md` — Entity definitions (LogEntry, CatalogItem, Stack, VitalsEntry, Alert, Insight)
- `quickstart.md` — Validation scenarios (10 E2E tests)
- `contracts/api.md` — API contracts (sync, interactions, vitals)

## User Motivation Context

From conversation:
> "The healthier I am the more resources (and money) I can throw your way (for upgrading your capabilities and stuff like that). The better this project is, the more opportunity I have to be healthier."

This is a self-reinforcing loop — project success directly tied to personal health outcomes. Worth noting for prioritization decisions.

## Lessons Learned

1. **Verify GitHub usernames** — `s-k-y-h-i-g-h` is not `SkyHigh`; API search is more reliable than web search
2. **Ask about v1 scope explicitly** — Don't assume mobile/native; users often want web-first
3. **Local-first is a feature, not a limitation** — Many personal tools benefit from offline operation
4. **Existing codebase is gold** — The `biohack` CLI saved significant implementation effort
5. **Platform decisions are iterative** — Users refine their thinking as they see proposals; expect course corrections
