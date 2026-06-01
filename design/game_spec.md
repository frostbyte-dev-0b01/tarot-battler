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
- Depth should come from synergies and trigger webs, not spreadsheet math. Power is expressed as a small set of named tiers rather than fine-grained multipliers, so players optimize interactions, timing, and counterplay instead of tuning decimals.
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
- up to seven ordered rules
- one aspect slot
- a formation position on a 3-column by 3-row grid

### Team Budget

Teams are built under a point budget so composition is a real value decision rather than "take the strongest of everything." Consistent with the synergy-over-spreadsheet pillar, costs are coarse tiers, not fine-grained prices:

- Only **characters and aspects** cost points. Abilities and passives are free within a character's pool.
- Character cost is a coarse tier (currently `1`–`3`); aspect cost is `0`–`2`.
- A team's total cost (sum of archetype costs plus aspect costs) must not exceed the **team budget** (currently `14`; a primary tuning knob).
- **Singletons:** one copy of each archetype and one copy of each aspect per team.
- Team size is variable up to the slot maximum, so the budget lets a team trade quantity for quality.

The budget is enforced in both the engine (`validate_team_config`) and the team builder, which shows a live budget meter. Seasonal/dynamic repricing is a later layer on top of these static costs.

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

These are the intended v1 stat names and roles. There is no mana stat: mana is a
universal "pip" resource (see Derived Resources), not a per-character attribute.

Provisional starting stat ranges are still being tuned, but the current design expectation is that all core stats should live in a similar midrange band. This keeps each stat comparably valuable and reduces integer breakpoint weirdness.

Conceptually, the likely direction is that each character has a fixed base spread and aspects provide most of the flexible pre-battle stat shaping. Those larger totals are a balance target, not a current engine requirement.

### Derived Resources

- `HP` is expected to scale as a multiple of `VIT`
- the current prototype uses `HP = 3 * VIT` as a working model
- `MP` = mana, a spendable battle resource used to cast abilities, tracked as discrete "pips"
- mana is universal: every character starts a battle at `0` MP and caps at `MAX_MP = 5`
- characters charge mana up by attacking, so abilities are an earned, occasional spend rather than a turn-one option; basic attacks are the default rhythm and abilities are rarer and more impactful
- characters do not passively regain MP between turns
- every character has a default `Basic Attack` action
- `Basic Attack` restores `1` mana pip after resolving (capped at `MAX_MP`)
- abilities generally cost `1`–`4` mana

`MP` is a universal runtime resource (no backing stat). Per-character mana
variation, if introduced, should come from passives/traits rather than a stat.

## Formation

Battles are played on a 3×3 formation grid. The board is drawn with each team's
depth running toward the enemy, so the three depth positions read as **columns**:

- front column (closest to the enemy)
- middle column
- back column

The three lateral positions are **lanes**. (For historical reasons the engine/JSON
stores the depth axis as the `row` field and the lateral axis as `col`; the UI and
ability text present depth as "column".)

### Column Protection

- enemies in the front column must be fully defeated before the middle column can be targeted
- enemies in the middle column must be fully defeated before the back column can be targeted
- abilities only bypass column protection if their targeting explicitly says so

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

When several characters become ready on the **same step**, the order in which
they act is randomized by the seeded battle RNG (rather than a fixed
team-then-index order). This removes the structural first-mover advantage and
gives close matchups outcome spread across seeds, while the battle stays fully
deterministic for any given seed (see Randomness and Determinism).

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

### Focus Policy (planned)

Focus is currently chosen by a fixed weakness heuristic. The planned addition
lets each unit carry **one focus policy** — a *selection rule* for who to
commit to among **legal (reachable) targets** when it needs a focus. It never
bypasses row protection; it only prioritizes among enemies it could already hit.

Focus stays **sticky**: a policy is applied only at *acquisition* (battle start,
the current target dying, or a forced refocus), never re-evaluated every turn.
Stickiness is what makes targeting-manipulation tools (`Taunt`, `Rebuke`,
`disorient`, `Command`, movement) lasting, meaningful actions rather than nudges
the autopilot overwrites. Dynamic "chase the lowest / snap to the setup target"
behavior is intentionally an **active ability** (a refocus, see below), not a
passive policy.

