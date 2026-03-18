# Team A Implementation Plan

## Purpose

This file defines the engine work needed to support the current draft Team A characters:

- The Emperor
- The Hierophant
- The Chariot

The goal is to align implementation with the drafted kits while keeping the rollout incremental and testable.

This plan assumes the full Team A design should be finalized before code changes begin. It is a readiness and sequencing document, not a commitment to implement immediately.

## Team A Draft Summary

### The Emperor

Passive:

- `Imperial Formation`
  Allies in The Emperor's row gain `Empower STR 1` and `Empower INT 1`.

Abilities:

- `Hold the Line`
  Apply `Ward 1` to allies in the user's row.
- `Command`
  One companion immediately makes a basic attack against the user's current target.
- `Taunt`
  Enemies with `STR > INT` change their target to the user.

### The Hierophant

Passive:

- `Sanctuary`
  Allies in Hierophant's row gain `Empower WIS 1`.

Abilities:

- `Smite`
  Deal magical damage to the user's current target.
- `Consecrate`
  Deal magical damage to the user's current target and all of that target's companions.
- `Blessing`
  Restore MP to allies in the user's row.

### The Chariot

Passive:

- `Pursuit`
  When an ally damages Chariot's current target, Chariot gains `Empower STR 1`.

Abilities:

- `Charge`
  Deal physical damage to the user's current target. If the tile one row forward in the same column is empty, move there.
- `Withdraw`
  Deal physical damage to the user's current target. If the tile one row backward in the same column is empty, move there.
- `Breakthrough`
  Deal physical damage to the user's current target. `Empower STR` on the user counts double for this attack.

## Current Engine Support

Already supported:

- sticky targets for basic attacks and `current_target`
- companion detection by adjacency
- physical and magical damage primitives
- MP restoration primitives
- status application and removal
- enemy positional targeting, including `same_row`
- passive triggers such as `on_battle_start`, `on_take_damage`, and `on_deal_damage`

Partially supported:

- row-based target logic exists for enemies, but not for ally-side effects
- `Empower` and `Fortify` exist as placeholder statuses for temporary design approximation

Not yet supported:

- ally-side `same_row` targeting
- retarget primitives
- commanded out-of-turn attacks
- movement abilities that change formation position mid-battle
- a true `Ward` mechanic
- damage calculations that selectively reinterpret status stacks for a single ability

## Design Decisions To Preserve

The implementation should preserve these gameplay intentions:

- Emperor and Hierophant reward row discipline
- Chariot moves between middle and front row as part of his normal loop
- `Command` is impactful and notable, not a hidden minor rider
- `Taunt` uses sticky-target mutation rather than target locking
- `Breakthrough` is a real payoff attack for `Empower`, not just a larger generic multiplier

## Recommended Rollout

## Phase 1: Row-Based Ally Targeting

### Goal

Support effects that target allies in the user's row.

### Needed for

- `Imperial Formation`
- `Hold the Line`
- `Sanctuary`
- `Blessing`

### Implementation

Extend ally-side target resolution to support positional filters analogous to enemy-side targeting:

- `same_row` for allies
- optionally `same_column` for allies if desired for future reuse

Recommended schema direction:

- keep using detailed target specs
- allow `position: "same_row"` for `category: "ally"` and `category: "companion"`

Example:

```json
{
  "category": "ally",
  "position": "same_row"
}
```

### Notes

This should be a generic engine improvement, not an Emperor-specific shortcut.

### Tests

- ally same-row target selection
- same-row ally status application
- same-row ally MP restoration
- row filter excludes allies in other rows

## Phase 2: Retarget Primitive

### Goal

Implement the v1 retargeting system described in [retargeting_spec.md](./retargeting_spec.md).

### Needed for

- `Taunt`

### Implementation

Add a new primitive:

- `retarget`

Fields:

- `target`
- `mode`
- optional `filter`

Initial supported modes:

