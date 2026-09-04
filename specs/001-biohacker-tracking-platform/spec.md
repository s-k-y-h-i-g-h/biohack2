# Feature Specification: Biohacker Tracking Platform

**Feature Branch**: `001-biohacker-tracking-platform`

**Created**: 2026-09-04

**Status**: Draft

**Input**: User description: "We want an application which helps biohackers by letting them log their supplement/medication/drug/food consumption and actions (like exercise, meditation or using red light therapy and stuff like that). We want to have information about all of those things available (e.g. for supplements we want details like the dosage range and duration of action). We want the application to allow users to inspect their logs so that they can use the information to improve their biohacking stacks/protocols. It might be useful if users can also log notes to remind them of realisations they have had while adjusting their stacks/protocols. The application should automatically flag dangerous drug interactions and stuff like that. The user should be able to log their vitals too. The application should alert the user when their vitals are abnormal and provide advice on how to correct them (using the log to figure out the correct things to do). It should be possible to create stacks/protocols and log them instead of having to log individual items (because there might be too many to log individually). This application needs to provide useful feedback and information and analysis and insights for biohackers."

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.

  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - Log Consumption and Actions (Priority: P1)

A biohacker opens the application and logs that they took Vitamin D3 5000 IU, did 20 minutes of meditation, and used red light therapy for 10 minutes. They also log that they ate a meal containing dairy. All entries are timestamped and stored.

**Why this priority**: Core logging functionality is the foundation of the entire application. Without it, nothing else works. This is the minimum viable feature set — a user who can only log items already has a useful personal record.

**Independent Test**: Can be fully tested by opening the app, selecting an item from the catalog, specifying a dosage/quantity, and confirming the log entry appears in the user's history with the correct timestamp. Delivers value as a personal journal.

**Acceptance Scenarios**:

1. **Given** the user is authenticated, **When** they select "Vitamin D3" from the supplement catalog and specify "5000 IU", **Then** the entry is saved with the current timestamp and appears in their log.
2. **Given** the user wants to log a non-catalogued item, **When** they create a custom entry with name, category, and dosage, **Then** the custom entry is saved and available for future log entries.
3. **Given** the user is logging an action like meditation, **When** they select "Meditation" from the actions catalog and specify duration, **Then** the entry is saved with start time and duration.

---

### User Story 2 - View and Inspect Logs (Priority: P2)

A biohacker opens the application and browses their history. They can filter by date range, category (supplement, medication, drug, food, action), and specific items. They can see a timeline view and a calendar view of their logged entries.

**Why this priority**: The ability to review past entries is essential for pattern recognition and protocol refinement. Without it, users cannot extract value from their logged data.

**Independent Test**: Can be fully tested by logging several items across different days, then opening the history view and verifying that all entries appear, can be filtered correctly, and display accurate timestamps and dosages.

**Acceptance Scenarios**:

1. **Given** the user has logged entries over multiple days, **When** they open the history view, **Then** all entries are displayed in reverse chronological order with accurate data.
2. **Given** the user has many entries, **When** they apply a date-range filter and a category filter, **Then** only entries matching both criteria are displayed.
3. **Given** the user wants to see their supplement intake over time, **When** they select "Supplements" and a date range, **Then** a summary of intake frequency and dosages is displayed.

---

### User Story 3 - Vitals Logging with Abnormal Alerting (Priority: P3)

A biohacker logs their blood pressure (130/85), heart rate (72 bpm), and weight (175 lbs). The system detects that blood pressure is above the user's personal baseline and triggers an alert with contextual advice derived from the user's log (e.g., "Your magnesium intake has been low this week — consider increasing it").

**Why this priority**: Vitals tracking and intelligent alerting converts raw data into actionable guidance, which is a key differentiator for this application. It turns passive logging into active health management.

**Independent Test**: Can be fully tested by logging vitals, configuring alert thresholds, and verifying that out-of-range values trigger alerts with log-derived recommendations. Delivers value as an early warning system.

**Acceptance Scenarios**:

1. **Given** the user has logged vitals within normal ranges, **When** they view their vitals dashboard, **Then** all values are displayed as normal with no alerts.
2. **Given** the user logs a blood pressure reading above their personal threshold, **When** the entry is saved, **Then** an alert is generated with contextual advice referencing the user's supplement and activity log.
3. **Given** the user dismisses an alert, **When** they log new vitals, **Then** the alert is updated or resolved based on the new values.

---

### User Story 4 - Stack and Protocol Management (Priority: P4)

A biohacker creates a "Morning Protocol" stack that includes Vitamin D3, Magnesium, Cold Exposure (10 min), and Coffee (200mg caffeine). They log the entire stack with one tap, and each component is individually recorded with the same timestamp.

