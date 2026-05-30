//! Ability definitions and execution.

use std::collections::HashMap;

use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::abilities_targeting::{
    resolve_ally_targets, resolve_enemy_targets, retarget_filter_matches, target_is_enemy_side,
};
use crate::logger::{BattleEvent, BattleLog};
use crate::models::{CharacterState, ConditionKind, Stat};
use crate::statuses::{StatusGroup, StatusMap, status_key};
use crate::targeting::select_target;

/// Simple target categories used by existing sample data.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SimpleAbilityTarget {
    CurrentTarget,
    CurrentTargetAndCompanions,
    TriggerTarget,
    TriggerAlly,
    BoundAlly,
    BoundEnemy,
    #[serde(rename = "self")]
    SelfChar,
    Companions,
    FrontRow,
    AllEnemies,
    AllAllies,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetCategory {
    Companion,
    Ally,
    Enemy,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionalCondition {
    Frontmost,
    Backmost,
    SameRow,
    SameColumn,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TargetSelector {
    HighestStat {
        stat: Stat,
    },
    LowestStat {
        stat: Stat,
    },
    HighestHp,
    LowestHp,
    HighestMp,
    LowestMp,
    MostStacks {
        status: String,
        #[serde(default)]
        stat: Option<Stat>,
    },
    FewestStacks {
        status: String,
        #[serde(default)]
        stat: Option<Stat>,
    },
    HasStatus {
        status: String,
        #[serde(default)]
        stat: Option<Stat>,
    },
    LacksStatus {
        status: String,
        #[serde(default)]
        stat: Option<Stat>,
    },
    HasCondition {
        condition: String,
    },
    LacksCondition {
        condition: String,
    },
    MostBuffs,
    Random,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RetargetMode {
    ToSelf,
    ToCompanion,
    DefaultRetarget,
    Disorient,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RetargetFilter {
    PhysicalAttackers,
    TargetingSelf,
    TargetingCompanion,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MoveDirection {
    Forward,
    Backward,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TargetSpec {
    pub category: TargetCategory,
    #[serde(default)]
    pub selector: Option<TargetSelector>,
    #[serde(default)]
    pub position: Option<PositionalCondition>,
    #[serde(default)]
    pub bypass_row_protection: bool,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct StatusRef {
    pub status: String,
    #[serde(default)]
    pub stat: Option<Stat>,
}

/// Who the ability primitive targets.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum AbilityTarget {
    Simple(SimpleAbilityTarget),
    Detailed(TargetSpec),
}

impl From<SimpleAbilityTarget> for AbilityTarget {
    fn from(value: SimpleAbilityTarget) -> Self {
        Self::Simple(value)
    }
}

/// Named attack-power tiers. Power is expressed as one of four tiers rather than
/// a free-form multiplier, so the player-facing language is pips/names instead of
/// decimals. The multipliers live only in data and tuning.
#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerTier {
    Light,
    Medium,
    Heavy,
    Ultimate,
}

impl PowerTier {
    pub fn multiplier(self) -> f64 {
        match self {
            PowerTier::Light => 1.0,
            PowerTier::Medium => 1.5,
            PowerTier::Heavy => 2.0,
            PowerTier::Ultimate => 2.5,
        }
    }

    /// Number of pips, for UI / replay presentation.
    pub fn pips(self) -> u8 {
        match self {
            PowerTier::Light => 1,
            PowerTier::Medium => 2,
            PowerTier::Heavy => 3,
            PowerTier::Ultimate => 4,
        }
    }
}

/// A single primitive effect composing an ability.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Primitive {
    BindTargets {
        target: AbilityTarget,
        primitives: Vec<Primitive>,
    },
    DealPhysicalDamage {
        target: AbilityTarget,
        power: PowerTier,
        #[serde(default)]
        double_empower_stat: Option<Stat>,
    },
    DealPhysicalDamageBonusVsStatus {
        target: AbilityTarget,
        power: PowerTier,
        status: String,
        #[serde(default)]
        stat: Option<Stat>,
        bonus_damage: u32,
    },
    DealMagicalDamage {
        target: AbilityTarget,
        power: PowerTier,
    },
    DealMagicalDamageCurrentTargetAndCompanions {
        primary_power: PowerTier,
        companion_power: PowerTier,
    },
    DealTrueDamage {
        target: AbilityTarget,
        amount: u32,
    },
    DealTrueDamageConsumeTargetStatus {
        target: AbilityTarget,
        status: String,
        #[serde(default)]
        stat: Option<Stat>,
        damage_per_stack: u32,
    },
    DealMagicalDamageConsumeStatus {
        target: AbilityTarget,
        power: PowerTier,
        status: String,
        #[serde(default)]
        stat: Option<Stat>,
        bonus_per_stack: u32,
    },
    DealPhysicalDamageConsumeSelfStatuses {
        target: AbilityTarget,
        power: PowerTier,
        statuses: Vec<StatusRef>,
        bonus_per_stack: u32,
    },
    DealTrueDamageConsumeSelfStatuses {
        target: AbilityTarget,
        statuses: Vec<StatusRef>,
        damage_per_stack: u32,
    },
    LoseCurrentHpPercent {
        percent: u32,
    },
    ApplyHaste {
        target: AbilityTarget,
        amount: u32,
    },
    RestoreHp {
        target: AbilityTarget,
        amount: u32,
    },
    RestoreMp {
        target: AbilityTarget,
        amount: u32,
    },
    ApplyStatus {
        target: AbilityTarget,
        status: String,
        #[serde(default)]
        stat: Option<Stat>,
        stacks: u32,
    },
    ApplyCondition {
        target: AbilityTarget,
        condition: String,
        stacks: u32,
    },
    RemoveStatus {
        target: AbilityTarget,
        status: String,
        #[serde(default)]
        stat: Option<Stat>,
        stacks: u32,
    },
    RemoveCondition {
        target: AbilityTarget,
        condition: String,
        stacks: u32,
    },
    SetFocusToTarget {
        target: AbilityTarget,
    },
    RemoveOneBuff {
        target: AbilityTarget,
    },
    Cleanse {
        target: AbilityTarget,
        amount: u32,
        #[serde(default)]
        group: Option<StatusGroup>,
    },
    Dispel {
        target: AbilityTarget,
        amount: u32,
        #[serde(default)]
        group: Option<StatusGroup>,
    },
    Retarget {
        target: AbilityTarget,
        mode: RetargetMode,
        #[serde(default)]
        filter: Option<RetargetFilter>,
    },
    RetargetEnemiesFocusingTargets {
        target: AbilityTarget,
        mode: RetargetMode,
    },
    RefocusSelf,
    RefocusAlliesToActorsTarget {
        target: AbilityTarget,
    },
    CommandAttack,
    Move {
        direction: MoveDirection,
        #[serde(default = "default_true")]
        if_empty: bool,
    },
    MoveTarget {
        target: AbilityTarget,
        direction: MoveDirection,
        #[serde(default = "default_true")]
        if_empty: bool,
    },
    TransformStatuses {
        target: AbilityTarget,
        from_status: String,
        to_status: String,
        stats: Vec<Stat>,
    },
    CleanseAndApplyStatusIfChanged {
        target: AbilityTarget,
        amount: u32,
        status: String,
        #[serde(default)]
        stat: Option<Stat>,
        stacks: u32,
    },
    IfTargetHasStatus {
        target: AbilityTarget,
        status: String,
        #[serde(default)]
        stat: Option<Stat>,
        primitives: Vec<Primitive>,
    },
    IfTargetLacksStatus {
        target: AbilityTarget,
        status: String,
        #[serde(default)]
        stat: Option<Stat>,
        primitives: Vec<Primitive>,
    },
    IfTargetHasCondition {
        target: AbilityTarget,
        condition: String,
        primitives: Vec<Primitive>,
    },
    IfTargetLacksCondition {
        target: AbilityTarget,
        condition: String,
        primitives: Vec<Primitive>,
    },
    IfTargetHasNoCompanions {
        target: AbilityTarget,
        primitives: Vec<Primitive>,
    },
    IfTargetHasAnyCondition {
        target: AbilityTarget,
        primitives: Vec<Primitive>,
    },
    IfTargetHpAtOrBelowPercent {
        target: AbilityTarget,
        percent: u32,
        primitives: Vec<Primitive>,
    },
    PurgeStatuses {
        target: AbilityTarget,
        statuses: Vec<StatusRef>,
        #[serde(default)]
        damage_target: Option<AbilityTarget>,
        #[serde(default)]
        damage_per_removed: u32,
    },
    CopyPassiveFromAlly {
        target: AbilityTarget,
    },
}

/// A complete ability definition.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AbilityDef {
    pub mp_cost: u32,
    pub primitives: Vec<Primitive>,
}

pub type AbilityMap = HashMap<String, AbilityDef>;

/// When a triggered passive fires.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
pub enum PassiveTrigger {
    OnBattleStart,
    OnDeath,
    OnKill,
    OnDealDamage,
    OnBasicAttack,
    OnTakeDamage,
    OnSelfBelowHalfHp,
    OnCompanionBelowHalfHp,
    OnTurnStart,
    OnAllyDamageMyTarget,
    OnRowAllyDamageMyTarget,
    OnAllyApplyOmen,
    OnAffectAllyWithAbility,
    OnFocusChanged,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AuraStatEffect {
    pub stat: Stat,
    pub amount: i32,
}

/// A passive ability definition — either a triggered effect or a permanent trait.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PassiveDef {
    Triggered {
        trigger: PassiveTrigger,
        #[serde(default)]
        once_per_tick: bool,
        #[serde(default)]
        once_per_battle: bool,
        primitives: Vec<Primitive>,
    },
    Trait {
        effect: crate::models::TraitEffect,
    },
    RowAura {
        effects: Vec<AuraStatEffect>,
    },
}

pub type PassiveMap = HashMap<String, PassiveDef>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DamageRecord {
    pub source_id: u32,
    pub target_id: u32,
    pub damage: u32,
}

pub struct ExecutionContext<'a> {
    pub actor_team: &'a mut [CharacterState],
    pub enemy_team: &'a mut [CharacterState],
    pub rng: &'a mut StdRng,
    pub log: &'a mut BattleLog,
    pub step: u32,
    pub status_defs: &'a StatusMap,
    pub actor_team_is_a: bool,
    pub trigger_target_id: Option<u32>,
    pub bound_ally_target_ids: Option<Vec<u32>>,
    pub bound_enemy_target_ids: Option<Vec<u32>>,
}

impl<'a> ExecutionContext<'a> {
    pub fn new(
        actor_team: &'a mut [CharacterState],
        enemy_team: &'a mut [CharacterState],
        rng: &'a mut StdRng,
        log: &'a mut BattleLog,
        step: u32,
        status_defs: &'a StatusMap,
        actor_team_is_a: bool,
    ) -> Self {
        Self {
            actor_team,
            enemy_team,
            rng,
            log,
            step,
            status_defs,
            actor_team_is_a,
            trigger_target_id: None,
            bound_ally_target_ids: None,
            bound_enemy_target_ids: None,
        }
    }

    pub fn with_trigger_target(mut self, trigger_target_id: Option<u32>) -> Self {
        self.trigger_target_id = trigger_target_id;
        self
    }
}

fn resolve_enemy_targets_for_context(
    ctx: &mut ExecutionContext<'_>,
    target: &AbilityTarget,
    actor_idx: usize,
) -> Vec<usize> {
    match target {
        AbilityTarget::Simple(SimpleAbilityTarget::BoundEnemy) => ctx
            .bound_enemy_target_ids
            .as_ref()
            .into_iter()
            .flatten()
            .filter_map(|id| ctx.enemy_team.iter().position(|c| c.id() == *id && c.is_alive()))
            .collect(),
        _ => resolve_enemy_targets(
            target,
            actor_idx,
            ctx.actor_team,
            ctx.enemy_team,
            ctx.rng,
            ctx.trigger_target_id,
        ),
    }
}

fn resolve_ally_targets_for_context(
    ctx: &mut ExecutionContext<'_>,
    target: &AbilityTarget,
    actor_idx: usize,
) -> Vec<usize> {
    match target {
        AbilityTarget::Simple(SimpleAbilityTarget::BoundAlly) => ctx
            .bound_ally_target_ids
            .as_ref()
            .into_iter()
            .flatten()
            .filter_map(|id| ctx.actor_team.iter().position(|c| c.id() == *id && c.is_alive()))
            .collect(),
        AbilityTarget::Simple(SimpleAbilityTarget::TriggerAlly) => ctx
            .trigger_target_id
            .and_then(|id| ctx.actor_team.iter().position(|c| c.id() == id && c.is_alive()))
            .into_iter()
            .collect(),
        _ => resolve_ally_targets(target, actor_idx, ctx.actor_team, ctx.rng),
    }
}

/// Execute an ability's primitives using a shared execution context.
///
/// Returns a list of damage records for defeat checking by the caller.
pub fn execute_ability_with_context(
    ctx: &mut ExecutionContext<'_>,
    actor_idx: usize,
    ability_name: &str,
    ability: &AbilityDef,
) -> Vec<DamageRecord> {
    let actor_id = ctx.actor_team[actor_idx].id();
    let actor_name = ctx.actor_team[actor_idx].base_name().to_string();

    ctx.log.push(BattleEvent::AbilityUsed {
        tick_count: ctx.step,
        actor_id,
        actor_name,
        ability_name: ability_name.to_string(),
        mp_cost: ability.mp_cost,
    });
    capture_context_snapshot(ctx);

    execute_primitives_with_context(ctx, actor_idx, ability_name, &ability.primitives)
}

/// Execute an ability's primitives.
///
/// Returns a list of damage records for defeat checking by the caller.
#[cfg_attr(not(test), allow(dead_code))]
#[allow(clippy::too_many_arguments)]
pub fn execute_ability(
    actor_idx: usize,
    ability_name: &str,
    ability: &AbilityDef,
    actor_team: &mut [CharacterState],
    enemy_team: &mut [CharacterState],
    rng: &mut StdRng,
    log: &mut BattleLog,
    step: u32,
    status_defs: &StatusMap,
) -> Vec<DamageRecord> {
    execute_ability_for_side(
        actor_idx,
        ability_name,
        ability,
        actor_team,
        enemy_team,
        true,
        rng,
        log,
        step,
        status_defs,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn execute_ability_for_side(
    actor_idx: usize,
    ability_name: &str,
    ability: &AbilityDef,
    actor_team: &mut [CharacterState],
    enemy_team: &mut [CharacterState],
    actor_team_is_a: bool,
    rng: &mut StdRng,
    log: &mut BattleLog,
    step: u32,
    status_defs: &StatusMap,
) -> Vec<DamageRecord> {
    let mut ctx = ExecutionContext::new(actor_team, enemy_team, rng, log, step, status_defs, actor_team_is_a);
    execute_ability_with_context(&mut ctx, actor_idx, ability_name, ability)
}

/// Execute a list of primitives (shared by abilities and passives) using a shared execution
/// context. Returns a list of damage records for defeat checking.
pub fn execute_primitives_with_context(
    ctx: &mut ExecutionContext<'_>,
    actor_idx: usize,
    _source_name: &str,
    primitives: &[Primitive],
) -> Vec<DamageRecord> {
    let mut damage_dealt: Vec<DamageRecord> = Vec::new();

    let actor_id = ctx.actor_team[actor_idx].id();

    // Pre-compute actor offensive stats for damage calculation
    let actor_int = ctx.actor_team[actor_idx].get_eff_stat(&Stat::MAG);

    for primitive in primitives {
        match primitive {
            Primitive::BindTargets { target, primitives } => {
                let previous_bound_allies = ctx.bound_ally_target_ids.clone();
                let previous_bound_enemies = ctx.bound_enemy_target_ids.clone();

                let bound_ally_ids = if target_is_enemy_side(target) {
                    None
                } else {
                    Some(
                        resolve_ally_targets_for_context(ctx, target, actor_idx)
                            .into_iter()
                            .filter_map(|idx| ctx.actor_team.get(idx))
                            .filter(|character| character.is_alive())
                            .map(CharacterState::id)
                            .collect::<Vec<_>>(),
                    )
                };
                let bound_enemy_ids = if target_is_enemy_side(target) {
                    Some(
                        resolve_enemy_targets_for_context(ctx, target, actor_idx)
                            .into_iter()
                            .filter_map(|idx| ctx.enemy_team.get(idx))
                            .filter(|character| character.is_alive())
                            .map(CharacterState::id)
                            .collect::<Vec<_>>(),
                    )
                } else {
                    None
                };

                ctx.bound_ally_target_ids = bound_ally_ids;
                ctx.bound_enemy_target_ids = bound_enemy_ids;
                let nested_damage =
                    execute_primitives_with_context(ctx, actor_idx, _source_name, primitives);
                damage_dealt.extend(nested_damage);
                ctx.bound_ally_target_ids = previous_bound_allies;
                ctx.bound_enemy_target_ids = previous_bound_enemies;
            }
            Primitive::DealPhysicalDamage {
                target,
                power,
                double_empower_stat,
            } => {
                let actor_str = match double_empower_stat {
                    Some(Stat::MGT) => ctx.actor_team[actor_idx].get_eff_stat_with_doubled_empower(&Stat::MGT),
                    _ => ctx.actor_team[actor_idx].get_eff_stat(&Stat::MGT),
                };
                let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                for tidx in target_indices {
                    let defender_for = ctx.enemy_team[tidx].get_eff_stat(&Stat::ARM);
                    let raw_damage =
                        scaled_damage_with_defense(actor_str, power.multiplier(), defender_for);
                    let damage = ctx.enemy_team[tidx].take_hit(raw_damage);
                    let tid = ctx.enemy_team[tidx].id();
                    let tname = ctx.enemy_team[tidx].base_name().to_string();
                    let hp = ctx.enemy_team[tidx].current_hp();
                    ctx.log.push(BattleEvent::AbilityDamage {
                        tick_count: ctx.step,
                        actor_id,
                        target_id: tid,
                        target_name: tname,
                        damage,
                        target_hp_remaining: hp,
                        damage_kind: "physical".to_string(),
                    });
                    capture_context_snapshot(ctx);
                    damage_dealt.push(DamageRecord {
                        source_id: actor_id,
                        target_id: tid,
                        damage,
                    });
                }
            }
            Primitive::DealPhysicalDamageBonusVsStatus {
                target,
                power,
                status,
                stat,
                bonus_damage,
            } => {
                let key = status_key(status, stat.as_ref());
                let actor_mgt = ctx.actor_team[actor_idx].get_eff_stat(&Stat::MGT);
                let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                for tidx in target_indices {
                    let defender_arm = ctx.enemy_team[tidx].get_eff_stat(&Stat::ARM);
                    let mut raw_damage = scaled_damage_with_defense(
                        actor_mgt,
                        power.multiplier(),
                        defender_arm,
                    );
                    if ctx.enemy_team[tidx].has_status(&key) {
                        raw_damage = raw_damage.saturating_add(*bonus_damage);
                    }
                    let damage = ctx.enemy_team[tidx].take_hit(raw_damage);
                    let tid = ctx.enemy_team[tidx].id();
                    let tname = ctx.enemy_team[tidx].base_name().to_string();
                    let hp = ctx.enemy_team[tidx].current_hp();
                    ctx.log.push(BattleEvent::AbilityDamage {
                        tick_count: ctx.step,
                        actor_id,
                        target_id: tid,
                        target_name: tname,
                        damage,
                        target_hp_remaining: hp,
                        damage_kind: "physical".to_string(),
                    });
                    capture_context_snapshot(ctx);
                    damage_dealt.push(DamageRecord {
                        source_id: actor_id,
                        target_id: tid,
                        damage,
                    });
                }
            }
            Primitive::DealMagicalDamage { target, power } => {
                let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                for tidx in target_indices {
                    let defender_wis = ctx.enemy_team[tidx].get_eff_stat(&Stat::RES);
                    let raw_damage =
                        scaled_damage_with_defense(actor_int, power.multiplier(), defender_wis);
                    let damage = ctx.enemy_team[tidx].take_hit(raw_damage);
                    let tid = ctx.enemy_team[tidx].id();
                    let tname = ctx.enemy_team[tidx].base_name().to_string();
                    let hp = ctx.enemy_team[tidx].current_hp();
                    ctx.log.push(BattleEvent::AbilityDamage {
                        tick_count: ctx.step,
                        actor_id,
                        target_id: tid,
                        target_name: tname,
                        damage,
                        target_hp_remaining: hp,
                        damage_kind: "magical".to_string(),
                    });
                    capture_context_snapshot(ctx);
                    damage_dealt.push(DamageRecord {
                        source_id: actor_id,
                        target_id: tid,
                        damage,
                    });
                }
            }
            Primitive::DealMagicalDamageCurrentTargetAndCompanions {
                primary_power,
                companion_power,
            } => {
                let Some(target_id) = ctx.actor_team[actor_idx].target() else {
                    continue;
                };
                let Some(target_idx) = ctx.enemy_team.iter().position(|c| c.id() == target_id && c.is_alive()) else {
                    continue;
                };

                let companion_ids = ctx.enemy_team[target_idx].effective_companion_ids();
                let companion_indices: Vec<usize> = companion_ids
                    .iter()
                    .filter_map(|id| {
                        ctx.enemy_team
                            .iter()
                            .position(|c| c.id() == *id && c.is_alive())
                    })
                    .collect();
                let target_and_companions: Vec<usize> = std::iter::once(target_idx)
                    .chain(companion_indices.into_iter())
                    .collect();

                for tidx in target_and_companions {
                    let multiplier = if tidx == target_idx {
                        primary_power.multiplier()
                    } else {
                        companion_power.multiplier()
                    };
                    let defender_res = ctx.enemy_team[tidx].get_eff_stat(&Stat::RES);
                    let raw_damage =
                        scaled_damage_with_defense(actor_int, multiplier, defender_res);
                    let damage = ctx.enemy_team[tidx].take_hit(raw_damage);
                    log_ability_damage(ctx, actor_id, tidx, damage, true, "magical");
                    damage_dealt.push(DamageRecord {
                        source_id: actor_id,
                        target_id: ctx.enemy_team[tidx].id(),
                        damage,
                    });
                }
            }
            Primitive::DealTrueDamage { target, amount } => {
                if target_is_enemy_side(target) {
                    let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                    for tidx in target_indices {
                        let damage = ctx.enemy_team[tidx].current_hp().min(*amount);
                        ctx.enemy_team[tidx].take_damage(*amount);
                        log_ability_damage(ctx, actor_id, tidx, damage, true, "true");
                        damage_dealt.push(DamageRecord {
                            source_id: actor_id,
                            target_id: ctx.enemy_team[tidx].id(),
                            damage,
                        });
                    }
                } else {
                    let target_indices = resolve_ally_targets_for_context(ctx, target, actor_idx);
                    for tidx in target_indices {
                        let damage = ctx.actor_team[tidx].current_hp().min(*amount);
                        ctx.actor_team[tidx].take_damage(*amount);
                        log_ability_damage(ctx, actor_id, tidx, damage, false, "true");
                        damage_dealt.push(DamageRecord {
                            source_id: actor_id,
                            target_id: ctx.actor_team[tidx].id(),
                            damage,
                        });
                    }
                }
            }
            Primitive::DealTrueDamageConsumeTargetStatus {
                target,
                status,
                stat,
                damage_per_stack,
            } => {
                let key = status_key(status, stat.as_ref());
                let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                for tidx in target_indices {
                    let consumed_stacks = ctx.enemy_team[tidx].status_stacks(&key);
                    if consumed_stacks > 0 {
                        ctx.enemy_team[tidx].remove_status(&key, consumed_stacks);
                    }
                    let raw_damage = consumed_stacks.saturating_mul(*damage_per_stack);
                    let damage = ctx.enemy_team[tidx].current_hp().min(raw_damage);
                    ctx.enemy_team[tidx].take_damage(raw_damage);
                    log_ability_damage(ctx, actor_id, tidx, damage, true, "true");
                    damage_dealt.push(DamageRecord {
                        source_id: actor_id,
                        target_id: ctx.enemy_team[tidx].id(),
                        damage,
                    });
                }
            }
            Primitive::DealMagicalDamageConsumeStatus {
                target,
                power,
                status,
                stat,
                bonus_per_stack,
            } => {
                let key = status_key(status, stat.as_ref());
                let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                for tidx in target_indices {
                    let defender_res = ctx.enemy_team[tidx].get_eff_stat(&Stat::RES);
                    let consumed_stacks = ctx.enemy_team[tidx].status_stacks(&key);
                    let raw_damage = scaled_damage_with_defense(
                        actor_int,
                        power.multiplier(),
                        defender_res,
                    )
                        + consumed_stacks.saturating_mul(*bonus_per_stack);
                    let damage = ctx.enemy_team[tidx].take_hit(raw_damage);
                    if consumed_stacks > 0 {
                        ctx.enemy_team[tidx].remove_status(&key, consumed_stacks);
                    }
                    let tid = ctx.enemy_team[tidx].id();
                    let tname = ctx.enemy_team[tidx].base_name().to_string();
                    let hp = ctx.enemy_team[tidx].current_hp();
                    ctx.log.push(BattleEvent::AbilityDamage {
                        tick_count: ctx.step,
                        actor_id,
                        target_id: tid,
                        target_name: tname,
                        damage,
                        target_hp_remaining: hp,
                        damage_kind: "magical".to_string(),
                    });
                    capture_context_snapshot(ctx);
                    damage_dealt.push(DamageRecord {
                        source_id: actor_id,
                        target_id: tid,
                        damage,
                    });
                }
            }
            Primitive::DealTrueDamageConsumeSelfStatuses {
                target,
                statuses,
                damage_per_stack,
            } => {
                let consumed_stacks: u32 = statuses
                    .iter()
                    .map(|status_ref| {
                        let key = status_key(&status_ref.status, status_ref.stat.as_ref());
                        let stacks = ctx.actor_team[actor_idx].status_stacks(&key);
                        if stacks > 0 {
                            ctx.actor_team[actor_idx].remove_status(&key, stacks);
                        }
                        stacks
                    })
                    .sum();

                let raw_damage = consumed_stacks.saturating_mul(*damage_per_stack);
                let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                for tidx in target_indices {
                    let damage = ctx.enemy_team[tidx].current_hp().min(raw_damage);
                    ctx.enemy_team[tidx].take_damage(raw_damage);
                    log_ability_damage(ctx, actor_id, tidx, damage, true, "true");
                    damage_dealt.push(DamageRecord {
                        source_id: actor_id,
                        target_id: ctx.enemy_team[tidx].id(),
                        damage,
                    });
                }
            }
            Primitive::DealPhysicalDamageConsumeSelfStatuses {
                target,
                power,
                statuses,
                bonus_per_stack,
            } => {
                let actor_mgt = ctx.actor_team[actor_idx].get_eff_stat(&Stat::MGT);
                let consumed_stacks: u32 = statuses
                    .iter()
                    .map(|status_ref| {
                        let key = status_key(&status_ref.status, status_ref.stat.as_ref());
                        let stacks = ctx.actor_team[actor_idx].status_stacks(&key);
                        if stacks > 0 {
                            ctx.actor_team[actor_idx].remove_status(&key, stacks);
                        }
                        stacks
                    })
                    .sum();
                let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                for tidx in target_indices {
                    let defender_arm = ctx.enemy_team[tidx].get_eff_stat(&Stat::ARM);
                    let raw_damage = scaled_damage_with_defense(
                        actor_mgt,
                        power.multiplier(),
                        defender_arm,
                    )
                        + consumed_stacks.saturating_mul(*bonus_per_stack);
                    let damage = ctx.enemy_team[tidx].take_hit(raw_damage);
                    let tid = ctx.enemy_team[tidx].id();
                    let tname = ctx.enemy_team[tidx].base_name().to_string();
                    let hp = ctx.enemy_team[tidx].current_hp();
                    ctx.log.push(BattleEvent::AbilityDamage {
                        tick_count: ctx.step,
                        actor_id,
                        target_id: tid,
                        target_name: tname,
                        damage,
                        target_hp_remaining: hp,
                        damage_kind: "physical".to_string(),
                    });
                    capture_context_snapshot(ctx);
                    damage_dealt.push(DamageRecord {
                        source_id: actor_id,
                        target_id: tid,
                        damage,
                    });
                }
            }
            Primitive::LoseCurrentHpPercent { percent } => {
                let current_hp = ctx.actor_team[actor_idx].current_hp();
                let amount = current_hp.saturating_mul(*percent) / 100;
                if amount > 0 {
                    let damage = ctx.actor_team[actor_idx].current_hp().min(amount);
                    ctx.actor_team[actor_idx].take_damage(amount);
                    log_ability_damage(ctx, actor_id, actor_idx, damage, false, "true");
                    damage_dealt.push(DamageRecord {
                        source_id: actor_id,
                        target_id: ctx.actor_team[actor_idx].id(),
                        damage,
                    });
                }
            }
            Primitive::ApplyHaste { target, amount } => {
                if target_is_enemy_side(target) {
                    let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                    for tidx in target_indices {
                        if ctx.enemy_team[tidx].is_alive() {
                            ctx.enemy_team[tidx].apply_haste(*amount);
                        }
                    }
                } else {
                    let target_indices = resolve_ally_targets_for_context(ctx, target, actor_idx);
                    for tidx in target_indices {
                        if ctx.actor_team[tidx].is_alive() {
                            ctx.actor_team[tidx].apply_haste(*amount);
                        }
                    }
                }
                capture_context_snapshot(ctx);
            }
            Primitive::RestoreHp { target, amount } => {
                let target_indices = resolve_ally_targets_for_context(ctx, target, actor_idx);
                for tidx in target_indices {
                    if ctx.actor_team[tidx].is_alive() {
                        let before = ctx.actor_team[tidx].current_hp();
                        ctx.actor_team[tidx].heal(*amount);
                        let after = ctx.actor_team[tidx].current_hp();
                        let healed = after.saturating_sub(before);
                        if healed > 0 {
                            ctx.log.push(crate::logger::BattleEvent::AbilityHeal {
                                tick_count: ctx.step,
                                actor_id,
                                target_id: ctx.actor_team[tidx].id(),
                                target_name: ctx.actor_team[tidx].base_name().to_string(),
                                amount: healed,
                                target_hp_remaining: after,
                            });
                            capture_context_snapshot(ctx);
                        }
                    }
                }
            }
            Primitive::RestoreMp { target, amount } => {
                let target_indices = resolve_ally_targets_for_context(ctx, target, actor_idx);
                for tidx in target_indices {
                    if ctx.actor_team[tidx].is_alive() {
                        let before = ctx.actor_team[tidx].current_mp();
                        ctx.actor_team[tidx].restore_mp(*amount);
                        let after = ctx.actor_team[tidx].current_mp();
                        let restored = after.saturating_sub(before);
                        if restored > 0 {
                            ctx.log.push(crate::logger::BattleEvent::AbilityMpRestore {
                                tick_count: ctx.step,
                                actor_id,
                                target_id: ctx.actor_team[tidx].id(),
                                target_name: ctx.actor_team[tidx].base_name().to_string(),
                                amount: restored,
                                target_mp_after: after,
                            });
                            capture_context_snapshot(ctx);
                        }
                    }
                }
            }
            Primitive::ApplyStatus {
                target,
                status,
                stat,
                stacks,
            } => {
                if let Some(def) = ctx.status_defs.get(status) {
                    let key = status_key(status, stat.as_ref());
                    if target_is_enemy_side(target) {
                        let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                        for tidx in target_indices {
                            if ctx.enemy_team[tidx].is_alive() {
                                let applied = ctx.enemy_team[tidx].add_status(
                                    &key,
                                    *stacks,
                                    actor_id,
                                    def,
                                    stat.clone(),
                                );
                                if applied {
                                    ctx.log.push(BattleEvent::StatusApplied {
                                        tick_count: ctx.step,
                                        actor_id,
                                        actor_name: ctx.actor_team[actor_idx].base_name().to_string(),
                                        target_id: ctx.enemy_team[tidx].id(),
                                        target_name: ctx.enemy_team[tidx].base_name().to_string(),
                                        status_name: key.clone(),
                                        stacks_added: *stacks,
                                        stacks_after: ctx.enemy_team[tidx].status_stacks(&key),
                                    });
                                    capture_context_snapshot(ctx);
                                }
                            }
                        }
                    } else {
                        let target_indices = resolve_ally_targets_for_context(ctx, target, actor_idx);
                        for tidx in target_indices {
                            if ctx.actor_team[tidx].is_alive() {
                                let applied = ctx.actor_team[tidx].add_status(
                                    &key,
                                    *stacks,
                                    actor_id,
                                    def,
                                    stat.clone(),
                                );
                                if applied {
                                    log_status_applied(
                                        ctx,
                                        actor_id,
                                        actor_idx,
                                        tidx,
                                        &key,
                                        *stacks,
                                        false,
                                    );
                                }
                            }
                        }
                    }
                }
            }
            Primitive::ApplyCondition {
                target,
                condition,
                stacks,
            } => {
                let Some(kind) = ConditionKind::from_key(condition) else {
                    continue;
                };
                if target_is_enemy_side(target) {
                    let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                    for tidx in target_indices {
                        if ctx.enemy_team[tidx].is_alive()
                            && ctx.enemy_team[tidx].add_condition(kind, *stacks, actor_id)
                        {
                            log_condition_applied(
                                ctx,
                                actor_id,
                                actor_idx,
                                tidx,
                                kind,
                                *stacks,
                                true,
                            );
                        }
                    }
                } else {
                    let target_indices = resolve_ally_targets_for_context(ctx, target, actor_idx);
                    for tidx in target_indices {
                        if ctx.actor_team[tidx].is_alive()
                            && ctx.actor_team[tidx].add_condition(kind, *stacks, actor_id)
                        {
                            log_condition_applied(
                                ctx,
                                actor_id,
                                actor_idx,
                                tidx,
                                kind,
                                *stacks,
                                false,
                            );
                        }
                    }
                }
            }
            Primitive::RemoveStatus {
                target,
                status,
                stat,
                stacks,
            } => {
                let key = status_key(status, stat.as_ref());
                let actor_id = ctx.actor_team[actor_idx].id();
                if target_is_enemy_side(target) {
                    let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                    for tidx in target_indices {
                        let before = ctx.enemy_team[tidx].status_stacks(&key);
                        ctx.enemy_team[tidx].remove_status(&key, *stacks);
                        let after = ctx.enemy_team[tidx].status_stacks(&key);
                        if before > after {
                            log_status_removed(ctx, actor_id, actor_idx, tidx, &key, before - after, after, true);
                        }
                    }
                } else {
                    let target_indices = resolve_ally_targets_for_context(ctx, target, actor_idx);
                    for tidx in target_indices {
                        let before = ctx.actor_team[tidx].status_stacks(&key);
                        ctx.actor_team[tidx].remove_status(&key, *stacks);
                        let after = ctx.actor_team[tidx].status_stacks(&key);
                        if before > after {
                            log_status_removed(ctx, actor_id, actor_idx, tidx, &key, before - after, after, false);
                        }
                    }
                }
            }
            Primitive::RemoveCondition {
                target,
                condition,
                stacks,
            } => {
                let Some(kind) = ConditionKind::from_key(condition) else {
                    continue;
                };
                if target_is_enemy_side(target) {
                    let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                    for tidx in target_indices {
                        ctx.enemy_team[tidx].remove_condition(kind, *stacks);
                    }
                } else {
                    let target_indices = resolve_ally_targets_for_context(ctx, target, actor_idx);
                    for tidx in target_indices {
                        ctx.actor_team[tidx].remove_condition(kind, *stacks);
                    }
                }
            }
            Primitive::SetFocusToTarget { target } => {
                let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                if let Some(&tidx) = target_indices.first()
                    && ctx.enemy_team[tidx].is_alive()
                {
                    let new_target_id = ctx.enemy_team[tidx].id();
                    ctx.actor_team[actor_idx].set_target_tracked(new_target_id, ctx.step);
                    log_retarget_event(
                        ctx,
                        ctx.actor_team[actor_idx].id(),
                        ctx.actor_team[actor_idx].base_name().to_string(),
                        Some(new_target_id),
                        lookup_name(ctx.enemy_team, Some(new_target_id)),
                        "to_ability_target".to_string(),
                    );
                }
            }
            Primitive::RemoveOneBuff { target } => {
                let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                for tidx in target_indices {
                    ctx.enemy_team[tidx].remove_one_buff();
                }
            }
            Primitive::Cleanse {
                target,
                amount,
                group,
            } => {
                if target_is_enemy_side(target) {
                    let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                    for tidx in target_indices {
                        ctx.enemy_team[tidx].cleanse(*amount, *group);
                    }
                } else {
                    let target_indices =
                        resolve_ally_targets_for_context(ctx, target, actor_idx);
                    for tidx in target_indices {
                        ctx.actor_team[tidx].cleanse(*amount, *group);
                    }
                }
            }
            Primitive::Dispel {
                target,
                amount,
                group,
            } => {
                if target_is_enemy_side(target) {
                    let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                    for tidx in target_indices {
                        ctx.enemy_team[tidx].dispel(*amount, *group);
                    }
                } else {
                    let target_indices =
                        resolve_ally_targets_for_context(ctx, target, actor_idx);
                    for tidx in target_indices {
                        ctx.actor_team[tidx].dispel(*amount, *group);
                    }
                }
            }
            Primitive::Retarget {
                target,
                mode,
                filter,
            } => {
                let target_indices = if target_is_enemy_side(target) {
                    resolve_enemy_targets_for_context(ctx, target, actor_idx)
                } else {
                    Vec::new()
                };
                for tidx in target_indices {
                    if !ctx.enemy_team[tidx].is_alive()
                        || !retarget_filter_matches(
                            &ctx.enemy_team[tidx],
                            &ctx.actor_team[actor_idx],
                            filter.as_ref(),
                        )
                    {
                        continue;
                    }

                    let new_target = match mode {
                        RetargetMode::ToSelf => Some(ctx.actor_team[actor_idx].id()),
                        RetargetMode::ToCompanion => {
                            let mut living_companion_ids: Vec<u32> = ctx.actor_team[actor_idx]
                                .effective_companion_ids()
                                .iter()
                                .filter_map(|id| {
                                    ctx.actor_team
                                        .iter()
                                        .find(|c| c.id() == *id && c.is_alive())
                                        .map(|c| c.id())
                                })
                                .collect();
                            living_companion_ids.shuffle(ctx.rng);
                            living_companion_ids.into_iter().next()
                        }
                        RetargetMode::DefaultRetarget => {
                            ctx.enemy_team[tidx].clear_target();
                            select_target(&ctx.enemy_team[tidx], ctx.actor_team, ctx.rng)
                        }
                        RetargetMode::Disorient => {
                            ctx.enemy_team[tidx].clear_target();
                            crate::targeting::select_disoriented_target(
                                &ctx.enemy_team[tidx],
                                ctx.actor_team,
                                ctx.rng,
                            )
                        }
                    };

                    if let Some(new_target_id) = new_target {
                        ctx.enemy_team[tidx].set_target_tracked(new_target_id, ctx.step);
                    }

                    let new_target_name = new_target.and_then(|target_id| {
                        ctx.actor_team
                            .iter()
                            .find(|c| c.id() == target_id)
                            .map(|c| c.base_name().to_string())
                    });
                    ctx.log.push(BattleEvent::Retargeted {
                        tick_count: ctx.step,
                        character_id: ctx.enemy_team[tidx].id(),
                        character_name: ctx.enemy_team[tidx].base_name().to_string(),
                        new_target_id: ctx.enemy_team[tidx].target(),
                        new_target_name,
                        mode: retarget_mode_label(mode).to_string(),
                    });
                    capture_context_snapshot(ctx);
                }
            }
            Primitive::RetargetEnemiesFocusingTargets { target, mode } => {
                let focused_target_ids: Vec<u32> =
                    resolve_ally_targets_for_context(ctx, target, actor_idx)
                        .into_iter()
                        .filter_map(|idx| ctx.actor_team.get(idx))
                        .filter(|character| character.is_alive())
                        .map(CharacterState::id)
                        .collect();
                if focused_target_ids.is_empty() {
                    continue;
                }

                for tidx in 0..ctx.enemy_team.len() {
                    if !ctx.enemy_team[tidx].is_alive()
                        || !ctx.enemy_team[tidx]
                            .target()
                            .is_some_and(|focus_id| focused_target_ids.contains(&focus_id))
                    {
                        continue;
                    }

                    let new_target = compute_retarget_target_id(
                        mode,
                        &ctx.enemy_team[tidx],
                        &ctx.actor_team[actor_idx],
                        ctx.actor_team,
                        ctx.rng,
                    );
                    if let Some(new_target_id) = new_target {
                        ctx.enemy_team[tidx].set_target_tracked(new_target_id, ctx.step);
                    }

                    log_retarget_event(
                        ctx,
                        ctx.enemy_team[tidx].id(),
                        ctx.enemy_team[tidx].base_name().to_string(),
                        ctx.enemy_team[tidx].target(),
                        lookup_name(ctx.actor_team, ctx.enemy_team[tidx].target()),
                        retarget_mode_label(mode).to_string(),
                    );
                }
            }
            Primitive::RefocusSelf => {
                let actor_clone = ctx.actor_team[actor_idx].clone();
                let new_target = select_target(&actor_clone, ctx.enemy_team, ctx.rng);
                if let Some(new_target_id) = new_target {
                    ctx.actor_team[actor_idx].set_target_tracked(new_target_id, ctx.step);
                }
                log_retarget_event(
                    ctx,
                    ctx.actor_team[actor_idx].id(),
                    ctx.actor_team[actor_idx].base_name().to_string(),
                    ctx.actor_team[actor_idx].target(),
                    lookup_name(ctx.enemy_team, ctx.actor_team[actor_idx].target()),
                    "default_retarget".to_string(),
                );
            }
            Primitive::RefocusAlliesToActorsTarget { target } => {
                let Some(actor_target_id) = ctx.actor_team[actor_idx].target() else {
                    continue;
                };
                let target_name = lookup_name(ctx.enemy_team, Some(actor_target_id));
                let target_indices = resolve_ally_targets_for_context(ctx, target, actor_idx);
                for tidx in target_indices {
                    if !ctx.actor_team[tidx].is_alive() {
                        continue;
                    }
                    ctx.actor_team[tidx].set_target_tracked(actor_target_id, ctx.step);
                    log_retarget_event(
                        ctx,
                        ctx.actor_team[tidx].id(),
                        ctx.actor_team[tidx].base_name().to_string(),
                        Some(actor_target_id),
                        target_name.clone(),
                        "to_actors_target".to_string(),
                    );
                }
            }
            Primitive::CommandAttack => {
                let target_id = match ctx.actor_team[actor_idx].target() {
                    Some(target_id) => target_id,
                    None => continue,
                };
                let Some(target_idx) = ctx.enemy_team.iter().position(|c| c.id() == target_id) else {
                    continue;
                };
                let Some(companion_idx) = ctx.actor_team[actor_idx]
                    .effective_companion_ids()
                    .iter()
                    .filter_map(|id| ctx.actor_team.iter().position(|c| c.id() == *id && c.is_alive()))
                    .max_by_key(|idx| ctx.actor_team[*idx].get_eff_stat(&Stat::MGT))
                else {
                    continue;
                };

                let attacker_str = ctx.actor_team[companion_idx].get_eff_stat(&Stat::MGT);
                let defender_for = ctx.enemy_team[target_idx].get_eff_stat(&Stat::ARM);
                // Commanded attack is a Light-tier physical hit.
                let raw_damage =
                    scaled_damage_with_defense(attacker_str, PowerTier::Light.multiplier(), defender_for);
                let damage = ctx.enemy_team[target_idx].take_hit(raw_damage);
                let source_id = ctx.actor_team[companion_idx].id();
                let source_name = ctx.actor_team[companion_idx].base_name().to_string();
                let hp = ctx.enemy_team[target_idx].current_hp();
                ctx.log.push(BattleEvent::BasicAttack {
                    tick_count: ctx.step,
                    actor_id: source_id,
                    actor_name: source_name,
                    target_id,
                    target_name: ctx.enemy_team[target_idx].base_name().to_string(),
                    damage,
                    target_hp_remaining: hp,
                    mp_restored: 0,
                    actor_mp_after: ctx.actor_team[companion_idx].current_mp(),
                });
                capture_context_snapshot(ctx);
                damage_dealt.push(DamageRecord {
                    source_id,
                    target_id,
                    damage,
                });
            }
            Primitive::Move {
                direction,
                if_empty,
            } => {
                try_move_character(
                    actor_idx,
                    ctx.actor_team,
                    ctx.enemy_team,
                    ctx.actor_team_is_a,
                    direction,
                    *if_empty,
                    ctx.log,
                    ctx.step,
                );
            }
            Primitive::MoveTarget {
                target,
                direction,
                if_empty,
            } => {
                let target_indices = resolve_ally_targets_for_context(ctx, target, actor_idx);
                for tidx in target_indices {
                    try_move_character(
                        tidx,
                        ctx.actor_team,
                        ctx.enemy_team,
                        ctx.actor_team_is_a,
                        direction,
                        *if_empty,
                        ctx.log,
                        ctx.step,
                    );
                }
            }
            Primitive::TransformStatuses {
                target,
                from_status,
                to_status,
                stats,
            } => {
                let Some(to_def) = ctx.status_defs.get(to_status) else {
                    continue;
                };
                if target_is_enemy_side(target) {
                    let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                    for tidx in target_indices {
                        transform_statuses(
                            &mut ctx.enemy_team[tidx],
                            from_status,
                            to_status,
                            stats,
                            actor_id,
                            to_def,
                        );
                    }
                } else {
                    let target_indices =
                        resolve_ally_targets_for_context(ctx, target, actor_idx);
                    for tidx in target_indices {
                        transform_statuses(
                            &mut ctx.actor_team[tidx],
                            from_status,
                            to_status,
                            stats,
                            actor_id,
                            to_def,
                        );
                    }
                }
            }
            Primitive::CleanseAndApplyStatusIfChanged {
                target,
                amount,
                status,
                stat,
                stacks,
            } => {
                let Some(def) = ctx.status_defs.get(status) else {
                    continue;
                };
                let key = status_key(status, stat.as_ref());
                if target_is_enemy_side(target) {
                    let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                    for tidx in target_indices {
                        if ctx.enemy_team[tidx].cleanse(*amount, None) {
                            let applied = ctx.enemy_team[tidx].add_status(
                                &key,
                                *stacks,
                                actor_id,
                                def,
                                stat.clone(),
                            );
                            if applied {
                                log_status_applied(ctx, actor_id, actor_idx, tidx, &key, *stacks, true);
                            }
                        }
                    }
                } else {
                    let target_indices =
                        resolve_ally_targets_for_context(ctx, target, actor_idx);
                    for tidx in target_indices {
                        if ctx.actor_team[tidx].cleanse(*amount, None) {
                            let applied = ctx.actor_team[tidx].add_status(
                                &key,
                                *stacks,
                                actor_id,
                                def,
                                stat.clone(),
                            );
                            if applied {
                                log_status_applied(
                                    ctx,
                                    actor_id,
                                    actor_idx,
                                    tidx,
                                    &key,
                                    *stacks,
                                    false,
                                );
                            }
                        }
                    }
                }
            }
            Primitive::IfTargetHasStatus {
                target,
                status,
                stat,
                primitives,
            } => {
                let key = status_key(status, stat.as_ref());
                let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                let should_execute = target_indices
                    .iter()
                    .any(|tidx| ctx.enemy_team[*tidx].is_alive() && ctx.enemy_team[*tidx].has_status(&key));
                if should_execute {
                    let nested_damage =
                        execute_primitives_with_context(ctx, actor_idx, _source_name, primitives);
                    damage_dealt.extend(nested_damage);
                }
            }
            Primitive::IfTargetLacksStatus {
                target,
                status,
                stat,
                primitives,
            } => {
                let key = status_key(status, stat.as_ref());
                let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                let should_execute = target_indices
                    .iter()
                    .any(|tidx| ctx.enemy_team[*tidx].is_alive() && !ctx.enemy_team[*tidx].has_status(&key));
                if should_execute {
                    let nested_damage =
                        execute_primitives_with_context(ctx, actor_idx, _source_name, primitives);
                    damage_dealt.extend(nested_damage);
                }
            }
            Primitive::IfTargetHasCondition {
                target,
                condition,
                primitives,
            } => {
                let Some(kind) = ConditionKind::from_key(condition) else {
                    continue;
                };
                let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                let should_execute = target_indices.iter().any(|tidx| {
                    ctx.enemy_team[*tidx].is_alive() && ctx.enemy_team[*tidx].has_condition(kind)
                });
                if should_execute {
                    let nested_damage =
                        execute_primitives_with_context(ctx, actor_idx, _source_name, primitives);
                    damage_dealt.extend(nested_damage);
                }
            }
            Primitive::IfTargetLacksCondition {
                target,
                condition,
                primitives,
            } => {
                let Some(kind) = ConditionKind::from_key(condition) else {
                    continue;
                };
                let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                let should_execute = target_indices.iter().any(|tidx| {
                    ctx.enemy_team[*tidx].is_alive() && !ctx.enemy_team[*tidx].has_condition(kind)
                });
                if should_execute {
                    let nested_damage =
                        execute_primitives_with_context(ctx, actor_idx, _source_name, primitives);
                    damage_dealt.extend(nested_damage);
                }
            }
            Primitive::IfTargetHasNoCompanions { target, primitives } => {
                let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                let should_execute = target_indices.iter().any(|tidx| {
                    ctx.enemy_team[*tidx].is_alive()
                        && ctx.enemy_team[*tidx].effective_companion_ids().is_empty()
                });
                if should_execute {
                    let nested_damage =
                        execute_primitives_with_context(ctx, actor_idx, _source_name, primitives);
                    damage_dealt.extend(nested_damage);
                }
            }
            Primitive::IfTargetHasAnyCondition { target, primitives } => {
                let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                let should_execute = target_indices.iter().any(|tidx| {
                    ctx.enemy_team[*tidx].is_alive() && !ctx.enemy_team[*tidx].conditions().is_empty()
                });
                if should_execute {
                    let nested_damage =
                        execute_primitives_with_context(ctx, actor_idx, _source_name, primitives);
                    damage_dealt.extend(nested_damage);
                }
            }
            Primitive::IfTargetHpAtOrBelowPercent {
                target,
                percent,
                primitives,
            } => {
                let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                let should_execute = target_indices.iter().any(|tidx| {
                    let target = &ctx.enemy_team[*tidx];
                    let max_hp = target.get_base_stat(&Stat::VIT) * 3;
                    target.is_alive() && target.current_hp() * 100 <= max_hp.saturating_mul(*percent)
                });
                if should_execute {
                    let nested_damage =
                        execute_primitives_with_context(ctx, actor_idx, _source_name, primitives);
                    damage_dealt.extend(nested_damage);
                }
            }
            Primitive::PurgeStatuses {
                target,
                statuses,
                damage_target,
                damage_per_removed,
            } => {
                let mut removed_total = 0;
                if target_is_enemy_side(target) {
                    let target_indices = resolve_enemy_targets_for_context(ctx, target, actor_idx);
                    for tidx in target_indices {
                        for status_ref in statuses {
                            let key = status_key(&status_ref.status, status_ref.stat.as_ref());
                            let stacks = ctx.enemy_team[tidx].status_stacks(&key);
                            if stacks > 0 {
                                ctx.enemy_team[tidx].remove_status(&key, stacks);
                                removed_total += stacks;
                            }
                        }
                    }
                } else {
                    let target_indices = resolve_ally_targets_for_context(ctx, target, actor_idx);
                    for tidx in target_indices {
                        for status_ref in statuses {
                            let key = status_key(&status_ref.status, status_ref.stat.as_ref());
                            let stacks = ctx.actor_team[tidx].status_stacks(&key);
                            if stacks > 0 {
                                ctx.actor_team[tidx].remove_status(&key, stacks);
                                removed_total += stacks;
                            }
                        }
                    }
                }
                if removed_total > 0 {
                    capture_context_snapshot(ctx);
                }
                if removed_total > 0 && *damage_per_removed > 0 {
                    if let Some(damage_target) = damage_target {
                        let target_indices = resolve_enemy_targets_for_context(ctx, damage_target, actor_idx);
                        if let Some(&tidx) = target_indices.first()
                            && ctx.enemy_team[tidx].is_alive()
                        {
                            let amount = removed_total * *damage_per_removed;
                            let damage = ctx.enemy_team[tidx].take_hit(amount);
                            ctx.log.push(BattleEvent::AbilityDamage {
                                tick_count: ctx.step,
                                actor_id,
                                target_id: ctx.enemy_team[tidx].id(),
                                target_name: ctx.enemy_team[tidx].base_name().to_string(),
                                damage,
                                target_hp_remaining: ctx.enemy_team[tidx].current_hp(),
                                damage_kind: "true".to_string(),
                            });
                            capture_context_snapshot(ctx);
                            damage_dealt.push(DamageRecord {
                                source_id: actor_id,
                                target_id: ctx.enemy_team[tidx].id(),
                                damage,
                            });
                        }
                    }
                }
            }
            Primitive::CopyPassiveFromAlly { target } => {
                let target_indices = resolve_ally_targets_for_context(ctx, target, actor_idx);
                if let Some(&tidx) = target_indices.first()
                    && ctx.actor_team[tidx].is_alive()
                {
                    let passive_name = ctx.actor_team[tidx].passive().to_string();
                    ctx.actor_team[actor_idx].gain_bonus_passive(&passive_name);
                    capture_context_snapshot(ctx);
                }
            }
        }
    }

    damage_dealt
}

/// Execute a list of primitives (shared by abilities and passives).
///
/// Returns a list of damage records for defeat checking.
#[allow(dead_code, clippy::too_many_arguments)]
pub fn execute_primitives(
    actor_idx: usize,
    source_name: &str,
    primitives: &[Primitive],
    actor_team: &mut [CharacterState],
    enemy_team: &mut [CharacterState],
    rng: &mut StdRng,
    log: &mut BattleLog,
    step: u32,
    status_defs: &StatusMap,
    trigger_target_id: Option<u32>,
) -> Vec<DamageRecord> {
    let mut ctx = ExecutionContext::new(actor_team, enemy_team, rng, log, step, status_defs, true)
        .with_trigger_target(trigger_target_id);
    execute_primitives_with_context(&mut ctx, actor_idx, source_name, primitives)
}

fn retarget_mode_label(mode: &RetargetMode) -> &'static str {
    match mode {
        RetargetMode::ToSelf => "to_self",
        RetargetMode::ToCompanion => "to_companion",
        RetargetMode::DefaultRetarget => "default_retarget",
        RetargetMode::Disorient => "disorient",
    }
}

fn lookup_name(team: &[CharacterState], target_id: Option<u32>) -> Option<String> {
    target_id.and_then(|id| {
        team.iter()
            .find(|character| character.id() == id)
            .map(|character| character.base_name().to_string())
    })
}

fn log_ability_damage(
    ctx: &mut ExecutionContext<'_>,
    actor_id: u32,
    target_idx: usize,
    damage: u32,
    enemy_side: bool,
    damage_kind: &str,
) {
    let (target_id, target_name, target_hp_remaining) = if enemy_side {
        (
            ctx.enemy_team[target_idx].id(),
            ctx.enemy_team[target_idx].base_name().to_string(),
            ctx.enemy_team[target_idx].current_hp(),
        )
    } else {
        (
            ctx.actor_team[target_idx].id(),
            ctx.actor_team[target_idx].base_name().to_string(),
            ctx.actor_team[target_idx].current_hp(),
        )
    };

    ctx.log.push(BattleEvent::AbilityDamage {
        tick_count: ctx.step,
        actor_id,
        target_id,
        target_name,
        damage,
        target_hp_remaining,
        damage_kind: damage_kind.to_string(),
    });
    capture_context_snapshot(ctx);
}

fn log_retarget_event(
    ctx: &mut ExecutionContext<'_>,
    character_id: u32,
    character_name: String,
    new_target_id: Option<u32>,
    new_target_name: Option<String>,
    mode: String,
) {
    ctx.log.push(BattleEvent::Retargeted {
        tick_count: ctx.step,
        character_id,
        character_name,
        new_target_id,
        new_target_name,
        mode,
    });
    capture_context_snapshot(ctx);
}

fn log_status_applied(
    ctx: &mut ExecutionContext<'_>,
    actor_id: u32,
    actor_idx: usize,
    target_idx: usize,
    key: &str,
    stacks_added: u32,
    enemy_side: bool,
) {
    let (target_id, target_name, stacks_after) = if enemy_side {
        (
            ctx.enemy_team[target_idx].id(),
            ctx.enemy_team[target_idx].base_name().to_string(),
            ctx.enemy_team[target_idx].status_stacks(key),
        )
    } else {
        (
            ctx.actor_team[target_idx].id(),
            ctx.actor_team[target_idx].base_name().to_string(),
            ctx.actor_team[target_idx].status_stacks(key),
        )
    };

    ctx.log.push(BattleEvent::StatusApplied {
        tick_count: ctx.step,
        actor_id,
        actor_name: ctx.actor_team[actor_idx].base_name().to_string(),
        target_id,
        target_name,
        status_name: key.to_string(),
        stacks_added,
        stacks_after,
    });
    capture_context_snapshot(ctx);
}

#[allow(clippy::too_many_arguments)]
fn log_status_removed(
    ctx: &mut ExecutionContext<'_>,
    actor_id: u32,
    actor_idx: usize,
    target_idx: usize,
    key: &str,
    stacks_removed: u32,
    stacks_after: u32,
    enemy_side: bool,
) {
    let (target_id, target_name) = if enemy_side {
        (
            ctx.enemy_team[target_idx].id(),
            ctx.enemy_team[target_idx].base_name().to_string(),
        )
    } else {
        (
            ctx.actor_team[target_idx].id(),
            ctx.actor_team[target_idx].base_name().to_string(),
        )
    };

    ctx.log.push(BattleEvent::StatusRemoved {
        tick_count: ctx.step,
        actor_id,
        actor_name: ctx.actor_team[actor_idx].base_name().to_string(),
        target_id,
        target_name,
        status_name: key.to_string(),
        stacks_removed,
        stacks_after,
    });
    capture_context_snapshot(ctx);
}

fn log_condition_applied(
    ctx: &mut ExecutionContext<'_>,
    actor_id: u32,
    actor_idx: usize,
    target_idx: usize,
    kind: ConditionKind,
    stacks_added: u32,
    enemy_side: bool,
) {
    let (target_id, target_name, stacks_after) = if enemy_side {
        (
            ctx.enemy_team[target_idx].id(),
            ctx.enemy_team[target_idx].base_name().to_string(),
            ctx.enemy_team[target_idx].condition_stacks(kind),
        )
    } else {
        (
            ctx.actor_team[target_idx].id(),
            ctx.actor_team[target_idx].base_name().to_string(),
            ctx.actor_team[target_idx].condition_stacks(kind),
        )
    };

    ctx.log.push(BattleEvent::ConditionApplied {
        tick_count: ctx.step,
        actor_id,
        actor_name: ctx.actor_team[actor_idx].base_name().to_string(),
        target_id,
        target_name,
        condition_name: kind.as_key().to_string(),
        stacks_added,
        stacks_after,
    });
    capture_context_snapshot(ctx);
}

fn compute_retarget_target_id(
    mode: &RetargetMode,
    affected_character: &CharacterState,
    actor: &CharacterState,
    actor_team: &[CharacterState],
    rng: &mut StdRng,
) -> Option<u32> {
    match mode {
        RetargetMode::ToSelf => Some(actor.id()),
        RetargetMode::ToCompanion => {
            let mut living_companion_ids: Vec<u32> = actor
                .effective_companion_ids()
                .iter()
                .filter_map(|id| {
                    actor_team
                        .iter()
                        .find(|c| c.id() == *id && c.is_alive())
                        .map(|c| c.id())
                })
                .collect();
            living_companion_ids.shuffle(rng);
            living_companion_ids.into_iter().next()
        }
        RetargetMode::DefaultRetarget => {
            let mut retarget_source = affected_character.clone();
            retarget_source.clear_target();
            select_target(&retarget_source, actor_team, rng)
        }
        RetargetMode::Disorient => {
            let mut retarget_source = affected_character.clone();
            retarget_source.clear_target();
            crate::targeting::select_disoriented_target(&retarget_source, actor_team, rng)
        }
    }
}

fn capture_context_snapshot(ctx: &mut ExecutionContext<'_>) {
    if ctx.actor_team_is_a {
        ctx.log.capture_latest_snapshot(ctx.actor_team, ctx.enemy_team);
    } else {
        ctx.log.capture_latest_snapshot(ctx.enemy_team, ctx.actor_team);
    }
}

/// Ratio-mitigation constant. A defense equal to K roughly halves incoming
/// damage; a defense ≈ the midrange stat gives ~50% mitigation.
pub(crate) const MITIGATION_K: f64 = 12.0;

/// Apply ratio mitigation to a raw damage amount: `raw * K / (K + defense)`,
/// rounded, with a minimum of 1. Defense gives smooth diminishing returns and
/// never fully negates a hit.
pub(crate) fn apply_mitigation(raw: f64, defense_stat: u32) -> u32 {
    let mitigated = raw * MITIGATION_K / (MITIGATION_K + defense_stat as f64);
    (mitigated.round().max(1.0)) as u32
}

fn scaled_damage_with_defense(attack_stat: u32, multiplier: f64, defense_stat: u32) -> u32 {
    apply_mitigation(attack_stat as f64 * multiplier, defense_stat)
}

fn default_true() -> bool {
    true
}

#[allow(clippy::too_many_arguments)]
fn try_move_character(
    character_idx: usize,
    moving_team: &mut [CharacterState],
    enemy_team: &[CharacterState],
    actor_team_is_a: bool,
    direction: &MoveDirection,
    if_empty: bool,
    log: &mut BattleLog,
    step: u32,
) {
    let current = moving_team[character_idx].position().clone();
    let next_row = match direction {
        MoveDirection::Forward => current.row.checked_sub(1),
        MoveDirection::Backward => current.row.checked_add(1).filter(|row| *row < 3),
    };
    let Some(next_row) = next_row else {
        return;
    };

    let destination = crate::models::Position {
        row: next_row,
        col: current.col,
    };
    let occupied = moving_team
        .iter()
        .enumerate()
        .any(|(idx, c)| idx != character_idx && c.is_alive() && c.position().row == destination.row && c.position().col == destination.col);
    if if_empty && occupied {
        return;
    }

    moving_team[character_idx].set_position(destination.clone());
    recompute_team_companions(moving_team);
    log.push(BattleEvent::Moved {
        tick_count: step,
        character_id: moving_team[character_idx].id(),
        character_name: moving_team[character_idx].base_name().to_string(),
        from_row: current.row,
        from_col: current.col,
        to_row: destination.row,
        to_col: destination.col,
    });
    if actor_team_is_a {
        log.capture_latest_snapshot(moving_team, enemy_team);
    } else {
        log.capture_latest_snapshot(enemy_team, moving_team);
    }
}

fn transform_statuses(
    target: &mut CharacterState,
    from_status: &str,
    to_status: &str,
    stats: &[Stat],
    source_id: u32,
    to_def: &crate::statuses::StatusDef,
) {
    for stat in stats {
        let from_key = status_key(from_status, Some(stat));
        let stacks = target.status_stacks(&from_key);
        if stacks == 0 {
            continue;
        }
        target.remove_status(&from_key, stacks);
        let to_key = status_key(to_status, Some(stat));
        target.add_status(&to_key, stacks, source_id, to_def, Some(stat.clone()));
    }
}

fn recompute_team_companions(team: &mut [CharacterState]) {
    let positions: Vec<(u32, crate::models::Position)> = team
        .iter()
        .filter(|c| c.is_alive())
        .map(|c| (c.id(), c.position().clone()))
        .collect();

    for c in team.iter_mut() {
        let companions: Vec<u32> = positions
            .iter()
            .filter(|(id, pos)| *id != c.id() && c.position().is_adjacent(pos))
            .map(|(id, _)| *id)
            .collect();
        c.set_companions(companions);
    }
}

#[cfg(test)]
#[path = "abilities_tests.rs"]
mod tests;
