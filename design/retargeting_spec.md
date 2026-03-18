# Retargeting Spec

## Purpose

This file defines the first-pass retargeting model for the battle engine.

Retargeting is intentionally narrow in scope for v1:

- it mutates sticky targets
- it does not add target locks
- it does not modify row protection rules
- it does not permanently alter targeting logic

The goal is to support battlefield control effects such as taunt, decoys, and forced target reevaluation without overbuilding a larger control system yet.

## Core Model

Each unit has a sticky target used for:

- basic attacks
- `current_target` ability effects

Retargeting effects overwrite or rebuild that sticky target for affected units.

## Primitive

```json
{
  "kind": "retarget",
  "target": "all_enemies",
  "mode": "to_self"
}
```

### Fields

- `kind: "retarget"`
- `target: AbilityTarget`
- `mode: RetargetMode`
- `filter?: RetargetFilter`

## Retarget Modes

### `to_self`

```json
{
  "kind": "retarget",
  "target": "all_enemies",
  "mode": "to_self"
}
```

Effect:

- each affected unit sets its sticky target to the caster

Rules:

- the caster must be alive
- the caster must be a valid target for the affected unit
- if invalid, the affected unit keeps its current target

Primary uses:

- taunt
- aggro control
- frontline protection

### `to_companion`

```json
{
  "kind": "retarget",
  "target": "all_enemies",
  "mode": "to_companion"
}
```

Effect:

- each affected unit sets its sticky target to one living companion of the caster

Rules:

- the caster must have at least one living companion
- companion selection for v1 is random among living companions
- the chosen companion must be a valid target for the affected unit
- if invalid, the affected unit keeps its current target

Primary uses:

- decoy effects
- bodyguard redirection
- protect-carry abilities

### `default_retarget`

```json
{
  "kind": "retarget",
  "target": "all_enemies",
  "mode": "default_retarget"
}
```

Effect:

- each affected unit clears its sticky target
- each affected unit immediately reruns normal target selection

Rules:

- the affected unit must be alive
- if no valid target exists, the new target becomes `null`

Primary uses:

- disruption
- confusion-style effects
- forcing target reevaluation after other battlefield changes

## Retarget Filters

V1 should keep filtering minimal.

The first recommended filter is:

- `physical_attackers`

Meaning:

- units whose `STR > INT`

This is enough to support early taunt-style abilities without committing to a broader filtering system.

Example:

```json
{
  "kind": "retarget",
  "target": "all_enemies",
  "mode": "to_self",
  "filter": "physical_attackers"
}
```

## Resolution Rules

For each affected unit:

1. skip dead units
2. apply the optional filter
3. resolve the new target based on `mode`
4. if a valid target is found, overwrite the sticky target
5. otherwise keep the existing target, except for `default_retarget`, which explicitly rebuilds target state

## Example Uses

### Taunt

```json
{
  "mp_cost": 2,
  "primitives": [
    {
      "kind": "retarget",
      "target": "all_enemies",
      "mode": "to_self",
      "filter": "physical_attackers"
    }
  ]
}
```

### Decoy Orders

```json
{
  "mp_cost": 2,
  "primitives": [
    {
      "kind": "retarget",
      "target": "all_enemies",
      "mode": "to_companion"
    }
  ]
}
```

### Confuse

```json
{
  "mp_cost": 2,
  "primitives": [
    {
      "kind": "retarget",
      "target": "all_enemies",
      "mode": "default_retarget"
    }
  ]
}
```
