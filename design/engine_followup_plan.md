# Engine Follow-Up Plan

## Goal

Close the highest-value remaining gaps between the current gameplay spec and the live engine without mixing in brainstorming-only mechanics.

## Main Remaining Gaps

### 1. Condition Layer

The spec now treats conditions as separate from buffs and debuffs, but the engine still only has statuses plus retarget effects.

Missing live support:

- `Marked`
- `Severed`

Desired outcome:

- conditions become a first-class engine layer
- replay snapshots and logs expose them clearly
- rules and abilities can reference them directly without overloading ordinary statuses
- `Stunned` is modeled as a non-stacking condition that loses `1` stack at end of turn
- `Marked` and `Severed` are modeled as stacking conditions that lose `1` stack at end of turn

Future candidate conditions:

- `Muted`

Suggested implementation order:

1. add runtime condition storage to `CharacterState`
2. add apply/remove/query primitives for conditions
3. add replay logging and snapshot serialization
4. update abilities that should use conditions instead of placeholder status logic

### 2. Compound Ability Atomic Targeting

Some multi-step abilities should bind a selected target once and reuse it across steps. `Rescue` is the clearest current example.

Desired outcome:

- primitives can opt into one bound target reused across nested steps
- target identity is stable across move/heal/refocus style sequences

Suggested implementation order:

1. add a temporary bound-target slot to execution context
2. add a primitive or wrapper to select and bind once
3. update `Rescue`
4. add tests for selector stability across chained steps

### 3. Ability Catalog Cleanup

The bundled ability file still contains deprecated prototype abilities that are no longer part of the intended live pool.

Current cleanup targets:

- remove `Channel`
- remove `Invocation`
- remove `Restoration`

Desired outcome:

- bundled live catalog only contains current roster abilities plus explicitly marked prototype leftovers if any remain
- UI catalogs and sample teams stop exposing replaced abilities

### 4. Status Grouping

The spec now groups effects into `Body`, `Mind`, and `Fate`, but the engine still uses generic polarity-only cleanse/dispel behavior.

Desired outcome:

- statuses carry an explicit group tag
- generic cleanse / dispel follows the intended group rules
- specialized cleanse / dispel can target a specific group later

Suggested implementation order:

1. add optional group metadata to status definitions
2. migrate bundled statuses
3. update generic cleanse / dispel behavior
4. add tests for grouped removal rules

### 5. Halving Decay

The intended decay model is still not live. The engine currently ticks down by `1`.

Desired outcome:

- `Omen` and `Restoration` halve after start-of-turn resolution
- `Empower`, `Weaken`, and `Lethality` halve at end of turn
- current conditions lose `1` stack at end of turn unless consumed or removed earlier

Suggested implementation order:

1. add decay-mode support to status definitions or status runtime
2. implement halving resolution points
3. migrate bundled live statuses
4. retune affected sample abilities if needed

### 6. Damage Model Evolution

The spec now points toward `flat base + multiplier`, but the engine only partially supports that direction.

Desired outcome:

- attacks can optionally use `base + multiplier`
- low-multiplier splash and setup attacks stay meaningful through defense
- existing multiplier-only abilities still remain supported

Suggested implementation order:

1. add optional flat base damage fields to physical and magical damage primitives
2. migrate a small set of abilities first
3. retune sample roster around the new numbers

This should be treated as a balance pass, not a small mechanical cleanup.

## Recommended Execution Order

1. condition layer
2. atomic target binding for compound abilities
3. ability catalog cleanup
4. status grouping
5. halving decay
6. optional flat-base damage migration

## Verification Checklist

After each stage:

- `cargo test -q`
- `cargo clippy -q`

After data changes:

- `cargo test -q bundled_content_references_are_valid`
- `cargo run -q -- 42`

After replay-impacting changes:

- load the latest replay in the UI
- verify new effects are legible in both snapshots and timeline text
