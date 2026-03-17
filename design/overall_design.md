
Complete Game Design Summary

Core Concept
An asynchronous autobattler where team construction, synergy building, rule scripting and meta-prediction are the primary skill expressions. Prices for characters, abilities and items adjust weekly based on prior season popularity and performance, rewarding contrarian but correct reads of the meta. Most of the game happens between rounds — theorycrafting, analyzing replays, predicting pricing shifts.

Theme
Tarot — 22 Major Arcana characters, each with mechanical identity and abilities derived from their symbolic meaning.

Engagement Loop
Daily

Two rounds per day, each round is a series of 5 battles against ELO-matched opponents
Battles run automatically against 5 different opponents per round
Return to watch replays and review results
Team must be locked in before a round starts to participate
Teams can be locked/unlocked and adjusted at any time between rounds
ELO decay begins after roughly 48 hours unlocked

Weekly

Leaderboard resets
Character, ability and item prices adjust based on prior week popularity and performance
Players predict next week's meta and rebuild accordingly


Pricing System

Characters, abilities and items all have individual point costs adjusted weekly
Costs reflect prior season popularity and performance
Team built within a fixed point budget
Rewards identifying undervalued picks before the market corrects
Multiple pricing layers — base character cost, ability costs, item costs
A cheap character with an expensive ability costs similar to an expensive character with a budget build


Character Construction
Each character has:

Base stat spread of ~30 points across 9 stats, fixed per character
6 adjustment points to distribute freely, with per-stat caps to prevent extremes
A role assignment — physical or magical attacker
1 passive chosen from a character-specific list of 4-5 options
2-3 abilities chosen from a larger list, partially character-specific
1 item slot, individually and dynamically priced each season


Stats — one job each
StatRoleCONHP poolSTRPhysical damageINTMagic damageFORPhysical resistanceWISMagic resistanceDEXTurn order/speedSPIAbility pool max and regenFOCAbility DC modifierRESAbility save modifier

Damage Formulas

Physical: max(STR - FOR, 1)
Magical: max(INT - WIS, 1)
Abilities modify base formulas explicitly and are tagged physical or magical


Save System

Triggered by abilities with save components
Attacker rolls 1d8 + FOC vs ability base DC
Defender rolls 1d8 + RES to resist
Guaranteed effects reserved for weaker abilities
Saves reserved for more powerful ones


Speed and Turns

Each character has a DEX counter starting at their DEX value
Counter ticks down by 1 each step
Character acts when counter reaches 1, resets to DEX + 2
Higher DEX acts more frequently, +2 reset softens difference between values


SPI and Ability Resources

SPI stat = starting pool size
Recovers at half SPI stat rounded down each round
Basic attacks are free
Ability cost must be met for a rule to be satisfied


Formation

4x4 grid, three rows — front, middle, back
Characters placed before battle, positions locked in
Front row must be fully defeated before middle can be targeted
Middle row must be fully defeated before back can be targeted
Abilities bypass row protection only if explicitly tagged
Adjacency is cardinal only, no diagonals

Terminology

Companions — characters who start in adjacent cells
Allies — all friendly characters on the team


Targeting

Offensive type: STR > INT → physical attacker, INT > STR → magical attacker, tie → random
Defensive type: FOR > WIS → physical defender, WIS > FOR → magical defender, tie → random
Target selection from frontmost occupied row of living enemies:

Physical attackers prefer magical defenders (weak to physical)
Magical attackers prefer physical defenders (weak to magical)
If no preferred-weakness match in front row, pick randomly from front row

Abilities can explicitly override targeting — taunt, misdirect, veil etc
Targeting modification is a distinct and powerful ability category


Rule System
Each character has up to 5 ordered rules governing ability usage
A rule is satisfied if:

All conditions are met AND current SPI >= ability cost
Rules checked top to bottom, first satisfied rule triggers
Insufficient SPI skips to next rule, does not fall back to basic attack immediately
No rule satisfied → basic attack

Available conditions:

Self stat/HP/SPI >/< value
Target stat/HP/SPI >/< value
Companion stat/HP/SPI >/< value — true if any adjacent ally matches
Ally stat/HP/SPI >/< value — true if any living teammate matches
Use count >/< value — total times this rule's ability has been used by this character
Turns since use >/< value — actor turns elapsed since this rule's ability was last used (never used = infinity, always passes >= checks)
Always — empty conditions list, explicit fallback override


Example Character — The Emperor
StatBaseCON10STR6INT4FOR3WIS2DEX4SPI5FOC5RES3

Role: Physical attacker
Passive: Authority — enemies with lower max HP start with -1 SPI
Abilities: Crush (cost 2, 1.5x STR physical damage), Embolden (cost 3, companions recover 1 SPI)


Example Rule Setup — The Emperor
PriorityAbilityCondition1CrushTarget HP <= 32EmboldenAny companion SPI < 23CrushAlways

Buff and Debuff System
Each character carries an active effects list at runtime:

Name, effect type, stat affected, magnitude, duration, source, on-expire handler
Stats always computed dynamically — base + sum of active modifiers
Base stats never mutated directly
Effects tick down each round, expired effects removed automatically
On-expire handlers support effects that trigger when they end

Effect types:

stat_modifier
damage over time
heal over time
behavior override


Passive System
Each character has 1 passive chosen from a character-specific list of 4-5 options. Passives define a character's reactive identity — how they behave beyond their active ability choices. A passive has a trigger and a list of primitives that execute when the trigger fires.