- `to_self`
- `to_companion`
- `default_retarget`

Initial supported filter:

- `physical_attackers`

Meaning:

- affected units with `STR > INT`

### Resolution rules

For each affected unit:

1. skip dead units
2. apply filter if present
3. resolve a new sticky target
4. overwrite sticky target if valid
5. otherwise keep the old target, except `default_retarget`, which rebuilds target state

### Tests

- `to_self` retargets only filtered physical attackers
- `to_companion` selects a living companion
- `default_retarget` rebuilds target using normal targeting logic
- invalid retarget leaves old target unchanged

## Phase 3: Commanded Companion Attack

### Goal

Allow one companion to immediately basic attack the user's current target.

### Needed for

- `Command`

### Implementation

Add a new primitive:

- `command_attack`

Recommended v1 semantics:

- choose one living companion of the caster
- companion makes an immediate basic attack
- target is the caster's current target
- commanded attack cannot trigger another ability
- commanded attack should still:
  - deal normal basic damage
  - trigger normal damage/death/passive resolution

Recommended companion selection rule for v1:

- highest STR living companion

This is more readable and strategic than random for Emperor.

### Open design choice

The plan should explicitly decide whether commanded attacks count as:

- the companion dealing damage for passive purposes
- an extra action for turn-count/resource purposes

Recommended behavior:

- yes for damage-triggered passives
- no for turn-count progression or MP regen

### Tests

- commanded companion attacks current target
- command does nothing if no current target exists
- command does nothing if no living companion exists
- command triggers normal defeat and passive resolution
- commanded attack does not spend MP or count as the companion's turn

## Phase 4: Movement Primitive

### Goal

Support same-column forward and backward movement if the destination tile is empty.

### Needed for

- `Charge`
- `Withdraw`

### Implementation

Add a movement primitive or movement rider support.

Recommended primitive:

- `move`

Fields:

- `direction: "forward" | "backward"`
- `if_empty: true`

Recommended v1 behavior:

- movement is always same-column
- moving forward means toward the enemy side
- moving backward means away from the enemy side
- movement only occurs if the destination tile exists and is unoccupied by an ally

Because Team A and Team B face opposite directions, the implementation must define movement in team-relative terms rather than raw row increment/decrement.

### Important follow-on behavior

Moving a character should update:

- position
- companion relationships
- row-based target effects for future ability resolution

Recommended v1 simplification:

- recompute companions for the moving unit's team immediately after movement

### Open design choice

Should movement force enemies currently targeting the moved unit to reevaluate?

Recommended v1:

- no automatic retarget
- sticky targets remain unless another effect changes them or the target later becomes invalid

### Tests

- move forward into empty tile
- fail to move when blocked
- fail to move when leaving board bounds
- same-column movement only
- companion relationships update after movement

## Phase 5: Real Ward Mechanic

### Goal

Replace placeholder `Fortify` approximations with a true `Ward` defensive effect.

### Needed for

- `Hold the Line`
- Chariot's intended `Withdraw after Ward absorbed` rule logic

### Implementation

Add a new status:

- `Ward`

Recommended v1 semantics:

- `Ward` is consumed when the character takes damage
- a consumed ward negates one hit entirely

Alternative simpler version:

- `Ward` reduces the next incoming hit to 0 and is removed

This should be implemented in damage intake rather than as a stat modifier.

### Why this matters

Chariot's intended scripted loop depends on whether he actually absorbed a Ward-triggered hit, not on generic defense stats.

### Tests

- ward negates next hit and is removed
- multiple wards consume one at a time if stacking is allowed
- ward works against physical and magical damage
- ward interaction with reflect should be defined and tested

## Phase 6: Status-Aware Rule Conditions

### Goal

Allow rules to react to statuses and stacks directly.

### Needed for

- Chariot `Withdraw` after having/consuming Ward
- Chariot `Breakthrough` setup and payoff logic
- future richer synergy scripting