Planned policy menu (grouped, with `Weakness: Auto` as the zero-config default):

- **Weakness** — `Auto` / `ARM` / `RES`: the enemy weakest to my [inferred /
  physical / magical] damage. The explicit ARM/RES variants let a player declare
  intent for hybrids or units buffed into a new lane mid-fight.
- **Execute** — lowest current HP (kill-chains: re-picks the next-lowest on a kill).
- **Setup** — `Omen` / `Marked`: the target with the most stacks of that effect
  (feeds the different payoff engines).
- **Isolate** — an enemy with no companions (pairs with `Sever` / The Hermit).
- **Focus-fire** — follow the **Captain's** current focus (the team converges).

Rules:

- **Universal fallback → `Weakness: Auto`** whenever a policy has no valid pick
  (no setup target yet, no isolated enemy, captain dead). One rule, no special
  cases. A captain set to Focus-fire is also treated as `Weakness: Auto`.
- Precedence: **forced focus > policy > fallback**. When a forced refocus ends,
  the unit **picks fresh** via its policy (no remembered target).
- Ties resolve deterministically (no random tiebreak).

### Refocus and Focus-as-Intent (planned)

Refocus abilities reuse the **Ability Targeting Model** below (target category +
selector + positional condition + optional row-bypass) to set a unit's focus —
e.g. *"focus the backmost enemy with the lowest HP, then deal magical damage."*
No separate refocus vocabulary.

Because a refocus can (with row-bypass) point focus past the front, **focus is
treated as intent**: an ordinary hit lands on the focus if it's legal, otherwise
on the frontmost reachable enemy, and the focus persists either way. Refocus
respects row protection unless the ability explicitly bypasses it.

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

Each character has up to 7 ordered rules.

These rules form one shared priority list for the character.

They are not grouped per ability.

Rules answer only one question:

- should this ability be used right now

If the first rule's conditions are met and the character has enough MP to pay the cost, that action is used. Otherwise the next rule is checked. If no rule is satisfied, the character uses `Basic Attack`.

### Logic Structure

The condition logic is deliberately flat — there are no nested boolean groups.

- Within a single rule, conditions combine with **AND** by default.
- The ordered list of rules provides an implicit **OR** across priorities.
- A per-rule `match_any` flag flips one rule to **OR** (fire if any condition
  holds) for the occasional case where a flat AND-list is not enough. This
  buys cheap OR without the cognitive cost of nesting.

The intent is a policy language that is shallow to read yet deep to play:
tactical depth comes from observable state and ordering, not boolean trees.

### Rule Condition Subjects

Rules can inspect these subjects:

- `self`
- `companion` — any **fixed** companion
- `any_ally` — any living ally (the whole team)
- `lowest_ally` — the living ally with the lowest current HP
- `target` — the current target
- `any_enemy` — any living enemy
- `lowest_enemy` — the living enemy with the lowest current HP
- `world`

`companion` is a fixed bond: companions are the allies cardinally adjacent at
battle start, and the bond persists even if units move. It does not imply
targeting that same companion. The `any_*` and `lowest_*` scopes are live —
they re-scan the relevant team each turn — and are intentionally distinct from
the fixed companion bond.

Example:

- `if any_ally HP < 30%, cast Restore`

This triggers if any living ally is below 30% HP. The ability still picks its
own target using its own targeting rules.

### Rule Properties

Character subjects (`self`, `companion`, `any_ally`, `lowest_ally`, `target`,
`any_enemy`, `lowest_enemy`) can inspect:

- any effective stat
- current HP, expressed as a **percentage of max HP** (`0`–`100`) so thresholds
  port across different stat lines
- current MP
- position information such as row for `self`
- `focused_by_count` — how many living enemies are currently focusing the
  subject (a threat/aggro signal that supports blind play)
- companion count for `self` and `target`
- stack count of a named effect using a status key such as `Empower:MGT`
- whether a named effect is present or absent using a key such as `Ward`

`world` can inspect:

- `tick_count`
- `ally_count`
- `enemy_count`

Position and bond queries include:

- `self_row`
- `self_companion_count`
- `target_companion_count`

### Rule Operators

- greater than or equal to
- less than or equal to
- equal to

### Special Rule Conditions

