# Engine Follow-Up Plan

## Goal

Close the highest-value remaining gaps between the current gameplay spec and the live engine without mixing in brainstorming-only mechanics.

## Main Remaining Gaps

### 1. Condition Layer

Status: complete.

What landed:

- conditions become a first-class engine layer
- replay snapshots and logs expose them clearly
- rules and abilities can reference them directly without overloading ordinary statuses
- `Stunned` is modeled as a non-stacking condition that loses `1` stack at end of turn
- `Marked` and `Severed` are modeled as stacking conditions that lose `1` stack at end of turn

Remaining follow-up:

- migrate live roster abilities to use `Marked` / `Severed`
- remove the older prototype `Stun` status once bundled data no longer depends on it

Future candidate conditions:

- `Muted`

### 2. Compound Ability Atomic Targeting

Status: complete.

What landed:

- primitives can opt into one bound target reused across nested steps
- target identity is stable across move/heal/refocus style sequences
- `Rescue` now binds its selected companion once and reuses it across move, heal, and enemy-refocus

Remaining follow-up:

- decide whether more bundled abilities should use bound targets
- keep compound target binding in mind when adding future multi-step support abilities

### 3. Ability Catalog Cleanup

Status: complete.

What landed:

- remove `Channel`
- remove `Invocation`
- remove `Restoration`

Result:

- bundled live catalog only contains current roster abilities plus explicitly marked prototype leftovers if any remain
- UI catalogs and sample teams stop exposing replaced abilities

### 4. Status Grouping

Status: complete.

What landed:

- statuses carry an explicit group tag
- stat-mod groups are derived from the affected stat when needed
- generic cleanse / dispel can optionally filter by group
- specialized cleanse / dispel can target a specific group later

Remaining follow-up:

- decide which bundled abilities should start using explicit group filters

### 5. Halving Decay

Status: complete.

What landed:

- `Omen` and `Restoration` halve after start-of-turn resolution
- `Empower`, `Weaken`, and `Lethality` halve at end of turn
- current conditions lose `1` stack at end of turn unless consumed or removed earlier

Remaining follow-up:

- retune affected bundled abilities if the new decay curve shifts balance too sharply
- decide later whether legacy placeholder effects should also migrate away from tick-down-by-1

### 6. Damage Model Evolution

Status: complete.

What landed:

- attacks can optionally use `base + multiplier`
- low-multiplier splash and setup attacks stay meaningful through defense
- existing multiplier-only abilities still remain supported

What was migrated:

- support was added to the main physical and magical hit primitives
- bundled low-multiplier magical setup and splash abilities now use explicit base damage floors
- multiplier-only attacks remain valid and still cover the rest of the live roster

Remaining follow-up:

- decide whether more physical utility attacks should migrate to `base + multiplier`
- continue ability-by-ability balance work rather than forcing the whole roster into one damage shape immediately

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
