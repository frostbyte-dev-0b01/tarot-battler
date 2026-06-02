# Metagame: Seasons, Pods, and the Draft

## Status

**Planned design — not yet implemented.** This describes the competitive run/arc
loop that wraps the battle engine. The battle engine, team builder, and replay
viewer already exist; this is the layer that gives them stakes, an arc, and a
reason to return.

## Design goals

- Turn the flat "submit a team → watch numbers → tweak" loop into a **run with an
  arc**: a beginning (commit a direction), rising action (a toolkit that grows),
  and a climax (finals) with persistent stakes (your tier).
- Reward high-effort team-craft over **a whole season**, not a single day.
- Personalize the experience without gating **power** behind play hours (tiers
  gate *content/complexity*, which is skill-gated and fair within a tier; raw
  strength is never time-gated).
- Recreate the **draft tension** that async play usually loses, by revealing the
  toolkit gradually over the season.

## The loop at a glance

- The unit of play is a **season = one month**, spent in a **pod** of same-tier
  players.
- You field and continuously refine **one team** across the season — you don't
  rebuild from scratch; you re-script and occasionally draft new tools.
- **Twice-weekly draft beats** on a fixed calendar schedule grow your toolkit and
  budget in small steps.
- **Daily** async battles against pod-mates accumulate **points**.
- Your **end-of-season point total** maps to fixed **tier thresholds** → your tier
  next season.
- The season ends in a cross-pod **Victors tournament** for cosmetics/titles;
  retired teams enter the **Hall of Honor** and return as **ghosts**.

## Pods & tiers

- **Tiers** (tarot-themed names TBD) form a ladder. Each tier has its **own draft
  option pool**: lower tiers offer a smaller, simpler selection (gentle
  onboarding); higher tiers unlock more characters / items / passives / mechanics.
  Higher-tier content should add **complexity and variety, not raw power**.
- A **pod** = ~20–50 players of the same tier, together for the whole season. It
  is the social + standings unit (a league table of rivals), not necessarily the
  content unit.
- Pod thinness is padded with **ghosts** (retired teams) so a pod always feels
  populated.

## The season arc (fixed draft schedule)

- **Opens small:** a starter set of ~5 characters and a budget to field ~3.
  Starting at a 3-character budget keeps week 1 easy to grok and fast to build.
- **Twice-weekly draft beats** on fixed calendar days (e.g. the 1st and 2nd pick
  each week), each a **small** choice — two small decisions a week rather than one
  big choice at the end.
- Beats follow a **fixed, learnable schedule** that alternates types, e.g.:
  character pick · passive pick · budget increase + item pick · character pick ·
  passive + budget increase · swap · … The *rhythm* is fixed and learnable; the
  *content* (which options appear) rotates per season (and possibly per pod).
- **Budget grows over the season** (3 → full team). The season is therefore its
  own difficulty ramp — simple early, complex by finals — which doubles as
  onboarding every month, for new players and veterans alike.
- **Divergence / per-pod metas** emerge from which options each player drafts (and,
  if offers vary per pod, from which options are even available). Because
  promotion is relative within a pod, differing pools do **not** create
  unfairness — only **internal viability** matters, so the per-pod/tier option
  generation must be curated to guarantee 2–3 viable archetypes and exclude
  degenerate combos.

## Drafting rules

- You must **claim** each draft pick before the next beat is revealed, or it is
  **auto-chosen at random** from that beat's offered options.
- **Exception — swaps are skipped, not auto-applied.** If a *swap* draft goes
  unclaimed, nothing is swapped; a character is never swapped against the player's
  intent.

## Adaptation model (soft inertia, not lock-in)

- The roster's **identity persists**; the primary adaptation lever is
  **re-scripting** rules / focus policy / positioning — cheap, expressive, and the
  heart of the authorship layer. The same four characters can answer a meta shift
  through doctrine changes alone.
- The **toolkit grows slowly** via drafts; full rebuilds are rare by design.
- **No explicit swap-budget in v1** — inertia is emergent (the effort of
  re-scripting + the slow drip of drafts). Add a hard constraint later only if
  players reinvent too freely.

## Battles & scoring

- **Daily** async battles against pod-mates' submitted teams; ghosts fill gaps.
- Points accumulate over the season into a **legible league table** — you can see
  exactly who you beat and the points that moved, rather than a hidden rating.
- **Open question (see below):** reconcile a "zero-sum point exchange" with
  "absolute tier thresholds." A clean candidate is **accumulation** (e.g. win/draw
  = +points) so totals are absolute and threshold-friendly; pure zero-sum is
  relative and fights absolute thresholds.

## Promotion / relegation (threshold-based)

- Your tier next season is set by your **end-of-season point total** crossing
  **fixed thresholds** (e.g. > 1000 → Silver).
