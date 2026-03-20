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
- valid columns are `0..=2`

## CharacterLoadout

```json
{
  "id": "the_emperor",
  "display_name": "The Emperor",
  "position": { "row": 0, "col": 0 },
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
  "actives": ["Hold the Line", "Command", "Taunt"],
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
  "vit": 7,
  "mgt": 8,
  "mag": 3,
  "arm": 7,
  "res": 3,
  "spd": 4,
  "wil": 4
}
```

Required stats:

- `vit`
- `mgt`
- `mag`
- `arm`
- `res`
- `spd`
- `wil`

## Rule

```json
{
  "ability": "Hold the Line",
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

Rules are character-level priorities, not per-ability subprograms.

Each character may have up to 5 rules total.

They are evaluated in array order:

- the first satisfied rule is used
- later rules are ignored once one rule fires
- if no rule is satisfied, the character uses `Rest`

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
        "vit": 7,
        "mgt": 8,
        "mag": 3,
        "arm": 7,
        "res": 3,
        "spd": 4,
        "wil": 4
      },
      "passive": "Imperial Formation",
      "actives": ["Hold the Line", "Command", "Taunt"],
      "item": null,
      "rules": [
        {
          "ability": "Hold the Line",
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
      "id": "the_hierophant",
      "display_name": "The Hierophant",
      "position": { "row": 0, "col": 2 },
      "stats": {
        "vit": 11,
        "mgt": 3,
        "mag": 7,
        "arm": 4,
        "res": 7,
        "spd": 3,
        "wil": 7
      },
      "passive": "Sanctuary",
      "actives": ["Smite", "Consecrate", "Blessing"],
      "item": null,
      "rules": [
        {
          "ability": "Blessing",
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
