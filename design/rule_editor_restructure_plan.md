# Rule Editor Restructure Plan

This file tracks the planned cleanup of the Team Builder rule editor.

## Goal

Make rule creation easier to understand and less noisy without changing the underlying rule JSON schema.

The current editor exposes all possible condition-detail fields at once:

- `Stat`
- `Status`

That works mechanically, but it creates confusion because only one of those fields is relevant for a given `value` type.

The restructure should make the UI reflect the actual rule model more clearly.

## Current Problem

Each condition row currently shows:

- `Subject`
- `Value`
- `Stat`
- `Status`
- `Operator`
- `Threshold`

Problems:

- `Stat` is only relevant when `value = stat`
- `Status` is only relevant when `value = has_status` or `status_stacks`
- many condition rows show irrelevant controls
- the editor feels more complex than the schema really is

## Target Condition Model

Each condition should be edited through these conceptual fields:

- `Subject`
- `Value Type`
- `Value Detail`
- `Operator`
- `Threshold`

`Value Detail` changes based on `Value Type`.

### Value Type Behavior

#### No extra detail required

These should not show any extra detail control:

- `hp`
- `mp`
- `use_count`
- `turns_since_use`
- `tick_count`
- `ally_count`
- `enemy_count`

#### Stat detail required

These should show a `Stat` dropdown:

- `stat`

#### Status detail required

These should show a `Status` input or dropdown:

- `has_status`
- `status_stacks`

## Planned Changes

### Phase 1: Dynamic Condition Detail Fields

Update the condition editor so only relevant detail controls are rendered.

Work:

- replace the always-visible `Stat` and `Status` controls with a single dynamic detail field area
- when `value_type = stat`, show a `Stat` dropdown
- when `value_type = has_status` or `status_stacks`, show a `Status` field
- otherwise render no extra detail control

Result:

- condition rows become significantly cleaner
- the UI better matches the schema

### Phase 2: Human-Friendly Labels

Improve the wording in dropdowns and labels.

Work:

- render display labels instead of raw internal values where possible
- examples:
  - `self` -> `Self`
  - `companion` -> `Any Companion`
  - `use_count` -> `Uses`
  - `turns_since_use` -> `Turns Since Use`
  - `ally_count` -> `Allies Alive`
  - `enemy_count` -> `Enemies Alive`
  - `has_status` -> `Has Status`
  - `status_stacks` -> `Status Stacks`

Result:

- the editor reads more like game logic and less like raw JSON

### Phase 3: Condition Preview Text

Add a compact human-readable preview for each condition.

Examples:

- `Self MP >= 4`
- `Target Has Status Omen >= 1`
- `Any Companion MP <= 2`
- `Self Status Stacks Empower:MGT >= 3`

Work:

- render a preview line in each condition card
- update it live as the condition changes

Result:

- rules are easier to scan and debug
- players can read the rule without mentally translating the raw fields

### Phase 4: Rule-Level Readability Pass

Polish the whole rule block after the condition cleanup lands.

Possible work:

- reduce vertical spacing where appropriate
- improve condition grouping visuals
- tighten action labels
- make empty-state text more helpful

Result:

- the full rule editor feels more deliberate and less form-heavy

## Out of Scope

This cleanup does not yet include:

- rule templates
- drag-and-drop rule editing
- status catalog autocomplete
- schema changes
- nested AND/OR logic

Those can come later if needed.

## Recommended Order

1. Phase 1: Dynamic Condition Detail Fields
2. Phase 2: Human-Friendly Labels
3. Phase 3: Condition Preview Text
4. Phase 4: Rule-Level Readability Pass

## Verification

After each phase:

- `node --check tools/ui/app.js`
- manual browser check of the Team Builder rule editor

Focus checks:

- switching `Value Type` updates visible controls correctly
- editing a condition still round-trips to the same JSON shape
- preview text matches the saved rule data
