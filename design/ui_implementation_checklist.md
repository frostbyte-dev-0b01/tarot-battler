# UI Implementation Checklist

## Purpose

This file turns `design/ui_spec.md` into a concrete implementation plan.

The target is a lightweight local developer tool built with:

- plain HTML
- plain CSS
- vanilla JavaScript

The tool should help with:

- authoring team JSON
- loading replay JSON
- scrubbing through battle state visually
- understanding battle outcomes more quickly than raw JSON logs allow

## Guiding Constraints

- keep the first version framework-free
- prefer simple file structure over abstractions
- prioritize replay usefulness before team-builder polish
- build around the schema docs, not ad hoc JSON
- keep the board state visual and stateful, not just text-log driven

## Proposed File Structure

Initial target:

- `tools/ui/index.html`
- `tools/ui/styles.css`
- `tools/ui/app.js`
- `tools/ui/sample-data/`

Split only if the JS becomes large:

- `tools/ui/team-builder.js`
- `tools/ui/replay-viewer.js`
- `tools/ui/state.js`
- `tools/ui/render.js`
- `tools/ui/validators.js`

## Phase 1: Static Shell

Goal:

- create the page shell and layout without data plumbing

Tasks:

- [x] create `tools/ui/index.html`
- [x] create `tools/ui/styles.css`
- [x] create `tools/ui/app.js`
- [x] add a top-level layout with Team Builder and Replay Viewer sections or tabs
- [x] add placeholder regions for:
  - [x] Team A editor
  - [x] Team B editor
  - [x] inspector
  - [x] battle board
  - [x] playback controls
  - [x] timeline
- [x] make the page render well on desktop and usable on smaller widths

Acceptance criteria:

- page loads locally as static HTML
- layout is visually clear even with placeholder content
- replay area already follows the intended inspector / board / timeline structure

Status:

- completed in `tools/ui/`

## Phase 2: Replay Loader and Validation

Goal:

- load replay JSON into the page and validate its top-level structure

Tasks:

- [x] add replay file input
- [x] add replay JSON paste area or sample replay loader
- [x] parse replay JSON safely with user-visible error messages
- [x] validate required top-level fields from `design/replay_schema.md`
- [x] render metadata summary:
  - [x] seed
  - [x] winner
  - [x] final tick count
  - [x] team names

Acceptance criteria:

- invalid replay JSON shows readable validation errors
- valid replay JSON loads without console-only debugging
- metadata appears in the UI after load

Status:

- completed with replay input, demo replay loading, top-level validation, and metadata rendering

## Phase 3: Initial Board Rendering

Goal:

- render both teams from the replay snapshot before any event playback

Tasks:

- [x] build board rendering from replay `teams`
- [x] preserve 4-column by 3-row layout
- [x] render empty spaces explicitly
- [x] render character tiles with:
  - [x] display name
  - [x] HP bar based on `max_hp`
  - [x] MP bar based on `max_mp`
  - [x] passive name or placeholder
- [x] distinguish Team A and Team B clearly

Acceptance criteria:

- both teams render in correct positions
- empty grid cells remain visible
- defeated styling is not needed yet, but tile states are ready for it

Status:

- completed with snapshot-driven board rendering and a richer built-in demo replay

## Phase 4: Replay State Model

Goal:

- create a replay state object that can be advanced event by event

Tasks:

- [x] define in-memory replay state initialized from the replay snapshot
- [x] track per-character:
  - [x] current HP
  - [x] current MP
  - [x] alive or defeated
  - [x] statuses and stacks
- [x] add event application logic for the minimum event set:
  - [x] `battle_start`
  - [x] `turn_start`
  - [x] `basic_attack`
  - [x] `ability_used`
  - [x] `damage`
  - [x] `healing`
  - [x] `status_applied`
  - [x] `status_removed`
  - [x] `status_tick`
  - [x] `passive_triggered`
  - [x] `turn_skipped`
  - [x] `resource_changed`
  - [x] `defeat`
  - [x] `battle_end`
- [x] rebuild board state from event index `0..N`

Acceptance criteria:

- replay state can be initialized and advanced deterministically
- current HP, MP, and defeat state match the selected event index
- the UI does not rely only on raw log text to describe state

Status:

- completed with an in-memory replay state model and event application pipeline feeding board rendering

## Phase 5: Playback Controls

Goal:

- scrub through replay events interactively

Tasks:

- [x] add previous-event button
- [x] add next-event button
- [x] add restart button
- [x] add event-index slider
- [x] show current event index
- [x] show current tick
- [x] ensure board re-renders from selected event state

Optional in this phase:

- [x] play
- [x] pause
- [ ] playback speed selector

Acceptance criteria:

- user can move event by event through the replay
- the board updates correctly at each step
- current tick and selected event are always visible

Status:

- completed with event-index scrubbing and lightweight autoplay controls

