# Ability Roster

This file tracks the current intended core active-ability pool.

For now, all abilities are considered globally available to all characters, even though the longer-term design will likely restrict each ability to a smaller subset of characters.

The current target is a core pool of 35 active abilities.

## Balance Framework

This is the current first-pass balancing framework for active abilities.

- `1 MP` should buy a light but meaningful effect.
- `2 MP` should buy a strong standard action.
- `3 MP` should buy a clearly high-impact action, multi-target action, or high-value utility action.
- `4 MP` and above should be reserved for signature payoff actions.

Current first-pass multiplier bands:

- below `1.0x` only for special low-damage utility attacks such as stun
- `1.0x` for weak but meaningful single-target damage
- `1.5x` for medium single-target damage
- `2.0x` for strong payoff damage
- `1.0x` per target is the normal floor for splash abilities; lower scaling should be reserved for rare utility attacks like stun

Current damage-shape principle:

- most damaging abilities should eventually use `flat base damage + stat multiplier`
- this keeps low-multiplier attacks relevant through defense
- it also creates healthier differentiation between:
  - reliable low-scaling attacks
  - pure scaling attacks
  - splash/setup attacks
  - payoff finishers

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

Design note:

- a few benchmark abilities are intentionally simple
- most of the core pool should have a dominant verb and a noticeable board-state, focus-state, or status-state impact
- if an ability reads like bookkeeping with a tiny rider, it is a candidate for replacement
- ability-first thinking should start from gameplay use, scripting value, discovered synergies, and counterplay
- names should usually come after the gameplay concept is clear
- exciting use cases should be phrased literally in game terms, e.g. "put this in the back with passive healers and gain value without losing meaningful HP"

### Formation and Control

- `Hold the Line`
  `MP 5`
  Apply `Ward 1` to allies in the user's row.
  Balance notes: row-wide protection, usually 2-3 total Ward in 3v3, so it should cost more than a one-for-one substitution ability

- `Command`
  `MP 3`
  One companion immediately makes a `1.0x MGT` physical attack against the user's focused enemy.
  Balance notes: this is action substitution, not action gain; its value comes from using the better attacker

- `Taunt`
  `MP 4`
  Enemies with higher `MGT` than `MAG` focus the user.

- `Rebuke`
  `MP 3`
  Deal `2 + 1.0x MAG` damage to the user's focused enemy. That enemy is disoriented.

- `Interpose`
  `MP 2`
  Enemies focusing any of the user's companions focus the user instead.

- `Decoy`
  `MP 2`
  Enemies focusing the user focus one of the user's companions instead.

- `Sever Thread`
  `MP 4`
  Apply `Severed` to the user's focused enemy for `3` turns.

- `Chorus`
  `MP 5`
  Until the user's next turn, when a companion uses an ability, all companions gain a copy of a random buff on that companion.

- `Shatter Faith`
  `MP 5`
  Remove all `Ward` and `Restoration` from the enemy team. For each effect removed, deal `2` true damage to the user's focused enemy.

### Physical Offense and Payoff

- `Charge`
  `MP 4`
  Deal `1.2x MGT` damage to the user's focused enemy. Then, if the tile one row forward in the same column is empty, move there. If the move succeeds, the user refocuses.
  Balance notes: engage tool that should update pressure as part of movement, not just advance position

- `Withdraw`
  `MP 4`
  Deal `1.0x MGT` damage to the user's focused enemy. Then, if the tile one row backward in the same column is empty, move there. If the move succeeds, enemies focusing the user refocus.
  Balance notes: this is intended to be a real disengage tool, not just backward movement

- `Breakthrough`
  `MP 6`
  Deal `1.5x MGT` damage to the user's focused enemy. `Empower MGT` on the user counts double for this attack.
  Balance notes: baseline hit is modest, ceiling comes from setup

- `Condemn`
  `MP 6`
  Deal `1.5x MGT` damage to the user's focused enemy. If that enemy has `Omen`, this attack instead uses `2.0x MGT`.

- `Verdict`
  `MP 8`
  Deal `2.0x MGT` damage to the user's focused enemy.
  Balance notes: clean finisher / heavy commitment button

- `Sunder`
  `MP 2`
  Deal `1.0x MGT` damage to the user's focused enemy. Apply `Weaken ARM 2`.

- `Sever`
  `MP 6`
  Deal `1.3x MGT` damage to the user's focused enemy. Consume that enemy's `Empower MGT` and `Empower ARM`, then deal `1` true damage per consumed stack.
  Balance notes: anti-buff physical payoff that punishes empowered frontliners without needing a separate dispel action

- `Concuss`
  `MP 4`
  Deal `2 + 0.8x MGT` damage to the user's focused enemy. Apply `Stunned 1`.

- `Pursue`
  `MP 4`
  Deal `2 + 1.5x MGT` damage to a `Marked` enemy. Consume `Marked` and focus that enemy.
  Balance notes: gives physical teams a real marked-target pursuit tool instead of relying only on normal sticky focus

- `Hunt the Weak`
  `MP 4`
  Deal `2 + 1.0x MGT` damage to the lowest-HP enemy. If that enemy survives, the user focuses it.

### Magical Offense and Omen Package

- `Smite`
  `MP 5`
  Deal `1.5x MAG` damage to the user's focused enemy.

