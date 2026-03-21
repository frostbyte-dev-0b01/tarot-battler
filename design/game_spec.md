# Game Spec

## Purpose

Tarot Battler is an asynchronous autobattler where deep team building is the core loop.

Players construct teams from tarot-inspired characters, choose passives and active abilities, script rule priorities, and predict the wider metagame. Most of the skill expression happens between battles:

- building synergistic teams
- pricing and value judgment
- writing good rules
- predicting common opponent structures
- reviewing replays and adjusting

This file is the primary gameplay spec. It describes the intended game rules, even where the current prototype does not fully match yet.

## Core Pillars

- Team building matters more than APM or manual execution.
- Formation, targeting, and rule scripting should create strategic depth.
- Characters should support tactical archetypes rather than only raw stat efficiency.
- Tarot flavor should live in the roster, names, status vocabulary, and presentation, while the tactical systems remain clear and legible.

## Match Structure

### Daily Loop

- Two rounds per day.
- Each round consists of 5 automated battles against ELO-matched opponents.
- Teams must be locked before a round begins.
- Players return to review readable replays and results, then update their teams before the next round.

### Weekly Loop

- Leaderboard resets weekly.
- Character, ability, and aspect prices adjust based on prior-week popularity and performance.
- Players are rewarded for identifying undervalued options before the market corrects.

## Team Construction

Each character loadout consists of:

- a tarot character template
- locked base stats from that template
- one passive selected from that character's passive pool
- two or three equipped active abilities
- up to five ordered rules
- one aspect slot
- a formation position on a 3-column by 3-row grid

The intended near-term direction is that players do **not** allocate raw base stats directly.

Instead:

- each arcana has a locked base stat profile
- aspects provide the main pre-battle stat augmentation layer
- runtime effective stats are then modified further by battle effects

### Character Stats

The current intended base stat set is:

- `VIT` — maximum HP
- `MGT` — physical offense
- `MAG` — magical offense
- `ARM` — physical defense
- `RES` — magical defense
- `SPD` — speed
- `WIL` — will; determines MP pool size and basic-attack MP recovery

These are the intended v1 stat names and roles.

Provisional starting stat ranges are still being tuned, but the current design expectation is that all core stats should live in a similar midrange band. This keeps each stat comparably valuable and reduces integer breakpoint weirdness.

Conceptually, the likely direction is that each character has a fixed base spread and aspects provide most of the flexible pre-battle stat shaping. Those larger totals are a balance target, not a current engine requirement.

### Derived Resources

- `HP` is expected to scale as a multiple of `VIT`
- the current prototype uses `HP = 3 * VIT` as a working model
- `MP` = spendable battle resource used to cast abilities
- characters begin battle with `MP = WIL`
- characters do not passively regain MP between turns
- every character has a default `Basic Attack` action
- `Basic Attack` restores `floor(WIL / 3)` MP after resolving

`WIL` is the base stat. `MP` is the runtime resource.

## Formation

Battles are played on a 3-column by 3-row formation grid:

- row 0: front
- row 1: middle
- row 2: back

### Row Protection

- enemies in the front row must be fully defeated before the middle row can be targeted
- enemies in the middle row must be fully defeated before the back row can be targeted
- abilities only bypass row protection if their targeting explicitly says so

### Position Terminology

- `companions` are cardinal-adjacent allies
- `allies` are all living teammates

Companion status matters for rules and ability targeting, but the specific companion that caused a rule to trigger does not automatically become the ability target.

## Speed and Turn Order

Each character has a speed counter derived from `SPD`.

- `max_ticks = 15 - SPD`
- at battle start, `ticks_until_turn = max(max_ticks, 1)`
- battle time advances in discrete steps
- each step reduces all living characters' counters by 1
- when a character's counter reaches 0, that character takes a turn
- escalation starts at `0`
- after each turn, increase `max_ticks` by `2`
- after each turn, set `ticks_until_turn = max(max_ticks, 1)`

This preserves the value of high `SPD` while softening the advantage over long fights.

Turn timing effects still happen if a character is stunned. Stun prevents the action itself, not the rest of turn processing.

Design note:

- the intent is that `SPD` should live in the same rough numeric range as the other stats
- the `15 - SPD` opener formula keeps that shared scale workable without making fast characters act too often

