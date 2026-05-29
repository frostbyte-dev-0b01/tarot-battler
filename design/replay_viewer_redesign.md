# Replay Viewer Redesign

## Purpose

This document defines an improved design for the replay viewer dev tool in
`tools/ui/`. It builds on `design/ui_spec.md` and `design/replay_schema.md` and
focuses on a single goal: **make a battle legible to a human watching it back.**

The viewer already renders snapshots, a timeline, and an inspector. The redesign
keeps that state-driven foundation and fixes the things that make a replay hard
to read.

## Problems With the Current Viewer

1. **`heal` events render as raw JSON.** The engine emits `type: "heal"`, but the
   viewer only formats `"healing"`, so heals dump `JSON.stringify(event)` into the
   timeline and current-event line, and are excluded from the "major" filter.
2. **Timeline filters are unreachable.** The major-only / selected-only inputs are
   `visually-hidden` and wired to a `.toggle-pill` element that does not exist.
3. **The event slider is hidden**, despite the spec requiring an event-index
   scrubber.
4. **Inspector and log cannot be seen at the same time.** They share one 300px
   rail behind an accordion, so the timeline is both cramped and hidden whenever
   you inspect a unit.
5. **Board cards are low-information.** HP/MP are 3px bars with no numbers, and
   there is no indication of statuses, conditions, or defeat cause on the card.
6. **Narration is de-emphasized and uncolored.** The "what just happened" line is a
   small, muted, right-aligned string with no color coding, and ability damage is
   phrased as "deals N ability damage".

## Design Principles

- **State first, log second.** Continue rendering board and inspector from
  `snapshots`; the log explains, it does not reconstruct.
- **Always-on context.** Board, inspector, and timeline should be visible together
  on a normal desktop width.
- **Color carries meaning.** Allies blue, enemies orange, damage red, healing
  green, status/buff purple — applied consistently in narration, timeline, and
  cards.
- **Numbers where they matter.** Show current/max HP and MP as text, not just bar
  fill.
- **No raw JSON ever reaches the player.** Every emitted event type has a readable
  template.

## Layout

A persistent three-column stage on desktop, collapsing to a single stacked column
on narrow screens:

```
+-------------+-----------------------------+-------------+
|  Inspector  |  time chips + legend        |  Battle Log |
|  (selected  |  NARRATION BANNER           |  filters:   |
|   unit)     |  +-----------------------+  |  All/Major/ |
|             |  |     Battle Board      |  |  Selected   |
|             |  +-----------------------+  |  grouped    |
|             |  prev play next restart    |  timeline   |
|             |  speed   slider   step     |             |
+-------------+-----------------------------+-------------+
```

- **Inspector column (left):** details for the selected unit — identity, HP/MP
  bars with numbers, passive, focus target, effective stats with deltas, statuses
  and conditions, and the unit's rule list.
- **Main column (center):**
  - **Time chips:** `Tick N` and `Step X / Total`, clearly distinct.
  - **Legend:** small source/target swatches so the board glows are
    self-explanatory.
  - **Narration banner:** the prominent, centered, color-coded description of the
    current event.
  - **Battle board:** both teams on the depth grid, source/target glow for the
    current event, defeated units shown in place but muted with an `✕` marker.
  - **Controls bar:** prev / play-pause / next / restart, speed buttons, a visible
    event-index slider, and the step counter.
- **Timeline column (right):** the full event log grouped by tick, with a working
  All / Major / Selected segmented filter and the selected event highlighted and
  auto-scrolled into view.

## Unit Card

Each board card shows:

- display name and a portrait glyph
- **HP and MP bars with `current/max` numbers**
- compact **status/condition chips** (name + stack count) with tooltips
- defeated state: muted card plus an `✕` marker
- source/target highlight when involved in the current event

## Narration and Timeline Text

- Add a `heal` template (alias of `healing`) and keep `mp_restore`.
- Phrase ability damage using the ability/source name when `damage_kind` is
  `"ability"` (e.g. "deals 12 damage to Justice with Charge") instead of "ability
  damage". Keep `physical` / `magical` wording when present.
- Color-code numeric magnitudes: **damage red**, **healing/MP green**, and status
  names in the **buff purple** family, in both the narration banner and timeline.
- Every event type the engine emits (`battle_start`, `turn_start`, `basic_attack`,
  `ability_used`, `damage`, `heal`, `mp_restore`, `status_applied`,
  `condition_applied`, `status_removed`, `status_tick`, `passive_triggered`,
  `turn_skipped`, `retargeted`, `moved`, `defeat`, `battle_end`) must have a
  readable template — no `JSON.stringify` fallback in practice.

## Playback Controls

- Visible **event-index slider** scrubbing by event, not tick.
- prev / play-pause / next / restart.
- Speed selector (0.5x / 1x / 2x / 4x).
- Step counter `current / total`.
- Keyboard: left/right (and a/d) step through events.

## Out of Scope

- Bespoke status/condition icon art (chips with text are enough for now).
- Damage/heal animations or particle effects.
- Any team-builder changes.

## Implementation Order

1. Event-text fidelity fixes (`heal`, ability-damage phrasing, color spans).
2. Persistent three-column layout (replace the accordion rail).
3. Richer unit cards (numbers + status chips + defeat marker).
4. Prominent color-coded narration banner.
5. Working timeline filter control and visible event slider.
</content>
</invoke>
