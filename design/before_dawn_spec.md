# Before Dawn: Prototype Design Spec (v0.1)

> **Working title.** This is a handoff spec for building the first playable prototype in a
> fresh repository. It's self-contained and assumes no prior context.
>
> **How to read it:**
> - §1–2 explain what the game is and why. Read them before making any judgment call this
>   spec doesn't cover.
> - §3–7 are **normative** rules and content. The rules must be implemented exactly as
>   written. The numbers are tuning placeholders that the simulator in §11 will adjust.
> - §8–10 cover the run structure, presentation, and recommended architecture.
> - §11–13 cover the simulator, required tests (including a golden battle trace), and
>   milestones.
> - §14–15 list what's out of scope and what's still open.
>
> "MUST" and "SHOULD" are used in the usual RFC sense.

---

## 1. Pitch

**Before Dawn** is a single-player roguelike autobattler built on Slay the Spire's run
structure. You're a necromancer dueling rival necromancers, hedge witches, and the holy
living through one long night.

You never play cards during a fight. Instead you **script** your deck:
- The deck plays one card per turn, in the order you set, and loops.
- Up to three **sigils** ("when X, play Y") let it react to what happens.
- Cards summon undead creatures. Each one hammers the enemy necromancer for a few turns
  and then crumbles into your graveyard.

A fight is a race to bring the other necromancer to 0 HP before dawn burns you both.

**The core loop:** map → fight (watch it) → reward (pick a card and decide where it goes
in your loop) → … → rest sites (heal, upgrade, or remove; re-sort the deck; edit sigils)
→ boss.

**Influences:**

| Game | What we take |
|---|---|
| Slay the Spire | Run structure, the act boss known in advance, HP that persists between fights |
| The Bazaar / Backpack Battles | Damage dealt to the player, racing engines, damage-over-time vs burst |
| Super Auto Pets | Readable trigger words |
| Final Fantasy XII Gambits | Ordered "condition → action" slots |
| Balatro / Peggle | Juice: escalating numbers and rising pitch |

## 2. Design pillars (read before making judgment calls)

1. **Scripting by arrangement.** During a fight, the player controls exactly two things:
   their **deck order** and up to **3 sigils**. There are no free-form rules and no typed
   numbers. A sigil threshold is a pin that snaps to a few notches.
2. **Nothing kills your engine by accident.** Creatures attack the enemy necromancer and
   never other creatures. A creature leaves because its Life runs out, or because an
   explicit card removes it.
   - Why: when units fight units, the first kill snowballs, because a side that loses a
     unit also loses its future damage. That makes outcomes feel random even when they're
     deterministic.
   - The one exception is **guardians**. They absorb damage aimed at their necromancer
     and can be destroyed, but destroying one never reduces its owner's damage output.
3. **Battles are deterministic, with no hidden randomness.** The same setups always produce
   the same battle. All randomness lives at the run level (map, rewards, encounters) and
   uses a seeded PRNG.
4. **Countable on fingers.** A player can predict every number from the board: no decimal
   multipliers and no hidden formulas.
5. **Order is for your engine; sigils are for the matchup.** The best deck order should
   depend mostly on your own cards. You adapt to a particular enemy through sigils, which
   you can only change at rest sites. Players must never feel they have to re-sort their
   deck before every fight.
6. **The strategic triangle.**
   - **Rush** (burst, short-lived creatures) beats **ramp** (scaling, graveyard), which dies
     before it scales.
   - **Ramp** beats **sustain** (guardians, healing), because stalling only feeds the ramp.
   - **Sustain** beats **rush**, by absorbing the burst.
   - Cross-counters keep hybrids viable: Spectral beats guardians, and Banish beats
     graveyards.
   - Because HP persists, pace is also a *run-level* trade. Rush wins fast and loses little
     HP; ramp wins reliably but bleeds HP every fight.
7. **Juice is a core feature, not polish.** Fights have to be satisfying to watch: one beat
   at a time, escalating numbers and pitch, and state shown as objects rather than
   numbers.

## 3. Glossary

| Term | Meaning |
|---|---|
| **Necromancer** | A side's avatar. It has HP and loses at 0. |
| **Side** | The player or the enemy. Each side has a necromancer, deck, board, graveyard, and sigils. |
| **Deck / loop** | An ordered list of cards. It plays the next card each turn and wraps around. |
| **Board** | Up to **6** creatures in a row. Leftmost is the oldest; new creatures enter on the right. |
| **Creature** | Summoned by a card. Has Attack and Life, and possibly keywords. |
| **Guardian** | A creature with **Guard**. It absorbs damage aimed at its necromancer and never attacks. |
| **Token** | A creature that comes from an effect rather than a deck card (e.g. Rat). |
| **Graveyard** | An ordered pile of crumbled creatures, with the most recent on top. |
| **Sigil** | A slot holding a condition tile, a card, and a pin if the tile uses one. |
| **Round** | One turn for each side. |
| **Dawn** | The fight clock. From the Dawn round on, the active necromancer takes escalating damage each turn. |
| **Opponent** | In card text, "opponent" always means the side opposite the card's owner. |

## 4. Battle rules (normative)

### 4.1 Setup

At the start of a fight, each side has:
- **Necromancer HP.** For the player, current run HP and max HP. For the enemy, its listed
  HP, which is also its max.
- **Deck:** an ordered list of `{cardId, upgraded}`, with the **deck pointer** at 0.
- **Board:** empty.
- **Graveyard:** empty.
- **Sigils:** all armed (not yet fired).
- **Pending Slow summon:** none.

