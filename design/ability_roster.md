# Ability Roster

This file tracks the current intended core active-ability pool.

For now, all abilities are considered globally available to all characters, even though the longer-term design will likely restrict each ability to a smaller subset of characters.

The current target is a core pool of 25 active abilities.

## Core Ability Pool

### Formation and Control

- `Hold the Line`
  Apply `Ward 1` to allies in the user's row.

- `Command`
  One companion immediately makes a standard physical attack against the user's current target.

- `Taunt`
  Enemies with stronger `MGT` than `MAG` change their target to the user.

- `Rebuke`
  Force the user's current target to choose a new target using default retargeting.

- `Interpose`
  Enemies targeting the user's companion change their target to the user.

- `Decoy`
  Enemies targeting the user change their target to one of the user's companions.

### Physical Offense and Payoff

- `Charge`
  Deal physical damage, then move one row forward in the same column if the tile is empty.

- `Withdraw`
  Deal physical damage, then move one row backward in the same column if the tile is empty.

- `Breakthrough`
  Deal physical damage. `Empower MGT` on the user counts double for this attack.

- `Condemn`
  Deal physical damage to the user's current target. This attack deals bonus damage if the target has `Omen`.

- `Verdict`
  Deal heavy physical damage to the user's current target.

- `Sunder`
  Deal physical damage and apply `Weaken ARM`.

- `Sever`
  Deal physical damage. Consume the user's `Empower MGT` and `Empower ARM` for bonus damage based on the consumed stacks.

### Magical Offense and Omen Package

- `Smite`
  Deal magical damage to the user's current target.

- `Consecrate`
  Deal magical damage to the user's current target and that target's companions.

- `Hex`
  Deal magical damage to the user's current target and apply `Omen 2`.

- `Eclipse`
  Deal magical damage to the user's current target and companions, then apply `Omen 1` to each.

- `Harvest Night`
  Deal magical damage to the user's current target, consume all `Omen` on that target, and add bonus damage equal to the consumed stacks.

- `Invocation`
  Deal magical damage. If the target has no `Omen`, apply `Omen 1`; otherwise restore `MP 1` to the user.

- `Transmute`
  Deal magical damage to the user's current target. If that target has `Omen`, restore `MP 2` to the user.

### Support, Sustain, and Utility

- `Blessing`
  Restore `MP` to allies in the user's row.

- `Channel`
  Restore `MP` to the lowest-`MP` ally.

- `Distill`
  Remove one buff from the user's current target. Apply `Omen 2` to that target.

- `Sanctify`
  Cleanse allies in the user's column.

- `Restoration`
  Restore HP to one companion.

## Brainstorming

Need to add MP cost to all the abilities. 
