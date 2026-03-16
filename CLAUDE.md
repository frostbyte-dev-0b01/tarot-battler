# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tarot Battler is an asynchronous autobattler where players build teams of Tarot characters (22 Major Arcana) within a point budget. Battles run automatically via a rule-based AI system. The complete game design spec lives in `design/overall_design.md`.

## Build Commands

```bash
cargo build        # compile
cargo run          # run the battle engine
cargo test         # run all tests
cargo test <name>  # run a single test by name
```

All cargo commands should be run from `battle_engine/`.

## Architecture

**Current state:** Early-stage Rust battle engine. No frontend, API, or database layers yet.

**Planned stack:** Rust engine → Python/FastAPI API → React/Svelte frontend, with PostgreSQL for persistence.

### Battle Engine (`battle_engine/`)

- `src/models.rs` — Core data types: `Stat`, `Position`, `CharacterConfig`, `CharacterState`, `Effect`/`EffectType`, rule/condition types (`Rule`, `Condition`, `ConditionSubject`, `QueryValue`, `Comparator`).
- `src/engine.rs` — `BattleState` drives the simulation loop: speed ticking, turn execution, rule evaluation → ability or basic attack, SPI regen, win conditions.
- `src/abilities.rs` — `AbilityDef`, `Primitive` (6 effect types), `AbilityMap`, `execute_ability()`. Damage/debuffs always target enemies; heals/buffs/restores always target allies.
- `src/rules.rs` — `evaluate_rules()` iterates a character's ordered rules, checking conditions and SPI cost. Returns first matching ability or None (basic attack fallback).
- `src/damage.rs` — Physical/magical damage calculation, basic attack type resolution.
- `src/targeting.rs` — Offensive/defensive type computation, front-row target selection with weakness preference.
- `src/loader.rs` — `load_characters()` and `load_abilities()` from JSON.
- `src/logger.rs` — `BattleEvent` enum (`BattleStart`, `BasicAttack`, `AbilityUsed`, `AbilityDamage`, `Defeat`, `BattleEnd`) and `BattleLog` with JSON serialization.
- `src/main.rs` — Entry point: loads JSON data, splits teams, runs battle, prints log.
- `src/data/characters.json` — Character configs (The Emperor has rules, others basic-attack only).
- `src/data/abilities.json` — Ability definitions (Crush, Embolden).

### Key Design Decisions

- **HP = 2 * CON.** Healing caps at this value.
- **Pool stats (CON, DEX, SPI) cannot be modified by effects.** `add_effect()` rejects `StatModifier` targeting these. Other stats (STR, INT, FOR, WIS, FOC, RES) are freely moddable.
- **CharacterState is fully encapsulated.** All fields are private; mutation happens through purpose-driven methods (`take_damage`, `heal`, `spend_spi`, `tick_speed`, `add_effect`, etc.) that enforce invariants like HP/SPI caps.
- **Two identity systems:** `base_name` is the archetype (e.g. "The Emperor"), `id` is a numeric runtime identifier assigned at battle setup. Players may later name custom loadouts separately.
- **Effective stats** are computed dynamically: `get_eff_stat()` sums `StatModifier` effects over the base. `get_base_stat()` returns the unmodified value.

### Key Game Mechanics

- **Formation:** 4x4 grid with front/middle/back rows. Row-based protection: must clear front before targeting middle. Companions = cardinal-adjacent teammates (set at battle start). Allies = all living teammates.
- **Targeting:** Offensive type (STR vs INT) and defensive type (FOR vs WIS) determine matchups. Physical attackers prefer magical defenders (weak to physical) and vice versa. Targets selected from frontmost occupied enemy row; random tiebreak.
- **Speed system:** DEX counter starts at DEX, decrements each step; character acts at counter=0. Reset escalates: DEX+2, DEX+4, DEX+6, etc. (`spd_max` tracks this), softening high-DEX dominance over time.
- **Rule system:** Each character has up to 5 ordered rules. A rule fires if all conditions are met (AND) AND current SPI >= ability cost. Falls back to basic attack if no rule matches. Available condition subjects: `SelfChar`, `Target`, `Companion` (adjacent), `Ally` (any teammate). Query values: `Stat`, `Hp`, `Spi`, `UseCount` (total ability uses), `TurnsSinceUse` (actor turns since last use, u32::MAX if never used). Comparators: `Gte`, `Lte`.
- **Abilities:** Tier 1 composed from JSON-defined primitives (`DealPhysicalDamage`, `DealMagicalDamage`, `RestoreHp`, `RestoreSpi`, `ApplyBuff`, `ApplyDebuff`). Damage/debuffs always target enemies; heals/buffs/restores always target allies. Tier 2 (custom Rust handlers) planned but not yet implemented.
- **Damage formulas:** Physical: `max(STR - FOR, 1)`, Magical: `max(INT - WIS, 1)`.
- **Effects:** Types are `StatModifier`, `DamageOverTime`, `HealOverTime`, `Incapacitate`. Duration 0 means permanent until explicitly removed.

### Design Principles

- Battle engine is pure logic with JSON I/O and deterministic simulation (seeded RNG).
- Characters and abilities are data-driven (JSON config).
- Rule evaluation is static and composable, not runtime-scripted.