## Phase 6: Timeline Panel

Goal:

- display a readable event timeline linked to playback state

Tasks:

- [x] render the full ordered event list
- [x] highlight the currently selected event
- [x] group or label entries by tick
- [x] render readable text for major event types
- [x] add major-events-only filter
- [x] add selected-character-only filter

Acceptance criteria:

- selecting an event updates the board
- timeline remains readable on dense ticks
- filtering makes the replay easier to inspect

Status:

- completed with clickable timeline events, tick grouping, and basic filters

## Phase 7: Inspector Panel

Goal:

- show detailed state for the selected character

Tasks:

- [x] support selecting a character tile
- [x] support selecting a character from the timeline
- [x] display:
  - [x] display name
  - [x] team
  - [x] position
  - [x] current HP / max HP
  - [x] current MP / max MP
  - [x] alive or defeated state
  - [x] passive
  - [x] actives
  - [x] statuses with stacks
  - [x] base stats
  - [x] effective stats
- [x] highlight selected unit on the board
- [x] highlight current event source or target on the board when applicable

Acceptance criteria:

- clicking a unit reveals useful debugging state
- inspector updates as replay position changes
- source/target highlighting makes event context easier to follow

Status:

- completed with board selection, timeline-driven focus, inspector rendering, and event source/target highlights

## Phase 8: Team Builder Loader and Validation

Goal:

- load and validate the interim team JSON format

Tasks:

- [x] add Team A file input
- [x] add Team B file input
- [x] add paste areas or sample team loaders
- [x] validate required fields from `design/team_builder_schema.md`
- [x] show validation errors inline
- [x] show team names and character counts

Acceptance criteria:

- valid teams load into the UI
- invalid teams show readable errors
- users can inspect both teams without editing raw DOM manually

Status:

- completed with Team A and Team B JSON loading, demo data, validation, and summary panels

## Phase 9: Team Builder Forms

Goal:

- support direct editing of team data in the UI

Tasks:

- [x] edit team name
- [x] add character
- [x] remove character
- [x] edit character identity fields
- [x] edit position
- [x] edit stats
- [x] edit passive
- [x] edit actives
- [x] edit item
- [x] edit rules

Rules editor tasks:

- [x] add rule
- [x] remove rule
- [x] move rule up
- [x] move rule down
- [x] edit rule ability
- [x] add condition
- [x] remove condition
- [x] edit subject, value type, operator, threshold
- [x] support stat selector when `value` is stat-based

Acceptance criteria:

- a user can build a valid team entirely in the UI
- edits round-trip cleanly to JSON
- rule editing is explicit and readable

Status:

- completed with live structured editing for team name, characters, stats, actives, items, rules, and conditions

## Phase 10: Import and Export Helpers

Goal:

- make the tool practical for everyday iteration

Tasks:

- [ ] export Team A JSON
- [ ] export Team B JSON
- [ ] copy team JSON to clipboard
- [ ] import from pasted JSON
- [ ] add sample team loader
- [ ] add sample replay loader

Acceptance criteria:

- common dev-tool loops do not require editing files by hand every time
- loading and exporting schemas is easy and predictable

## Phase 11: Polish Pass

Goal:

- improve readability without turning the tool into a full product UI

Tasks:

- [ ] make board tiles visually distinct by team
- [ ] mute defeated units clearly
- [ ] improve HP and MP bar readability
- [ ] make status summaries compact and scannable
- [ ] improve responsive layout for narrower windows
- [ ] ensure keyboard focus works reasonably for core controls

Acceptance criteria:

- tool feels intentionally usable, not only technically functional
- replay analysis is faster than reading raw JSON

## Recommended Build Order

Recommended execution order:

1. Phase 1: Static Shell
2. Phase 2: Replay Loader and Validation
3. Phase 3: Initial Board Rendering
4. Phase 4: Replay State Model
5. Phase 5: Playback Controls
6. Phase 6: Timeline Panel
7. Phase 7: Inspector Panel
8. Phase 8: Team Builder Loader and Validation
9. Phase 9: Team Builder Forms
10. Phase 10: Import and Export Helpers
11. Phase 11: Polish Pass

This order prioritizes replay inspection first, which is likely the highest-value part of the tool.

## Risks and Watchouts

- event-only rendering is not enough; the replay must drive a mutable state model
- replay schema and current engine output may not match exactly yet
- rule editing can become noisy if too much abstraction is added early
- drag-and-drop formation editing should be deferred until the basic editor works
- timeline rendering can get cluttered if major and minor events are not distinguishable

## Suggested First Milestone

The first milestone worth shipping locally is:

- static UI shell
- replay loader
- board rendering from initial snapshot
- event-index scrubber
- basic event-driven HP/MP updates
- timeline with selected-event highlight

If that milestone is solid, the replay viewer will already be useful for engine iteration before the team builder is fully finished.
