# Ability Targeting Taxonomy

## Goal

The targeting system should help abilities express role and archetype, not just inherit the same target as a unit's basic attack.

`current_target` is still useful, but it should not be the default assumption for most signature abilities. A richer target taxonomy makes formation, team structure, and matchup prediction more important.

The design goal is to add enough selectors to create tactical variety without making targeting unreadable or overly bespoke.

## Current Problem

If most offensive abilities use `current_target`, many abilities collapse into the same tactical shape:

- basic attack but stronger
- basic attack plus status
- basic attack plus self-buff

That reduces the strategic value of unique abilities and makes formation matter mostly through row protection rather than through actual skill design.

## Recommended Target Selector Set

The best next target selectors are:

### Core Enemy Selectors

- `current_target`
  Use when the ability is meant to be a direct extension of normal offensive routing.

- `frontmost_enemy`
  Explicitly pressures the front line. Useful for tank breakers, shield strippers, and challenge-style attacks.

- `backmost_enemy`
  Pressures supports, mages, and fragile utility pieces. Useful for snipers, assassins, and anti-backline tools.

- `lowest_hp_enemy`
  Useful for finishers, execution effects, and snowball units that want to secure kills.

- `highest_hp_enemy`
  Useful for anti-tank or setup abilities that want to mark or poison the enemy anchor.

- `random_enemy`
  Useful for chaos, trickster, or gamble-themed effects.

- `same_column_enemy`
  Strong formation-aware target. Good for duelists, beam attacks, artillery, and line-based pressure.

### Enemy Area / Secondary Selectors

- `all_enemies`
  Broad pressure, debuff application, or low-multiplier AOE.

- `adjacent_enemies_of_target`
  Splash, cleave, or blast effects centered on a chosen target.

- `same_row_enemies`
  Sweeps, line attacks, and anti-cluster tools.

### Core Ally Selectors

- `self`
  Self-buffs, self-heals, and self-resource tools.

- `companions`
  Supports adjacency archetypes and makes formation matter.

- `all_allies`
  Raid-style healing, buffs, or team support.

- `lowest_hp_ally`
  The best default for healer identity. More interesting than generic `all_allies` healing in many cases.

- `lowest_spi_ally`
  Useful for support casters and resource batteries.

- `highest_spi_ally`
  Useful for “feed the carry” support patterns.

- `ally_in_same_column`
  Creates column-based support structures and promotes lane play.

## Priority Order

The first five selectors that would add the most strategic value are:

1. `lowest_hp_enemy`
2. `lowest_hp_ally`
3. `same_column_enemy`
4. `backmost_enemy`
5. `adjacent_enemies_of_target`

These create more tactical variety immediately without requiring a large rewrite of the combat model.

## Design Heuristics

### Keep `current_target` for:

- straightforward martial strikes
- efficient core attacks
- abilities meant to reinforce a unit's existing attack routing

### Prefer custom selectors for:

- signature character abilities
- assassins and backline hunters
- healers and rescue tools
- lane, adjacency, or formation specialists
- control and setup abilities

## Tarot-Flavored Examples

- **The Emperor**
  Often fine with `current_target` or `frontmost_enemy`
  Ruler / battlefield commander identity fits direct, structured pressure

- **The Tower**
  Good fit for `all_enemies` or `adjacent_enemies_of_target`
  Collapse, shockwave, and battlefield disruption themes

- **The Chariot**
  Good fit for `frontmost_enemy` or `same_column_enemy`
  Direct, aggressive, lane-breaking momentum

- **The High Priestess**
  Good fit for `lowest_hp_ally`, `ally_in_same_column`, or `backmost_enemy`
  Protective or precise mystical intervention

- **The Hermit**
  Good fit for `backmost_enemy` or `same_column_enemy`
  Precision, isolation, and deliberate selection

- **The Fool**
  Good fit for `random_enemy` or flexible conditional selectors
  Chaos, risk, improvisation

- **The Magician**
  Good fit for `same_column_enemy`, `highest_spi_ally`, or `lowest_spi_ally`
  Resource shaping and controlled redirection

- **Strength**
  Good fit for `frontmost_enemy`, `adjacent_enemies_of_target`, or `companions`
  Protective ferocity and frontal domination

- **The Moon**
  Good fit for `lowest_hp_enemy`, `random_enemy`, or `backmost_enemy`
  Elusive pressure and deceptive target selection

- **The Star**
  Good fit for `lowest_hp_ally` and `all_allies`
  Hope, rescue, and restoration

## Structural Recommendation

Abilities should be thought of as:

- **target selector**
- **effect payload**

Example:

- selector: `lowest_hp_enemy`
- payload: `deal_magical_damage`, `apply_poison`

This scales better than treating every new tactical targeting pattern as a unique hardcoded ability.

## Practical Recommendation

When adding new abilities:

- use `current_target` for generic attacks
- use custom selectors for defining character moves
- tie selectors to tarot identity and formation play

The best version of the system is one where targeting itself is part of team building and matchup prediction, not just a hidden implementation detail.
