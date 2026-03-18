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
- Character, ability, and item prices adjust based on prior-week popularity and performance.
- Players are rewarded for identifying undervalued options before the market corrects.

## Team Construction

Each character loadout consists of:

- a tarot character
- a base stat spread
- a small number of stat adjustment points
- one passive selected from that character's passive pool
- two or three equipped active abilities
- up to five ordered rules
- one item slot
- a formation position on a 4-column by 3-row grid

### Character Stats

The current intended base stat set is:

- `CON` — maximum HP
- `STR` — physical offense
- `INT` — magical offense
- `FOR` — physical defense
- `WIS` — magical defense
- `DEX` — speed
- `SPI` — spirit stat; determines MP pool size and MP regeneration

Stat names and exact effects are still somewhat provisional, but this is the intended v1 structure.

### Derived Resources

- `HP` = `2 * CON`
- `MP` = spendable battle resource used to cast abilities
- characters begin battle with `MP = SPI`
- characters regenerate MP at the end of their own turns
- current intended regeneration rate is `floor(SPI / 2)` as a placeholder rate, though this will likely change during balance tuning

`SPI` is the base stat. `MP` is the runtime resource.

## Formation

Battles are played on a 4-column by 3-row formation grid:

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

Each character has a speed counter derived from `DEX`.

- `max_ticks = 10 - DEX`
- at battle start, `ticks_until_turn = max(max_ticks, 1)`
- battle time advances in discrete steps
- each step reduces all living characters' counters by 1
- when a character's counter reaches 0, that character takes a turn
- escalation starts at `0`
- after each turn, increase `max_ticks` by `2`
- after each turn, set `ticks_until_turn = max(max_ticks, 1)`

This preserves the value of high `DEX` while softening the advantage over long fights.

Turn timing effects still happen if a character is stunned. Stun prevents the action itself, not the rest of turn processing.

### Turn Resolution Order

Each character turn resolves in this order:

1. start-of-turn passive triggers resolve
2. start-of-turn status ticks resolve
3. if the character is stunned, it skips its action; otherwise it evaluates rules and acts
4. death resolution completes for anything killed by that turn
5. end-of-turn effects resolve
6. MP regeneration resolves
7. speed reset and escalation resolve

## Targeting

Targeting is split into two separate systems:

- rules decide whether a character uses an ability
- ability targeting decides who that ability affects

These systems are intentionally independent.

## Sticky Targeting

Each character maintains a sticky target for:

- basic attacks
- any ability that targets `current_target`

### Initial Sticky Target Selection

At battle start, each character picks an enemy from the frontmost occupied enemy row:

- if `STR > INT`, prefer the enemy with the lowest `FOR`
- if `INT > STR`, prefer the enemy with the lowest `WIS`
- if `STR == INT`, choose randomly

If multiple legal targets tie for the same best defensive stat, choose randomly among those tied targets.

### Sticky Target Updates

A character keeps its sticky target until:

- that target is defeated
- an effect explicitly forces the character to select a new target

When the sticky target is lost, a new one is selected using the same rules.

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

Rules answer only one question:

- should this ability be used right now

If the first rule's conditions are met and the character has enough MP to pay the cost, that ability is used. Otherwise the next rule is checked. If no rule is satisfied, the character uses a basic attack against its sticky target.

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
- stack count of a named effect using a status key such as `Empower:STR`
- whether a named effect is present or absent using a key such as `Ward`

`world` can inspect:

- `tick_count`
- `ally_count`
- `enemy_count`

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

### Basic Attacks

- basic attacks use the actor's sticky target
- physical basic attacks use `STR` against `FOR`
- magical basic attacks use `INT` against `WIS`
- fallback action is currently a basic attack

The idea of replacing basic attacks with a `Rest` action remains a future design option, not part of the current spec.

### Active Abilities

Abilities are currently modeled as:

- name
- MP cost
- target definition
- one or more effect primitives

Most abilities should be expressible through data-driven primitives. Unique one-off handlers are acceptable for especially distinct signature abilities.

### Ability Primitives

The current primitive families are:

- deal physical damage
- deal magical damage
- restore HP
- restore MP
- apply a status or effect
- remove a status or effect
- modify targeting

This list can expand as the game's tactical needs become clearer.

## Damage and Defense

Current intended baseline formulas:

- physical damage: `max(STR - FOR, 1)`
- magical damage: `max(INT - WIS, 1)`

Abilities can scale these formulas up, down, or transform them with custom logic.

## Status and Effect System

Statuses are a core team-building axis, but the final status vocabulary should be more tarot-specific than the current prototype.

### Current Prototype Direction

The current implementation uses familiar effects such as:

- `Bleed`
- `Poison`
- `Regen`
- stat buffs and debuffs
- `Stun`

This is acceptable for prototyping, but not assumed to be the final thematic vocabulary.

### Intended Design Direction

Statuses should:

- fit the tarot theme better
- support tactical archetypes rather than only generic damage over time
- create interactions such as payoff, conversion, cleansing, dispelling, transfer, or detonation

Good long-term status families will likely include:

- pressure or burden effects
- omen or setup effects
- ward or protection effects
- blessing or radiance effects
- escalation or momentum effects

### Stacking and Duration

The current intended effect model includes:

- stackable timed effects
- non-stackable effects that refresh or overwrite
- permanent effects that persist until removed
- opposing effects that cancel each other out

Exact final status vocabulary is still open, but the engine should support these interaction patterns.

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

### Permanent Traits

Some passives are permanent traits that alter engine behavior rather than firing a one-time effect list.

Examples:

- MP cost reduction
- debuff resistance
- damage reflection
- row-bypass permission
- target redirection

## Stun and Turn Denial

If a character is stunned when its turn arrives:

- it skips its action
- its speed counter still resets normally
- the normal escalation still applies
- end-of-turn effects still occur normally, including MP regeneration and other turn processing

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
- final status vocabulary
- exact item system
- field effects
- reversed character mode
- pricing formula details

These should be tracked in implementation notes and future design docs rather than treated as settled rules.