Passive triggers:

on_battle_start — fires once at the start of battle. Opening buffs, debuffs, status application. Example: Authority (Emperor) weakens all enemies' STR.
on_death — fires when this character dies. Example: Collapse (Tower) deals damage to all enemies.
on_ally_death — fires when any ally (or specifically a companion) dies. Example: Grief (Lovers) gains Empower when a companion falls.
on_kill — fires when this character kills an enemy. Example: Transformation (Death) gains permanent Fortify on kill.
on_deal_damage — fires when this character deals damage (basic attack or ability). Example: Venom (Moon) applies 1 Poison on hit.
on_take_damage — fires when this character takes damage. Example: Fury (Chariot) gains 1 Empower:STR when hit.
on_turn_start — fires at the start of each of this character's turns, before action selection. Example: Fortune's Wheel (Wheel of Fortune) randomly buffs or debuffs self.
permanent_trait — always active, modifies character behavior rather than executing primitives. These are not trigger-based; they modify engine rules directly. Example: Resourcefulness (Magician) reduces ability SPI costs by 1.

Permanent traits are the exception — they don't execute primitives on a trigger, they alter how the engine processes the character. Each trait is a named behavior recognized by the engine, stored as an enum variant on CharacterState. Adding a new trait requires engine code at the relevant code path.

Trait categories and examples:

Resource modification:
- spi_cost_reduction { amount } — abilities cost less SPI (minimum 1). Magician's Resourcefulness.
- spi_regen_bonus { amount } — recovers extra SPI each regen tick. Hierophant's devotion.

Status modification:
- status_tick_bonus { status, extra_stacks } — named status ticks extra stacks per turn on targets afflicted by this character. "Bleed applied by this character ticks 2 stacks instead of 1."
- debuff_resistance { count } — first N debuffs applied to this character are negated. Fool's Innocence.
- status_potency { extra_stacks } — statuses applied by this character gain extra stacks.

Companion/formation override:
- universal_companion — this character counts as a companion to all allies regardless of position. Lovers' bond.
- row_bypass — can target any row, ignoring formation protection. Hermit sees through illusions.

Targeting override:
- taunt_aura — enemies must target this character while it lives (or while it's in the frontmost row). Can coexist with the targeting system.

Speed/turn modification:
- no_speed_escalation — speed reset is always base DEX, doesn't escalate with +2 per turn. Chariot's relentless pace.
- first_strike — acts before anyone else on the first step regardless of DEX. Star's foresight.

Damage modification:
- damage_floor { amount } — basic attacks and ability damage deal at least this much, overriding the max(ATK - DEF, 1) floor.
- damage_reflect { amount } — attackers take fixed damage when they hit this character.


Ability Architecture
Two tier system:

Simple abilities composed from primitive effects defined entirely in JSON
Complex unique abilities point to named Rust handler functions

Primitive effects cover majority of cases:

deal_physical_damage
deal_magical_damage
restore_hp
restore_spi
apply_status
remove_status
modify_targeting

Ability triggers:

on_turn — normal rule based activation via the rule system
on_round_start — passive regeneration or start-of-round effects


Tech Stack
Battle engine

Rust — pure logic, JSON in and battle log out, no rendering
serde + serde_json — JSON serialization
rand — dice rolls

Backend

Python + FastAPI — web server, ELO system, round scheduling, pricing adjustments
Calls Rust battle engine as subprocess
PostgreSQL — persistent storage for accounts, teams, ELO, battle logs, pricing history

Frontend

React or Svelte — team builder UI, formation grid, rule scripting interface
PixiJS or plain HTML/CSS — battle replay viewer reads structured battle log and animates it
PWA — mobile optimized, push notifications for round results

Optional

Discord bot — round results, ELO updates, simplified battle summaries


Battle Log Structure
json[
  {
    "step": 1,
    "turn": 1,
    "event_type": "action_basic_attack",
    "actor": "Emperor",
    "target": "Fool",
    "damage": 3,
    "target_hp_remaining": 4
  },
  {
    "step": 2,
    "turn": 1,
    "event_type": "rule_evaluated",
    "actor": "Emperor",
    "rule_index": 1,
    "satisfied": true,
    "ability": "Crush"
  }
]
```

---

**Rust Folder Structure**
```
battle_engine/
  ├── Cargo.toml
  └── src/
        ├── main.rs
        ├── models.rs
        ├── engine.rs
        ├── rules.rs
        ├── abilities.rs
        ├── effects.rs
        ├── targeting.rs
        ├── passives.rs
        ├── logger.rs
        └── data/
              ├── characters.json
              ├── abilities.json
              └── items.json

Cargo.toml Dependencies
toml[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rand = "0.8"

Development Path

Pen and paper — verify math and rule system feel right
Rust battle engine — two hardcoded characters, basic attacks only, text log output
Add rule system and abilities
Add passives and buff/debuff system
Add full 4x4 grid, formation and targeting
Add saves and items
Stress test with full teams
Python FastAPI wrapper
Simple web UI — unstyled is fine early
Battle replay viewer — text log first, animations later
Polish — art, mobile PWA, Discord bot


Open Questions

Exact team point budget size
Per-stat adjustment cap within 6 point character budget
Exact pricing adjustment formula between seasons
Shared vs character-specific ability pools
Full ability list beyond The Emperor's starting moves
ELO decay rate
