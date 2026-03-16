# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tarot Battler is an asynchronous autobattler where players build teams of Tarot characters (22 Major Arcana) within a point budget. Battles run automatically via a rule-based AI system. The complete game design spec lives in `notes.md`.

## Build Commands

```bash
cargo build        # compile
cargo run          # run the battle engine
cargo test         # run all tests
cargo test <name>  # run a single test by name
```

## Architecture

**Current state:** Early-stage Rust battle engine. No frontend, API, or database layers yet.

**Planned stack:** Rust engine → Python/FastAPI API → React/Svelte frontend, with PostgreSQL for persistence.

### Battle Engine (`battle_engine/`)

- `src/models.rs` — Core data structures: `Stat` enum (9 stats: CON, STR, INT, FOR, WIS, DEX, SPI, FOC, RES), `Position`, `CharacterConfig`, `CharacterState`. Stats are computed dynamically as base + sum of active effect modifiers.
- `src/main.rs` — Entry point (currently a stub).

### Key Game Mechanics

- **Formation:** 4x4 grid with front/middle/back rows. Row-based protection: must clear front before targeting middle.
- **Speed system:** DEX counter decrements each step; character acts at counter=1, resets to DEX+2.
- **Rule system:** Each character has up to 5 ordered rules. A rule fires if all conditions are met AND current SPI >= ability cost. Falls back to basic attack if no rule matches.
- **Two-tier abilities:** Tier 1 abilities are composed from JSON-defined primitive effects (deal_damage, apply_buff, etc.). Tier 2 abilities point to custom Rust handler functions.
- **Damage formulas:** Physical: `max(STR - FOR, 1)`, Magical: `max(INT - WIS, 1)`.
- **Effects:** Active effects list per character with dynamic stat recomputation. Types: stat_modifier, damage_over_time, heal_over_time, behavior_override.

### Design Principles

- Battle engine is pure logic with JSON I/O and deterministic simulation (seeded RNG).
- Characters and abilities are data-driven (JSON config).
- Rule evaluation is static and composable, not runtime-scripted.