### Implementation

Add rule query support for:

- `has_status`
- `status_stacks`

Example:

```json
{ "subject": "self", "value": { "has_status": "Ward" }, "op": "gte", "threshold": 1 }
```

or

```json
{ "subject": "self", "value": { "status_stacks": "Empower", "stat": "str" }, "op": "gte", "threshold": 2 }
```

### Recommendation

This phase is important enough that it should happen before Chariot's full scripted behavior is considered complete.

### Tests

- self has_status checks
- self status_stacks checks
- target status checks
- interaction with stat-keyed statuses like `Empower:STR`

## Phase 7: Breakthrough Payoff Support

### Goal

Support an attack that treats `Empower STR` as double for one ability.

### Needed for

- `Breakthrough`

### Implementation options

#### Option A: Dedicated primitive

Add:

- `deal_physical_damage_with_doubled_empower`

Pros:

- simple and explicit

Cons:

- not very reusable

#### Option B: Damage modifier flag

Add optional fields to physical-damage primitives:

- `double_empower_stat: "str"`

Pros:

- more reusable
- cleaner for future special attacks

Cons:

- requires slightly more damage-calculation plumbing

### Recommendation

Use Option B.

This keeps the primitive library from exploding while still supporting notable payoff attacks.

### Tests

- damage increases correctly with `Empower STR`
- only `Empower STR` is doubled
- doubled empower applies only to that one ability

## Phase 8: Data Integration

### Goal

Add Team A's drafted passives and abilities to the bundled catalogs and sample teams.

### Needed data additions

Passives:

- `Imperial Formation`
- `Sanctuary`
- `Pursuit`

Abilities:

- `Hold the Line`
- `Command`
- `Taunt`
- `Smite`
- `Consecrate`
- `Blessing`
- `Charge`
- `Withdraw`
- `Breakthrough`

### Sample formation

Recommended initial Team A sample:

- Emperor in front row
- Hierophant in same row as Emperor
- Chariot in middle row

This supports the intended Chariot loop:

- begin protected
- `Charge` forward
- absorb support
- `Withdraw` back
- save MP for `Breakthrough`

## Phase 9: Replay and UI Support

### Goal

Ensure the replay viewer and team builder expose the new mechanics clearly.

### Needed updates

- replay events for movement
- replay events for retarget changes if useful for debugging
- status display for `Ward`
- builder catalogs updated with new passives and abilities

### Recommendation

Movement events are likely worth logging explicitly. Retarget changes should at least be visible in text replay even if not yet visualized on the board.

## Recommended Implementation Order

1. Phase 1: row-based ally targeting
2. Phase 2: retarget primitive
3. Phase 3: commanded companion attack
4. Phase 4: movement primitive
5. Phase 5: real Ward mechanic
6. Phase 6: status-aware rule conditions
7. Phase 7: Breakthrough payoff support
8. Phase 8: data integration
9. Phase 9: replay and UI support

## Suggested Milestones

### Milestone A

Emperor and Hierophant are functional:

- row-based ally effects work
- `Taunt` works
- `Smite` and `Consecrate` work using existing damage primitives
- `Blessing` works

### Milestone B

Chariot movement loop is functional:

- movement works
- `Command` works
- `Ward` works
- basic row-dance play pattern is visible in logs

### Milestone C

Team A scripting and payoff are functional:

- status-aware rules work
- `Breakthrough` payoff works
- replay viewer can show the resulting battle clearly

## Risks

- movement plus sticky targets can create subtle targeting edge cases
- a real `Ward` mechanic affects many combat interactions, not just Team A
- commanded attacks may create unexpected passive-trigger chains
- status-aware rules expand the rule language and need careful validation

## Recommendation

Do not implement all of this at once.

The safest first coding slice is:

- Phase 1
- Phase 2
- Phase 3

That gets Emperor mostly online and gives a good foundation for the rest of Team A.