- `always` (implemented: a rule with no conditions)
- `probability(X%)` (**planned**): the rule fires only if a seeded roll passes
  the given chance — a *visible, scriptable* source of variance the player can
  build around (e.g. "30% chance to Taunt"). Implementing it threads the seeded
  battle RNG into rule evaluation; the roll stays deterministic per seed, so
  replays remain reproducible. A small number of kits should use it so variance
  is designed and legible rather than hidden in damage math.

### World State Timing

`tick_count` is the number of world ticks that have elapsed.

`ally_count` and `enemy_count` use live values only after the current death resolution is complete.

Rules do not observe half-resolved action states.

## Combat Actions

### Basic Attack

- every character has a default `Basic Attack` action
- `Basic Attack` is the fallback action when no rule is satisfied
- `Basic Attack` restores `1` mana pip after resolving (capped at `MAX_MP`)

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

### Damage Tiers

Attack power is expressed as one of four named tiers, never a free-form multiplier. Players see a tier (shown as pips), not a decimal:

- `Light` (⚔) — `×1.0`: default single-target, AOE, and utility hits where the rider is the point
- `Medium` (⚔⚔) — `×1.5`: a committed single-target attack
- `Heavy` (⚔⚔⚔) — `×2.0`: a high-MP power hit
- `Ultimate` (⚔⚔⚔⚔) — `×2.5`: a premium or conditional finisher

In the UI these tiers render as clustered damage symbols — swords for physical,
sparks for magical — derived from the ability's primitives (so the symbol count
always matches the real tier). Ability descriptions carry only the rider text.

The underlying multipliers exist only in data; the design language is the tier. Conditions and setups should express their payoff as bumping an attack up a tier — for example, "if the target has `Omen`, `Condemn` hits one tier harder" — rather than swapping raw multipliers. This keeps power legible and pushes design toward interactions instead of decimal tuning.

### Damage Formula

Physical and magical hits use ratio-based mitigation:

- `raw = effective attack stat × tier multiplier` (`MGT` for physical, `MAG` for magical)
- `damage = round(raw * K / (K + defense))`, with a minimum of `1`
- `K` is a global constant, currently `12` (≈ the midrange stat), so a defense equal to the midrange roughly halves incoming damage

Defense gives smooth diminishing returns: it always matters and never fully negates a hit, and there is no flat armor-subtraction cliff. Attacks carry **no flat base damage**; the `min(1)` floor is a safety net, not a balancing knob.

- physical hits use `MGT` against `ARM`
- magical hits use `MAG` against `RES`
- `Omen` resolves separately at start of turn as true damage, bypassing mitigation
- true damage bypasses mitigation entirely
- `Lethality`, if it returns, adds flat damage after mitigation

Offensive `Empower` / `Weaken` modify the attack stat before the tier multiplier, so they scale damage cleanly. Defensive `Empower` / `Weaken` on `ARM` / `RES` feed the mitigation term.

### Damage Resolution Order

For a normal physical or magical hit:

1. calculate effective `MGT` or `MAG`, including active `Empower` or `Weaken` stacks
2. multiply by the tier multiplier to get `raw`
3. apply ratio mitigation: `raw * K / (K + effective ARM or RES)`
4. round and apply the `max(result, 1)` floor
5. add any `Lethality` stacks flat (bypassing mitigation)
6. apply the result to target HP

`Omen` resolves separately at start of turn before the character acts.

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

Decay varies by status family:

- start of turn: `Omen` deals damage, then loses `1` stack (tick-down), so setup/payoff scripting is reliable
- start of turn: `Restoration` heals, then halves — kept self-limiting on purpose, so sustain stacks are hard to snowball and it never becomes a payoff engine
- `Empower` and `Weaken` are **permanent**: they do not decay and are removed only by dispel, cleanse, opposing-effect cancellation, or consume effects such as `Sever`, `Transmute`, and `Cleanse the Throne`
- end of turn: current conditions lose `1` stack unless consumed or removed earlier
- `Lethality`, if it returns to the live roster, uses halving decay as a short-lived burst window

`Empower` and `Weaken` stacks are **capped per stat** (currently `8`) so permanence rewards setup without becoming uncatchable. Because they are sticky, application amounts are a primary balance knob and should stay small.

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

## Captain and Banners (planned)

