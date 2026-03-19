# Team B Implementation Plan

## Purpose

This file defines the engine work needed to support the current draft Team B characters:

- The Moon
- The Magician
- Justice

The goal is to align implementation with the drafted kits while keeping the rollout incremental and testable.

This plan assumes the Team B draft is still a working design. The intent is to identify the minimum engine work needed to make the roster playable and evaluable in real matchups against Team A.

## Team B Draft Summary

### The Moon

Passive:

- `Foreboding`
  When The Moon deals magical damage to an enemy, apply `Omen 1` to that enemy.

Abilities:

- `Hex`
  Deal magical damage to the user's current target. Apply `Omen 2` to that target.
- `Eclipse`
  Deal magical damage to the user's current target and all of that target's companions. Apply `Omen 1` to each damaged target.
- `Harvest Night`
  Deal magical damage to the user's current target. Consume all `Omen` on that target and add bonus damage equal to the consumed stacks.

### The Magician

Passive:

- `Catalyst`
  The first time each tick an ally applies `Omen` to an enemy, apply `Omen 1` to that enemy.

Abilities:

- `Channel`
  Restore MP to the user or one ally.
- `Distill`
  Remove 1 buff from the user's current target. Apply `Omen 2` to that target.
- `Transmute`
  Deal magical damage to the user's current target. If that target has `Omen`, restore `MP 2` to the user.

### Justice

Passive:

- `Sentence`
  When Justice deals damage to an enemy with `Omen`, apply `Weaken MGT 1` to that enemy.

Abilities:

- `Condemn`
  Deal physical damage to the user's current target. If that target has `Omen`, this attack deals bonus damage.
- `Verdict`
  Deal heavy damage to the user's current target. This ability can only be used if that target has at least `Omen 3`.
- `Rebuke`
  Force the user's current target to choose a new target using default retargeting.

## Current Engine Support

Already supported:

- sticky targets and default retargeting
- single-target and enemy-companion splash targeting
- physical and magical damage primitives
- MP restoration primitives
- status application and removal
- status-aware rule conditions such as `has_status` and `status_stacks`
- movement, retargeting, and replay support from Team A work

Partially supported:

- generic damage triggers exist, but there is no status-specific ally event trigger
- `Bleed` and `Poison` exist as prototype DOT statuses, but `Omen` is not yet implemented as the intended true-damage setup effect
- status ticking exists, but the intended halving-decay model for `Omen` is not in place

Not yet supported:

- a real `Omen` status definition with the intended semantics
- status consumption for bonus damage
- a passive trigger for "ally applies Omen"
- conditional bonus damage based on target status without fully consuming the status
- a clean way to model "while targeting Justice" passive behavior

## Design Decisions To Preserve

The implementation should preserve these gameplay intentions:

- Team B should fight on a setup-and-payoff axis rather than simple row buffs or raw stat pressure
- `Omen` should feel distinct from generic DOTs and should be the core status identity for the team
- Moon should both set up and cash out `Omen`
- Magician should improve consistency and tempo for the `Omen` plan rather than becoming the main carry
- Justice should be a frontline control anchor, not just another generic bruiser
- `Harvest Night` should visibly consume `Omen` rather than merely checking for it
- `Rebuke` should use the existing retargeting model rather than inventing a new control subsystem

## Recommended Rollout

## Phase 1: Real Omen Status

Status:

- implemented

### Goal

Replace placeholder prototype expectations with a first-class `Omen` status.

### Needed for

- `Foreboding`
- `Hex`
- `Eclipse`
- `Harvest Night`
- `Distill`
- `Transmute`
- `Verdict`
- `Sentence`

### Implementation

Add a new named status:

- `Omen`

Recommended v1 semantics:

- `Omen` is a stack-based debuff
- at the start of the target's turn, `Omen` deals true damage equal to its current stacks
- after triggering, `Omen` decays

Current v1 note:

- the current engine uses tick-down-by-1 for timed stacks
- the long-term design wants halving decay
- `Omen` now ships with current tick semantics and is explicitly treated as a temporary implementation compromise until status decay is redesigned

### Tests

- `Omen` deals unmitigated damage at turn start
- `Omen` damages before the target acts
- `Omen` can kill and still respects normal death resolution
- `Omen` stack decay is explicitly tested for whatever v1 behavior is chosen

## Phase 2: Omen Consumption Payoff

Status:

- implemented

### Goal

Support abilities that consume `Omen` stacks for bonus damage.

### Needed for

- `Harvest Night`

### Implementation

Add a primitive or primitive rider such as:

- `consume_status_for_bonus_damage`

Recommended v1 behavior:

- target one enemy
- read current `Omen` stacks
- remove all `Omen` from that target
- add bonus damage equal to the removed stack count

This can be implemented either:

- as a dedicated primitive for `Harvest Night`
- or as a more reusable status-consumption primitive with:
  - `status`
  - `consume_all`
  - `bonus_per_stack`

Implemented result:

- the engine now supports a reusable magical consume-status damage primitive
- it reads current stacks of a named status, adds flat bonus damage per stack, then removes the status from the target
- this is sufficient for `Harvest Night` without introducing a wider general-purpose rider system yet

### Tests

- damage scales with consumed `Omen`
- all `Omen` is removed after the hit
- zero-stack use adds no bonus and consumes nothing
- death resolution still behaves correctly on the payoff hit

## Phase 3: Conditional Bonus Damage on Marked Targets

Status:

- implemented

### Goal

Support attacks that gain a damage rider if the target has `Omen`.

### Needed for

- `Condemn`

### Implementation

There are two reasonable v1 options:

