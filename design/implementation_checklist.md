# Engine Alignment Checklist

This checklist tracks the work required to align the Rust battle engine with the current design spec in:

- [game_spec.md](/home/frostbyte/Work/tarot-battler/design/game_spec.md)
- [character_design.md](/home/frostbyte/Work/tarot-battler/design/character_design.md)
- [implementation_notes.md](/home/frostbyte/Work/tarot-battler/design/implementation_notes.md)

## Phase 1: Stat and Resource Cleanup

- [x] Remove `FOC` and the older save-style `RES` concept from engine models, loaders, sample data, and tests
- [x] Keep `WIL` as the base stat
- [x] Rename runtime resource terminology from WIL to MP in code where practical
- [x] Update rules, primitives, helpers, and logs to use MP terminology
- [x] Update docs to reflect completed engine alignment
- [x] Run test suite
- [x] Commit Phase 1

## Phase 2: Speed System Rewrite

- [x] Replace current SPD timing model with `max_ticks = 10 - SPD`
- [x] Clamp `ticks_until_turn` with `max(max_ticks, 1)`
- [x] Increase `max_ticks` by `2` after each turn
- [x] Update tests for new speed progression
- [x] Update docs to reflect completed engine alignment
- [x] Run test suite
- [x] Commit Phase 2

## Phase 3: Turn Resolution Ordering

- [x] Enforce turn order: on-turn-start passives, status ticks, action or stun skip, death resolution, end-of-turn effects, MP regen, speed reset
- [x] Ensure stunned characters still receive all non-action turn processing
- [x] Add regression tests for stunned-turn behavior
- [x] Update docs to reflect completed engine alignment
- [x] Run test suite
- [x] Commit Phase 3

## Phase 4: Rule System Alignment

- [x] Remove `Ally` as a rule subject
- [x] Keep only `SelfChar`, `Companion`, `Target`, and `World`
- [x] Make rule stat checks use effective stats
- [x] Rename `round_count` semantics to `tick_count` in exposed rule vocabulary
- [x] Add regression tests for companion semantics and world counters
- [x] Update docs to reflect completed engine alignment
- [x] Run test suite
- [x] Commit Phase 4

## Phase 5: MP Regen and Tick Semantics

- [x] Replace global step-based resource regen with per-character end-of-turn MP regen
- [x] Align logs and exposed names with `tick_count`
- [x] Add tests for per-turn MP regen behavior
- [x] Update docs to reflect completed engine alignment
- [x] Run test suite
- [x] Commit Phase 5

## Phase 6: Targeting Model Expansion

- [x] Add single-target `ally` and `enemy` selectors
- [x] Add selector conditions for single-target ability resolution
- [x] Add positional enemy conditions: `frontmost`, `backmost`, `same_row`, `same_column`
- [x] Preserve sticky targeting for `current_target`
- [x] Add deterministic tests for new targeting resolution
- [x] Update docs to reflect completed engine alignment
- [x] Run test suite
- [x] Commit Phase 6

## Phase 7: Sample Data Cleanup

- [x] Remove obsolete stat/resource references from sample data
- [x] Update sample characters, abilities, and passives to valid post-alignment shapes
- [x] Final documentation sync
- [x] Run test suite
- [x] Commit Phase 7
