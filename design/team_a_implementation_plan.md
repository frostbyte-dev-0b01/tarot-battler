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
- enemy companion targeting
- passives that react to an ally damaging this character's current target
- dynamic row-aura passives that update as units move

## Design Decisions To Preserve

The implementation should preserve these gameplay intentions:

- Emperor and Hierophant reward row discipline
- Chariot moves between middle and front row as part of his normal loop
- `Command` is impactful and notable, not a hidden minor rider
- `Taunt` uses sticky-target mutation rather than target locking
- `Breakthrough` is a real payoff attack for `Empower`, not just a larger generic multiplier
- row-based passives should follow current formation state rather than only battle-start snapshots
- `Consecrate` should use enemy-companion splash rather than a generic all-enemies fallback
- `Pursuit` should care about coordinated focus fire on Chariot's current target, not generic ally damage

## Recommended Rollout

## Phase 1: Row-Based Ally Targeting

Status:

- implemented

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

Status:

- implemented

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

Status:

- implemented

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

Status:

- implemented

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

Status:

- implemented

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

Status:

- implemented

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
{ "subject": "self", "value": { "status_stacks": "Empower:STR" }, "op": "gte", "threshold": 2 }
```

### Recommendation

This phase is important enough that it should happen before Chariot's full scripted behavior is considered complete.

### Tests

- self has_status checks
- self status_stacks checks
- target status checks
- interaction with stat-keyed statuses like `Empower:STR`

## Phase 7: Breakthrough Payoff Support

Status:

- implemented

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

### Blocked on

- Phase 8A: enemy companion targeting
- Phase 8B: dynamic row auras
- Phase 8C: focus-fire passive trigger

Without those capabilities, Team A can only be approximated and Chariot's intended loop will not behave correctly.

## Phase 8A: Enemy Companion Targeting

Status:

- implemented

### Goal

Support effects that hit the user's current target and that target's companions.

### Needed for

- `Consecrate`

### Implementation

Add a new enemy targeting shape that resolves:

- the user's current target
- all living companions of that target

Recommended v1 shape:

- `current_target_and_companions`

This should remain target-resolution logic, not a one-off Hierophant shortcut.

### Tests

- includes current target
- includes all living companions of current target
- excludes non-companions
- handles missing current target cleanly

## Phase 8B: Dynamic Row Aura Support

### Goal

Support row-based passives that stay correct as formation changes during battle.

### Needed for

- `Imperial Formation`
- `Sanctuary`

### Implementation

Add a lightweight row-aura system for passives that:

- applies aura statuses to qualifying allies
- removes aura statuses when the source moves, dies, or row membership changes
- reevaluates after battle start and after movement

Recommended v1 scope:

- only ally same-row aura passives
- only stat-mod aura statuses

This avoids baking Team A behavior into one-time `on_battle_start` applications that would not follow Chariot when he changes rows.

### Tests

- same-row ally gains aura status while sharing row
- ally loses aura status after source or ally moves out of row
- dead source removes aura effect
- overlapping row auras stack predictably or are explicitly defined not to

## Phase 8C: Focus-Fire Passive Trigger

### Goal

Support passives that react when an ally damages this character's current target.

### Needed for

- `Pursuit`

### Implementation

Add a new passive trigger concept for:

- ally damages my current target

Recommended semantics:

- fires on actual damage dealt, not on zero-damage hits
- checks the passive owner's current sticky target at the time the ally damage resolves
- does not fire from the owner's own damage
- can fire from commanded attacks if the commanded unit is an ally and hits the correct target

Long-term direction:

- this v1 trigger is intentionally narrow
- a more flexible future system should likely generalize this into ally-event passives with event-type and relationship filters

### Tests

- fires when an ally damages the passive owner's current target
- does not fire on unrelated enemy targets
- does not fire on self damage
- fires off commanded companion attacks if target matches

## Phase 9: Data Integration

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

## Phase 10: Replay and UI Support

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
8. Phase 8A: enemy companion targeting
9. Phase 8B: dynamic row aura support
10. Phase 8C: focus-fire passive trigger
11. Phase 9: data integration
12. Phase 10: replay and UI support

## Suggested Milestones

### Milestone A

Emperor and Hierophant are functional:

- row-based ally effects work
- `Taunt` works
- `Smite` and `Consecrate` work using existing damage primitives
- `Blessing` works
- row auras update as formation changes

### Milestone B

Chariot movement loop is functional:

- movement works
- `Command` works
- `Ward` works
- basic row-dance play pattern is visible in logs
- `Pursuit` can build Empower during coordinated focus fire

### Milestone C

Team A scripting and payoff are functional:

- status-aware rules work
- `Breakthrough` payoff works
- replay viewer can show the resulting battle clearly
- bundled Team A data and sample battle are aligned with the intended kits

## Risks

- movement plus sticky targets can create subtle targeting edge cases
- a real `Ward` mechanic affects many combat interactions, not just Team A
- commanded attacks may create unexpected passive-trigger chains
- status-aware rules expand the rule language and need careful validation
- dynamic auras can create subtle add/remove timing bugs after movement and death
- focus-fire passive triggers can overfire if damage attribution is not kept precise

## Recommendation

Do not implement all of this at once.

The safest first coding slice is:

- Phase 1
- Phase 2
- Phase 3

That gets Emperor mostly online and gives a good foundation for the rest of Team A.
