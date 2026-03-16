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

- `src/models.rs` — Core data types: `Stat`, `Position`, `CharacterConfig`, `CharacterState`, `StatusTick`, rule/condition types (`Rule`, `Condition`, `ConditionSubject`, `QueryValue`, `Comparator`).
- `src/statuses.rs` — Named status effect system: `StackType` (TickDown, NoStack, Permanent), `StatusBehavior` (DamagePerStack, HealPerStack, StatModPerStack, SkipTurn), `StatusDef`, `StatusInstance`, `StatusMap`. Helper functions `status_key()` and `opposite_key()` for key construction.
- `src/engine.rs` — `BattleState` drives the simulation loop: speed ticking, turn execution, rule evaluation → ability or basic attack, status ticking (batch-resolve), incapacitate check, on_battle_start passives, SPI regen, win conditions.
- `src/abilities.rs` — `AbilityDef`, `Primitive` (6 types: `DealPhysicalDamage`, `DealMagicalDamage`, `RestoreHp`, `RestoreSpi`, `ApplyStatus`, `RemoveStatus`), `AbilityMap`, `PassiveDef`, `PassiveMap`, `execute_ability()`, `execute_primitives()`. `ApplyStatus` auto-determines targeting by behavior (damage/negative stat-mod → enemies; heal/positive stat-mod → allies). Targets: `CurrentTarget`, `SelfChar`, `Companions`, `AllEnemies`, `AllAllies`.
- `src/rules.rs` — `evaluate_rules()` iterates a character's ordered rules, checking conditions and SPI cost. Returns first matching ability or None (basic attack fallback).
- `src/damage.rs` — Physical/magical damage calculation, basic attack type resolution.
- `src/targeting.rs` — Offensive/defensive type computation, front-row target selection with weakness preference.
- `src/loader.rs` — `load_characters()`, `load_abilities()`, `load_passives()`, and `load_statuses()` from JSON.
- `src/logger.rs` — `BattleEvent` enum (`BattleStart`, `BasicAttack`, `AbilityUsed`, `AbilityDamage`, `StatusDamage`, `StatusHeal`, `TurnSkipped`, `PassiveTriggered`, `Defeat`, `BattleEnd`) and `BattleLog` with JSON serialization.
- `src/main.rs` — Entry point: loads JSON data (characters, abilities, passives, statuses), splits teams, runs battle, prints log.
- `src/data/characters.json` — Character configs (The Emperor has rules, others basic-attack only). 10 characters with passives assigned.
- `src/data/abilities.json` — Ability definitions (Crush, Embolden).
- `src/data/passives.json` — Passive definitions (Authority, Hope, Inner Fire) using `ApplyStatus` primitives with `on_battle_start` trigger.
- `src/data/statuses.json` — Named status effect definitions (Bleed, Poison, Regen, Empower/Weaken, Fortify/Enfeeble, Stun).

### Key Design Decisions

- **HP = 2 * CON.** Healing caps at this value.
- **Pool stats (CON, DEX, SPI) cannot be modified by status effects.** `add_status()` rejects `StatModPerStack` targeting these. Other stats (STR, INT, FOR, WIS, FOC, RES) are freely moddable.
- **CharacterState is fully encapsulated.** All fields are private; mutation happens through purpose-driven methods (`take_damage`, `heal`, `spend_spi`, `tick_speed`, `add_status`, etc.) that enforce invariants like HP/SPI caps.
- **Two identity systems:** `base_name` is the archetype (e.g. "The Emperor"), `id` is a numeric runtime identifier assigned at battle setup. Players may later name custom loadouts separately.
- **Effective stats** are computed dynamically: `get_eff_stat()` sums `StatModPerStack` status magnitudes × stacks over the base. `get_base_stat()` returns the unmodified value.

### Key Game Mechanics

- **Formation:** 4x4 grid with front/middle/back rows. Row-based protection: must clear front before targeting middle. Companions = cardinal-adjacent teammates (set at battle start). Allies = all living teammates.
- **Targeting:** Offensive type (STR vs INT) and defensive type (FOR vs WIS) determine matchups. Physical attackers prefer magical defenders (weak to physical) and vice versa. Targets selected from frontmost occupied enemy row; random tiebreak.
- **Speed system:** DEX counter starts at DEX, decrements each step; character acts at counter=0. Reset escalates: DEX+2, DEX+4, DEX+6, etc. (`spd_max` tracks this), softening high-DEX dominance over time.
- **Rule system:** Each character has up to 5 ordered rules. A rule fires if all conditions are met (AND) AND current SPI >= ability cost. Falls back to basic attack if no rule matches. Available condition subjects: `SelfChar`, `Target`, `Companion` (adjacent), `Ally` (any teammate). Query values: `Stat`, `Hp`, `Spi`, `UseCount` (total ability uses), `TurnsSinceUse` (actor turns since last use, u32::MAX if never used). Comparators: `Gte`, `Lte`.
- **Abilities:** Tier 1 composed from JSON-defined primitives (`DealPhysicalDamage`, `DealMagicalDamage`, `RestoreHp`, `RestoreSpi`, `ApplyStatus`, `RemoveStatus`). `ApplyStatus` auto-determines targeting by behavior. Tier 2 (custom Rust handlers) planned but not yet implemented.
- **Passives:** Each character has an optional passive ability. Passives fire at battle start (`on_battle_start` trigger) and execute the same primitives as abilities. Defined in `passives.json`.
- **Damage formulas:** Physical: `max(STR - FOR, 1)`, Magical: `max(INT - WIS, 1)`.
- **Named status effects:** Data-driven definitions in `statuses.json`. Each status has a `behavior` (DamagePerStack, HealPerStack, StatModPerStack, SkipTurn), a `stack_type` (TickDown, NoStack, Permanent), and an optional `opposes` field for cancellation. Status keys include the stat for stat-mod statuses (e.g. `"Empower:STR"`), plain name otherwise (e.g. `"Bleed"`).
  - **TickDown:** All stacks fire each turn, then one stack falls off (3 Bleed = 3+2+1 = 6 total damage over 3 turns).
  - **NoStack:** Reapplying replaces stacks only if higher. One stack falls off per tick.
  - **Permanent:** Never decays; only removed by `RemoveStatus`.
  - **Opposing cancellation:** Empower/Weaken and Fortify/Enfeeble cancel each other on the same stat (e.g. applying 5 Weaken:STR against 2 Empower:STR = 3 Weaken:STR).
  - **Batch-resolve ticking:** All damage/heal from statuses is collected, applied as a net HP change, then death is checked. Order of evaluation never matters.
  - **Current statuses:** Bleed, Poison, Regen, Empower (opposes Weaken), Weaken (opposes Empower), Fortify (opposes Enfeeble), Enfeeble (opposes Fortify), Stun.

### Design Principles

- Battle engine is pure logic with JSON I/O and deterministic simulation (seeded RNG).
- Characters and abilities are data-driven (JSON config).
- Rule evaluation is static and composable, not runtime-scripted.