**Why this priority**: Biohackers often run multiple interventions simultaneously. Manually logging 10+ individual items per day is tedious and error-prone. Stacks reduce friction and increase logging compliance.

**Independent Test**: Can be fully tested by creating a named stack with multiple items, logging the stack, and verifying that each item in the stack appears as a separate log entry with the same timestamp.

**Acceptance Scenarios**:

1. **Given** the user has created a stack, **When** they tap "Log Stack" and confirm, **Then** every item in the stack is logged individually with the current timestamp.
2. **Given** the user modifies an existing stack (adds an item), **When** they save the updated stack, **Then** future logs of the stack include the new item.
3. **Given** the user wants to log a variation of their stack, **When** they select a stack and adjust dosages before logging, **Then** the adjusted values are logged.

---

### User Story 5 - Drug Interaction Safety Alerts (Priority: P5)

A biohacker attempts to log a new supplement while taking a prescription medication. The system cross-references the combination against known interaction databases and flags a dangerous interaction with a warning and recommended action.

**Why this priority**: Safety is paramount. Flagging dangerous interactions is a critical trust feature that protects users from harm and differentiates the app from a simple journaling tool.

**Independent Test**: Can be fully tested by logging two items known to interact dangerously, and verifying that an interaction warning is displayed before the entry is saved.

**Acceptance Scenarios**:

1. **Given** the user attempts to log two items with a known dangerous interaction, **When** the second item is added, **Then** a prominent warning is displayed describing the interaction risk.
2. **Given** the user acknowledges a drug interaction warning and proceeds, **When** the entry is saved, **Then** the entry is flagged as acknowledged-interaction in the log.
3. **Given** the user logs a new item, **When** the system checks for interactions, **Then** the check completes within 2 seconds.

---

### User Story 6 - Insights and Analysis (Priority: P6)

A biohacker opens the insights dashboard and sees correlations between their supplement intake and vital signs over time. For example: "Your sleep quality has improved 30% on days when you took Magnesium after 6pm."

**Why this priority**: Delivering insights transforms raw data into actionable knowledge. This is the "wow factor" feature that makes the app indispensable for serious biohackers.

**Independent Test**: Can be fully tested by logging correlated data (supplements and vitals) over a period, then viewing the insights dashboard and verifying that relevant correlations are surfaced with supporting data.

**Acceptance Scenarios**:

1. **Given** the user has logged consistent data over 2+ weeks, **When** they open the insights dashboard, **Then** at least one correlation or trend is displayed with supporting data points.
2. **Given** the user wants to understand a specific correlation, **When** they click on an insight, **Then** a detailed view shows the relevant log entries that contributed to the insight.

---

### User Story 7 - Notes and Realizations (Priority: P7)

A biohacker writes a note attached to a specific log entry: "Noticed increased anxiety after taking Ashwagandha at night — will try morning only next time." The note is linked to the entry and visible in the history view.

**Why this priority**: Contextual notes capture the qualitative dimension of biohacking that numbers cannot. They help users remember why they made changes and support long-term protocol refinement.

**Independent Test**: Can be fully tested by adding a note to a log entry, then finding that entry in history and verifying the note is displayed alongside it.

**Acceptance Scenarios**:

1. **Given** the user is viewing a log entry, **When** they add a note, **Then** the note is saved and visible on the entry in the history view.
2. **Given** the user has many notes, **When** they search for "anxiety", **Then** all entries with notes containing "anxiety" are returned.

---

### Edge Cases

- What happens when the user logs an item that is not in the catalog? The system allows custom item creation with basic fields (name, category, unit).
- How does the system handle drug interaction data unavailability? The system displays a disclaimer that interaction data may be incomplete and users should consult a healthcare professional.
- What happens when vital thresholds are not configured? The system uses clinically established reference ranges as defaults, and allows users to set personal baselines.
- How does the system handle missing data in insights? The insights engine only generates correlations when sufficient data points exist (minimum 7 overlapping entries).
- What happens when a user logs a stack that includes an item they have previously flagged as causing a bad reaction? The system displays a warning based on the user's historical notes.

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right functional requirements.
-->

### Functional Requirements