### Turn Resolution Order

Each character turn resolves in this order:

1. start-of-turn passive triggers resolve
2. start-of-turn status ticks resolve
3. if the character is stunned, it skips its action; otherwise it evaluates rules and acts
4. death resolution completes for anything killed by that turn
5. end-of-turn effects resolve
6. speed reset and escalation resolve

## Targeting

Targeting is split into two separate systems:

- rules decide whether a character uses an ability
- ability targeting decides who that ability affects

These systems are intentionally independent.

## Focus

Each character maintains a sticky focus for:

- basic attacks
- any ability that targets `current_target`

`focus` is the ongoing attack intent a unit carries between actions. It is distinct from an ability's immediate target selection.

### Initial Focus Selection

At battle start, each character picks an enemy from the frontmost occupied enemy row:

- if `MGT > MAG`, prefer the enemy with the lowest `ARM`
- if `MAG > MGT`, prefer the enemy with the lowest `RES`
- if `MGT == MAG`, choose randomly

If multiple legal targets tie for the same best defensive stat, choose randomly among those tied targets.

### Focus Updates

A character keeps its current focus until:

- that focused enemy is defeated
- an effect explicitly forces the character to select a new focus

When the current focus is lost, a new one is selected using the same rules.

### Refocus Effects

Refocus effects mutate a character's current focus directly.

Design-facing ability text can use two control terms here:

- `refocus` — choose a new focus using normal targeting rules
- `disorient` — choose a new focus using a less favorable targeting rule

The current engine still names these as retargeting modes internally. Design-facing ability text should prefer `focus` / `refocus`.

Current supported retargeting modes are:

- `to_self`
- `to_companion`
- `default_retarget`

These effects are intentionally narrow:

- they change the current focus
- they do not add target locks
- they do not permanently rewrite row-protection rules
- they do not create a broader control subsystem by themselves

This keeps battlefield control effects like taunt and forced refocus legible without overcomplicating the core targeting model.

## Ability Targeting Model

Abilities define their own targets independent of rules.

The recommended model is:

- target side or category
- selector condition
- optional positional condition
- optional row-bypass flag

### Target Categories

- `self`
- `companion`
- `all_companions`
- `ally`
- `all_allies`
- `current_target`
- `current_target_and_companions`
- `enemy`
- `front_row`
- `all_enemies`

### Selector Conditions

Single-target `companion`, `ally`, and `enemy` targeting can use selectors such as:

- highest or lowest of any stat
- highest or lowest current HP
- highest or lowest current MP
- most or fewest stacks of a named effect
- has or lacks a named effect
- random

Enemy targeting should also support positional conditions such as:

- frontmost
- backmost
- same row
- same column

### Row Protection

- any effect targeting enemies respects row protection by default
- ally targeting does not use row protection
- row bypass is an explicit targeting property, not an implicit trait of damage type or status type

## Rule System

Each character has up to 5 ordered rules.

These rules form one shared priority list for the character.

They are not grouped per ability.

Rules answer only one question:

- should this ability be used right now

If the first rule's conditions are met and the character has enough MP to pay the cost, that action is used. Otherwise the next rule is checked. If no rule is satisfied, the character uses `Basic Attack`.

### Rule Condition Groups

Rules can inspect only:

- `self`
- `companion`
- `target`
- `world`

`companion` means any adjacent ally. It does not imply targeting that same companion.

Example:

- `if companion HP < 4, cast Restore`

This triggers if any companion is below 4 HP. The ability still picks its own target using its own targeting rules.

### Rule Properties

`self`, `companion`, and `target` can inspect:

- any effective stat
- current HP
- current MP
- position information such as row for `self`
- companion count for `self` and `target`
- stack count of a named effect using a status key such as `Empower:MGT`
- whether a named effect is present or absent using a key such as `Ward`

`world` can inspect:

- `tick_count`
- `ally_count`
- `enemy_count`

This is expected to include queries such as:

- `self_row`
- `self_companion_count`
- `target_companion_count`

### Rule Operators

- greater than
- less than
- equal to

### Special Rule Conditions

- `always`
- `probability(X%)`

### World State Timing

`tick_count` is the number of world ticks that have elapsed.

`ally_count` and `enemy_count` use live values only after the current death resolution is complete.

Rules do not observe half-resolved action states.

