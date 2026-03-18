# Team Builder Schema

## Purpose

This file defines the interim JSON schema for team builder and replay tooling.

It is intentionally simple:

- one JSON file per team
- each file contains full character loadouts
- validation requires at least 1 character per team

This is a temporary builder-facing format. The intended long-term direction is:

- predefined roster entries keyed by `character_id`
- team files that store `character_id + overrides`

The current engine can load this format directly with:

```bash
cargo run -- --team-a path/to/team_a.json --team-b path/to/team_b.json
```

For now, full loadouts are easier to edit by hand and simpler to support in dev tooling.

## Top-Level Shape

```json
{
  "version": 1,
  "name": "Imperial Phalanx",
  "characters": []
}
```

## TeamConfig

- `version: number`
- `name: string`
- `characters: CharacterLoadout[]`

### Validation Rules

- a team must contain at least 1 character
- character `id` values must be unique within the team
- character positions must be unique within the team
- valid rows are `0..=2`
- valid columns are `0..=3`

## CharacterLoadout

```json
{
  "id": "the_emperor",
  "display_name": "The Emperor",
  "position": { "row": 0, "col": 0 },
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
  "actives": ["Crush", "Embolden"],
  "item": null,
  "rules": []
}
```

### Fields

- `id: string`
- `display_name?: string`
- `position: Position`
- `stats: StatBlock`
- `passive: string`
- `actives: string[]`
- `item: string | null`
- `rules: Rule[]`

### Notes

- `id` should be stable and machine-friendly
- `display_name` is intended for UI and replay readability
- `stats` are the final allocated stats for now, not base stats plus modifiers
- `passive` may be an empty string when the character has no passive equipped
- `actives` may be empty when the character has no active abilities equipped
- `item` remains nullable until item design and implementation are expanded
- bundled engine JSON now uses the same lowercase stat keys and `when` / `op` rule fields
- the current UI exposes up to three active slots explicitly, though the stored JSON remains `actives: string[]`

## Position

```json
{
  "row": 0,
  "col": 0
}
```

- `row: number`
- `col: number`

## StatBlock

```json
{
  "con": 7,
  "str": 8,
  "int": 3,
  "for": 7,
  "wis": 3,
  "dex": 4,
  "spi": 4
}
```

Required stats:

- `con`
- `str`
- `int`
- `for`
- `wis`
- `dex`
- `spi`

## Rule

```json
{
  "ability": "Crush",
  "when": [
    {
      "subject": "self",
      "value": "mp",
      "op": "gte",
      "threshold": 2
    }
  ]
}
```

### Fields

- `ability: string`
- `when: Condition[]`

## Condition

```json
{
  "subject": "target",
  "value": { "stat": "for" },
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
- `"use_count"`
- `"turns_since_use"`
- `"tick_count"`
- `"ally_count"`
- `"enemy_count"`
- `{ "stat": "con" | "str" | "int" | "for" | "wis" | "dex" | "spi" }`
- `{ "has_status": "Ward" }`
- `{ "status_stacks": "Empower:STR" }`

## Example Team

```json
{
  "version": 1,
  "name": "Imperial Phalanx",
  "characters": [
    {
      "id": "the_emperor",
      "display_name": "The Emperor",
      "position": { "row": 0, "col": 0 },
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
      "actives": ["Crush", "Embolden"],
      "item": null,
      "rules": [
        {
          "ability": "Crush",
          "when": [
            {
              "subject": "self",
              "value": "mp",
              "op": "gte",
              "threshold": 2
            }
          ]
        }
      ]
    },
    {
      "id": "the_tower",
      "display_name": "The Tower",
      "position": { "row": 0, "col": 1 },
      "stats": {
        "con": 8,
        "str": 7,
        "int": 2,
        "for": 8,
        "wis": 3,
        "dex": 2,
        "spi": 2
      },
      "passive": "Collapse",
      "actives": ["Rubble", "Shatter"],
      "item": null,
      "rules": [
        {
          "ability": "Shatter",
          "when": [
            {
              "subject": "self",
              "value": "mp",
              "op": "gte",
              "threshold": 2
            }
          ]
        }
      ]
    }
  ]
}
```

## Future Direction

The expected long-term format is lighter:

- roster data defines each tarot character
- team files reference roster entries by `character_id`
- team files only store overridden stats, loadout choices, rules, and position

That future format should preserve the same broad structure and validation rules where possible so the team builder can migrate without a full redesign.
