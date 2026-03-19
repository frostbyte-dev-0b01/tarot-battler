# UI Spec

## Purpose

This file defines the initial developer-facing UI for:

- building teams
- loading battle replays
- scrubbing through battles visually

The goal is not to build the final product UI yet. The goal is to create a lightweight tool that makes it much easier to:

- author team JSON
- run trial battles
- inspect board state over time
- understand why rules and abilities produced a specific outcome

The first implementation should be plain HTML, CSS, and vanilla JavaScript.

## Design Goals

- lightweight and fast to build
- easy to run locally without a larger frontend stack
- directly compatible with `design/team_builder_schema.md`
- directly compatible with `design/replay_schema.md`
- stateful and visual, not just raw JSON and log text

## Recommended Structure

The v1 tool should be a single page with two major work areas:

- Team Builder
- Replay Viewer

These can be:

- separate tabs
- or separate sections on the same page

Either approach is acceptable. Tabs are likely cleaner once the page grows.

## V1 Layout

The recommended replay layout is:

- top: replay load controls and current-event summary
- full-width board row
- lower detail row with inspector and event timeline
- bottom: raw replay JSON, metadata, and validation

The recommended team builder layout is:

- left panel: Team A editor
- right panel: Team B editor
- optional bottom panel: raw JSON import/export and validation messages

## Replay Viewer

### Main Board

The center of the replay viewer should display both teams on a 4-column by 3-row grid.

Each team board should:

- preserve row and column placement from the replay data
- clearly distinguish front, middle, and back rows
- show defeated characters in-place rather than removing them visually

Each character tile should display:

- display name
- current HP
- current MP
- alive or defeated state
- short status summary
- passive name with hover text from the passive catalog when available

Optional but useful:

- passive name
- current effective stats in a tooltip or inspector
- visual highlight when the character is the source or target of the selected event

### Inspector Panel

The left panel should show details for the currently selected unit.

Suggested fields:

- display name
- team
- position
- current HP and max HP
- current MP and max MP
- alive or defeated state
- passive
- actives
- passive and active descriptions via hover tooltip when available
- current statuses with stacks
- base stats
- effective stats

Optional later additions:

- sticky target
- last action taken
- rule list with the last triggered rule highlighted

### Playback Controls

The replay viewer should scrub by event index, not only by tick.

Reason:

- multiple meaningful events happen on the same tick
- tick-only scrubbing is too coarse for debugging

The control area should include:

- previous event
- next event
- play
- pause
- restart
- event-index slider
- current event index
- current tick display

Optional:

- jump to previous tick
- jump to next tick
- playback speed selector

### Timeline Panel

The right column should display the event log as a readable timeline.

The selected event should be highlighted.

The timeline should support at least:

- full event list
- major-events-only filter
- selected-character-only filter

The timeline should remain readable even when many events happen on the same tick, so it should group or visually label events by tick.

The viewer should also render replay events for movement and retargeting in both the timeline and the current-event summary.

## Team Builder

### Team Editor

The initial version should edit two separate team JSON documents using the interim full-loadout schema.

Each team panel should allow:

- editing team name
- adding a character
- removing a character
- editing character fields
- validating the team
- exporting the team JSON

Each character editor should include:

- `id`
- `display_name`
- position
- stats
- passive chosen from the passive catalog, or left empty
- `active_1`, `active_2`, and `active_3` style slots backed by the ability catalog, each optional
- item
- rules

The builder does not need:

- drag-and-drop
- roster browser
- card art
- advanced visual polish

Structured forms are enough for v1.

### Rules Editor

Rules are the most complex editable part of the team schema, so the UI should keep them explicit and readable.

A rule editor should support:

- ordered list of rules
- ability name
- list of conditions under `when`
- add/remove condition
- move rules up and down

A condition editor should support:

- subject
- value type
- stat selector when relevant
- status selector when relevant
- operator
- threshold

### JSON Import and Export

The team builder should support:

- loading a team JSON file from disk
- exporting the current team to JSON
- pasting raw JSON into a text area for quick iteration

This is important because:

- dev tooling will evolve quickly
- hand-editing JSON will still be common for a while
- import/export helps compare UI behavior against the schema docs directly

## State Model

The replay viewer should not derive meaning only from raw text log lines.

Instead, it should build a replay state model:

- initialize from the replay's initial team snapshot
- walk events in order
- update HP, MP, statuses, and alive state
- render the board from current replay state

This is the key design requirement for the viewer:

- the UI should be state-based, not log-only

The log is an explanation layer.
The board state is the primary visualization.

## Data Flow

### Team Builder Flow

1. user loads or edits Team A JSON
2. user loads or edits Team B JSON
3. tool validates both against the team builder schema
4. tool exports JSON for use by the engine

### Replay Viewer Flow

1. user loads a replay JSON file
2. tool validates the replay shape
3. tool builds an initial board state from the replay snapshot
4. tool applies events up to the selected event index
5. tool renders board, inspector, and timeline

Current engine loop:

1. run the engine
2. engine writes `tools/ui/sample-data/latest_replay.json`
3. replay viewer loads that file by default on page open
4. `Load Latest Replay` re-fetches it on demand

Current sample-team loop:

1. click `Load Sample Team` in either team panel
2. edit the structured team form or raw JSON
3. export the team JSON for the engine

Current bundled sample note:

- the built-in sample teams mirror the live bundled 3v3 matchup, currently Team A versus the Team B omen roster

## Recommended File Structure

The first implementation can stay very small:

- `index.html`
- `styles.css`
- `app.js`
- optional `sample-data/` directory for fixture JSON

If the JS grows, split later into:

- `team-builder.js`
- `replay-viewer.js`
- `state.js`
- `render.js`
- `schemas.js`

Do not start with a framework unless the tool outgrows vanilla JS.

## V1 Minimum Feature Set

The minimum useful version should include:

- load Team A JSON
- load Team B JSON
- edit and export team JSON
- load replay JSON
- render both teams on the board
- render HP and MP bars
- render defeated state
- render a simple status summary
- scrub by event index
- highlight the selected timeline event
- show selected unit details in the inspector

If those pieces work well, the tool will already be valuable for development.

## V1 Nice-to-Haves

- play and pause controls
- timeline filters
- import raw JSON by paste
- copy-to-clipboard export
- sample replay loader
- sample team loader
- event search by character name or ability name

## Explicit Non-Goals for V1

The first pass should not try to include:

- final production styling
- account systems
- backend persistence
- drag-and-drop formation editing
- roster drafting flow
- market or shop UI
- final combat animations
- complex visual effects

Those are future product concerns, not immediate dev-tool concerns.

## Implementation Order

Recommended order:

1. static page shell and layout
2. replay JSON file loader
3. board renderer from initial team snapshot
4. event-index scrubber
5. event application into replay state
6. timeline panel
7. inspector panel
8. team builder forms
9. import/export helpers
10. validation messaging

This order gets replay inspection working early, which is likely the highest-value piece.

## Design Notes

- The board should visibly preserve empty spaces in the 4x3 grid
- The right-hand log should explain events, not replace state rendering
- The viewer should highlight source and target units for the selected event
- Defeated units should remain visible but muted
- Statuses should start as compact text labels, not bespoke icons
- The UI should prioritize clarity and debugging value over visual flair
