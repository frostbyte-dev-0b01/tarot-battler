# Replay Snapshot Plan

## Goal

Move the replay format from:

- initial team snapshot
- event log only
- UI-side state reconstruction

to:

- initial team snapshot
- ordered event log
- full battle-state snapshots for replay playback

The replay viewer should use snapshots for board rendering and state inspection, while events remain the source for timeline text.

## Why

The current replay viewer reconstructs battle state by replaying events in JavaScript. That has a few problems:

- rewinding and jumping are more expensive than they need to be
- the UI must partially understand engine rules
- new combat mechanics tend to force replay-viewer logic changes
- state inspection is limited by what the UI can derive from the event stream

State snapshots solve those issues for the local dev-tool workflow.

## Proposed Replay Shape

Keep the current top-level replay structure and add:

- `snapshots: ReplaySnapshot[]`

Each snapshot should represent the full battle state after a given event index.

Recommended indexing model:

- `snapshots[0]` is the initial state before any events are applied
- `snapshots[n + 1]` is the state after `events[n]`

That makes playback trivial:

- event index `-1` -> `snapshots[0]`
- event index `n` -> `snapshots[n + 1]`

## Snapshot Contents

Each snapshot should include:

- `tick`
- `event_index`
- `teams`

Each character state in a snapshot should include:

- stable id
- display name
- alive / defeated state
- position
- current HP / max HP
- current MP / max MP
- base stats
- effective stats
- passive
- actives
- current target
- companions
- statuses / conditions with stack counts

This is intended to be a full UI-facing view of the battle state.

## Implementation Stages

### Stage 1: Schema And Doc Update

- update `design/replay_schema.md`
- document the new `snapshots` array
- document the snapshot indexing model
- note that the UI should stop reconstructing state from events once snapshots exist

### Stage 2: Engine Snapshot Model

- add replay snapshot structs in `battle_engine/src/logger.rs`
- add helpers to serialize full team and character runtime state
- include:
  - base stats
  - effective stats
  - statuses
  - current target
  - companions

Status: complete

### Stage 3: Snapshot Capture During Battle

- capture an initial snapshot before any events are applied
- capture a new snapshot after each logged event
- prefer true per-event snapshots instead of coarse action-boundary snapshots

This likely means:

- adding a `BattleState` helper that appends snapshots when new log events are emitted
- instrumenting the event-producing call sites so each event gets a matching post-event snapshot

Status: complete

### Stage 4: Replay JSON Output

- update `BattleLog::to_replay_json`
- include `snapshots`
- keep the current `events` output for timeline text

Status: complete

### Stage 5: Replay Viewer Refactor

- stop deriving replay state from the event stream
- use `snapshots[event_index + 1]` for board rendering and inspector state
- keep the event log and current-event summary based on `events`
- simplify or remove the old event-application helpers in `tools/ui/app.js`

Status: complete

### Stage 6: Validation And Regression Tests

- update replay validation in the UI
- add engine tests for snapshot count and key fields
- confirm:
  - initial snapshot exists
  - snapshot count is `events.len() + 1`
  - moved / retargeted / status / HP / MP changes appear in snapshots

Status: complete

## Notes

- Replay files will get larger. That is acceptable for the local dev-tool workflow.
- The event log remains useful for readable replay text and debugging.
- If true per-event snapshots turn out to be too invasive, the fallback is action-boundary snapshots, but that should be treated as a fallback, not the target design.
