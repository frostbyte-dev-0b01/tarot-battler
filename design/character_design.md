# Character Design

## Purpose

This file captures the intended character design framework for Tarot Battler:

- tactical archetypes
- role types
- pivot units
- how status systems and targeting should support team-building depth

The goal is to make characters interesting because of how they shape compositions, not just because they have strong numbers.

## Tactical Archetypes

Team archetypes should be defined by how they win and what they force the opponent to answer.

Tarot flavor belongs in the characters. Archetypes should stay tactically clear.

### Front-to-Back Pressure

Goal:

- break the enemy frontline efficiently
- win by stable formation advantage

Typical tools:

- strong direct damage
- anti-tank pressure
- durable frontliners
- row-locked support

Natural counters:

- strong sustain
- backline reach
- disruptive control

### Backline Reach

Goal:

- punish fragile supports and carries behind the frontline

Typical tools:

- row bypass
- backline selectors
- same-column attacks
- splash centered on protected enemies

Natural counters:

- good screening
- intercept effects
- anti-dive control

### Status Pressure

Goal:

- accumulate layered disadvantages the opponent must answer

Typical tools:

- stack application
- status amplification
- status payoff abilities
- spread, transfer, consume, or detonate effects

Natural counters:

- cleanse
- dispel
- status resistance
- faster tempo kills

### Sustain / Stabilization

Goal:

- survive early pressure and win long fights

Typical tools:

- rescue healing
- team healing
- cleanse
- defensive buffs
- MP recovery

Natural counters:

- anti-heal
- backline reach
- burst

### Tempo / Burst

Goal:

- create short windows of overwhelming pressure before the opponent stabilizes

Typical tools:

- opening buffs
- MP acceleration
- kill chaining
- focused damage windows
- snowball triggers

Natural counters:

- disruption
- durable formations
- denial effects that interrupt the burst turn

### Disruption / Control

Goal:

- interfere with enemy execution rather than only racing damage

Typical tools:

- dispels
- taunt or intercept
- target redirection
- turn denial
- anti-trigger effects
- anti-buff or anti-heal tools

Natural counters:

- redundant threats
- stable front-to-back pressure
- teams that are less ability-dependent

### Optional Expansion Archetypes

These should be supported later once the core archetypes are healthy:

- Resource Advantage
- Death / Sacrifice

## Character Role Types

When designing a character, it is useful to classify its intended strategic role.

### Specialist

A specialist is strongest in one clear job:

- burst attacker
- healer
- frontline tank
- cleanse support
- backline hunter

Specialists should be best-in-class in a narrower area than pivot units.

### Enabler

An enabler makes other units stronger by shaping formation, resources, target flow, or status interactions.

Examples:

- companion buffer
- MP battery
- mark applier
- passive trigger amplifier

### Counter-Tech Piece

A counter-tech piece is included to answer specific team styles.

Examples:

- anti-heal support
- dispel specialist
- anti-backline screen
- anti-status purifier

Counter-tech should be strong in its intended matchup but not universally efficient.

### Pivot Unit

A pivot unit can fill one of two roles depending on loadout and rules.

This is one of the strongest tools for deep team building because it links:

- loadout choice
- stat allocation
- formation
- rules
- matchup prediction

Pivot units should trade peak efficiency for flexibility.

They should not be the best specialist in both lanes.

## Current Team Drafts

These are the current working character-kit drafts for the first small-scale 3v3 prototype.

### Team A

#### The Emperor

Role:

- frontline commander
- formation enabler
- target-flow control

Passive:

- `Imperial Formation`
  Allies in The Emperor's row gain `Empower STR 1` and `Empower INT 1`.

Abilities:

- `Hold the Line`
  Apply `Ward 1` to allies in the user's row.
- `Command`
  One companion immediately makes a basic attack against the user's current target.
- `Taunt`
  Enemies with `STR > INT` change their target to the user.

#### The Hierophant

Role:

- row-based magical protector
- companion support bruiser
- secondary magical pressure

Passive:

- `Sanctuary`
  Allies in Hierophant's row gain `Empower WIS 1`.

Abilities:

- `Smite`
  Deal magical damage to the user's current target.
- `Consecrate`
  Deal magical damage to the user's current target and all of that target's companions.
- `Blessing`
  Restore MP to allies in the user's row.

## Pivot Unit Design

### Why Pivot Units Matter

Pivot units create strategic ambiguity:

- the same character can behave differently in different teams
- team preview becomes less revealing
- rule writing and loadout choices matter more

### Good Pivot Patterns

- offense / offense
  physical or magical pressure depending on target profile

- offense / support
  attacks in some matchups, stabilizes in others

- sustain / disruption
  healer that can also cleanse or dispel

- frontline / payoff
  durable unit that becomes threatening once conditions are met

### Example: Death

Death is a strong candidate for a pivot design:

- balanced `STR` and `INT`
- both physical and magical actives equipped
- rules that choose damage lane based on defender profile
- execute behavior once the target is low enough

That makes Death:

- matchup-sensitive
- scriptable
- strategically flexible

It should be slightly less efficient than a pure specialist unless the player scripts it well.

### Good Pivot Candidates

Tarot cards that naturally fit pivot design:

- Death
- Temperance
- Magician
- Lovers
- Justice
- Wheel of Fortune
- Devil
- Judgement

## Targeting as Character Identity

Targeting should help signature abilities feel distinct.

Good examples:

- frontliners using `current_target` or `front_row`
- assassins using `enemy` targeting with a `backmost` row condition
- rescue healers using `lowest_hp_ally`
- formation specialists using same-column row conditions or companion-based targeting

Signature abilities should often use more expressive targeting than generic attacks.

## Statuses as Team-Building Systems

Statuses should support archetypes rather than only serving as generic DOT or stat math.

Useful status functions:

- pressure
- setup
- protection
- escalation
- conversion fuel
- control

Strong status ecosystems allow characters to:

- apply
- amplify
- cleanse
- dispel
- transfer
- consume
- copy
- transform

That creates real composition-based play instead of isolated effect spam.

## Formation and Companion Design

Formation should matter beyond row protection.

High-value design spaces:

- companion buffs
- same-column protection
- intercept behavior
- isolation rewards
- row-bypass counterplay
- movement abilities

Companion interactions are especially valuable because they make team layout a meaningful build decision.

## Character Design Heuristics

When designing a new character, ask:

1. Which tactical archetype does this character support or counter?
2. Is it a specialist, enabler, counter-tech piece, or pivot unit?
3. Does it make formation, targeting, MP use, or status interactions more interesting?
4. Does it create a new composition pattern or answer an existing one?
5. Is its role legible in battle logs and replays?

If a character mostly adds raw damage or raw stat buffs without changing team behavior, it is usually not adding much strategic depth.
