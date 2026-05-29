# Team Builder Schema

## Purpose

This file defines the live JSON contract used by the Team Builder and the battle engine.

The current system is template-based:

- teams do not author raw stats directly
- each character references an archetype template
- the template provides locked base stats
- equipped aspects add pre-battle stat bonuses and role identity
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
  "aspect": "aspect_of_grace",
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
- `aspect: string | null`
- `rules: Rule[]`

### Notes

- `id` should be stable and machine-friendly within the team
- `template_id` must reference a defined archetype template
- `display_name` is a cosmetic/player-facing override
- `stats` are intentionally absent from team JSON
- `passive` and `actives` are loadout choices validated against the template pools
- `aspect` is nullable, but when present must reference a defined aspect
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
    "active_pool": ["Hold the Line", "Command", "Taunt", "Interpose", "Sunder"],
    "aspect_slots": 1
  }
}
```

### Fields

- `display_name: string`
- `stats: StatBlock`
- `default_passive: string`
- `passive_pool: string[]`
- `active_pool: string[]`
- `aspect_slots: number`
- `cost: number`

### Notes

- `stats` are the locked base stats for that arcana
- the engine treats these as authoritative
- `default_passive` supports builder defaults
- `passive_pool` and `active_pool` define legal loadout choices
- `aspect_slots` should currently be `1`
- `cost` is the archetype's coarse point cost for the team budget (currently `1`–`3`); aspect definitions carry an analogous `cost` (`0`–`2`). A team's total archetype + aspect cost must not exceed the team budget (currently `14`), and each archetype and aspect may appear at most once per team.

## Aspect Catalog

The engine loads aspects from:

- `battle_engine/src/data/aspects.json`

### AspectDef

```json
{
  "aspect_of_ruin": {
    "display_name": "Aspect of Ruin",
    "description": "A high-pressure role that rewards attacking conditioned enemies.",
    "stat_bonuses": {
      "mgt": 2,
      "mag": 2,
      "wil": 1,
      "vit": -2,
      "arm": -1
    },
    "passive": "Ruinous"
  }
}
```

### Fields

- `display_name: string`
- `description: string`
- `stat_bonuses: Partial<StatBlock>`
- `passive?: string`
- `active?: string`

### Notes

- each character can equip only one aspect
- a team cannot repeat the same aspect
- aspects should generally shift stats by about `+/-5` total, not provide huge stat spikes
- aspects should usually grant one defining passive or active, most often a passive

### First Aspects

- `Aspect of Ruin`
  - stats: `MGT +2`, `MAG +2`, `WIL +1`, `VIT -2`, `ARM -1`
  - passive: `Ruinous`
    - The first time each tick the user damages an enemy with a condition, deal `2` true damage.

- `Aspect of Grace`
  - stats: `VIT +2`, `RES +2`, `WIL +1`, `MGT -1`, `MAG -1`
  - passive: `Grace`
    - The first time each tick the user affects an ally with an ability, that ally restores `2 HP`.

## Resolution Model

The engine resolves authored loadouts into runtime characters like this:

1. look up the referenced archetype
2. copy the archetype base stats
3. apply aspect stat bonuses
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
      "aspect": "aspect_of_grace",
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
