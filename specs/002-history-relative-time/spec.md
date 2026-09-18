# Feature Specification: Show Relative Elapsed Time on History Log

**Feature Branch**: `002-history-relative-time`

**Created**: 2026-09-17

**Status**: Draft

**Input**: User description: "The history page log entries should show how much time has passed between when they were logged and now"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See Time Since an Entry Was Logged (Priority: P1)

A biohacker opens the history page and scrolls through their past supplement, medication, food, action, and vitals entries. Each entry displays, in addition to its timestamp, how long ago it was logged (e.g., "2 minutes ago", "3 hours ago", "5 days ago").

**Why this priority**: This is the core and only requirement of the feature. Relative time gives the user immediate contextual understanding of the recency of each entry without having to mentally compare timestamps against the current time.

**Independent Test**: Can be fully tested by logging entries at known times, opening the history page, and verifying each entry shows a relative elapsed-time label that matches the actual time difference and updates correctly over time. Delivers instant temporal context for every history entry.

**Acceptance Scenarios**:

1. **Given** the user has an entry logged 2 minutes ago, **When** they open the history page, **Then** the entry displays a relative time of "2 minutes ago" alongside its full timestamp.
2. **Given** the user keeps the history page open for several minutes, **When** time passes, **Then** the displayed relative time updates automatically without requiring a page refresh.
3. **Given** an entry was logged more than a day ago, **When** it is displayed, **Then** the relative time is expressed in days (e.g., "5 days ago"), and entries older than a threshold show a stable, readable form (e.g., weeks, months) rather than ever-growing day counts.

---

### Edge Cases

- **Future-dated entries**: What happens when an entry's timestamp is slightly in the future (due to clock skew or a recorded scheduled/start time)? The system should not show a nonsensical negative value (e.g., "-2 minutes ago"); it should show the entry as "just now" or "in <duration>".
- **Very recent entries**: Entries logged within the last few seconds should display "just now" rather than "0 seconds ago".
- **App left open / tab inactive**: When the app regains focus after being backgrounded for a long time, relative labels refresh to the correct current values.
- **Old entries**: Very old entries (e.g., months old) should degrade to a readable relative form and ultimately to the absolute date rather than an unwieldy long duration string.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The history page MUST display, for each log entry, a relative elapsed-time label indicating how long ago the entry was logged (e.g., "just now", "5 minutes ago", "3 hours ago", "2 days ago").
- **FR-002**: Each entry MUST continue to display its absolute timestamp alongside the relative label.
- **FR-003**: Relative elapsed-time labels MUST auto-update over time without requiring a manual page refresh, and MUST reflect accurate values immediately after the page regains focus.
- **FR-004**: The system MUST handle future-dated timestamps gracefully by showing a sensible value (e.g., "just now" or "in <duration>") rather than a negative duration.
- **FR-005**: The system MUST select an appropriate unit (seconds, minutes, hours, days, weeks, months) based on the elapsed duration, with sub-minute entries shown as "just now".
- **FR-006**: Relative labels and absolute timestamps MUST remain consistent with the same underlying timezone/clock that produced the original entry times.

### Key Entities *(include if feature involves data)*

- **Log Entry**: A record of a logged action (supplement/medication/drug/food intake, action, or vitals reading). Has a timestamp of when it was logged, which the relative elapsed-time label is computed from.
- **Relative Elapsed Time Label**: A derived, display-only value computed from the entry timestamp and the current time. It is not stored; it is rendered at display time so it always reflects the true elapsed duration.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of history page entries display a correct relative elapsed-time label that matches the true time difference within 1 minute of when the page is viewed.
- **SC-002**: Relative elapsed-time labels update automatically within 1 minute of the actual elapsed time passing, with zero page refreshes required.
- **SC-003**: A user viewing the history page can determine the recency of any single entry at a glance with no calculation, for 100% of entries.
- **SC-004**: No entry ever displays an incorrect or nonsensical elapsed time (negative values, stale labels after refocus) in normal usage.

## Assumptions

- The history page already displays a timestamp for each entry; this feature augments it with a relative label rather than replacing the absolute timestamp.
- Relative time is computed client-side from the local device clock at the moment of display, so it stays accurate without server round-trips.
- The feature applies uniformly to all entry categories shown on the history page (supplements, medications, drugs, food, actions, vitals) — there is no category-specific behavior.
- No persistence model change is required: elapsed time is derived from the existing logged timestamp and is not stored.