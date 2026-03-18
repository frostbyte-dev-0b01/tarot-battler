# Implementation Notes

## Purpose

This file collects:

- prototype-specific implementation notes
- known mismatches between the current engine and the intended design
- future ideas that are worth keeping visible but are not yet part of the core spec

Use [game_spec.md](/home/frostbyte/Work/tarot-battler/design/game_spec.md) as the source of truth for gameplay rules.

## Current Prototype Notes

### Status Vocabulary

The current prototype still uses familiar RPG effects such as:

- `Bleed`
- `Poison`
- `Regen`
- `Empower`
- `Weaken`
- `Fortify`
- `Enfeeble`
- `Stun`

This is useful for prototyping, but the intended design direction is now:

- `Omen` as the official true-damage setup effect
- `Lethality` as a post-mitigation offensive stack
- `Fortify` / `Weaken` as the main offensive and defensive stat-mod families
- `Regen` and `Stunned` as the main sustain and turn-denial effects

`Bleed` and `Poison` should currently be treated as implementation placeholders so the design docs and engine behavior do not get confused.

### Stat Naming

The current v1 stat set is:

- `VIT`
- `MGT`
- `MAG`
- `ARM`
- `RES`
- `SPD`
- `WIL`

These names are now the intended design direction. Numeric tuning and exact formulas may still evolve.

### MP Terminology

The design now distinguishes:

- `WIL` as the base will stat
- `MP` as the spendable runtime resource

The code and sample data now use this terminology. Remaining references should be treated as cleanup bugs.

## Engine Alignment Tasks

These design decisions are now settled and should be reflected in the engine before further major combat expansion:


## Open Balance and Tuning Questions

These are real design questions but not yet settled enough to be part of the core spec:

- exact MP regeneration rate
- final SPD curve and escalation tuning
- exact stat point budgets and adjustment caps
- exact team point budget size
- season-to-season pricing formula
- the exact long-term numeric stat scale, even though the likely direction is much larger totals than the current prototype

## Future Design Ideas

These are intentionally not part of the core game spec yet.

### Rest as Default Action

Alternative to basic attacks as the fallback action:

- if no rule fires, the character `Rests`
- `Rest` restores `floor(WIL / 2)` MP

Potential upside:

- makes failed rule matches less mindless
- creates natural charge-and-burst pacing
- gives expensive loadouts a clearer identity

Potential downside:

- slows battle pace
- reduces baseline pressure
- requires a larger rebalance of ability costs and action expectations

### Additional or Revised Stats

Possible future additions or revisions:

- reintroducing control-oriented or save-oriented stats
- renaming current stats to better fit the final theme
- merging or simplifying offensive and defensive categories if the stat model feels too dense

### Save System

A save-based layer may still be useful later for control and resistible effects.

This was previously considered with `FOC` and `RES`, but those stats are not part of the current core spec.

If saves return, they should be reintroduced intentionally rather than left as half-supported legacy stats.

### Thematic Status Migration

Potential long-term direction:

- replace generic DOTs and generic buff names with tarot-flavored effect families
- ensure status families create tactical ecosystems, not just passive penalties
- keep permanent and non-stacking effects where useful, but migrate timed stack decay away from tick-down-by-1 toward halving decay

Possible effect families include:

- burden or pressure
- omen or setup
- ward or protection
- blessing or radiance
- momentum or escalation
- lethality or finishing pressure

### Field Effects

Weather-like or battlefield-wide effects remain an open expansion idea.

Questions still open:

- are they tied to abilities
- tied to certain characters
- or randomly assigned per battle

### Reversed Characters

Long-term idea:

- each character can appear in upright or reversed form
- reversed form changes ability and passive access
- upright and reversed kits should not mix freely within one loadout

Not part of v1.

### Ability and Effect Ideas

Useful spaces to explore later:

- heals for self, companion, ally, or all allies
- MP restoration for self, companion, ally, or all allies
- taunts keyed to physical or magical attackers
- cure-all debuff removal
- swap positions with a companion
- consume or trigger status stacks for payoff
- attacks that invert expected stat usage
- block-next-hit effects
- force enemies to retarget
- voluntarily change your own target
- move while attacking
- sweep an entire row
- ignore row protection to hit the lowest HP enemy

### UI / Tooling Implications

Because rules use effective stats, the team builder and replay inspection tools should eventually show:

- base stats
- item bonuses
- live Fortify and Weaken effects
- resulting effective stat totals

This is especially important once threshold unlocks and item-based identity shifts are in the main design.

## Consolidated Source of Truth

The intended design is now split across:

- [game_spec.md](/home/frostbyte/Work/tarot-battler/design/game_spec.md)
- [character_design.md](/home/frostbyte/Work/tarot-battler/design/character_design.md)
- [implementation_notes.md](/home/frostbyte/Work/tarot-battler/design/implementation_notes.md)

Older brainstorming has been consolidated into this file and the current design docs.