1. rule-only version
- make `Condemn` a stronger attack that Justice only uses when the target has enough `Omen`
- simplest content path, but less expressive

2. engine-supported conditional rider
- add a damage rider like:
  - if target has status X, add flat or multiplicative bonus

Implemented result:

- the engine now supports a reusable physical damage primitive with a flat bonus against targets that have a named status
- this keeps `Condemn` data-driven without collapsing it into a pure rule-gated generic attack

### Tests

- bonus applies only when target has `Omen`
- bonus does not consume `Omen`
- damage remains normal when target lacks `Omen`

## Phase 4: Ally Applies Omen Trigger

Status:

- implemented

### Goal

Support passives that react when an ally applies `Omen` to an enemy.

### Needed for

- `Catalyst`

### Implementation

Add a new passive trigger concept:

- `on_ally_apply_omen`

Recommended semantics:

- fires when an ally successfully applies at least one stack of `Omen`
- does not fire on self-application by the passive owner unless explicitly intended
- fires after the original `Omen` application resolves
- should carry enough context to target the same enemy

Implemented result:

- the engine now logs real `status_applied` events
- passives can react to ally `Omen` applications through `on_ally_apply_omen`
- passives can target the just-marked enemy through `trigger_target`

Long-term note:

- like `on_ally_damage_my_target`, this is a pragmatic v1 trigger
- long-term design likely wants a broader ally-event trigger system with filters for event type and status name

### Tests

- ally applies `Omen` -> `Catalyst` adds one more stack
- self application behavior is explicitly tested
- non-`Omen` status applications do not trigger it
- multiple applications in a tick obey the intended "first time each tick" rule if that limit is implemented in passive data or engine logic

## Phase 5: Tick-Limited Ally-Event Passives

Status:

- implemented

### Goal

Support "first time each tick" limits on passive triggers.

### Needed for

- `Catalyst`

### Implementation

Add lightweight per-tick passive trigger tracking for a passive owner, such as:

- passive trigger fired this tick

Implemented result:

- triggered passives now support `once_per_tick` in data
- the battle engine tracks passive firings by owner, passive name, and tick
- this is sufficient to cap `Catalyst` cleanly without affecting unrelated passives

### Tests

- capped passive fires once per tick
- it can fire again on a later tick
- unrelated passives are not blocked

## Phase 6: While-Targeting Trigger or Simplification

Status:

- implemented

### Goal

Support Justice's passive if kept as written.

### Needed for

- `Sentence`

### Implementation

Original wording:

- enemies with `Omen` deal `Weaken MGT 1` to themselves while targeting Justice

Implemented design decision:

- simplify before coding instead of adding a targeting-state passive subsystem
- `Sentence` now reads:
  - "When Justice deals damage to an enemy with `Omen`, apply `Weaken MGT 1` to that enemy."

This keeps Justice omen-aware and control-oriented while fitting existing trigger and targeting surfaces.

### Tests

- covered during Team B data integration

## Phase 7: Data Integration

### Goal

Add the drafted Team B passives and abilities to the content catalogs and sample teams.

### Needed data additions

Passives:

- `Foreboding`
- `Catalyst`
- `Sentence`

Abilities:

- `Hex`
- `Eclipse`
- `Harvest Night`
- `Channel`
- `Distill`
- `Transmute`
- `Condemn`
- `Verdict`
- `Rebuke`

Statuses:

- `Omen`

### Sample formation

Recommended initial Team B sample:

- Justice in the front row
- Moon in the middle or back row
- Magician protected but close enough to support

The sample should demonstrate:

- steady `Omen` application
- at least one `Harvest Night` or `Verdict` payoff
- `Rebuke` changing target flow

### Tests

- sample content validation passes
- bundled battle runs successfully
- replay shows `Omen` application, ticking, and payoff clearly

## Phase 8: Replay and UI Support

### Goal

Ensure the replay viewer and team builder expose `Omen` clearly.

### Needed updates

- status display should make `Omen` stacks obvious
- current-event summaries should clearly show `Omen` application and consumption
- sample teams in the dev UI should update once Team B is implemented

### Recommendation

If `Omen` consumption gets its own replay event, add viewer support for it. If not, ensure the resulting status removal and bonus-damage sequence is still legible in the timeline.

## Recommended Implementation Order

1. Phase 1: real `Omen` status
2. Phase 2: `Omen` consumption payoff
3. Phase 3: conditional bonus damage on marked targets
4. Phase 4: ally-applies-`Omen` trigger
5. Phase 5: tick-limited ally-event passives if needed
6. Phase 6: simplify or support Justice passive
7. Phase 7: data integration
8. Phase 8: replay and UI support

## Suggested Milestones

### Milestone A

Moon is functional:

- `Omen` exists
- `Hex` and `Eclipse` apply it
- `Harvest Night` can consume it

### Milestone B

Magician meaningfully supports the omen plan:

- `Channel` and `Transmute` are useful
- `Catalyst` or its simplified equivalent improves consistency

### Milestone C

Justice completes the team:

- a final passive wording is chosen
- `Condemn`, `Verdict`, and `Rebuke` produce visible control/payoff moments
- the 3v3 matchup against Team A is ready for replay-based evaluation

## Main Risks

- `Omen` semantics may drift if the engine still uses prototype status timing while the design moves toward halving decay
- `Catalyst` can become noisy or overtuned if passive trigger limiting is not defined clearly
- Justice's drafted passive is the least implementation-ready part of Team B and may need simplification before coding
- if `Harvest Night` bonus damage is not logged clearly, replay readability will suffer even if the mechanic is correct