## Combat Actions

### Basic Attack

- every character has a default `Basic Attack` action
- `Basic Attack` is the fallback action when no rule is satisfied
- `Basic Attack` restores `floor(WIL / 3)` MP after resolving

### Basic Attacks

- `Basic Attack` should also be scriptable in the rule system as a normal action choice
- when a character makes an attack, physical attacks use `MGT` against `ARM`
- when a character makes an attack, magical attacks use `MAG` against `RES`

### Active Abilities

Abilities are currently modeled as:

- name
- MP cost
- target definition
- one or more effect primitives

Most abilities should be expressible through data-driven primitives. Unique one-off handlers are acceptable for especially distinct signature abilities.

Each ability should be impactful and notable in its own right. Filler actions that are only "deal ordinary damage with a slightly different multiplier" should be kept rare. The main reason to add abilities is to create team-building depth, role shifts, or tactical interaction.

Ability design should begin from:

- gameplay use
- rule-scripting value
- discovered synergies
- counterplay
- whether the ability creates a visible swing in replay or board state

Names should generally come after the gameplay concept is clear.

### Ability Primitives

The current primitive families are:

- deal physical damage
- deal magical damage
- deal true damage
- restore HP
- restore MP
- apply a status or effect
- remove a status or effect
- modify targeting
- move or reposition units
- transform one state into another

This list can expand as the game's tactical needs become clearer.

## Damage and Defense

Current intended formulas:

- physical damage: `max(MGT * multiplier - ARM, 1)`
- magical damage: `max(MAG * multiplier - RES, 1)`
- omen damage: current stacks as true damage, with no mitigation
- lethality: flat damage added after normal damage resolution, bypassing `ARM` and `RES`

The current design direction is that many damaging abilities should eventually use:

- flat base damage
- plus a stat multiplier

This helps low-multiplier attacks stay meaningful through defense and creates more room to differentiate reliable hits, splash attacks, and payoff abilities.

Current implementation note:

- the live engine now supports optional `flat base + multiplier` damage on physical and magical hit primitives
- only part of the bundled roster uses that model so far; multiplier-only attacks remain fully supported

### Damage Resolution Order

For a normal physical or magical hit:

1. calculate effective `MGT` or `MAG`, including active Fortify or Weaken stacks
2. multiply by the ability multiplier
3. add any flat base damage on the ability
4. subtract `ARM` or `RES`
5. apply the `max(result, 1)` floor
6. add any `Lethality` stacks flat
7. apply the result to target HP

`Omen` resolves separately at start of turn before the character acts.

### Common Ability Multipliers

These are intended balancing anchors, not hard-coded categories:

- sub-`1.0x`: reserved for special low-damage utility attacks such as stun
- weak hit: around `1.0x`
- medium hit: around `1.5x`
- strong hit: around `2.0x`
- AOE per target: around `0.8x` to `1.0x`
- execute: around `2.0x` to `2.5x`, usually conditional

Multipliers apply before defense subtraction. This means offensive Empower and Weaken effects on `MGT` or `MAG` naturally scale ability damage up or down.

## Status and Effect System

Statuses are a core team-building axis, and the intended vocabulary should support archetypes and payoff patterns rather than only generic RPG effects.

### First Balance-Pass Status Set

For the first real balance pass, the live status list should stay tight:

- `Ward` — blocks the next hit and is then consumed
- `Omen` — true-damage setup effect that triggers at start of turn
- `Restoration` — HP restoration over time
- `Empower(stat)` — positive stack-based modifier to `MGT`, `MAG`, `ARM`, or `RES`
- `Weaken(stat)` — negative stack-based modifier to `MGT`, `MAG`, `ARM`, or `RES`

`Ward` should remain a status rather than a condition. It behaves like a scarce defensive layer or buff, not like a special rule-state flag.

`Omen` is the official name for the intended true-damage setup effect.

### Conditions

Conditions are separate from buffs and debuffs.

They should stay relatively limited and should usually represent:

- action denial
- script denial
- target/focus disruption
- relationship changes that are not simple stat shifts

Current intended condition list:

- `Stunned`
- `Marked`
- `Severed`

More conditions can be added later, but the list should stay tight. Conditions should feel qualitatively different from ordinary buffs and debuffs, not just like another place to store small numeric modifiers.

