# Replay Schema

## Purpose

This file defines the proposed JSON schema for saved battle replays.

The replay viewer should be able to render:

- battle metadata
- initial team state
- a readable timeline
- board state changes over time
- full point-in-time battle state without replaying combat logic in the UI

The schema is designed to be frontend-friendly. It should not require the UI to reimplement battle logic in order to display HP, MP, statuses, defeats, or event causes.

The live replay format now also includes battle `conditions` separately from ordinary `statuses`.

The current engine writes replay JSON to:

```text
tools/ui/sample-data/latest_replay.json
```

You can also override the output path with:

```bash
cargo run -- --json-out path/to/replay.json
```

`cargo run -- --json` prints the same replay-schema JSON to stdout.

## Top-Level Shape

```json
{
  "version": 1,
  "seed": 42,
  "winner": "team_a",
  "tick_count": 27,
  "teams": {
    "team_a": {
      "name": "Imperial Phalanx",
      "characters": []
    },
    "team_b": {
      "name": "Omen Tribunal",
      "characters": []
    }
  },
  "events": [],
  "snapshots": []
}
```

## ReplayConfig

- `version: number`
- `seed: number`
- `winner: "team_a" | "team_b" | "draw"`
- `tick_count: number`
- `teams: ReplayTeams`
- `events: ReplayEvent[]`
- `snapshots: ReplaySnapshot[]`

### Notes

- `tick_count` is the final world tick when the battle ends
- `teams` is the initial battle snapshot, not live mutable state
- `events` is the ordered battle timeline
- `snapshots` is the ordered full-state playback stream for the UI

## Team Snapshot

```json
{
  "name": "Imperial Phalanx",
  "characters": [
    {
      "id": "the_emperor",
      "display_name": "The Emperor",
      "position": { "row": 0, "col": 0 },
      "max_hp": 14,
      "max_mp": 4,
      "stats": {
        "vit": 7,
        "mgt": 8,
        "mag": 3,
        "arm": 7,
        "res": 3,
        "spd": 4,
        "wil": 4
      },
      "passive": "Imperial Formation",
      "actives": ["Hold the Line", "Command", "Taunt"]
    }
  ]
}
```

### TeamSnapshot

- `name: string`
- `characters: ReplayCharacter[]`

### ReplayCharacter

- `id: string`
- `display_name: string`
- `position: Position`
- `max_hp: number`
- `max_mp: number`
- `stats: StatBlock`
- `passive: string | null`
- `actives: string[]`

### Notes

- `id` should be stable across the replay
- `display_name` is intended for UI labels and replay text
- `max_hp` and `max_mp` are included so the viewer can render bars without recomputing derived values

## Replay Snapshots

The replay viewer should render board state from `snapshots`, not by replaying `events` in JavaScript.

Recommended indexing model:

- `snapshots[0]` is the initial battle state before any events are applied
- `snapshots[n + 1]` is the battle state after `events[n]`

That means:

- event index `-1` maps to `snapshots[0]`
- event index `n` maps to `snapshots[n + 1]`

### ReplaySnapshot

```json
{
  "tick": 4,
  "event_index": 7,
  "teams": {
    "team_a": {
      "name": "Imperial Phalanx",
      "characters": []
    },
    "team_b": {
      "name": "Omen Tribunal",
      "characters": []
    }
  }
}
```

- `tick: number`
- `event_index: number`
- `teams: ReplayTeamsState`

### ReplayCharacterState

```json
{
  "id": "the_emperor",
  "display_name": "The Emperor",
  "alive": true,
  "position": { "row": 0, "col": 0 },
  "current_hp": 36,
  "max_hp": 36,
  "current_mp": 8,
  "max_mp": 12,
  "stats": {
    "vit": 12,
    "mgt": 12,
    "mag": 8,
    "arm": 7,
    "res": 5,
    "spd": 8,
    "wil": 12
  },
  "effective_stats": {
    "vit": 12,
    "mgt": 13,
    "mag": 8,
    "arm": 7,
    "res": 5,
    "spd": 8,
    "wil": 12
  },
  "passive": "Imperial Formation",
  "actives": ["Hold the Line", "Command", "Taunt"],
  "current_target_id": "justice",
  "companions": ["the_hierophant"],
  "statuses": [
    {
      "name": "Empower:MGT",
      "stacks": 1
    }
  ],
  "conditions": [
    {
      "name": "Marked",
      "stacks": 2
    }
  ]
}
```

- `id: string`
- `display_name: string`
- `alive: boolean`
- `position: Position`
- `current_hp: number`
- `max_hp: number`
- `current_mp: number`
- `max_mp: number`
- `stats: StatBlock`
- `effective_stats: StatBlock`
- `passive: string | null`
- `actives: string[]`
- `current_target_id: string | null`
- `companions: string[]`
- `statuses: ReplayStatusState[]`
- `conditions: ReplayConditionState[]`

### Notes

- snapshots are the source of truth for playback rendering
- events remain the source of truth for timeline text
- conditions should be rendered separately from statuses in the inspector and timeline when present
- snapshots should include enough state for the inspector to render without recomputing combat rules
- this includes effective stats, live status stacks, current targets, and live positions