- `Consecrate`
  `MP 8`
  Deal `2 + 1.5x MAG` damage to the user's focused enemy and `1 + 1.0x MAG` damage to that enemy's companions.
  Balance notes: intended to be a real bomb when enemy formation clusters, not just light splash

- `Hex`
  `MP 4`
  Deal `2 + 1.0x MAG` damage to the user's focused enemy. Apply `Omen 2`.

- `Eclipse`
  `MP 6`
  Deal `1.0x MAG` damage to the user's focused enemy and that enemy's companions. Apply `Omen 1` to each damaged enemy.

- `Harvest Night`
  `MP 6`
  Deal `2 + 1.0x MAG` damage to the user's focused enemy. Then consume all `Omen` on that enemy and deal `2` true damage per stack consumed.

- `Seal`
  `MP 4`
  Deal `2 + 1.0x MAG` damage to the user's focused enemy. Apply `Weaken MAG 4`.

- `Transmute`
  `MP 4`
  Deal `1.0x MAG` damage to the user's focused enemy. Transform that enemy's `Empower MGT` and `Empower ARM` into equal `Weaken MGT` and `Weaken ARM`.

- `Detonate Mark`
  `MP 4`
  Deal `2 + 1.0x MAG` damage to the user's focused enemy. If that enemy has `Marked`, consume `Marked` and deal `6` true damage.
  Balance notes: explicit magical mark payoff that stays meaningful through defense

### Support, Sustain, and Utility

- `Blessing`
  `MP 4`
  Apply `Restoration 3` to the user and the user's companions.
  Balance notes: this is now a real sustain shell enabler rather than a light coordination rider

- `Offer`
  `MP 2`
  Lose `20%` current HP. Gain `MP 4` and `Empower MAG 4`.

- `Distill`
  `MP 4`
  Dispel `1` from the user's focused enemy. Apply `Omen 2`.
  Balance notes: even with no buff present, the Omen still applies; this is the cleaner anti-buff omen setup tool

- `Sanctify`
  `MP 4`
  Cleanse allies in the user's column. Allies cleansed this way gain `Ward 1`.
  Balance notes: column identity plus an immediate visible defensive swing

- `Rescue`
  `MP 4`
  Restore `6 HP` to one companion. If the tile one row backward in the same column is empty, move that companion there. Enemies focusing that companion refocus.
  Balance notes: support healing should often reposition or reset pressure, not only add HP

  Team concept note:
  this kind of ability becomes much more interesting once rules can inspect `self_row`, allowing a tank to begin as a frontline anchor, get rescued backward, and then switch into a different backline damage plan

- `Death Mark`
  `MP 3`
  Apply `Marked 1` to the furthest back enemy.
  Balance notes: baseline marked applier should be scriptable, backline-facing, and not require broader focus rewrites

- `Profane Exchange`
  `MP 5`
  Move all debuffs from the user onto the user's focused enemy.

- `Inheritance`
  `MP 6`
  Choose one companion. The user gains that companion's passive permanently for the rest of battle.

## Brainstorming

These ideas still look promising, but are lower priority than the near-term promotions above.

- `Brand of Ruin`
  Apply a new status: `Marked`.
  When allies attack a `Marked` enemy, remove `Marked` and deal `3` true damage.
  Strong use case:
  mark a target, then use focus tools and coordinated rules to collapse on that enemy.

- `Covenant`
  Until the user's next turn, allies in the user's row cannot be reduced below `1 HP` by direct damage.

- `Blood Rite`
  Deal `5` true damage to each companion. Then deal `1.0x MAG` damage to the user's focused enemy, plus `1` true damage for each damage dealt this way.
  Strong use case:
  put this on a backline caster between passive healers or durable companions so the team converts survivable self-damage into burst.

- `Last Rites`
  Deal `2 + 1.5x MAG` damage to the lowest-HP enemy.

- `Cleanse the Throne`
  Target the enemy with the most buffs. Remove all buffs from that enemy. That enemy is disoriented.

- `Wake the Dead`
  Each defeated companion immediately makes a `1.0x MGT` physical attack against the user's focused enemy.

- `Execution Order`
  Target the lowest-HP enemy. All allies who refocus before your next turn must focus that enemy.

## Cut / Shelved Ideas

These are intentionally not current priorities, but are kept here as reference.

- `Split`
  Too movement-focused for the current design direction.

- `Coronate`
  Awkward short-window scripting and unclear payoff.

- `Sanctuary`
  Interesting, but too rules-heavy and interaction-sensitive for now.

- `Tether`
  Likely wants a dedicated linked-damage system rather than a one-off ability.

- `Silence`
  Depends on `Muted`, which is explicitly shelved for now.

- `Revelation`
  Too meta/tooling-oriented for the current gameplay focus.

- `Invert`
  Interesting, but likely too weird and swingy for the first pass.

- `Anchor`
  Weak design: anti-synergy between `Ward` and raw mitigation, plus poor scripting value.

- `Echo`
  Promising space, but more implementation-heavy than the near-term list.

- `Exile`
  Movement/displacement is not where the current design wants to invest most depth.

- `Mirror`
  Too reactive and narrow compared to stronger state-change abilities.

- `Hush`
  Depends on a `Delayed` condition/system that does not exist yet.

- `Judgment Day`
  Splashy, but too global and hard to tune cleanly for an early pass.

- `Last Stand`
  Serviceable, but less interesting and less identity-rich than the stronger options above.