Current intended meanings:

- `Stunned` — the unit cannot take actions while it has `Stunned`
- `Marked` — has no intrinsic effect, but can be consumed or triggered by abilities
- `Severed` — the unit is treated as having no companions for scripted and ability purposes

Current intended condition behavior:

- all current conditions lose `1` stack at end of turn unless they are consumed or removed earlier
- `Stunned` does not stack and is usually applied as `1`
- `Marked` does not stack and does not decay naturally; it remains until consumed or explicitly removed
- `Severed` stacks

Future candidate conditions:

- `Muted` — cannot reuse the same ability as the last turn

Current implementation note:

- the engine now has a separate live condition layer for `Stunned`, `Marked`, and `Severed`
- the live roster now uses both `Marked` and `Severed`
- the older prototype `Stun` status has been removed from the live status catalog

### Status Groups

Buffs and debuffs are grouped for cleanse and dispel behavior.

The current intended groups are:

- `Body`
  - `Empower MGT`
  - `Weaken MGT`
  - `Empower ARM`
  - `Weaken ARM`
- `Mind`
  - `Empower MAG`
  - `Weaken MAG`
  - `Empower RES`
  - `Weaken RES`
- `Fate`
  - `Omen`
  - `Restoration`
  - `Ward`

`Stunned` and other conditions are separate from the `Body`, `Mind`, and `Fate` groups.

Current implementation note:

- the live engine now carries explicit `Body` / `Mind` / `Fate` group metadata for statuses
- stat-mod groups are derived from the affected stat, and explicit group tags are used for effects such as `Omen`
- generic `cleanse` / `dispel` still works broadly by polarity unless a group filter is specified

### Cleanse and Dispel

These terms have specific meanings:

- `cleanse` means remove `1 tick` of debuffs
- `dispel` means remove `1 tick` of buffs

Abilities can scale that amount explicitly:

- `cleanse 2` means remove `2 ticks` of debuffs
- `dispel 2` means remove `2 ticks` of buffs

Default generic removal behavior is intentionally broad but incremental:

- ally-side `cleanse` reduces all debuffs on the affected allies by `1 tick`
- enemy-side `dispel` reduces all buffs on the affected enemies by `1 tick`

This means:

- each matching timed effect loses `1` current stack or tick
- if an effect reaches `0`, it is removed
- permanent effects are not removed unless an ability explicitly says it can remove permanent effects
- conditions such as `Stunned` are not removed unless an ability explicitly says it removes conditions

More specialized abilities can still target a specific group:

- remove `1 tick` of all `Body` debuffs
- remove `1 tick` of all `Mind` buffs
- remove all `Fate` debuffs

### First-Pass Live Status Set

The first balance-pass live status set is:

- `Omen`
- `Restoration`
- `Ward`
- `Empower`
- `Weaken`

`Stunned`, `Marked`, and `Severed` are conditions, not statuses.

### Intended Design Direction

Statuses should:

- fit the tarot theme better
- support tactical archetypes rather than only generic damage over time
- create interactions such as payoff, conversion, cleansing, dispelling, transfer, or detonation

### Stacking and Duration

The intended effect model includes:

- stackable timed effects
- non-stackable effects that refresh or overwrite
- permanent effects that persist until removed
- opposing effects that cancel each other out on application

### Decay Model

For timed stacked effects, the intended default decay behavior is:

- halve stacks at the appropriate resolution point
- round down
- remove the effect when stacks reach `0`

This creates a self-limiting equilibrium when the same effect is applied repeatedly.

Current intended timing:

- start of turn: `Omen` deals damage, then halves
- start of turn: `Restoration` heals, then halves
- end of turn: `Empower`, `Weaken`, `Ward`, and `Omen` lose value according to their own rules, with the main halving-decay family currently being `Omen`, `Restoration`, `Empower`, and `Weaken`
- end of turn: current conditions lose `1` stack unless consumed or removed earlier

The older prototype tick-down-by-1 behavior is an implementation detail, not the intended long-term design.

Current implementation note:

- the live engine now uses the intended halving decay model for `Omen`, `Restoration`, `Empower`, and `Weaken`
- the live status catalog has been trimmed to the first-pass set above

## Compound Ability Resolution

Multi-step abilities resolve in written order.

