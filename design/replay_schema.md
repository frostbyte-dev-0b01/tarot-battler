# Replay Schema

## Purpose

This file defines the proposed JSON schema for saved battle replays.

The replay viewer should be able to render:

- battle metadata
- initial team state
- a readable timeline
- board state changes over time

The schema is designed to be frontend-friendly. It should not require the UI to reimplement battle logic in order to display HP, MP, statuses, defeats, or event causes.

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
      "name": "Arcane Gambit",
      "characters": []
    }
  },
  "events": []
}
```

## ReplayConfig

- `version: number`
- `seed: number`
- `winner: "team_a" | "team_b" | "draw"`
- `tick_count: number`
- `teams: ReplayTeams`
- `events: ReplayEvent[]`

### Notes

- `tick_count` is the final world tick when the battle ends
- `teams` is the initial battle snapshot, not live mutable state
- `events` is the ordered battle timeline

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
        "con": 7,
        "str": 8,
        "int": 3,
        "for": 7,
        "wis": 3,
        "dex": 4,
        "spi": 4
      },
      "passive": "Authority",
      "actives": ["Crush", "Embolden"]
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

### `ability_used`

```json
{
  "tick": 4,
  "type": "ability_used",
  "actor_id": "the_emperor",
  "ability": "Crush",
  "mp_cost": 2
}
```

### `damage`

```json
{
  "tick": 4,
  "type": "damage",
  "source_id": "the_emperor",
  "target_id": "the_fool",
  "amount": 6,
  "damage_kind": "physical",
  "source_kind": "ability",
  "source_name": "Crush",
  "target_hp_after": 0
}
```

### `healing`

```json
{
  "tick": 7,
  "type": "healing",
  "source_id": "the_star",
  "target_id": "strength",
  "amount": 4,
  "source_kind": "ability",
  "source_name": "Restore",
  "target_hp_after": 6
}
```

### `status_applied`

```json
{
  "tick": 8,
  "type": "status_applied",
  "source_id": "the_moon",
  "target_id": "the_tower",
  "status": "Poison",
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
  "status": "Empower:STR",
  "stacks_removed": 1,
  "stacks_after": 0
}
```

### `status_tick`

```json
{
  "tick": 10,
  "type": "status_tick",
  "target_id": "the_tower",
  "status": "Poison",
  "amount": 2,
  "kind": "damage",
  "target_hp_after": 5
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

### `resource_changed`

```json
{
  "tick": 11,
  "type": "resource_changed",
  "actor_id": "the_magician",
  "resource": "mp",
  "delta": 2,
  "value_after": 4,
  "reason": "turn_regen"
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
- `resource_changed`
- `defeat`
- `battle_end`

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