- **FR-001**: The system MUST allow users to log consumption items from a catalog of supplements, medications, drugs, and foods, with dosage/quantity and timestamp.
- **FR-002**: The system MUST allow users to log actions from a catalog of activities (exercise, meditation, red light therapy, cold exposure, etc.) with duration and timestamp.
- **FR-003**: The system MUST allow users to create custom catalog entries for items or actions not yet in the system.
- **FR-004**: The system MUST store all log entries with complete metadata (item reference, dosage, timestamp, user notes if any) in a user-specific data store.
- **FR-005**: The system MUST provide a history view that displays all logged entries with filtering by date range, category, and specific items.
- **FR-006**: The system MUST provide a catalog of items and actions with detailed information including dosage ranges, duration of action, and relevant warnings.
- **FR-007**: The system MUST allow users to log their vitals (blood pressure, heart rate, weight, blood glucose, sleep quality, etc.) with timestamp.
- **FR-008**: The system MUST alert users when logged vitals fall outside their personal baseline or clinically established reference ranges.
- **FR-009**: The system MUST provide contextual advice for abnormal vitals by cross-referencing the user's logged supplements, medications, and actions.
- **FR-010**: The system MUST allow users to create named stacks and protocols composed of multiple catalog items and actions.
- **FR-011**: The system MUST allow users to log an entire stack with a single action, creating individual entries for each component.
- **FR-012**: The system MUST automatically check for known dangerous drug and supplement interactions when a new item is logged.
- **FR-013**: The system MUST display prominent warnings for dangerous interactions and require user acknowledgment before saving.
- **FR-014**: The system MUST allow users to attach free-text notes to any log entry.
- **FR-015**: The system MUST provide an insights dashboard that surfaces correlations between logged data and vital sign changes over time.
- **FR-016**: The system MUST store user data in a user-specific data store that is accessible only by the authenticated user. Data syncs across the user's own devices if cloud sync is enabled; otherwise, data remains on-device only.
- **FR-017**: The system MUST allow users to export their data in a standard format (CSV/JSON) for personal backup or analysis.
- **FR-018**: The system MUST meet the performance target of loading the history view in under 2 seconds for up to 1,000 entries.

### Key Entities *(include if feature involves data)*

- **LogEntry**: Represents a single logged event. Attributes: id, userId, itemReference (catalog or custom), quantity, unit, timestamp, stackId (if logged as part of a stack), notes, isAcknowledgedInteraction (boolean).
- **CatalogItem**: Represents a supplement, medication, drug, food, or action. Attributes: id, name, category, dosageRange, unit, durationOfAction, warnings, isCustom (boolean).
- **Stack**: Represents a named collection of catalog items and actions. Attributes: id, userId, name, description, items (ordered list with quantities).
- **VitalsEntry**: Represents a set of vital measurements logged at a point in time. Attributes: id, userId, timestamp, bloodPressureSystolic, bloodPressureDiastolic, heartRate, weight, bloodGlucose, sleepQuality, customMetrics.
- **Alert**: Represents a notification triggered by abnormal vitals or dangerous interactions. Attributes: id, userId, type (vital or interaction), severity, message, recommendation, isAcknowledged, linkedEntryId.
- **Insight**: Represents a correlation or trend derived from logged data. Attributes: id, userId, type, title, description, supportingDataPoints, generatedAt.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria.
  These must be technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: Users can log a consumption item or action in under 15 seconds from app open to confirmation.
- **SC-002**: Users can view their complete log history with a date-range filter applied in under 2 seconds.
- **SC-003**: Dangerous drug interaction warnings are displayed within 3 seconds of submitting a new log entry that triggers an interaction.
- **SC-004**: Abnormal vital alerts are triggered within 5 seconds of saving a vitals entry that is out of range.
- **SC-005**: The insights dashboard generates at least one correlation when the user has logged 7 or more overlapping data points across two categories.
- **SC-006**: Users can create, save, and log a stack containing 10 or more items in under 30 seconds.
- **SC-007**: The system correctly flags at least 90% of known dangerous interactions in a benchmarked test dataset.
- **SC-008**: Users can export their complete log history (up to 5 years of daily entries) as a downloadable file within 10 seconds.
- **SC-009**: The application is usable on both mobile and desktop web browsers.
- **SC-010**: 80% of users who log a stack complete the logging flow without abandoning it.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
-->

- Users have a smartphone or desktop browser and stable internet connectivity for cloud features.
- Users are familiar with basic biohacking concepts and terminology (supplements, protocols, stacks).
- The catalog of supplements, medications, and drugs will initially be populated from a reputable open-source database (e.g., SelfHacked, Examine.com, or similar), with a pathway for community contributions.
- Drug interaction data will be sourced from publicly available databases (e.g., DrugBank, FDA data) or a licensed API.
- Clinically established reference ranges for vitals will be used as defaults; users can override with personal baselines.
- Cross-device synchronization is desirable but not required for v1 — on-device storage with manual export is acceptable as an initial approach.
- The application targets individuals engaged in personal biohacking, not clinical or medical use; it is not intended to replace professional medical advice.
- v1 will support a single user per installation (no multi-user/team features).
- Alerts and recommendations are informational only; the application does not provide medical diagnoses.