Default intended behavior:

- an ability selects its targets for each step according to that step's own targeting text
- if a later step depends on the result of an earlier step, that dependency should be explicit in the text
- if an ability is intended to bind one target once and reuse it across all steps, that should be treated as atomic targeting and called out clearly in primitive design

Live implementation note:

- atomic targeting is now a live opt-in behavior through bound-target primitives
- if an ability does not opt into binding, each step still selects its targets when that step resolves

This matters for abilities such as `Rescue`, where:

- move
- heal
- enemy refocus

are conceptually about the same companion, and `Rescue` now binds that companion once for the full sequence.

## Passives and Traits

Each character has one passive selected from a character-specific pool.

Passives should define reactive identity rather than just flat efficiency.

### Triggered Passives

Triggered passives fire on battle events such as:

- battle start
- turn start
- dealing damage
- taking damage
- killing an enemy
- dying
- ally death
- an ally damaging this character's current target

Some passives are dynamic auras rather than one-time triggers. These should update with live formation state so row-based bonuses remain correct when units move or die.

### Permanent Traits

Some passives are permanent traits that alter engine behavior rather than firing a one-time effect list.

Examples:

- MP cost reduction
- debuff resistance
- damage reflection
- row-bypass permission
- target redirection

## Aspects and Build Depth

Build depth should come from two complementary systems:

- character-specific identity through stat allocation and ability thresholds
- flexible external identity through aspects

### Ability Threshold Unlocks

Abilities may gain one secondary unlock when a less-obvious secondary stat crosses a threshold.

Design rules:

- the base ability must be competitive without the unlock
- the threshold stat should not be the obvious primary stat for that role
- the unlock should shift character identity rather than only adding raw efficiency
- thresholds should live in the upper third of the reachable range, so they require real investment

Examples:

- `Breakthrough`: high `VIT` could add self-healing, turning it toward bruiser play
- `Harvest Night`: high `SPD` could refund MP or grant a follow-up effect if it secures a kill, turning it toward assassin play
- `Taunt`: high `MAG` could also Weaken enemy `MAG`, turning it toward a hexblade/control angle
- `Blessing`: high `MGT` could also grant `Fortify MGT`, turning it toward battle support

### Aspects

Each character equips one aspect.

The intended first implementation of aspects should be simpler:

- stat bonuses
- one defining passive or active, usually passive
- a seasonal price

Aspects should act as identity packages rather than generic efficiency bundles.

Each team should be restricted to one copy of each aspect.

First aspect direction:

- `Aspect of Ruin`
  - stats: `MGT +2`, `MAG +2`, `WIL +1`, `VIT -2`, `ARM -1`
  - passive: `Ruinous`
    - The first time each tick the user damages an enemy with a condition, deal `2` true damage.

- `Aspect of Grace`
  - stats: `VIT +2`, `RES +2`, `WIL +1`, `MGT -1`, `MAG -1`
  - passive: `Grace`
    - The first time each tick the user affects an ally with an ability, that ally restores `2 HP`.

Aspects should usually stay around a modest `+/-5` total stat swing, not a massive raw-stat package.

Aspect prices are expected to shift with seasonal popularity and performance, creating budget pressure and meta-based tradeoffs.

### Effective Stats

Rule conditions check effective stats during battle.

Effective stats include:

- base stats
- aspect bonuses
- active Fortify and Weaken stacks

The team builder and inspection tools should make effective stats legible so rule scripting remains understandable.

## Stun and Turn Denial

If a character is stunned when its turn arrives:

- it skips its action
- its speed counter still resets normally
- the normal escalation still applies
- end-of-turn effects still occur normally, including other turn processing

Stun denies actions. It does not freeze the speed system.

## Design Direction for Depth

The core loop should reward:

- formation planning
- role compression
- counter-tech
- status ecosystems
- resource engines
- targeting control
- reactive passives and trigger webs
- matchup-aware rule writing

Mechanics that only add raw damage or raw stats are lower priority than mechanics that change how a team behaves.

## Open Design Areas

These are intentionally not locked in yet:

- final stat names and exact stat tuning
- final MP regeneration rate
- exact aspect pricing and budget rules
- field effects
- reversed character mode
- pricing formula details

These should be tracked in implementation notes and future design docs rather than treated as settled rules.
