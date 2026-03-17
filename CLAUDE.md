# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Note: `AGENTS.md` is a symlink to this file so the same instructions are shared by both Claude and Codex.

## Project Overview

Tarot Battler is an asynchronous autobattler where players build teams of Tarot characters (22 Major Arcana) within a point budget. Battles run automatically via a rule-based AI system.

Primary design references:
- `design/game_spec.md` — primary gameplay spec and source of truth
- `design/character_design.md` — tactical archetypes, role types, and pivot unit guidance
- `design/implementation_notes.md` — prototype alignment notes, future ideas, and open design items
- `design/team_builder_schema.md` — interim JSON contract for team builder and replay tooling
- `design/replay_schema.md` — proposed replay JSON contract for battle viewer tooling
- `design/ui_spec.md` — lightweight dev-tool UI structure for team editing and replay viewing
- `design/ui_implementation_checklist.md` — phased UI build checklist and execution order

## Build Commands

```bash
cargo build        # compile
cargo run          # run the battle engine
cargo test         # run all tests
cargo test <name>  # run a single test by name
```

All cargo commands should be run from `battle_engine/`.

## Architecture

**Current state:** Early-stage Rust battle engine with a lightweight static UI scaffold in `tools/ui/`. No API or database layers yet.

**Planned stack:** Rust engine → Python/FastAPI API → React/Svelte frontend, with PostgreSQL for persistence.

### UI Dev Tools (`tools/ui/`)

- `index.html` — static shell for Team Builder and Replay Viewer tabs
- `styles.css` — responsive layout and placeholder styling for the dev tool
- `app.js` — lightweight UI bootstrapping, currently tab switching plus replay loading, team loading, validation, structured team editing, metadata rendering, snapshot board rendering, replay state application, playback controls, timeline rendering, and inspector state
- `sample-data/` — placeholder location for replay and team JSON fixtures

### Battle Engine (`battle_engine/`)

- `src/models.rs` — Core data types: `Stat`, `Position`, `CharacterConfig`, `CharacterState`, `StatusTick`, `TraitEffect`, rule/condition types (`Rule`, `Condition`, `ConditionSubject`, `QueryValue`, `Comparator`).
- `src/statuses.rs` — Named status effect system: `StackType` (TickDown, NoStack, Permanent), `StatusBehavior` (DamagePerStack, HealPerStack, StatModPerStack, SkipTurn), `StatusDef`, `StatusInstance`, `StatusMap`. Helper functions `status_key()` and `opposite_key()` for key construction.
- `src/engine.rs` — `BattleState` drives the simulation loop: speed ticking, turn execution, on-turn-start passive hooks, start-of-turn status ticks, rule evaluation → ability or basic attack, stunned-turn skips, end-of-turn MP regen, win conditions, and death-side effect handling. Re-entrancy guard (`in_passive_phase`) prevents passive cascading.
- `src/abilities.rs` — `AbilityDef`, `Primitive` (6 types: `DealPhysicalDamage`, `DealMagicalDamage`, `RestoreHp`, `RestoreMp`, `ApplyStatus`, `RemoveStatus`), `AbilityMap`, `PassiveDef` (tagged enum: `Triggered` or `Trait`), `PassiveMap`, `execute_ability()`, `execute_primitives()`. Targeting supports legacy simple categories plus detailed single-target specs with selectors and positional enemy conditions.
- `src/rules.rs` — `evaluate_rules()` iterates a character's ordered rules, checking conditions and MP cost (reduced by `MpCostReduction` trait, minimum 1). Returns first matching ability or None (basic attack fallback).
- `src/damage.rs` — Physical/magical damage calculation, basic attack type resolution.
- `src/targeting.rs` — Offensive/defensive type computation, front-row target selection with weakness preference.
- `src/loader.rs` — `load_characters()`, `load_abilities()`, `load_passives()`, and `load_statuses()` from JSON, plus content validation for references, positions, target legality, and status-shape correctness.
- `src/logger.rs` — `BattleEvent` enum (`BattleStart`, `BasicAttack`, `AbilityUsed`, `AbilityDamage`, `StatusDamage`, `StatusHeal`, `TurnSkipped`, `PassiveTriggered`, `DamageReflect`, `Defeat`, `BattleEnd`) and `BattleLog` with JSON and human-readable replay formatting grouped by `tick_count`.
- `src/main.rs` — Entry point: loads JSON data (characters, abilities, passives, statuses), validates content, splits teams, runs battle, prints readable text replay by default or JSON with `--json`.
- `src/data/characters.json` — Sample 5v5 roster with themed rules for trial battles.
- `src/data/abilities.json` — Ability definitions including direct attacks, buffs, healing, MP support, cleanse/dispel, and status payoff tools.
- `src/data/passives.json` — Passive definitions: triggered passives and permanent traits used by the sample roster.
- `src/data/statuses.json` — Named status effect definitions (Bleed, Poison, Regen, Empower/Weaken, Fortify/Enfeeble, Stun).

