# Ability Roster

This file tracks the current intended core active-ability pool.

For now, all abilities are considered globally available to all characters, even though the longer-term design will likely restrict each ability to a smaller subset of characters.

The current target is a core pool of 25 active abilities.

## Balance Framework

This is the current first-pass balancing framework for active abilities.

- `1 MP` should buy a light but meaningful effect.
- `2 MP` should buy a strong standard action.
- `3 MP` should buy a clearly high-impact action, multi-target action, or high-value utility action.
- `4 MP` and above should be reserved for signature payoff actions.

Current first-pass multiplier bands:

- `0.5x-0.7x` per target for splash abilities
- `0.8x-1.0x` for light single-target damage with extra utility
- `1.2x-1.4x` for strong standard single-target damage
- `1.6x-2.0x` for heavy payoff attacks

Current balancing shorthand:

- `1 MP` is roughly worth a light attack, `cleanse 1`, `dispel 1`, `Omen 1`, or `restore 2-3 MP`
- `2 MP` is roughly worth a strong single-target attack, `Omen 2`, a targeted focus-control effect, or a strong row/column utility effect
- `3 MP` is roughly worth splash damage, broad support, or strong conditional payoff
- `4 MP` is for finishers and high-ceiling payoff buttons

Current design direction for economy:

- all characters have `Rest`
- `Rest` restores `floor(WIL / 2)` MP
- `WIL` and MP costs will likely both scale upward later for more granularity

## Notation

- `cleanse 1` means remove `1 tick` of debuffs
- `dispel 1` means remove `1 tick` of buffs
- `focus` means a unit's sticky ongoing attack intent, distinct from an ability's immediate target
- `refocus` means clear the affected unit's current focus and choose a new one using default targeting rules
- `disorient` means clear the affected unit's current focus and choose a new one using a less favorable targeting rule
- `Omen N` means apply `N` Omen stacks
- `Ward 1` blocks the next incoming hit
- `Lethality N` adds `N` true damage after normal damage resolution
- `Empower` / `Weaken` amounts refer to stat-specific stacks

## Core Ability Pool

### Formation and Control

- `Hold the Line`
  `MP 5`
  Apply `Ward 1` to allies in the user's row.
  Balance notes: row-wide protection, usually 2-3 total Ward in 3v3, so it should cost more than a one-for-one substitution ability

- `Command`
  `MP 4`
  One companion immediately makes a standard physical attack against the user's current target with `1.0x MGT` multiplier.
  Balance notes: effectively compresses a second action, so cost stays above a light attack

- `Taunt`
  `MP 4`
  Enemies with higher `MGT` than `MAG` focus the user.

- `Rebuke`
  `MP 2`
  The user's focused enemy is disoriented.

- `Interpose`
  `MP 2`
  Enemies focusing the user's companion change focus to the user.

- `Decoy`
  `MP 2`
  Enemies focusing the user change focus to one of the user's companions.

### Physical Offense and Payoff

- `Charge`
  `MP 4`
  Deal physical damage with `1.2x MGT`, then move one row forward in the same column if the tile is empty. If the move succeeds, the user refocuses.
  Balance notes: engage tool that should update pressure as part of movement, not just advance position

- `Withdraw`
  `MP 4`
  Deal physical damage with `0.8x MGT`, then move one row backward in the same column if the tile is empty. If the move succeeds, enemies focusing the user refocus.
  Balance notes: this is intended to be a real disengage tool, not just backward movement

- `Breakthrough`
  `MP 6`
  Deal physical damage with `1.0x MGT`. `Empower MGT` on the user counts double for this attack.
  Balance notes: baseline hit is modest, ceiling comes from setup

- `Condemn`
  `MP 6`
  Deal physical damage with `1.4x MGT` to the user's current target. If that target has `Omen`, this attack instead uses `1.6x MGT`.

- `Verdict`
  `MP 8`
  Deal heavy physical damage with `1.8x MGT` to the user's current target.
  Balance notes: clean finisher / heavy commitment button

- `Sunder`
  `MP 2`
  Deal physical damage with `0.8x MGT` and apply `Weaken ARM 1`.

- `Sever`
  `MP 6`
  Deal physical damage with `1.4x MGT`. Consume the user's `Empower MGT` and `Empower ARM`, then gain `Lethality 1` per consumed stack for this attack.
  Balance notes: physical self-buff payoff, mirrors `Harvest Night`


### Magical Offense and Omen Package

- `Smite`
  `MP 4`
  Deal magical damage with `1.4x MAG` to the user's current target.

- `Consecrate`
  `MP 6`
  Deal magical damage to the user's current target and that target's companions.
  Uses `0.7x MAG` per target.

- `Hex`
  `MP 4`
  Deal magical damage to the user's current target and apply `Omen 2`.
  Uses `0.8x MAG`.

- `Eclipse`
  `MP 6`
  Deal magical damage to the user's current target and companions, then apply `Omen 1` to each.
  Uses `0.7x MAG` per target.

- `Harvest Night`
  `MP 8`
  Deal magical damage to the user's current target, consume all `Omen` on that target, and add bonus damage equal to the consumed stacks.
  Uses `1.2x MAG`, then gains `Lethality 1` per consumed `Omen`.

- `Invocation`
  `MP 4`
  Deal magical damage. If the target has no `Omen`, apply `Omen 1`; otherwise restore `MP 1` to the user.
  Uses `1.0x MAG`.

- `Transmute`
  `MP 4`
  Deal magical damage to the user's current target. If that target has `Omen`, restore `MP 2` to the user.
  Uses `1.0x MAG`.

### Support, Sustain, and Utility

- `Blessing`
  `MP 4`
  Restore `MP` to allies in the user's row.
  Restores `MP 2` to each ally in the row.

- `Channel`
  `MP 2`
  Restore `MP` to the lowest-`MP` ally.
  Restores `MP 3`.

- `Distill`
  `MP 4`
  Remove one buff from the user's current target. Apply `Omen 2` to that target.
  Balance notes: even with no buff present, the Omen still applies

- `Sanctify`
  `MP 4`
  Cleanse allies in the user's column.
  Applies `cleanse 1` to each ally in the column.

- `Restoration`
  `MP 4`
  Restore HP to one companion.
  Restores `6 HP`.

## Brainstorming

New ability ideas and balance experiments can be added here before they are promoted into the core pool.
