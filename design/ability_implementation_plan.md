# Ability Implementation Plan

This file tracks the staged engine and data work needed to support the next set of core abilities from [ability_roster.md](/home/frostbyte/Work/tarot-battler/design/ability_roster.md).

## Scope

New abilities to implement:

- `Sanctify`
- `Sunder`
- `Invocation`
- `Sever`
- `Restoration`
- `Interpose`
- `Decoy`

## Current Support

Already supported well enough:

- `Sunder`
  - physical damage
  - apply `Weaken ARM`
- `Restoration`
  - restore HP to a companion target

Partially supported or needs engine work:

- `Sanctify`
  - needs generic `cleanse` behavior rather than named status removal
- `Invocation`
  - needs a target-lacks-status conditional
- `Sever`
  - needs bonus damage that consumes multiple statuses from the user
- `Interpose`
  - needs retarget filtering for enemies targeting a companion
- `Decoy`
  - needs retarget filtering for enemies targeting the user

## Stages

### Stage 1: Cleanse and Dispel Primitives

Add reusable primitives for the new removal vocabulary:

- `cleanse`
- `dispel`

Behavior for v1:

- `cleanse N` removes `N` ticks from matching debuffs
- `dispel N` removes `N` ticks from matching buffs
- initial implementation should support the broad default behavior from the design spec

This stage unlocks:

- `Sanctify`

Status:

- completed
- engine now supports broad `cleanse` and `dispel` primitives for timed effects
- conditions and permanent effects remain untouched

### Stage 2: Conditional Target-Lacks-Status Primitive

Add a conditional primitive for:

- `if_target_lacks_status`

This stage unlocks:

- `Invocation`

Status:

- completed
- engine now supports `if_target_lacks_status`

### Stage 3: Consume-Own-Statuses Damage Primitive

Add a reusable physical damage primitive that:

- deals normal physical damage
- consumes selected statuses from the user
- adds bonus damage based on consumed stacks

This stage unlocks:

- `Sever`

Status:

- completed
- engine now supports physical bonus-damage attacks that consume selected self-statuses

### Stage 4: Retarget Filter Expansion

Extend retarget filtering to support:

- enemies currently targeting the user
- enemies currently targeting any of the user's companions

This stage unlocks:

- `Interpose`
- `Decoy`

### Stage 5: Data Integration

Add the new abilities to:

- [abilities.json](/home/frostbyte/Work/tarot-battler/battle_engine/src/data/abilities.json)

Update:

- [ability_roster.md](/home/frostbyte/Work/tarot-battler/design/ability_roster.md)
- relevant UI descriptions if needed

### Stage 6: Verification and Follow-Up

Add regression tests for:

- generic cleanse on ally-side targets
- target-lacks-status conditional branching
- consuming multiple self-buffs for bonus damage
- `Interpose`
- `Decoy`

Then verify:

- `cargo test -q`
- `cargo clippy -q`
- `node --check tools/ui/app.js`

## Notes

- This plan intentionally stops at reusable primitives and the seven new abilities.
- It does not yet implement the full grouped `Body / Mind / Fate` removal model beyond the broad generic cleanse/dispel behavior needed immediately.
- The long-term grouped-removal model should stay documented in [game_spec.md](/home/frostbyte/Work/tarot-battler/design/game_spec.md) even if the first engine implementation is narrower internally.