## Common Event Envelope

Every replay event should include:

- `tick: number`
- `type: string`

Additional fields depend on the event type.

Example:

```json
{
  "tick": 4,
  "type": "ability_used",
  "actor_id": "the_emperor"
}
```

## Event Types

### `battle_start`

```json
{
  "tick": 0,
  "type": "battle_start"
}
```

### `turn_start`

```json
{
  "tick": 4,
  "type": "turn_start",
  "actor_id": "the_emperor",
  "current_hp": 11,
  "current_mp": 2
}
```

### `basic_attack`

```json
{
  "tick": 4,
  "type": "basic_attack",
  "actor_id": "the_emperor",
  "target_id": "the_fool",
  "damage_kind": "physical"
}
```

### `rest`

```json
{
  "tick": 4,
  "type": "rest",
  "actor_id": "the_emperor",
  "mp_restored": 2,
  "mp_after": 4
}
```

### `ability_used`

```json
{
  "tick": 4,
  "type": "ability_used",
  "actor_id": "the_emperor",
  "ability": "Hold the Line",
  "mp_cost": 2
}
```

### `damage`

```json
{
  "tick": 4,
  "type": "damage",
  "source_id": "justice",
  "target_id": "the_hierophant",
  "amount": 6,
  "damage_kind": "physical",
  "source_kind": "ability",
  "source_name": "Condemn",
  "target_hp_after": 0
}
```

### `healing`

```json
{
  "tick": 7,
  "type": "healing",
  "source_id": "the_hierophant",
  "target_id": "the_emperor",
  "amount": 3,
  "source_kind": "ability",
  "source_name": "Blessing",
  "target_hp_after": 6
}
```

### `status_applied`

```json
{
  "tick": 8,
  "type": "status_applied",
  "source_id": "the_moon",
  "target_id": "the_hierophant",
  "status": "Omen",
  "stacks_added": 2,
  "stacks_after": 3
}
```

### `status_removed`

```json
{
  "tick": 9,
  "type": "status_removed",
  "source_id": "the_hermit",
  "target_id": "strength",
  "status": "Empower:MGT",
  "stacks_removed": 1,
  "stacks_after": 0
}
```

### `status_tick`

```json
{
  "tick": 10,
  "type": "status_tick",
  "target_id": "the_hierophant",
  "status": "Omen",
  "amount": 2,
  "kind": "damage",
  "target_hp_after": 5
}
```

### `moved`

```json
{
  "tick": 10,
  "type": "moved",
  "actor_id": "the_chariot",
  "to_row": 0,
  "to_col": 2
}
```

### `retargeted`

```json
{
  "tick": 10,
  "type": "retargeted",
  "actor_id": "the_fool",
  "new_target_id": "the_emperor",
  "mode": "to_self"
}
```

### `passive_triggered`

```json
{
  "tick": 10,
  "type": "passive_triggered",
  "actor_id": "the_tower",
  "passive": "Collapse",
  "trigger": "on_death"
}
```

### `turn_skipped`

```json
{
  "tick": 11,
  "type": "turn_skipped",
  "actor_id": "the_fool",
  "reason": "stun"
}
```

### `defeat`

```json
{
  "tick": 12,
  "type": "defeat",
  "actor_id": "the_moon"
}
```

### `battle_end`

```json
{
  "tick": 27,
  "type": "battle_end",
  "winner": "team_a"
}
```

## Viewer-Critical Fields

The replay viewer should not need to reconstruct full battle state from raw damage math alone.

For that reason, state-changing events should include:

- `target_hp_after`
- `value_after` for MP changes
- `stacks_after` for status application and removal

These fields make timeline rendering and board playback much simpler and less brittle.

## Recommended Minimum Event Set

The replay format should support at least these event families:

- `battle_start`
- `turn_start`
- `basic_attack`
- `ability_used`
- `damage`
- `healing`
- `status_applied`
- `status_removed`
- `status_tick`
- `passive_triggered`
- `turn_skipped`
- `defeat`
- `battle_end`

## Current Engine Coverage

The current engine writes this replay schema directly, but it does not yet emit every event family listed above.

Currently emitted:

- `battle_start`
- `turn_start`
- `basic_attack`
- `ability_used`
- `damage`
- `status_tick`
- `passive_triggered`
- `turn_skipped`
- `defeat`
- `battle_end`

Still missing from engine output:

- `healing` as a standalone event
- `status_applied`
- `status_removed`

Those gaps should be filled later so the replay viewer can show status state with full fidelity without reconstructing hidden engine details.

This is enough for:

- a readable timeline
- a board-state replay viewer
- per-character side panels
- basic post-battle summaries derived from the event stream

## Design Notes

- The event stream should stay ordered exactly as the engine resolves events
- The replay schema should use stable character IDs rather than positional indices alone
- Derived summary stats should be computed separately rather than embedded in every replay
- The current engine log format does not yet match this full schema exactly; this file is the intended frontend-facing contract to work toward