- You stay in your **current tier's pool all season** regardless of your running
  total; only the final total promotes or relegates you.
- **Rationale:** every point matters. Even in last place it is worth scoring a few
  more points (toward a threshold, or to avoid dropping below the relegation line),
  so there is never a reason to stop playing.

## Auto-submit & absence

- **Auto-submit is on by default** — your team keeps competing while you're away,
  so you can rarely "miss" a day.
- Missed draft picks auto-resolve at random (per the claim rule); missed swaps are
  skipped.
- True absence (no valid team at all) = **sit out, retain points, no decay** —
  absence should never punish; auto-submit makes decay unnecessary.

## End of season

- **Victors tournament:** the top teams from each pod auto-compete **cross-pod**.
  Champions **bring their season-built teams** (so the month-long build gets its
  showcase), and rewards are **cosmetics / titles / seeding only — never power.**
  This is also the one **shared spectacle** that unites otherwise-different pods.
- **Hall of Honor:** season teams are archived (a personal record + a global
  featured wall) and reused as the **ghosts** that populate pods and PvE.

## Open questions / concerns

1. **Scoring model.** Zero-sum exchange (relative, very legible) vs accumulation
   (absolute, fits the threshold promotion). Leaning accumulation. Either way, the
   thresholds must be tuned for **healthy** tier movement (only clear over/under-
   performers move; most players stay), and to mean roughly the same thing across
   pods.
2. **Per-tier vs per-pod options.** Do all pods in a tier share identical draft
   options (shared, discussable meta) or does each pod draw a different subset from
   the tier pool (more variety, more anti-netdeck, less shared learning)?
3. **Tier-gated content.** Ensure higher tiers grant **more complexity, not more
   power**, or the ladder becomes a power-creep climb.
4. **Mid-season newcomers.** Bulk of players re-pod each season boundary (synced).
   Newcomers reaching a tier mid-season: catch-up into a live pod (instant
   catch-up drafts, neutral standing, promotion-only that season) vs a rolling
   newcomer bracket. Leaning catch-up.
5. **Match-day granularity.** Daily continuous results vs one weekly round-robin
   event. (Currently: daily.)
6. **Calendar fairness.** Fixed pick days (e.g. "first Tuesday") across time zones;
   draft windows should be generous.
7. **Population** at high tiers and at season boundaries; ghost padding helps but
   matchmaking density is a real constraint early.

## Addendum — resolved decisions (follow-up)

### Scoring model — resolved (both zero-sum *and* absolute)

- Every player has a single persistent **point total** — the source of truth for
  their tier.
- A battle awards **+5 win / −5 loss / 0 draw**, exchanged **within the pod**, so
  it is simultaneously **zero-sum** (the pod's total is conserved) and
  **absolute** (your total is a real number a threshold can read).
- New players start at a **baseline** (e.g. 800). A tier is a **point band**
  (e.g. Bronze < 1000, Silver 1000–2000, …). At each season boundary, players are
  re-sorted into tier pods by their total. You stay in your pod all season; only
  the final total moves you.
- A **floor at 0** prevents negative totals (no points are lost below 0).
- Exact numbers (points-per-battle, battles-per-day, band widths) are **tuning**:
  calibrate so a season produces roughly one band of movement for clear over- and
  under-performers while most players hold their tier.

### Authorship & telemetry — direction (needs its own deep dive)

The expression payoff is **legible cause→effect**, never a "cleverness" grade.
Build toward:

- A **beefier post-game stats page**: per-team and **per-character** breakdowns
  (damage, healing, kills, abilities used, **rule firings**).
- **Aggregate stats across matches**: your record + stat lines **vs a specific
  opponent team** and **vs the field**, with trends over the season.
- **Per-rule telemetry**: how often each rule fired and what it led to
  (kills / saves), to drive iteration — outcomes, not a score.
- **Replay rule-pinning**: pin a rule / character / event while watching to mark
  and jump to each time it fires on the timeline.

These warrant a focused follow-up on *which metrics matter*.

### Tone — locked

- **Register: earnest-mystical with a wry, warm edge ("elegant occult").** Fate
  spoken with gravitas, but self-aware enough not to be pompous; the cards carry
  weight, the experience carries warmth.
- The **Major Arcana as a living pantheon** — each card's tarot meaning is its
  personality and colors its kit; **Reversed = its shadow aspect**.
- **Ambient voice-of-fate + the cards' own voices**, rather than a single mascot
  NPC.
- **Divination as the pervasive vocabulary** — battles are Readings, the meta is
  the Wheel turning, drafts are the cards fate deals you, a defeat is a card spent.
- Atmosphere **ennobles the mechanics** (determinism = fate is fixed once the
  cards are laid; the seeded turn-order = fortune's small turns) rather than
  masking them; the tactics stay legible beneath the mood.

