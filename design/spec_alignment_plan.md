# Spec Alignment Plan

## Goal

Bring the engine, bundled data, replay output, and tooling closer to the current gameplay spec and core ability roster.

This plan intentionally separates:

- **current core-pool alignments** that should become live
- **future brainstorm mechanics** that should stay in design docs only

The target is not "implement every brainstorm idea." The target is to make the engine faithfully support the current intended core rules and the currently promoted core ability pool.

## Main Gaps Today

### Rules

The spec now expects more expressive rule queries than the engine currently exposes cleanly:

- `self_row`
- `self_companion_count`
- `target_companion_count`

These are important because several current core abilities are only worth scripting if topology information is visible.

### Focus-Control Language

The design now distinguishes:

- `focus`
- `refocus`
- `disorient`

The engine still mostly expresses this as retargeting modes internally, and does not yet have a negative `disorient`-style focus reset.

### Condition Layer

The spec now treats conditions as separate from buffs/debuffs, with the initial intended list:

- `Stunned`
- `Muted`
- `Marked`
- `Severed`

The engine currently only has a direct turn-skip status and ordinary status tracking. It does not yet distinguish conditions as a separate gameplay-facing layer.

### Damage Shape

The spec now points toward many abilities using:

- flat base damage
- plus a stat multiplier

The engine currently only supports multiplier-based physical and magical damage plus special-case true-damage-like payoff handling.

### Core Ability Drift

Several bundled abilities still reflect older versions of the design. Examples:

- `Hold the Line` still costs `4` in data
- `Charge` / `Withdraw` use older numbers and do not yet align with current focus wording/effects
- `Rebuke` is still a pure retarget effect
- `Blessing` is still only MP restoration
- `Consecrate`, `Eclipse`, and `Harvest Night` do not match the latest current design
- `Invocation` still exists in the old form even though `Seal` is the newer intended replacement
- `Transmute` still uses the earlier MP-refund idea
- support abilities like `Sanctify` / `Rescue` are not yet aligned to the newer "visible swing" standard

## Scope

### In Scope

- rule query additions needed by current intended scripting
- engine support for current core focus-control vocabulary
- engine support for true-damage riders and transform-style effects used by promoted core abilities
- data updates for the promoted core ability pool
- sample roster and rules updates where needed
- replay/logging updates if new effects need explicit visibility
- docs and UI hover descriptions aligned with the new live definitions

### Out Of Scope For This Pass

- implementing all brainstorming abilities
- full condition-system overhaul for every proposed future condition
- full migration to flat-base-plus-multiplier damage across the entire engine
- large item-system or passive-pool expansion
- broad movement-system expansion

Those should remain separate follow-up work once the promoted core pool is stable.

## Implementation Stages

### Stage 1: Doc And Roster Cleanup

- align `game_spec.md`, `ability_roster.md`, and `implementation_notes.md`
- make implemented-vs-planned boundaries explicit
- lock the intended current core ability wording for the live subset

Verification:

- docs agree on focus/conditions terminology
- docs clearly distinguish current core pool from brainstorming

### Stage 2: Rule Query Support

Status: completed in `rule query support` stage commit.

- add rule query support for:
  - `self_row`
  - `self_companion_count`
  - `target_companion_count`
- update validation and team-builder support for the new query keys

Verification:

- rules can reference the new values in JSON
- tests cover each new query path

### Stage 3: Focus And Target-Control Support

Status: completed in `focus-control support` stage commit.

- add a `disorient`-style targeting mode or equivalent effect primitive
- keep internal retargeting mechanics, but expose the design-facing distinction in data/docs
- ensure refocus-style effects can be applied to:
  - self
  - focused enemy
  - enemies focusing a given ally
  - allies in a row where current core abilities need it

Verification:

- targeted focus changes show up correctly in replay snapshots and timeline text
- tests cover positive and negative focus changes

### Stage 4: Ability Primitive Extensions

- add or generalize primitives needed by the revised live pool:
  - true damage rider / direct true-damage hit
  - split primary/secondary damage for target-and-companions abilities
  - transform status stacks from one family to another
  - compound support actions like heal + reposition + force refocus
  - row-wide ally refocus helpers where needed

Verification:

- new primitives are data-driven where possible
- tests cover each primitive directly

### Stage 5: Core Ability Data Alignment

Update bundled live abilities to their current intended versions, including at least:

- `Hold the Line`
- `Command`
- `Taunt`
- `Rebuke`
- `Charge`
- `Withdraw`
- `Breakthrough`
- `Condemn`
- `Verdict`
- `Sunder`
- `Sever`
- `Smite`
- `Consecrate`
- `Hex`
- `Eclipse`
- `Harvest Night`
- `Seal`
- `Transmute`
- `Blessing`
- `Offer`
- `Distill`
- `Sanctify`
- `Rescue`

This stage also includes removing or replacing old bundled live abilities that are no longer in the intended core pool.

Verification:

- `abilities.json` matches the current roster wording and costs
- descriptions used by the UI match the live mechanics

### Stage 6: Sample Roster And Rule Alignment

- update bundled characters to use the revised live abilities
- update MP thresholds and scripts to use the new costs and new rule queries
- make at least one sample team show off the newer support/control directions cleanly

Verification:

- bundled sample battle still runs
- replay shows the updated mechanics clearly

### Stage 7: Replay, UI, And Validation Cleanup

- update replay formatting / timeline text where new mechanics need clearer names
- update UI catalogs and hover descriptions
- update team validation if new value keys or status names are required
- add targeted regression tests for replay visibility if needed

Verification:

- replay viewer remains usable with the new mechanics
- new conditions / transforms / focus changes are legible in the UI

## Recommended Commit Boundaries

One commit per stage:

1. docs cleanup
2. rule query support
3. focus-control support
4. primitive extensions
5. ability data alignment
6. sample roster alignment
7. replay/UI cleanup

Each stage should run the smallest relevant verification set before commit.

## Verification Checklist

After each code stage:

- `cargo test -q`
- `cargo clippy -q`

After data and UI stages:

- `cargo run -q -- 42`
- `node --check tools/ui/app.js`

Final manual sanity checks:

- load latest replay in the UI
- verify focus-changing abilities read clearly in the timeline
- verify sample teams still produce understandable battles
