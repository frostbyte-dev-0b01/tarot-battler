
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


Ability Architecture
Two tier system:

Simple abilities composed from primitive effects defined entirely in JSON
Complex unique abilities point to named Rust handler functions

Primitive effects cover majority of cases:

deal_physical_damage
deal_magical_damage
restore_hp
restore_spi
apply_buff
apply_debuff
modify_targeting

Ability triggers:

on_turn — normal rule based activation
on_death — Tower collapse etc
on_hit — reactive abilities
on_round_start — passive regeneration
on_battle_start — opening effects like Authority
on_row_cleared — formation reactive abilities


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
