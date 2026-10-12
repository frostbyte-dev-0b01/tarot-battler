# Redesign Proposal: The Spread

## Status

**Proposal, not yet implemented.** This is a distilled redesign of Tarot Battler into a
small, readable autobattler in the Super Auto Pets / Backpack Battles family, PvE
first. It does not replace `game_spec.md` yet. If the prototype (see Build Plan)
proves fun, this becomes the new primary spec and the current engine is archived.

## TL;DR

- You still **script** your cards, but with three gestures instead of rule forms:
  **Place** a card in the spread, **Order** the blocks on its track, **Flip** it
  upright or reversed.
- Cards have two numbers (**Power / Health**), a 3-cell **track** of abilities they
  loop through, and two faces (**Upright ▲ / Reversed ▼**), each with a one-sentence
  trait.
- Battles are short, deterministic, and every number on screen can be counted by hand.
- PvE comes first: a **Reading** is a roguelike run of 9 face-up encounters across
  three acts (Past, Present, Future). Async PvP comes later through ghosts.

## Why the old version felt like a spreadsheet

The tactical ideas were good: setup and payoff, protecting the backline, reading the
opponent. The problem was how players reached them:

| Pain point | Where it came from |
|---|---|
| Programming, not playing | Up to 7 ordered rules per unit, each with a subject, a query, a comparator, and a threshold |
| Hidden math | Six stats, physical/magical split, ratio mitigation (`K / (K + defense)`), four damage tiers |
| Hidden intent | Sticky focus, weakness heuristics, focus policies, companions vs allies |
| Invisible timing | Speed ticks with escalation, MP pips, re-entrancy rules for passives |
| Many small knobs | Passive pools, ability pools, aspects, items, banners, point budget |

Every one of these knobs was reasonable on its own. Together they made each decision
small, numeric, and hard to see in a replay.

## What we borrow from the genre

- **SAP:** a line of units where order *is* the strategy, simple trigger words
  ("start of battle", "when hurt", "when this falls"), two numbers per unit, and runs
  against ghosts.
- **Backpack Battles:** arrangement as the build. Items have **shapes**, and fitting
  them together is tactile.
- **Both:** battles you can watch in under a minute, cause and effect you can see, and
  short runs with drafting variety.

## The core idea: script with three gestures

| Gesture | What you do | What it replaces |
|---|---|---|
| **Place** | Drag the card to a slot in your spread | Focus, targeting selectors, row protection, companions, speed |
| **Order** | Drag ability blocks along the card's 3-cell track | Rule priority list, MP cost, cooldowns |
| **Flip** | Tap to turn the card upright ▲ or reversed ▼ | Conditional rules, passive pools |

Each card has exactly these three knobs: drag it, drag its blocks, tap it. Between them
they cover the questions the old rule system asked:

- **Who** does it affect? → **Place**. All targeting is positional, so where a card
  sits decides who it hits and who it helps.
- **When** does it act? → **Order** (its proactive rhythm) and **Place** (its turn
  order within a round).
- **How** does it react? → **Flip**. Each face carries a different triggered trait.

Reversed as the card's shadow side is already the locked tone (see `metagame.md`, Tone).
Upright cards get a gold frame and reversed cards a violet one, which gives you the
"change its color" control for free.

## Battle rules

### The card

- **Power**: Strike damage. **Health**: how much damage the card can take before it falls.
- **Track**: 3 cells. The card plays one cell per round and loops back to the start.
- **Kit**: 2–3 named **blocks**, each 1, 2, or 3 cells wide. A kit holds more total
  width than a track fits, so you choose what to bring as well as the order.
  Every block may appear at most once. Empty cells play **Strike**.
- **Faces**: an Upright trait and a Reversed trait, one sentence each with one
  trigger word.

### The spread

- Each side has a **line of 5 slots**. Slot 1 is the **front**.
- **Neighbors** are the cards directly ahead and behind.
- When a card falls, the cards behind it **step forward**. Neighbors and slots update
  immediately.

### Rounds