Two fight-level settings:
- **Turn order:** the player goes first each round, unless the encounter has `ambush`, in
  which case the enemy goes first each round.
- **Dawn round:** 12 by default. An encounter may override it (the Abbess of Dawn uses 8).

### 4.2 Turn structure

On round `r`, the active side **S** (opponent **O**) takes its turn in these steps:

1. **Dawn.** If `r ≥ dawnRound`, S's necromancer loses `r − dawnRound + 1` HP. Guardians
   never absorb Dawn damage.
2. **Start-of-turn specials.** Encounter-specific effects resolve here (e.g. the Lich
   Lord's Grave Theft, §7.3).
3. **Choose an action.** Take the first of these that applies:
   - a. **Pending Slow summon.** If S has one, it arrives (summon procedure, §4.5). That is
     S's whole action. Sigils are not checked this turn.
   - b. **Sigil.** Otherwise, check S's sigils in slot order (1 → 2 → 3). The first
     **armed** sigil whose condition is true **fires**: mark it fired, then play its card
     (§4.4). The deck pointer does **not** advance.
   - c. **Deck.** Otherwise, play `deck[pointer]`, then advance the pointer, wrapping to 0
     after the last card.
4. **Creatures act.** Snapshot S's board, left to right. For each creature in the snapshot
   that is still on the board:
   - it performs its action (§4.6);
   - it then loses 1 Life;
   - if its Life is now 0, it crumbles (§4.7).

   Creatures that enter the board *during* this step don't act this turn. Creatures that
   entered during step 3 are in the snapshot and **do** act this turn.
5. **End of turn.** Nothing happens here in v0. It exists as a hook for later content.

**Ending a battle:**
- The battle ends **immediately** when any necromancer's HP is ≤ 0, and every remaining
  step is skipped. That side loses.
- Ties can't happen, because only one side acts at a time and no effect damages both
  necromancers at once.
- Safety cap: if round 40 completes, the enemy wins. Dawn should make this unreachable.

**Trigger queue:**
- Triggered effects (Arrive, Crumble, "Whenever…") go into a **FIFO queue**.
- They resolve after the effect that caused them has finished, and before the next
  creature acts or the next step begins.
- Triggers raised while the queue is resolving are appended to the same queue.
- Safety: abort after 500 triggers in a single turn. Log it as a content bug.

### 4.3 Damage, guardians, healing

**Sources of damage:**
- **Attack damage.** When a creature with current Attack ≥ 1 acts, it deals damage equal to
  its current Attack to O's necromancer.
- **Effect damage.** "Deal N" means N damage to O's necromancer.

**How guardians absorb damage:**
- Unless the source is **Spectral** or the effect says "ignoring guardians", O's guardians
  absorb damage **oldest-first**.
- Each guardian absorbs up to its current Guard, and its Guard drops by that amount.
- Any damage left after all guardians reaches the necromancer.
- A guardian whose Guard reaches 0 crumbles once that damage instance has finished
  resolving.

**Rules for specific cases:**
- **Dawn damage** is never absorbed.
- **"Hits"**: an attack *hits* if it deals ≥ 1 damage to the necromancer itself. Damage that
  guardians fully absorb isn't a hit. On-hit effects (Leech Bats) use this definition.
- **Healing** raises necromancer HP, capped at max HP.
- Each damage effect is **one damage instance**, so it produces one number pop. Corpse
  Explosion, for example, deals its total as a single hit.

### 4.4 Playing a card

- **Spell:** resolve its effect. Spells never enter the graveyard. They stay in the deck
  loop, or in their sigil.
- **Creature:** summon it (§4.5).
- **Slow creature:** create a *pending Slow summon* for that card, shown as a summoning
  circle on S's side. On S's next turn, step 3a summons it.
  - A Slow card therefore costs two turns.
  - A side can have at most one pending Slow summon, which step 3a's priority enforces.
  - If an effect would start a Slow summon while one is already pending, the new one
    **fizzles**.
- **Echo memory:** every card played, whether from the deck, a sigil, or an Echo, is
  recorded in S's play history for Echo (§5).

### 4.5 Summon procedure

1. **Push out if full.** If S's board already has 6 creatures, the oldest (leftmost) is
   **pushed out**. It crumbles (§4.7), and its Crumble trigger is queued.
2. **Place.** Append the new creature at the right end, with its base stats (upgraded
   stats if the card is upgraded) and full Life.
3. **Arrive.** Queue its Arrive trigger, if it has one.
4. **Resolve** the queue.

Summoning a token from inside a trigger follows the same procedure, so it can push out
another creature.

### 4.6 Creature action

A creature's action resolves in this order:
1. **Attack.** If its *base* Attack is ≥ 1, it attacks for its current Attack (§4.3).
   Creatures with no base Attack (guardians, Rat Nest, Bone Drummer, Ghoul) skip this
   step, even if a bonus would apply.
2. **"Each action" effects.** For example, Rat Nest summons a Rat here.
3. **Grow.** If it has Grow N, it permanently gains +N Attack.

**Current Attack** = base Attack + permanent gains (Grow, Gravedigger) + dynamic bonuses
(Lich's graveyard count, Bone Drummer's aura). Dynamic bonuses are computed at the moment
of the attack.

**Extra actions:** Frenzy grants an extra action. It runs the same three steps but costs no
Life.

### 4.7 Crumbling and banishing

- **Crumble.**
  1. Remove the creature from the board.
  2. Push a record (`{cardId, upgraded}`, or the token id) onto the **top** of its owner's
     graveyard.
  3. Queue its Crumble trigger, plus any "whenever another of your creatures crumbles"
     triggers on its owner's board.
- **A creature crumbles when:**
  - its Life reaches 0;
  - it's pushed out by a full board;
  - its Guard reaches 0;
  - an effect says it crumbles (Torch, Offering).
- **Banish.** Remove the creature from the board. It does **not** enter the graveyard, and it
  triggers neither its own Crumble effect nor any "whenever crumbles" effects.
- After any removal, the board closes the gap, and the remaining creatures keep their
  order.

### 4.8 Keywords

| Keyword | Rule |
|---|---|
| **Attack N** | Each action, deal N to the opponent's necromancer (§4.3). |
| **Life N** | The creature acts N times, then crumbles. Shown as candle pips. |
| **Guard N** | Guardian. Absorbs damage aimed at its necromancer until Guard reaches 0, then crumbles. Guardians also have Life. |
| **Arrive:** | Triggers when the creature is summoned, including when it returns via Raise Dead. |
| **Crumble:** | Triggers when the creature crumbles, for any of the reasons in §4.7. |
| **Each action:** | Happens every time the creature acts (§4.6 step 2). |
| **Grow N** | After each action, +N Attack, permanently for this fight. |
| **Spectral** | Its attacks ignore guardians. |
| **Slow** | Takes two turns to summon (§4.4). |
| **Banish** | Removal that bypasses the graveyard and all Crumble triggers (§4.7). |

### 4.9 Sigils

- **Slots.** A side has up to 3 sigil slots. Each holds a **condition tile**, a **card**,
  and a **pin** (the threshold) if the tile uses one.
- **The card leaves the deck.** A card assigned to a sigil is taken out of the deck loop.
  When the sigil fires, that card is played exactly as if it came from the deck.
- **When sigils are checked:** only in step 3b. Each sigil fires **at most once per fight**,
  and all sigils re-arm at the start of every fight.
- **Priority:** slot 1, then slot 2, then slot 3. At most one sigil fires per turn, and
  firing replaces that turn's deck card.
- **Conditions** are listed in §6.
- **Enemies.** The engine MUST support sigils for enemies too, so both sides run on the same
  rules. No v0 enemy uses them.

### 4.10 Visibility

- **Before a fight:** show the enemy's HP, full deck loop, specials, and Dawn round.
- **During a fight:** for each side, show the next action (the next deck card, or the
  pending Slow arrival), each sigil's condition and whether it's armed or fired, the deck
  loop strip with its pointer, and the graveyard count.

## 5. Player cards (normative; numbers are placeholders)

### 5.1 Tokens

| Token | Stats |
|---|---|
| **Rat** | Attack 1, Life 3 |
| **Shade** | Attack 1, Life 2. The fallback form of Mirror Shade. |

### 5.2 Card pool (24 cards)

**Rarity weights:** Starter cards appear in the starting deck only. Rewards are drawn from
Common, Uncommon, and Rare (§8.3). Bone Wall and Rat Swarm are both starters and
reward-eligible Commons.

**Archetype tags** are used for UI color and for building simulator decks.

**Rush** (front-loaded, short-lived)

| Card | Rarity | Type | Text | Upgraded |
|---|---|---|---|---|
| Ghoul | Common | Creature | Life 1. **Arrive:** deal 5. | Arrive: deal 8 |
| Hellhound | Common | Creature | Attack 4, Life 2 | Attack 5 |
| Death Knight | Uncommon | Creature, **Slow** | Attack 6, Life 2 | Attack 8 |
| Bone Spear | Common | Spell | Deal 4, ignoring guardians. | Deal 6 |
| Frenzy | Rare | Spell | All your creatures act now, in board order, with no Life cost. | Also, attackers get +1 Attack for this extra action. |

**Swarm** (damage over time: low Attack, long Life, many bodies)

| Card | Rarity | Type | Text | Upgraded |
|---|---|---|---|---|
| Rat Swarm | Starter / Common | Spell | Summon 2 Rats. | Summon 3 Rats |
| Rat Nest | Uncommon | Creature | Life 3. **Each action:** summon a Rat. | Life 4 |
| Plague Bearer | Common | Creature | Attack 1, Life 4. **Crumble:** summon 2 Rats. | Crumble: summon 3 Rats |
| Leech Bats | Common | Creature | Attack 1, Life 5. Whenever it hits, heal 1. | Life 7 |
| Wraith | Common | Creature | Attack 2, Life 3, **Spectral**. | Attack 3 |

**Scaling**

| Card | Rarity | Type | Text | Upgraded |
|---|---|---|---|---|
| Bone Golem | Uncommon | Creature, **Slow** | Attack 1, Life 6, **Grow 1**. | Attack 2 |
| Lich | Rare | Creature | Attack 1, +1 Attack per creature in your graveyard. Life 4. | Life 6 |
| Gravedigger | Uncommon | Creature | Attack 1, Life 4. Whenever another of your creatures crumbles, +1 Attack. | Attack 2 |

**Graveyard**

| Card | Rarity | Type | Text | Upgraded |
|---|---|---|---|---|
| Raise Dead | Rare | Spell | Remove the top creature of your graveyard, and summon a fresh copy of it with full Life. Its Arrive triggers. If your graveyard is empty, nothing happens. | Raise the top 2, one at a time, top first |
| Corpse Explosion | Uncommon | Spell | Remove up to 5 creatures from the top of your graveyard. Deal 2 per creature removed, as one damage instance. | Up to 8 |
| Offering | Uncommon | Spell | Your oldest creature crumbles, and its Crumble triggers. Deal damage equal to double its current Attack, measured before it's removed. Does nothing if your board is empty. | Triple |

**Order**

| Card | Rarity | Type | Text | Upgraded |
|---|---|---|---|---|
| Echo | Rare | Spell | Play again the most recent non-Echo card you played this fight. If there is none, nothing happens. | Replay the 2 most recent non-Echo cards, older first |
| Bone Drummer | Uncommon | Creature | Life 4. No Attack. Creatures to its right with base Attack ≥ 1 get +1 Attack. | Life 6 |
| Mirror Shade | Rare | Creature | **Arrive:** becomes a fresh copy of the creature directly to its left, with full Life. The copy's Arrive does *not* trigger. If nothing is to its left, it stays a Shade token. | The copy also gets +1 Attack |

**Defense**

| Card | Rarity | Type | Text | Upgraded |
|---|---|---|---|---|
| Bone Wall | Starter / Common | Creature (guardian) | Guard 6, Life 4 | Guard 9 |
| Ossuary Gate | Uncommon | Creature (guardian), **Slow** | Guard 14, Life 4 | Guard 20 |
| Soul Siphon | Common | Spell | Deal 4, then heal 4. | Deal 6, heal 6 |
| Exorcism | Uncommon | Spell | Banish the opponent's 2 oldest creatures. | 3 oldest |

**Basic**

| Card | Rarity | Type | Text | Upgraded |
|---|---|---|---|---|
| Skeleton | Starter | Creature | Attack 2, Life 3 | Life 4 |

Counts: 9 Common, 9 Uncommon, 5 Rare, and 1 starter-only card (Skeleton).

### 5.3 Clarifications

- **Ghoul** has no base Attack. It acts once doing nothing, then crumbles. In practice it's
  a 5-damage hit that adds one creature to the graveyard.
- **Echo** can replay a card that was played from a sigil. If it replays a Slow card while
  another Slow summon is pending, that replay fizzles (§4.4).
- **Frenzy's** extra action includes "Each action" effects and Grow. Creatures summoned
  *during* Frenzy don't get an extra action.
- **Raise Dead** works on tokens too: raising a Rat gives a Rat.
- **Offering** can target a guardian (Attack 0). The damage is 0, but its Crumble still
  triggers.
- **Lich** counts every creature in its owner's graveyard, tokens included, at the moment it
  attacks.

### 5.4 Starting state

- **HP:** 60 / 60.
- **Starter deck, in this order:** `Skeleton, Skeleton, Bone Wall, Skeleton, Bone Wall,
  Rat Swarm`.
- **Sigils:** 1 slot, starting empty. Owned condition tiles: **Wounded** and **Omen**
  (§6).
- **Before the first fight,** the player may arrange the deck and sigils once, as at a rest
  site but without heal, upgrade, or remove.

## 6. Sigil condition tiles (normative)

Pins snap to the listed notches. Use integer math for percentages:
`hp * 100 <= pin * maxHp`.

| Tile | Condition (checked in step 3b) | Pin notches |
|---|---|---|
| **Wounded** | My HP ≤ pin % of my max HP | 75 / 50 / 25 |
| **Execute** | Opponent HP ≤ pin % of their max HP | 75 / 50 / 25 |
| **Overrun** | Opponent has ≥ pin creatures on board (guardians and tokens count) | 2 / 3 / 4 / 5 |
| **Shielded** | Opponent has at least one guardian | n/a |
| **Omen** | Opponent has a pending Slow summon | n/a |
| **Harvest** | My graveyard has ≥ pin creatures | 3 / 5 / 8 |
| **Late Hour** | Current round ≥ pin | 4 / 6 / 8 / 10 |
| **Crowded** | My board has 6 creatures | n/a |

Each tile the player owns can be used in at most one slot.

## 7. Enemies (normative; numbers are placeholders)

Enemies follow exactly the same rules as the player: a deck loop, a board, a graveyard,
and optional specials and sigils. Their loops are visible before the fight. **Each
encounter is designed to ask the player's deck one question.**

### 7.1 Enemy-only cards

| Card | Type | Text |
|---|---|---|
| Shovel | Spell | Deal 3. |
| Lantern | Creature (guardian) | Guard 4, Life 3 |
| Pitchfork | Creature | Attack 2, Life 2 |
| Torch | Spell | The opponent's oldest creature crumbles (Crumble triggers). |
| Thornling | Creature | Attack 1, Life 6, Grow 1 |
| Hex Shield | Creature (guardian) | Guard 5, Life 3 |
| Tower Shield | Creature (guardian) | Guard 8, Life 3 |
| Lance | Spell | Deal 4. |
| Zealot | Creature | Attack 3, Life 2 |
| Purge | Spell | Banish every opponent creature with 3 or more Life remaining. |
| Lumber | Spell | Deal 3. |
| Golem Fist | Creature, Slow | Attack 12, Life 1 |
| Banshee | Creature | Attack 2, Life 3, Spectral |
| Wail | Spell | Each opponent guardian's Guard becomes `floor(Guard / 2)`. |
| Acolyte | Creature (guardian) | Guard 6, Life 3 |
| Sunbeam | Spell | Deal 6, then banish the opponent's oldest creature. |
| Choir | Spell | Heal 8. |

Enemies may also use player cards (Rat Swarm, Plague Bearer, Skeleton, Wraith, Bone Wall).

### 7.2 Encounters (Act 1)

| Encounter | Tier / pool | HP | Loop | The question | Intended answers |
|---|---|---|---|---|---|
| **Grave Robber** | Normal, easy | 20 | Shovel · Shovel · Lantern | None; it's the tutorial | Anything |
| **Rat King** | Normal, easy and standard | 28 | Rat Swarm · Rat Swarm · Plague Bearer | Can you handle a wide board of small hits? | Exorcism, Overrun sigil, guardians, rush |
| **Hedge Witch** | Normal, easy and standard | 26 | Thornling · Hex Shield · Thornling | Can you kill her before her creatures scale? | Rush (Hellhound, Ghoul, Death Knight), Execute sigil |
| **Torch Mob** | Normal, standard | 32 | Pitchfork · Pitchfork · Torch | Do you rely on one long-lived creature? Torch also *feeds* Crumble decks. | Swarm, Plague Bearer, Gravedigger |
| **Templar Squire** | Normal, standard | 30 | Tower Shield · Lance · Tower Shield | Can you get through guardians? | Wraith, Bone Spear, Shielded sigil, ramp |
| **The Inquisitor** | Elite | 55 | Zealot · Zealot · Purge | Purge is telegraphed. Can you time long-lived creatures around it, or go short? | Rush, short Life, Raise Dead |
| **Flesh Golem** | Elite | 70 | Lumber · Lumber · Golem Fist | One telegraphed haymaker. Can you have Guard up when it lands? | Omen sigil → Bone Wall, racing it |
| **Banshee Sisters** | Elite | 45 | Banshee · Banshee · Wail | Your guardians don't work here. What's plan B? | Racing, healing (Leech Bats, Soul Siphon via Wounded), Exorcism |
| **The Abbess of Dawn** | Boss | 110 | Acolyte · Sunbeam · Choir | **Dawn comes at round 8.** A pure rush test. | Burst, Spectral, Bone Spear |
| **The Lich Lord** | Boss | 120 | Skeleton · Wraith · Bone Wall · Skeleton | **Grave Theft** (below): your graveyard feeds her. | Corpse Explosion on your own graveyard, Raise Dead, Exorcism, lean graveyards |

### 7.3 Specials

- **Grave Theft** (Lich Lord), at turn step 2. If the player's graveyard isn't empty:
  1. Remove its top creature.
  2. Summon a fresh copy onto the Lich Lord's board, using the normal summon procedure
     (Arrive triggers, and it can push).
  3. When that creature later crumbles, it goes to *her* graveyard.

## 8. Run structure (Act 1 only in v0)

### 8.1 Run start
- A run seed is drawn and stored. All run randomness uses a seeded PRNG.
- The player starts with the state in §5.4.
- The **boss** (Abbess or Lich Lord) is chosen by seed and shown on the map from the start,
  so the player can draft toward an answer.

### 8.2 Map
- **Layout:**
  - Floor 1: 3 nodes, all Fights from the easy pool.
  - Floors 2–3: Fights from the easy pool, or a Rest site (floor 3 only).
  - Floors 4–8: Fight (standard pool) 50%, Elite 22%, Rest 28%.
  - Floor 9: a single Rest node.
  - Floor 10: the Boss.
- Each of floors 1–8 has 2–4 nodes. Each node links to 1–2 nodes on the next floor, and
  paths don't cross, as in Slay the Spire.
- **Constraints.** Regenerate the map until all of these hold:
  - there are at least 2 Elite nodes;
  - at least one path from start to boss has ≤ 1 Elite;
  - at least one path has ≥ 2 Elites;
  - no two Rest nodes are directly connected.
- **Encounter selection:**
  - normal fights draw from their pool, avoiding the encounter just fought;
  - elites don't repeat within a run.

### 8.3 Rewards
- **After any fight win:** a card reward.
  - Offer 3 distinct cards.
  - Normal fights roll rarity at Common 60 / Uncommon 30 / Rare 10.
  - Elites roll at 35 / 45 / 20.
  - The player may skip.
  - If they take a card, they choose its **insertion position** in the deck loop (any gap).
    The default is the end. This is the only time order can change outside a rest site.
- **After an elite win, additionally:**
  - unlock the next sigil slot, up to a maximum of 3;
  - choose 1 of 2 random condition tiles the player doesn't own yet.
- **No gold, shops, relics, or events in v0.**

### 8.4 Rest sites
- **Choose one:**
  - **Heal** 30% of max HP, rounded up;
  - **Upgrade** a card;
  - **Remove** a card (the deck has a minimum size of 4).
- **Also, free and unlimited:**
  - **reorder** the deck;
  - **edit sigils:** move cards between the deck and sigil slots, assign tiles, and set
    pins.
- These edits are the **only** way to re-sort the deck or change sigils (pillar 5).

### 8.5 HP, death, victory
- HP persists between fights. Max HP is 60 throughout v0.
- At 0 HP the run is lost. Defeating the boss wins the run.
- The end screen shows: floors cleared, final HP, final deck and sigils, and per-fight
  stats (damage dealt by card, HP lost, rounds).

### 8.6 Saving
- Store the run state in `localStorage` under a versioned key, e.g.
  `before-dawn:run:v1`.
- Battles are deterministic, so saving the pre-fight state is enough. Reloading mid-fight
  simply replays the fight from the start.

## 9. Presentation

### 9.1 Screens
1. **Title:** New Run / Continue / Sandbox.
2. **Map:** the boss portrait is visible from the start.
3. **Pre-fight:** the enemy's portrait, HP, loop strip, specials, and Dawn round; your loop
   strip and sigils (read-only); and a **Fight** button.
4. **Battle** (§9.2).
5. **Reward:** card choice, then dragging the card into the loop strip to set its position.
6. **Rest site:** heal / upgrade / remove, plus the deck and sigil editor.
7. **Run end.**
8. **Sandbox** (dev tool): any player cards in any order, any sigils, versus any encounter,
   with a step-through mode and the event log visible.

### 9.2 Battle layout

```
[Enemy necromancer  HP ████░░ 34/45]  [loop strip ▸ next card]   [☠ 3]
[ enemy board: 6 slots ……………………………………………… ]  [summoning circle]
              ~ night sky → Dawn gradient ~   Round 5 · Dawn in 7
[ player board: 6 slots …………………………………………… ]  [summoning circle]
[Player necromancer HP █████░ 48/60]  [loop ▸ next] [sigils ① ② ③]  [☠ 5]
```

The layout must work at phone width (portrait) and on desktop.

### 9.3 Visual language
- **Creature tile:**
  - a sprite (placeholder emoji or simple SVG), colored by archetype: rush red, swarm
    green, scaling purple, graveyard bone/grey, order gold, defense blue;
  - an **Attack** badge (sword);
  - **Life** as candle pips;
  - **Guard** as a bone-shield badge.
- **Size shows power.** Sprite scale = `1 + 0.08 × (currentAttack − baseAttack)`, capped at
  1.6×, so Grow and Lich are visible at a glance.
- **Slow:** a pulsing summoning circle at the board's right edge.
- **Crumble:** the sprite collapses into bones that fly to the graveyard pile, and the
  count ticks up with a *clack*.
- **Banish:** a ghostly white dissolve, with no clack and no change to the graveyard.
- **Damage numbers:**
  - size scales with `log2(amount)`;
  - colors: white for attacks, violet for spells, orange for Dawn, green for heals;
  - 10 or more damage catches fire.
- **Hit-stop:** `30ms + 6ms × amount`, capped at 120ms. Screen shake for 8 or more damage.
- **Sigil fire:** the slot glows, a chime plays, and a banner shows the condition and the
  card, e.g. *"Wounded (≤ 50%) → Bone Wall"*.
- **Dawn:** the background interpolates indigo → violet → orange from round 1 to the Dawn
  round. At Dawn the sun rises, and each Dawn tick shows an orange number.
- **Pacing:**
  - 450ms per major beat at 1×;
  - the creature actions in a turn start at 220ms apart and get 10% faster with each
    creature, to a minimum of 80ms;
  - speed controls: 1× / 2× / 4× / **Skip** (jump to the result).
- **Combat log panel:** every event, with attribution (e.g. "Skeleton hits for 2", "Lantern
  absorbs 2").
- **Hover or tap** any creature, card, or sigil to see its full text.
- **Reduced-motion setting:** disables shake and hit-stop.

### 9.4 Audio

All sounds are synthesized with Web Audio, so v0 needs no audio assets.

| Sound | When | How |
|---|---|---|
| **Blip** | Each hit | Within one turn's creature sequence, each consecutive hit plays one semitone higher than the last. The pitch resets each turn. |
| **Thud** | Hits of 8 or more | Low noise burst |
| **Clack** | Crumble | Short noise |
| **Chime** | Sigil fires | Two sines |
| **Whoosh** | Spells | Filtered noise sweep |
| **Low drone** | Dawn approaching | Swells as Dawn nears |

Include master volume and mute controls.

## 10. Technical architecture (recommended)

- **Stack:** TypeScript (strict), Vite, Vitest, ESLint + Prettier. Playwright for a smoke
  test is optional.
- **Rendering:** plain DOM + CSS transforms + Web Audio is enough for v0 and the easiest to
  test. Move to a canvas library (e.g. PixiJS) only if the juice needs it.
- **Layout:**
  - `src/engine/`: the pure, deterministic battle engine. No DOM, no `Math.random`, no
    `Date`.
  - `src/content/`: card, token, enemy, encounter, and sigil tile definitions as typed data.
    Validate it at startup: unknown ids, missing upgrades, and illegal values all fail
    loudly.
  - `src/run/`: the run state machine, map generation, and rewards. Uses a seeded PRNG
    (e.g. mulberry32 or sfc32), with the seed stored in the run state.
  - `src/sim/`: the Node CLI simulator and solver (§11).
  - `src/ui/`: rendering and input. It consumes engine event logs and never mutates engine
    state.
- **Engine API (sketch):**
  ```ts
  type CardRef = { id: string; upgraded: boolean };
  type SigilSetup = { tile: TileId; pin?: number; card: CardRef };
  type SideSetup = {
    hp: number; maxHp: number;
    deck: CardRef[]; sigils: SigilSetup[];
    specials?: SpecialId[];
  };
  type BattleInput = {
    player: SideSetup; enemy: SideSetup;
    dawnRound: number; ambush?: boolean;
  };
  type BattleResult = {
    winner: 'player' | 'enemy';
    rounds: number;
    events: BattleEvent[];
    final: { player: SideSummary; enemy: SideSummary };
  };
  function runBattle(input: BattleInput): BattleResult;
  ```
  The engine computes the whole battle up front. The UI animates it by replaying
  `events`.
- **Content model:**
  - Cards are data: stats, keywords, and a list of triggered effects built from a small set
    of operations, for example:
    - `dealDamage {amount, ignoreGuardians?}`, `heal {amount}`
    - `summonToken {token, count}`
    - `crumbleOldest {side}`, `banishOldest {side, count}`, `banishWhere {minLife}`
    - `allAct {noLifeCost, bonusAttack?}`
    - `raiseTop {count}`, `consumeGraveyard {max, perCreature}`
    - `replayLast {count}`, `copyLeft {bonusAttack?}`, `halveGuard {side}`
  - Dynamic Attack (Lich) and auras (Bone Drummer) are declared properties.
  - Bespoke handlers by id are acceptable for odd cards.
- **Event log (minimum set).** Every event carries a sequence index.
  - `round_start {round}`, `turn_start {side, round}`
  - `dawn {side, amount, hpAfter}`
  - `special {side, specialId, details}`
  - `sigil_fired {side, slot, tile, pin, card}`
  - `card_played {side, card, source: 'deck'|'sigil'|'echo', deckIndex?}`
  - `slow_begun {side, card}`, `slow_fizzled {side, card}`
  - `summoned {side, creatureId, card|token, index, attack, life, guard?}`
  - `creature_acted {side, creatureId, extra: boolean}`
  - `damage {sourceSide, source: {kind:'creature'|'card'|'dawn', id}, absorbed:[{creatureId, amount}], toNecromancer, hpAfter}`
  - `heal {side, amount, hpAfter}`
  - `stat_changed {creatureId, attack?, life?, guard?, reason}`
  - `crumbled {side, creatureId, cause:'life'|'pushed'|'guard'|'effect'}`
  - `banished {side, creatureId}`
  - `graveyard_changed {side, size}`
  - `battle_end {winner, round}`

## 11. Simulator and tuning (required before UI polish)

### 11.1 CLI commands
- **`sim battle --deck <ids> --sigils <tile:pin:card,...> --enemy <id> [--hp 60] [--trace]`**
  prints the result and a readable log.
- **`sim sweep`** runs sample decks against every encounter and outputs win rate, average
  rounds, and average HP lost.
  - Sample decks are built from archetype templates, plus simulated drafts that use the
    run's reward rules.
- **`sim solve --deck <ids> --enemy <id> [--sigils …]`**
  - Enumerates every distinct permutation of the deck. Identical cards are deduplicated;
    ≤ 8 cards means ≤ 40,320 orders.
  - Optionally enumerates every pin notch for the given sigils.
  - Reports the best, median, and worst outcome.

### 11.2 Comparing outcomes
1. A win beats a loss.
2. Among wins, less HP lost is better, and ties go to fewer rounds.
3. Among losses, less enemy HP remaining is better.

### 11.3 Health metrics and initial targets

Write these out as a markdown/CSV report.

1. **Fight length:** normal fights median 6–10 rounds, with fewer than 10% reaching Dawn.
2. **HP lost per fight** (reasonable decks): normal 4–12, elite 10–25. The boss should be
   tight.
3. **Starter deck:** beats every easy-pool encounter, losing at most 15 HP each.
4. **Order sensitivity:** in at least 30% of sampled deck/enemy pairs (6–8 card decks), the
   best order saves at least 4 HP over the median order.
5. **Order stability** (pillar 5): an order optimized against one enemy lands in the top
   25% of orders against at least half of the other enemies.
6. **The questions are real:** for each encounter's intended answers (§7.2), swapping one
   answer card into a deck reduces average HP lost by at least 25% against that
   encounter.
7. **No dominant card:** across sweep-optimal decks, no single card appears in more than
   40%.
8. **Sigil value:** the best pin notch is not always the earliest one. Pin choice should
   change outcomes.

## 12. Required tests

### 12.1 Golden battle trace (normative)

- **Player:** HP 60/60, starter deck in the order from §5.4, no sigils, goes first.
- **Enemy:** Grave Robber, HP 20, loop `Shovel · Shovel · Lantern`.
- **Dawn round:** 12.

Labels in the trace:
- Player creatures: A–E are Skeletons, W1/W2 are Bone Walls, R1/R2 are Rats.
- Enemy creatures: L1/L2 are Lanterns.
- **gy** is the player's graveyard size.

| Rd | Side | Action | Creature actions and results | Robber HP | Player HP |
|---|---|---|---|---|---|
| 1 | P | Skeleton **A** | A hits 2 (A Life 2) | 18 | 60 |
| 1 | E | Shovel | Deals 3 | 18 | 57 |
| 2 | P | Skeleton **B** | A hits 2 (Life 1); B hits 2 (Life 2) | 14 | 57 |
| 2 | E | Shovel | Deals 3 | 14 | 54 |
| 3 | P | Bone Wall **W1** (Guard 6) | A hits 2, Life 0, crumbles (gy 1); B hits 2 (Life 1); W1 Life 3 | 10 | 54 |
| 3 | E | Lantern **L1** (Guard 4, Life 3) | L1 acts (Life 2) | 10 | 54 |
| 4 | P | Skeleton **C** | B's 2 absorbed by L1 (Guard 2); B crumbles (gy 2); W1 Life 2; C's 2 absorbed by L1 (Guard 0), L1 crumbles; C Life 2 | 10 | 54 |
| 4 | E | Shovel | 3 absorbed by W1 (Guard 3) | 10 | 54 |
| 5 | P | Bone Wall **W2** | W1 Life 1; C hits 2 (Life 1); W2 Life 3 | 8 | 54 |
| 5 | E | Shovel | 3 absorbed by W1 (Guard 0); W1 crumbles (gy 3) | 8 | 54 |
| 6 | P | Rat Swarm → **R1**, **R2** | C hits 2, crumbles (gy 4); W2 Life 2; R1 hits 1 (Life 2); R2 hits 1 (Life 2) | 4 | 54 |
| 6 | E | Lantern **L2** | L2 acts (Life 2) | 4 | 54 |
| 7 | P | Skeleton **D** (deck wraps) | W2 Life 1; R1's 1 absorbed (L2 Guard 3); R2's 1 absorbed (Guard 2); D's 2 absorbed (Guard 0), L2 crumbles; R1, R2 Life 1; D Life 2 | 4 | 54 |
| 7 | E | Shovel | 3 absorbed by W2 (Guard 3) | 4 | 54 |
| 8 | P | Skeleton **E** | W2 Life 0, crumbles (gy 5); R1 hits 1, crumbles (gy 6); R2 hits 1, crumbles (gy 7); D hits 2 → **Robber 0, player wins** (E never acts) | 0 | 54 |

**Final state:** winner `player`, round 8, player HP 54, player graveyard 7, enemy graveyard
2, player deck pointer 2.

### 12.2 Rule unit tests (each MUST exist)

1. **Push-out.** A board of 6 Skeletons, plus a 7th summoned: the oldest crumbles (graveyard
   +1, Crumble triggers queued), and the new one sits at index 5.
2. **Absorption overflow.** Guardians with Guard 3 and Guard 4 take a 9-damage attack: both
   crumble, and the necromancer takes 2, so it counts as a hit. A 7-damage attack would
   break both but would not be a hit.
3. **Spectral.** A Wraith ignores an opposing guardian and hits the necromancer for 2.
4. **Slow.** Death Knight played on turn t: no creature appears and a pending summon is shown.
   On turn t+1: no deck card is played and the pointer is unchanged; the Death Knight
   arrives and attacks for 6 that same turn, ending at Life 1.
5. **Sigil.**
   - Wounded at 50, with HP 30/60: fires instead of the deck card, and the pointer is
     unchanged.
   - The next turn, with HP still 30: it does not fire again.
   - With HP 31/60: it does not fire.
6. **Omen.** It's true on the turn after the opponent starts a Slow summon, and false once
   that summon has arrived.
7. **Banish.** Exorcism removes 2 creatures: the graveyard is unchanged, Plague Bearer's
   Crumble does not trigger, and Gravedigger does not grow.
8. **Mid-step summons.** Rat Nest's Rat, summoned during step 4, does not act that turn and
   acts on the next turn with Life 3.
9. **Crumble chain.** Plague Bearer reaching Life 0 summons 2 Rats via the queue. If the
   board was full, each Rat pushes out the oldest creature.
10. **Echo.**
    - After a deck Skeleton, Echo summons a Skeleton.
    - After a sigil-fired Bone Spear, Echo casts Bone Spear.
    - Echo directly after Echo replays the last non-Echo card.
11. **Frenzy.** Every creature acts an extra time without losing Life, and Grow applies on
    that extra action.
12. **Lich.** With 4 creatures in the owner's graveyard, it attacks for 5. After Corpse
    Explosion removes all of them, it attacks for 1.
13. **Dawn.** With dawnRound 12, the active necromancer loses 1 at the start of its round-12
    turn and 2 in round 13. This damage ignores guardians.
14. **Mirror Shade.**
    - With a Hellhound to its left, it becomes a Hellhound (Attack 4, Life 2), and no Arrive
      triggers.
    - Summoned onto an empty board, it becomes a Shade (1/2).
15. **Grave Theft.** The player's top graveyard creature leaves their graveyard and appears
    on the Lich Lord's board. A raised Ghoul deals 5 to the player on arrival.
16. **Determinism.** The same `BattleInput` run twice produces identical event logs.

## 13. Milestones

| # | Milestone | Acceptance criteria |
|---|---|---|
| M0 | Scaffold | TS strict, Vite, Vitest, lint/format, CI running tests |
| M1 | Engine + content | All §4–7 rules and content implemented; content validation; §12 tests green; `sim battle --trace` prints readable logs |
| M2 | Simulator + first tuning pass | `sim sweep` and `sim solve` working; §11.3 report generated; numbers adjusted toward targets, with changes noted in the report |
| M3 | Battle viewer + sandbox | Any deck/sigils vs any encounter, animated from the event log, with all §9.3 juice, §9.4 audio, speed controls, log panel, and reduced motion |
| M4 | Run loop | Map gen with constraints, pre-fight screen, rewards with insertion, rest sites with deck and sigil editors, elite sigil unlocks, boss known from the start, save/continue, run-end summary |
| M5 | Polish + tuning | Playtest feedback applied; re-run the §11.3 report; no known rule bugs |

**Order of work:** get the rules right in the engine and simulator before investing in
visuals. Once M1 passes, the viewer should *feel* good early (M3), because whether watching
is fun is one of the core questions.

## 14. Out of scope for v0

- Shops and gold
- Relics
- Events
- Acts 2–3
- Meta-progression and unlocks
- Real art and audio assets
- Accounts
- Mobile packaging
- PvP

**Planned later** (don't build now, but don't design against them):
- **Daily seeded run with a leaderboard.** Everyone plays the same seed. Determinism makes
  this fair.
- **Ghost bosses.** Winning player decks appear as bosses in other players' runs.
- **Async PvP** against ghost decks, if a player base ever exists.
- **Relics** that bend rules, such as "your first sigil can fire twice" or "Echo replays
  upgraded".

## 15. Tuning knobs and open questions

- **Knobs:**
  - Dawn round (12)
  - board cap (6)
  - starting HP (60) and rest-site heal (30%)
  - Rat stats (1/3)
  - all card and enemy numbers
  - rarity weights
  - map node weights
  - how fast sigil slots unlock (currently on elite wins)
- **Open questions:**
  - Should the player always act first in PvE, or should some normal fights use `ambush`
    for variety?
  - Is a starting deck of 6 cards the right size, given that loop length interacts with
    how often your best cards come around?
  - Should sigil cards also be allowed back into the deck mid-run outside rest sites?
    (Currently no.)
  - Does Frenzy need a cap on very wide boards (6 creatures acting twice)? Watch it in the
    simulator.
  - Should upgraded cards show visually on the board (e.g. a glow)? Probably yes; it's
    cheap.