Each team designates **one captain**. Designation is free (no budget cost); the
captain is a single, legible strategic decision that gives the team a spine, a
small effect, and — via the **Focus-fire** policy — a rally point. It is also
the counterplay hook: killing the captain removes its persistent banner and the
focus-fire director.

### Banners

When captained, a card flies a **banner** chosen from **5 options**: its own
**signature banner** (a property of the card, distinct from its passive pool)
plus **4 shared banners** available to any captain. Banner space spans two axes:

- **scope:** self / companions / team
- **duration:** opener (fires once at battle start) / persistent (while the
  captain lives) / triggered (on an event)

Banner duration should track the captain's survivability, which makes "who is my
captain" a real risk read: fragile captains favor **opener** banners (snipe-proof
— already fired) or **self-sustain** (snipe-resistant); durable captains carry
**persistent team auras** worth both protecting and sniping.

The four shared banners span the grid so every captain has range:

- **Rally** (opener / team) — the team's first turn comes sooner
- **Bulwark** (persistent / team) — allies in the captain's column take less from the first hit each turn
- **Resolve** (persistent / self) — the captain regains a little HP each turn
- **Last Stand** (triggered / team) — the first time the captain drops below 50%, the team gains `Ward 1`

### Captain and focus

Under the **Focus-fire** policy, allies copy the captain's current focus at
acquisition, so the captain's *own* focus policy becomes the team's targeting
doctrine. A persistent team banner that rewards attacking the captain's focus
(e.g. the Emperor's) plus allies on Focus-fire forms a self-reinforcing
focus-fire engine. If the captain dies, focus-fire units keep their current
target (sticky) and fall back to `Weakness: Auto` on their next acquisition, like
every other policy.

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
  - stats: `MGT +2`, `MAG +2`, `SPD +1`, `VIT -2`, `ARM -1`
  - passive: `Ruinous`
    - When the user damages an enemy that has a condition, deal `2` true damage.

- `Aspect of Grace`
  - stats: `VIT +2`, `RES +2`, `SPD +1`, `MGT -1`, `MAG -1`
  - passive: `Grace`
    - When the user affects an ally with an ability, that ally restores `2 HP`.

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

## Randomness and Determinism

Determinism and variance are treated as **orthogonal**:

- **Per-seed determinism = reproducibility.** A battle is fully determined by
  `(team A, team B, seed)`. This is non-negotiable: replays, debugging, and the
  gauntlet runner all depend on it. All randomness draws from the single seeded
  battle RNG — never wall-clock or unseeded sources.
- **Cross-seed variance = the outcome distribution.** A tuning dial. Skill
  (team, rules, formation, captain) sets the *mean* of a matchup's win-rate
  distribution; variance sets the *spread*. Skill should dominate the mean;
  variance should mainly decide *close* matchups — "fair noise" that flips
  coin-flip games without overriding a real advantage. The daily loop's **5
  battles per round** then averages per-battle variance into an expected score,
  so a round is not a single dice roll.

Calibration targets (win rate for the favored side):

- clear advantage: ~85–97% (rare upsets), never a guaranteed 100%
- favored ~65–75% · even ~50% · counter/underdog ~5–20% (never an auto-0%)

Sources of variance, by preference:

1. **Turn order among simultaneously-ready units** (implemented) — seed-shuffled
   each step. Preserves the coarse-tier damage model, doesn't override authored
   decisions, and doubles as a fairness fix (removes the first-mover bias).
2. **`probability(X%)` rule conditions and proc passives** (planned) — variance
   that is visible and scriptable, so players build around it.
3. **Global damage rolls** are intentionally avoided: they fight the
   coarse-tier / no-decimals pillar and make breakpoints opaque. Reach for a
   small flat jitter only if turn-order variance proves insufficient.

Things that stay deterministic regardless: the MP economy, status stacks and
durations, the speed formula, damage tiers, and anything the player explicitly
authored (team, rules, formation). The Arena's win-rate + confidence interval
display is the calibration instrument for tuning variance magnitude.

## Open Design Areas

These are intentionally not locked in yet:

- final stat names and exact stat tuning
- final MP regeneration rate
- exact aspect pricing and budget rules
- field effects
- reversed character mode
- pricing formula details

These should be tracked in implementation notes and future design docs rather than treated as settled rules.