- Slots resolve front to back: every slot-1 card acts, then every slot-2 card, and so on.
- At each slot, the side with **initiative** acts first. Initiative alternates every
  round, and you have it in round 1.
- On its turn a card plays its current track cell, and then the track advances one cell.
- A card acts at most once per round, even if it steps forward mid-round.

### Tracks and blocks

- A block **resolves on its last cell**. Its earlier cells are **wind-up**, shown as a
  filling bar on the card, so the opponent's big plays are telegraphed.
- A 3-wide block therefore fires on rounds 3, 6, 9. A 2-wide block at cells 1–2 fires
  on rounds 2, 5, 8, and at cells 2–3 on rounds 3, 6, 9.
- Width is the cost: big effects buy their power with idle rounds and the risk of
  dying mid-wind-up. This replaces mana.

### Targets (positional words only)

- Enemy side: **the enemy front** (default), **the enemy back**, **every enemy**, and
  sparingly **the weakest enemy** (lowest Health).
- Your side: **self**, **the card ahead**, **the card behind**, **neighbors**,
  **every ally**.
- **Strike**: deal damage equal to Power to the enemy front.

### Keywords (four, and that's all)

| Keyword | Meaning |
|---|---|
| **Shield N** | Absorbs N damage before Health. Doesn't decay. |
| **Omen N** | Does nothing alone. Payoff cards consume it. |
| **Burn N** | At end of round, lose N Health, then Burn drops by 1. |
| **Delay** | The card's next turn is skipped and its track doesn't advance, so its rhythm slides back a round. |

Buffs are plain stat changes (`+2 Power`), as in SAP. Healing can't exceed starting
Health unless the text says otherwise.

### Trigger words

Start of battle · End of round · When hurt · When this falls · When a neighbor falls ·
When an ally falls · When this defeats an enemy.

### End of battle

- A side with no cards loses. A mutual wipe counts as a loss in PvE.
- **Midnight**: from round 10, at the end of each round every card loses Health equal
  to `round − 9`. Battles should usually end around rounds 5–8.
- **No hidden randomness.** Given the same arrangements, a battle always plays out the
  same way. Variance belongs to the run (deals, encounters), and any random card says
  so on its face.

## Content design rules

1. **Each face wants a different place.** A good flip changes where the card wants to
   sit or how its track should be ordered, as well as its numbers. This is what makes
   the three gestures interlock instead of acting as three separate knobs.
2. **One sentence, one trigger.** Every trait and block fits on the card at a glance.
3. **Every block does something useful in any state.** Heals overflow into Shield,
   payoffs Strike if there's nothing to consume, and so on. Nothing is wasted silently.
4. **Countable on fingers.** If a player can't predict a number from the board, it's
   wrong.
5. **Signature over filler.** No "Strike but 1.3×". A block exists only if it creates
   a decision about placement or timing.

## Starter roster (8 cards)

The current eight arcana, rebuilt. Numbers are placeholders for the prototype.

**The Fool · 0** (2/5): the wildcard
- Blocks: *Leap* (1): swap with the card behind you, then Strike. *Hunt the Weak* (2):
  deal double Power to the weakest enemy.
- ▲ *Beginner's Luck*: Start of battle, copy the Upright trait of the card behind you.
- ▼ *Folly*: When this falls, deal 4 damage to the enemy front.
- Upright wants to sit ahead of a strong trait. Reversed wants the front, to fall early.

**The Magician · I** (3/4): the conductor
- Blocks: *Transmute* (1): turn the enemy front's Shield into Burn. *Grand Working* (3):
  deal 3 damage to every enemy.
- ▲ *Manifest*: Start of battle, your neighbors' tracks begin one cell ahead, so their
  big blocks land a round sooner.
- ▼ *Sleight of Hand*: Start of battle, swap the enemy front with the card behind it.
- Upright is a timing tool for your own team. Reversed rearranges the enemy's spread.

