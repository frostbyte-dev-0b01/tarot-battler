# Combat Formula Rework — Implementation Plan

## Locked design decisions

1. **Ratio mitigation.** `damage = round(eff_attack × tier_mult × K / (K + defense))`, minimum 1.
   `K = 12`. No flat base damage on attacks. True damage and Omen bypass mitigation.
   Both the ability damage path and the basic/command-attack path use this formula.
2. **Four power tiers (enum, not float).** Stored in ability data as `"power"`:
   - `strike` ×1.0, `strong` ×1.5, `heavy` ×2.0, `execute` ×2.5.
   Players see pips/names, never decimals.
3. **Empower / Weaken are permanent** (no decay), removed only by dispel/cleanse/
   opposing-cancellation/consume. Capped at **8 stacks per stat**.
4. **Decay:** Omen → tick-down by 1. Restoration → stays halving (deliberate
   exception so sustain can't snowball). Lethality (dormant) → halving family.
   Conditions unchanged (tick down by 1, end of turn).
5. **Philosophy:** depth comes from synergies and trigger webs, not spreadsheet
   math. Recorded in the spec.

## Steps

1. **Docs** — update `game_spec.md` (Core Pillars note, Damage and Defense,
   Decay Model), `implementation_notes.md` (status vocabulary + decay), and this
   plan. Source of truth must match the locked decisions.
2. **PowerTier + formula** — add `PowerTier` enum with `multiplier()`; rewrite
   `scaled_damage_with_defense` to ratio (K=12), drop the `base_damage` param;
   add a shared `apply_mitigation` helper; route the command-attack and
   `execute_basic_attack_action` paths through it.
3. **Primitive fields** — replace `multiplier: f64` (+ `primary_/companion_`)
   with `power: PowerTier` and remove `base_damage` across the damage primitives;
   fix the call sites in `abilities.rs`.
4. **Statuses** — set Empower/Weaken `stack_type` to `permanent` in
   `statuses.json`; update `status_decay_rule` (Omen tick-down, Restoration
   halve-only, stat-mods permanent); add the per-stat stack cap in `add_status`.
5. **Ability data** — convert every damage primitive in `abilities.json` to a
   `power` tier, drop `base_damage`, and rewrite descriptions in tier language.
6. **Tests** — fix primitive constructors in `test_support.rs`; recompute the
   damage/decay expectations in `abilities_tests.rs` and `models.rs` against the
   new formula and decay model.
7. **Verify** — `cargo build`, `cargo test`, run a sample battle, eyeball a
   replay for sane damage numbers. Commit and push.

## Notes / risks

- The biggest churn is test expectation values; recompute, don't rubber-stamp.
- Permanent Empower (e.g. Pursuit granting +1 per ally hit on focus) snowballs by
  design; the 8-stack cap is the leash. Revisit application amounts during tuning.
- Removing flat base lowers some low-tier hits; ratio mitigation keeps them
  relevant (a Strike is never floored to 1 except against extreme defense).
</content>
