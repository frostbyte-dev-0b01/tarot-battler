# Roster Implementation Plan

This plan turns the current roster/spec docs into live engine and UI content in staged commits.

## Goals

- align the engine with the current 8-character, 35-ability roster direction
- replace `item` terminology and data flow with `aspect`
- implement the first two aspects
- implement the new passives and supporting mechanics
- add the promoted and newly-added core abilities
- keep the repo valid after each stage

## Stage 1: Foundations

- update condition behavior so:
  - `Stunned` remains non-stacking and decays at end of turn
  - `Marked` becomes non-stacking and does not decay naturally
  - `Severed` remains stackable and decays at end of turn
- add `Haste` support as an immediate timing effect
- replace engine/catalog terminology from `item` to `aspect`
- add `aspects.json`
- update loader/runtime resolution to apply aspect stat bonuses and optional granted passive/active content

## Stage 2: Character and Passive Data

- update existing archetype stats to match the new direction
- add `The Hermit`
- add `The Fool`
- update passive data for:
  - `Imperial Formation`
  - `Sanctuary`
  - `Pursuit`
  - `Sentence`
  - `Foreboding`
  - `Catalyst`
  - `Lonely Road`
  - `Chance Opening`
- add any passive runtime support required by:
  - target-change triggers
  - below-50%-HP triggers
  - no-companion target checks
  - per-tick trigger limits

## Stage 3: Core Ability Expansion

- add the missing core abilities:
  - `Concuss`
  - `Pursue`
  - `Detonate Mark`
  - `Death Mark`
  - `Sever Thread`
  - `Chorus`
  - `Profane Exchange`
  - `Shatter Faith`
  - `Hunt the Weak`
  - `Inheritance`
- add any missing targeting selectors or primitives needed by those abilities
- update archetype active pools to curated 5-ability sets

## Stage 4: UI and Schema Alignment

- rename Team Builder/UI references from `item` to `aspect`
- load the aspect catalog in the browser UI
- expose aspect selection in the builder
- ensure derived stat display reflects aspect bonuses
- keep the rule builder and replay viewer working with the updated roster

## Stage 5: Content Refresh and Verification

- refresh sample teams for the new roster
- remove dead teams/builds that no longer validate
- rerun battle royale
- regenerate latest replay
- run:
  - `cargo test`
  - `cargo run -- --teams ...`
  - `python3 scripts/battle_royale.py`
  - `node --check tools/ui/app.js`
