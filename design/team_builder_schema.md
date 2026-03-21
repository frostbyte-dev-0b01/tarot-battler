# Team Builder Schema

## Purpose

This file defines the live JSON contract used by the Team Builder and the battle engine.

The current system is template-based:

- teams do not author raw stats directly
- each character references an archetype template
- the template provides locked base stats
- equipped items add pre-battle stat bonuses
- battle-time effects then modify those resolved stats further

## Top-Level Shape

```json
{
  "version": 2,
  "name": "Imperial Phalanx",
  "characters": []
}
```

## TeamConfig

- `version: number`
- `name: string`
- `characters: CharacterLoadout[]`

### Validation Rules

- `version` must be `2`
- a team must contain at least 1 character
- character `id` values must be unique within the team
- character positions must be unique within the team
- valid rows are `0..=2`
- valid columns are `0..=2`

## CharacterLoadout

```json
{
  "id": "emperor_anchor",
  "template_id": "the_emperor",
  "display_name": "The Emperor",
  "position": { "row": 0, "col": 0 },
  "passive": "Imperial Formation",
  "actives": ["Hold the Line", "Command", "Taunt"],
  "item": "vitality_charm",
  "rules": []
}
```

### Fields

- `id: string`
- `template_id: string`
- `display_name?: string`
- `position: Position`
- `passive: string`
- `actives: string[]`
- `item: string | null`
- `rules: Rule[]`

### Notes

- `id` should be stable and machine-friendly within the team
- `template_id` must reference a defined archetype template
- `display_name` is a cosmetic/player-facing override
- `stats` are intentionally absent from team JSON
- `passive` and `actives` are loadout choices validated against the template pools
- `item` is nullable, but when present must reference a defined item
- the current UI exposes up to three active slots explicitly

## Position

```json
{
  "row": 0,
  "col": 0
}
```

- `row: number`
- `col: number`

## Rule

```json
{
  "ability": "Hold the Line",
  "when": [
    {
      "subject": "self",
      "value": "mp",
      "op": "gte",
      "threshold": 5
    }
  ]
}
```

Rules are character-level priorities, not per-ability subprograms.

Each character may have up to 5 rules total.

They are evaluated in array order:

- the first satisfied rule is used
- later rules are ignored once one rule fires
- if no rule is satisfied, the character uses `Basic Attack`

### Fields

- `ability: string`
- `when: Condition[]`

## Condition

```json
{
  "subject": "target",
  "value": { "stat": "arm" },
  "op": "gte",
  "threshold": 6
}
```

### Fields

- `subject: "self" | "target" | "companion" | "world"`
- `value: QueryValue`
- `op: "gte" | "lte"`
- `threshold: number`

## QueryValue

Allowed values:

- `"hp"`
- `"mp"`
- `"self_row"`
- `"self_companion_count"`
- `"target_companion_count"`
- `"use_count"`
- `"turns_since_use"`
- `"tick_count"`
- `"ally_count"`
- `"enemy_count"`
- `{ "stat": "vit" | "mgt" | "mag" | "arm" | "res" | "spd" | "wil" }`
- `{ "has_status": "Ward" }`
- `{ "status_stacks": "Empower:MGT" }`

## Archetype Catalog

The engine loads archetypes from:

- `battle_engine/src/data/archetypes.json`

### ArchetypeTemplate

```json
{
  "the_emperor": {
    "display_name": "The Emperor",
    "stats": {
      "vit": 12,
      "mgt": 12,
      "mag": 8,
      "arm": 7,
      "res": 5,
      "spd": 8,
      "wil": 12
    },
    "default_passive": "Imperial Formation",
    "passive_pool": ["Imperial Formation"],
    "active_pool": ["Hold the Line", "Command", "Taunt", "Interpose", "Decoy", "Sunder"],
    "item_slots": 1
  }
}
```

### Fields

- `display_name: string`
- `stats: StatBlock`
- `default_passive: string`
- `passive_pool: string[]`
- `active_pool: string[]`
- `item_slots: number`

### Notes

- `stats` are the locked base stats for that arcana
- the engine treats these as authoritative
- `default_passive` supports builder defaults
- `passive_pool` and `active_pool` define legal loadout choices

## Item Catalog

The engine loads items from:

- `battle_engine/src/data/items.json`

### ItemDef

```json
{
  "vitality_charm": {
    "display_name": "Vitality Charm",
    "description": "Gain +2 VIT.",
    "stat_bonuses": {
      "vit": 2
    }
  }
}
```

### Fields

- `display_name: string`
- `description: string`
- `stat_bonuses: Partial<StatBlock>`

### Notes

- the current implementation keeps items simple: stat bonuses only
- passive-like item effects can come later

## Resolution Model

The engine resolves authored loadouts into runtime characters like this:

1. look up the referenced archetype
2. copy the archetype base stats
3. apply item stat bonuses
4. validate passive and active choices against the template pools
5. build the resolved runtime `CharacterConfig`

This means team JSON is authoring input, not final combat state.

## Example Team

```json
{
  "version": 2,
  "name": "Imperial Phalanx",
  "characters": [
    {
      "id": "the_emperor",
      "template_id": "the_emperor",
      "display_name": "The Emperor",
      "position": { "row": 0, "col": 0 },
      "passive": "Imperial Formation",
      "actives": ["Hold the Line", "Command", "Taunt"],
      "item": "vitality_charm",
      "rules": [
        {
          "ability": "Hold the Line",
          "when": [
            {
              "subject": "self",
              "value": "mp",
              "op": "gte",
              "threshold": 5
            }
          ]
        }
      ]
    }
  ]
}
```