**The Emperor · IV** (2/9): the wall
- Blocks: *Hold the Line* (2): Shield 3 to self and neighbors. *Command* (1): the card
  behind you Strikes immediately (its track doesn't move).
- ▲ *Imperial Formation*: Start of battle, neighbors gain Shield 2.
- ▼ *Tyrant*: When a neighbor falls, gain +2 Power and Shield 4.

**The Hierophant · V** (1/6): the keeper
- Blocks: *Blessing* (1): heal the card ahead 3, with any excess becoming Shield.
  *Sanctify* (2): clear Omen and Burn from self and neighbors, and each gains Shield 2.
- ▲ *Sanctuary*: End of round, heal the card ahead 1.
- ▼ *Martyr*: When this falls, every ally gains Shield 3.
- Upright sits behind the tank. Reversed sits in front as the sacrifice.

**The Chariot · VII** (4/4): the charge
- Blocks: *Charge* (1): swap with the card ahead of you, then Strike. *Breakthrough* (2):
  deal double Power to the enemy front, and damage beyond its Health carries into the
  card behind it.
- ▲ *Momentum*: When this defeats an enemy, Strike again.
- ▼ *Wild Ride*: Its Strikes hit the enemy back instead of the front.
- Upright chains kills up the front. Reversed dives the backline.

**The Hermit · IX** (2/6): the loner
- Blocks: *Lantern* (1): strip the enemy front's Shield, then Strike. *Sever Thread* (2):
  the enemy front loses its trait for the rest of the battle.
- ▲ *Solitude*: +1 Power and +1 Health for each empty slot in your spread.
- ▼ *Isolation*: When an ally falls, gain +2 Power.
- Upright asks whether you should field fewer cards, which brings back the old
  quality-versus-quantity budget question with no point costs.

**Justice · XI** (3/6): the payoff
- Blocks: *Condemn* (1): Strike, with +2 damage if the target has Omen. *Verdict* (3):
  consume all Omen on the enemy front and deal 2 damage per Omen.
- ▲ *Balance*: When hurt, deal 1 damage back to the attacker.
- ▼ *Retribution*: When a neighbor is hurt, the attacker gains Omen 1.
- Upright wants the front. Reversed wants to stand beside whoever takes the hits.

**The Moon · XVIII** (2/4): the setup
- Blocks: *Hex* (1): Omen 2 to the enemy front. *Eclipse* (2): Omen 1 to every enemy.
  *Harvest Night* (3): every enemy takes damage equal to its Omen, then loses it.
- ▲ *Foreboding*: Start of battle, Omen 1 to every enemy.
- ▼ *Lunacy*: Enemies with 3+ Omen are maddened, so their Strikes hit the card behind
  them instead (if any).

## Worked example: every number is countable

Your spread is `[Emperor ▲] [Justice ?] [Moon ▲]`. Justice's track is `[Verdict ■■■]`,
which fires on round 3. The Moon's track is `[Hex] [Eclipse ■■]`, so Hex fires on
round 1 and Eclipse on round 3. The enemy front (a Knight of Wands with an all-Strike
track) hits your front every round.

How much Omen is on the enemy front when Verdict resolves on round 3?

| Arrangement | Omen when Verdict lands | Verdict damage |
|---|---|---|
| Justice ▲ (base) | Foreboding 1 + Hex 2 = **3** | 6 |
| **Flip** Justice ▼ Retribution | 3 + three hits on the Emperor = **6** | **12** |
| …then **Place** the Moon ahead of Justice | 1 + 2 + Eclipse 1 = **4** (Justice no longer stands beside the Emperor) | 8 |
| …or **Order** the Moon as `[Eclipse ■■] [Hex]` | 1 + Eclipse 1 + 3 = **5** (Hex now lands after Verdict) | 10 |

The "obvious" fixes in the last two rows are worse. The replay shows why in one line
("Hex: round 3, slot 3, after Verdict"), and a player who reads it can count the
difference on their fingers. That's the loop we want: watch, spot the cause, make one
gesture, and try again.

## The screens

### Build (the table)

```
 ENEMY   [Knight of Wands 3/4]  [Page of Cups 1/5]  [Five of Swords 2/3]
          [·][·][·]               [Mend][·][·]         [...]

 YOU     [Emperor ▲ 2/9]   [Justice ▼ 3/6]   [Moon ▲ 2/4]   [ empty ]   [ empty ]
          [Hold the Line■■][·] [Verdict ■■■]   [Hex][Eclipse■■]
```

- Drag cards between slots (**Place**). Tap a card to rotate it 180° and its trait text
  swaps (**Flip**). Drag the blocks under each card (**Order**).
- In PvE the enemy spread and tracks are visible, so you're planning against something
  you can see.

### Timeline (the sequencer)

```
            R1        R2        R3          R4
 Emperor    wind      ▶Hold     · Strike    wind
 Justice    wind      wind      ▶Verdict    wind
 Moon       ▶Hex      wind      ▶Eclipse    ▶Hex
 ─────────────────────────────────────────────────
 Knight     Strike    Strike    Strike      Strike
```

A pre-battle preview shows when each block fires if nobody falls or gets Delayed. It
looks and works like a drum machine. After the battle, the same grid becomes the
replay, with each event attributed to the block or trait that caused it. (This is the
current replay's rule attribution, simplified.)

## PvE: The Reading

A run is a **Reading**, three acts of three fights each, about 20–30 minutes.

- **Acts**: Past, Present, Future. Each act has two encounters and a boss.
- **Start**: you're dealt 5 Major Arcana and keep 3. Your spread has 5 slots, and you
  may hold up to 2 more cards in reserve.
- **Face-up encounters**: before each fight you see the enemy spread, faces, and tracks.
  In later acts, 1–2 enemy cards stay face-down until the fight starts.
- **After a win you get a Deal**. Choose 1 of 3 offers:
  - a new Major Arcana;
  - a Major you already own, which **Ascends** it (+1/+1, and its block numbers improve,
    like an SAP merge);
  - a **Minor Arcana** claimed from the enemies you just beat, attached as a **Charm**
    (one per Major).
- **After a loss**, spend a **Candle** (you start with 3), rearrange, and retry the same
  encounter. When the Candles run out, the Reading ends. Losing becomes a lesson
  rather than a dead end, and the watch-and-adjust loop becomes the heart of the game.
- **Rearranging is always free.** The gestures are never rationed.
- **Meta-progression**: finishing Readings unlocks more of the 22 Major Arcana. Your
  score is the Candles you have left.

### Enemies are the Minor Arcana

- The four suits give each encounter a flavor:
  - **Wands**: burn and aggression.
  - **Cups**: healing and Shield.
  - **Swords**: reach and precision.
  - **Pentacles**: thick Health and slow, heavy blocks.
- Pips are the rank and file. Courts (Page, Knight, Queen, King) are elites.
- **Charms** use the same cards. A claimed Minor gives a small suit bonus:
  - Wands: +1 Power.
  - Cups: +3 Health.
  - Swords: your first Strike deals double.
  - Pentacles: start of battle, Shield 3.

  Claimed courts carry their own one-line trait.

### Bosses are Major Arcana that break a rule

Each boss has a visible, rule-breaking track:
- **The Tower**: every 3rd round, *Lightning* strips all Shield from your side and hits
  your front for 6.
- **The Devil**: *Chains* bind your back card, which is Delayed every other round.
- **Death**: when any card falls, Death gains its Power.
- **The High Priestess**: her track is face-down until each block fires.

## Later: async PvP through ghosts

When you finish a Reading, your final spread (cards plus all three gestures) is saved as
a **ghost**. Other players then meet ghosts as the Future act of their Readings, and
eventually in an SAP-style arena where enemy spreads are hidden. The existing `server/`
already has the right bones for this (redb storage, a daily runner, replays). It would
store and serve ghosts instead of hosting season pods.

## The other fourteen arcana (one hook each)

| Card | Hook |
|---|---|
| High Priestess · II | Hidden track. Her blocks are face-down to the enemy. |
| Empress · III | Growth: end of round, neighbors gain +1 Health. |
| Lovers · VI | A bonded pair with the card behind, who share damage taken. |
| Strength · VIII | ▲ When hurt, +1 Power. ▼ +3 Power, but loses 1 Health each round. |
| Wheel of Fortune · X | Turns the wheel: advances every track on both sides by a cell. |
| Hanged Man · XII | Delays itself on purpose, and each Delay makes its next block stronger. |
| Death · XIII | When this falls, the card that steps up transforms: it gains +3/+3. |
| Temperance · XIV | Evens out Health between its neighbors. |
| Devil · XV | Chains the enemy front so it shares damage with the card behind it. |
| Tower · XVI | Strips all Shield on both sides. |
| Star · XVII | End of round, heal every ally 1. |
| Sun · XIX | Upright allies get +1 Power. |
| Judgement · XX | The first ally to fall returns with 1 Health. |
| World · XXI | If your spread is full, every ally gets +1/+1. |

A few of these care about how many of your cards are upright or reversed. That light
"color" synergy layer is worth testing, but it isn't a requirement for v1.

## What happens to the current codebase

**Keep (concepts and infrastructure):**
- The Major Arcana pantheon, the tone, and Omen as the setup/payoff resource.
- Deterministic simulation, JSON-defined content, and replay attribution.
- The static-site deploy (`DEPLOY.md`), Node unit tests, and Playwright smoke tests.
- `server/` as the future ghost store.

**Cut from the new mode:**
- Six stats, the physical/magical split, mitigation formulas, and damage tiers.
- MP pips, tick-based speed with escalation, and focus (sticky targets and focus
  policies).
- The rule system (subjects, queries, comparators, `match_any`), and companions vs allies.
- The 3×3 grid and row protection.
- Aspects, items, banners, team passives, and point costs.

**Engine:** write a fresh, small engine rather than retrofitting `battle_engine/`. The
rules above should fit in about 1k lines, while the current engine is about 20k lines
built mostly around the systems being cut. The old engine and dev tool stay untouched
as reference until the new mode proves itself.

## Build plan

**M1 Sandbox: is arranging fun, and are battles readable?**
- A DOM-free JS engine module, unit-tested with `node --test` the way `rules-lib.js` is,
  plus a new page under `tools/spread/`.
- The 8 starter cards and 3–4 hand-made encounters.
- A build screen with drag (Place), tap-to-flip (Flip), and block tracks (Order).
- An animated battle and the timeline replay.
- JS first, because the rules will change a lot while we look for the fun. One file
  that both the page and the tests load means no WASM rebuild step.

**M2 The Reading**
- The run loop: acts, Deals, Ascend, Charms, Candles, and 3 bosses.

**M3 Content and balance**
- All 22 Major Arcana and the Minor Arcana enemy sets.
- An auto-sim harness that plays every card pairing across encounters, the same idea
  as `build-replays.sh`'s round-robin, to flag outliers.

**M4 Ghosts and PvP**
- Port the engine to Rust once the rules settle, so the server can run battles
  natively.
- Repoint `server/` at ghosts.

## Open questions

1. **Line or two rows?** A line of 5 is the most legible option and is proven by SAP.
   Two rows of 3 add lanes and more puzzle, and also more fiddliness. *Leaning: line.*
2. **Retry after a loss, or push on?** Retrying makes the replay-and-adjust loop the
   core and gives a puzzle feel. Pushing on is more roguelike. *Leaning: retry, at the
   cost of a Candle.*
3. **Face-up enemies?** They make scripting purposeful, since you plan against what you
   see, but they risk making fights "solved". *Leaning: face-up, with face-down cards in
   later acts.*
4. **Block widths, or plain ordering?** Widths give a natural cost and telegraphed big
   plays, but they're one more concept. *Leaning: widths. They're the most
   Backpack-Battles-like part.*
5. **Economy: Deals only, or a gold shop?** Deals are simpler, while a shop adds
   sell/reroll texture. *Leaning: Deals for M2, and revisit after playtests.*
6. **Initiative**: alternate every round, or always the player first in PvE?
   *Leaning: alternate, so PvP doesn't need a rules change later.*