### Key Design Decisions

- **HP = 2 * CON.** Healing caps at this value.
- **Pool stats (CON, DEX, SPI) cannot be modified by status effects.** `add_status()` rejects `StatModPerStack` targeting these. Other current prototype stats are freely moddable.
- **CharacterState is fully encapsulated.** All fields are private; mutation happens through purpose-driven methods (`take_damage`, `heal`, `spend_mp`, `restore_mp`, `tick_speed`, `add_status`, etc.) that enforce invariants like HP/MP caps.
- **Two identity systems:** `base_name` is the archetype (e.g. "The Emperor"), `id` is a numeric runtime identifier assigned at battle setup. Players may later name custom loadouts separately.
- **Effective stats** are computed dynamically: `get_eff_stat()` sums `StatModPerStack` status magnitudes × stacks over the base. `get_base_stat()` returns the unmodified value.

### Key Game Mechanics

- **Formation:** 4-column by 3-row grid with front/middle/back rows. Row-based protection: must clear front before targeting middle, and middle before targeting back. Companions = cardinal-adjacent teammates (set at battle start). Allies = all living teammates.
- **Targeting:** The intended design uses sticky targets for basic attacks and `current_target` abilities, with ability-side targeting kept separate from rule evaluation. See `design/game_spec.md`.
- **Speed system:** The engine uses `max_ticks = 10 - DEX`, clamps `ticks_until_turn` to at least `1`, then adds `+2` to `max_ticks` after each turn before resetting the countdown. This preserves fast openers while softening high-DEX advantage over time.
- **Rule system:** Rule groups are `SelfChar`, `Companion`, `Target`, and `World`. `Companion` means any adjacent ally and does not imply that same companion becomes the ability target. World queries currently support live `ally_count`, `enemy_count`, and step-based `tick_count`.
- **Abilities:** Tier 1 composed from JSON-defined primitives (`DealPhysicalDamage`, `DealMagicalDamage`, `RestoreHp`, `RestoreMp`, `ApplyStatus`, `RemoveStatus`). Target definitions support both simple categories and detailed selector-based targeting, and content validation enforces legal buff/debuff targeting. Tier 2 (custom Rust handlers) planned but not yet implemented.
- **Passives:** Each character has an optional passive ability. Two kinds: **triggered** passives fire on specific game events and execute primitives like abilities; **permanent traits** are applied at battle start and modify engine rules for the duration of the battle. Six triggers: `on_battle_start` (step 0), `on_turn_start` (each turn, even when stunned), `on_deal_damage` (once per action if any damage dealt), `on_take_damage` (from defender's perspective), `on_kill` (for the killer), `on_death` (from dead character's perspective). Re-entrancy guard prevents passive cascading — passives triggered during passive execution only log defeats, no further passives fire. Trait types: `MpCostReduction` (reduces ability MP cost, minimum 1), `DebuffResistance` (negates first N debuffs — DamagePerStack, SkipTurn, or negative StatModPerStack), `DamageReflect` (flat damage back to attackers, can kill). Defined in `passives.json` as a tagged enum (`"type": "triggered"` or `"type": "trait"`).
- **Damage formulas:** Physical: `max(STR - FOR, 1)`, Magical: `max(INT - WIS, 1)`.
- **Named status effects:** Data-driven definitions in `statuses.json`. Each status has a `behavior` (DamagePerStack, HealPerStack, StatModPerStack, SkipTurn), a `stack_type` (TickDown, NoStack, Permanent), and an optional `opposes` field for cancellation. Status keys include the stat for stat-mod statuses (e.g. `"Empower:STR"`), plain name otherwise (e.g. `"Bleed"`).
  - **TickDown:** All stacks fire each turn, then one stack falls off (3 Bleed = 3+2+1 = 6 total damage over 3 turns).
  - **NoStack:** Reapplying replaces stacks only if higher. One stack falls off per tick.
  - **Permanent:** Never decays; only removed by `RemoveStatus`.
  - **Opposing cancellation:** Empower/Weaken and Fortify/Enfeeble cancel each other on the same stat (e.g. applying 5 Weaken:STR against 2 Empower:STR = 3 Weaken:STR).
  - **Batch-resolve ticking:** All damage/heal from statuses is collected, applied as a net HP change, then death is checked. Order of evaluation never matters.
- **Current statuses:** Bleed, Poison, Regen, Empower (opposes Weaken), Weaken (opposes Empower), Fortify (opposes Enfeeble), Enfeeble (opposes Fortify), Stun. These are prototype statuses, not necessarily the final thematic vocabulary.

### Design Principles

- Battle engine is pure logic with JSON I/O and deterministic simulation (seeded RNG).
- Characters and abilities are data-driven (JSON config).
- Rule evaluation is static and composable, not runtime-scripted.
