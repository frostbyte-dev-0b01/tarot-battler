# Rest Fallback Implementation Plan

This file tracks the staged engine changes needed to align the prototype with the updated design:

- characters do not passively regain MP between turns
- every character has a default `Rest` action
- `Rest` restores `floor(WIL / 2)` MP
- `Rest` is the fallback action when no rule is satisfied

## Scope

This plan updates:

- battle execution
- replay logging and replay JSON
- UI replay formatting for the new action
- affected tests and docs

It does not remove commanded attacks or other explicit attack effects. Abilities such as `Command` still cause an ally to make an attack.

## Stages

### Stage 1: Add Rest as a Logged Action

Add engine and tooling support for a `Rest` action without changing fallback behavior yet.

Work:

- add a `Rest` battle event
- add replay JSON export for `rest`
- add readable replay formatting for `Rest`
- update the UI timeline/current-event formatting to display `Rest`
- clarify wording for `Command` so it explicitly describes a standard attack using a normal physical attack profile

Result:

- the engine and tooling understand `Rest`
- no gameplay behavior changes yet

### Stage 2: Change Fallback from Basic Attack to Rest

Change the battle loop so that when no rule is satisfied, the actor uses `Rest` instead of a universal basic attack.

Work:

- replace fallback basic-attack execution in `engine.rs` / `turns.rs`
- implement `Rest` execution as `restore floor(WIL / 2) MP`
- log `Rest`
- update tests that currently expect fallback `BasicAttack`
- update docs to note the engine now matches the intended fallback action

Result:

- only explicit attack effects and active abilities generate attacks
- fallback behavior matches the current game spec

### Stage 3: Remove Passive End-of-Turn MP Regeneration

Remove automatic MP restoration at end of turn.

Work:

- remove turn-end MP regen from turn finishing
- update tests that currently expect `turn_regen`
- update docs and repo notes to remove the implementation mismatch

Result:

- `Rest` is the primary MP recovery mechanism
- no passive turn-by-turn MP gain remains

## Verification

After each stage:

- `cargo test -q`
- `cargo clippy -q`
- `node --check tools/ui/app.js`

## Notes

- `basic_attack` replay events should remain for commanded attacks and any future explicit attack effects
- `damage.rs` can remain, since the engine still needs standard attack damage resolution for commanded attacks
- once this plan is complete, the prototype and the design docs should match on the fallback/resource model

