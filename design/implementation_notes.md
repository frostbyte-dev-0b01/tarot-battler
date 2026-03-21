# Implementation Notes

## Purpose

This file collects:

- prototype-specific implementation notes
- known mismatches between the current engine and the intended design
- future ideas that are worth keeping visible but are not yet part of the core spec

Use [game_spec.md](/home/frostbyte/Work/tarot-battler/design/game_spec.md) as the source of truth for gameplay rules.

## Current Prototype Notes

### Status Vocabulary

The live status catalog is now trimmed to the intended first-pass set:

- `Omen` as the official true-damage setup effect
- `Restoration` as the main sustain effect
- `Empower` / `Weaken` as the main offensive and defensive stat-mod families
- `Ward` as the scarce defensive layer

`Stunned` is no longer a status concern at all. It is a condition.

The live engine currently uses halving decay for `Omen`, `Restoration`, `Empower`, and `Weaken`.

Current likely future direction:

- move `Omen` back toward tick-down-by-`1` so setup/payoff scripting is more reliable
- keep `Restoration` on halving decay so sustain stacks are self-limiting
- keep `Lethality` in the halving-decay family as a short-lived burst effect if it returns
- make `Empower` / `Weaken` much stickier, potentially permanent until removed, while also making them harder to apply

Legacy placeholder effects such as `Bleed`, `Poison`, `Regen`, `Fortify`, `Enfeeble`, and the older `Stun` status have been removed from the live status catalog. The engine still supports generic status behaviors internally for testing and future experimentation, but those names should not be treated as live gameplay content.

The intended design now also groups timed effects into `Body`, `Mind`, and `Fate`, with generic ally cleanse reducing all debuffs by `1 tick` and generic enemy dispel reducing all buffs by `1 tick`. The engine now carries explicit group metadata and supports optional group-aware `cleanse` / `dispel` targeting, though bundled live abilities still mostly use the broad generic form.

The design now also distinguishes:

- `focus` as the sticky ongoing attack intent
- `target` as the immediate ability target
- `conditions` such as `Stunned`, `Marked`, and `Severed` as a separate layer from buffs and debuffs

The engine now has a first-class condition layer for `Stunned`, `Marked`, and `Severed`, with:

- `Stunned` as non-stacking action denial
- `Marked` as a non-stacking ability hook that persists until consumed or removed
- `Severed` as a stackable "no companions" relationship break

Bundled roster data now uses both `Marked` and `Severed`.

`Muted` is still a future candidate condition, not part of the intended near-term core set.

Compound abilities can now opt into atomic target binding. `Rescue` uses a bound companion target so its move, heal, and enemy-refocus steps stay on the same unit through the full sequence.

The engine now supports optional `flat base + multiplier` damage on physical and magical hit primitives, and the live first-pass roster uses that shape broadly for setup, splash, and payoff attacks.

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

## Open Balance and Tuning Questions

These are real design questions but not yet settled enough to be part of the core spec:

- exact basic-attack MP recovery rate
- final SPD curve and escalation tuning
- exact stat point budgets and adjustment caps
- exact team point budget size
- season-to-season pricing formula
- deeper balance tuning on the new shared stat scale, now that opener timing uses `15 - SPD` and the live damage bands have been moved to the `1.0 / 1.5 / 2.0` framework

## Future Design Ideas

These are intentionally not part of the core game spec yet.

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
- keep permanent and non-stacking effects where useful, but allow different status families to use different decay models rather than forcing one global rule

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
- conditions and state transforms that create strong build-around or counter-tech stories

### UI / Tooling Implications

Because rules use effective stats, the team builder and replay inspection tools should eventually show:

- base stats
- aspect bonuses
- live Fortify and Weaken effects
- resulting effective stat totals

This is especially important once threshold unlocks and aspect-based identity shifts are in the main design.

### Template-Based Character Builds

The engine and Team Builder now use the template-based `version: 2` team schema:

- archetype templates own locked base stats
- team files reference a `template_id`
- aspects should provide the first layer of pre-battle stat augmentation
- the engine resolves authored loadouts into final runtime characters before battle start

The Team Builder now treats stats as derived output rather than direct input:

- template base stats
- aspect bonuses
- final pre-battle totals

The older direct-stat team format has been retired.

## Consolidated Source of Truth

The intended design is now split across:

- [game_spec.md](/home/frostbyte/Work/tarot-battler/design/game_spec.md)
- [character_design.md](/home/frostbyte/Work/tarot-battler/design/character_design.md)
- [implementation_notes.md](/home/frostbyte/Work/tarot-battler/design/implementation_notes.md)
- [team_builder_schema.md](/home/frostbyte/Work/tarot-battler/design/team_builder_schema.md)
- [replay_schema.md](/home/frostbyte/Work/tarot-battler/design/replay_schema.md)
- [ui_spec.md](/home/frostbyte/Work/tarot-battler/design/ui_spec.md)

Older brainstorming has been consolidated into this file and the current design docs.
