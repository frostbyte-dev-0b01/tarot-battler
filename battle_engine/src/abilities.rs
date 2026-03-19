//! Ability definitions and execution.

use std::collections::HashMap;

use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::abilities_targeting::{
    resolve_ally_targets, resolve_enemy_targets, retarget_filter_matches, target_is_enemy_side,
};
use crate::logger::{BattleEvent, BattleLog};
use crate::models::{CharacterState, Stat};
use crate::statuses::{StatusMap, status_key};
use crate::targeting::select_target;

/// Simple target categories used by existing sample data.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SimpleAbilityTarget {
    CurrentTarget,
    CurrentTargetAndCompanions,
    TriggerTarget,
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
    Random,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RetargetMode {
    ToSelf,
    ToCompanion,
    DefaultRetarget,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RetargetFilter {
    PhysicalAttackers,
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

/// A single primitive effect composing an ability.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Primitive {
    DealPhysicalDamage {
        target: AbilityTarget,
        multiplier: f64,
        #[serde(default)]
        double_empower_stat: Option<Stat>,
    },
    DealPhysicalDamageBonusVsStatus {
        target: AbilityTarget,
        multiplier: f64,
        status: String,
        #[serde(default)]
        stat: Option<Stat>,
        bonus_damage: u32,
    },
    DealMagicalDamage {
        target: AbilityTarget,
        multiplier: f64,
    },
    DealMagicalDamageConsumeStatus {
        target: AbilityTarget,
        multiplier: f64,
        status: String,
        #[serde(default)]
        stat: Option<Stat>,
        bonus_per_stack: u32,
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
    RemoveStatus {
        target: AbilityTarget,
        status: String,
        #[serde(default)]
        stat: Option<Stat>,
        stacks: u32,
    },
    RemoveOneBuff {
        target: AbilityTarget,
    },
    Cleanse {
        target: AbilityTarget,
        amount: u32,
    },
    Dispel {
        target: AbilityTarget,
        amount: u32,
    },
    Retarget {
        target: AbilityTarget,
        mode: RetargetMode,
        #[serde(default)]
        filter: Option<RetargetFilter>,
    },
    CommandAttack,
    Move {
        direction: MoveDirection,
        #[serde(default = "default_true")]
        if_empty: bool,
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
    OnTakeDamage,
    OnTurnStart,
    OnAllyDamageMyTarget,
    OnAllyApplyOmen,
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
    pub trigger_target_id: Option<u32>,
}

impl<'a> ExecutionContext<'a> {
    pub fn new(
        actor_team: &'a mut [CharacterState],
        enemy_team: &'a mut [CharacterState],
        rng: &'a mut StdRng,
        log: &'a mut BattleLog,
        step: u32,
        status_defs: &'a StatusMap,
    ) -> Self {
        Self {
            actor_team,
            enemy_team,
            rng,
            log,
            step,
            status_defs,
            trigger_target_id: None,
        }
    }

    pub fn with_trigger_target(mut self, trigger_target_id: Option<u32>) -> Self {
        self.trigger_target_id = trigger_target_id;
        self
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

    execute_primitives_with_context(ctx, actor_idx, ability_name, &ability.primitives)
}

/// Execute an ability's primitives.
///
/// Returns a list of damage records for defeat checking by the caller.
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
    let mut ctx = ExecutionContext::new(actor_team, enemy_team, rng, log, step, status_defs);
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
            Primitive::DealPhysicalDamage {
                target,
                multiplier,
                double_empower_stat,
            } => {
                let actor_str = match double_empower_stat {
                    Some(Stat::MGT) => ctx.actor_team[actor_idx].get_eff_stat_with_doubled_empower(&Stat::MGT),
                    _ => ctx.actor_team[actor_idx].get_eff_stat(&Stat::MGT),
                };
                let target_indices = resolve_enemy_targets(
                    target,
                    actor_idx,
                    ctx.actor_team,
                    ctx.enemy_team,
                    ctx.rng,
                    ctx.trigger_target_id,
                );
                for tidx in target_indices {
                    let defender_for = ctx.enemy_team[tidx].get_eff_stat(&Stat::ARM);
                    let base = (actor_str as i32 - defender_for as i32).max(1) as u32;
                    let raw_damage = ((base as f64 * multiplier).max(1.0)) as u32;
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
                    });
                    damage_dealt.push(DamageRecord {
                        source_id: actor_id,
                        target_id: tid,
                        damage,
                    });
                }
            }
            Primitive::DealPhysicalDamageBonusVsStatus {
                target,
                multiplier,
                status,
                stat,
                bonus_damage,
            } => {
                let key = status_key(status, stat.as_ref());
                let actor_mgt = ctx.actor_team[actor_idx].get_eff_stat(&Stat::MGT);
                let target_indices = resolve_enemy_targets(
                    target,
                    actor_idx,
                    ctx.actor_team,
                    ctx.enemy_team,
                    ctx.rng,
                    ctx.trigger_target_id,
                );
                for tidx in target_indices {
                    let defender_arm = ctx.enemy_team[tidx].get_eff_stat(&Stat::ARM);
                    let base = (actor_mgt as i32 - defender_arm as i32).max(1) as u32;
                    let mut raw_damage = ((base as f64 * multiplier).max(1.0)) as u32;
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
                    });
                    damage_dealt.push(DamageRecord {
                        source_id: actor_id,
                        target_id: tid,
                        damage,
                    });
                }
            }
            Primitive::DealMagicalDamage { target, multiplier } => {
                let target_indices = resolve_enemy_targets(
                    target,
                    actor_idx,
                    ctx.actor_team,
                    ctx.enemy_team,
                    ctx.rng,
                    ctx.trigger_target_id,
                );
                for tidx in target_indices {
                    let defender_wis = ctx.enemy_team[tidx].get_eff_stat(&Stat::RES);
                    let base = (actor_int as i32 - defender_wis as i32).max(1) as u32;
                    let raw_damage = ((base as f64 * multiplier).max(1.0)) as u32;
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
                    });
                    damage_dealt.push(DamageRecord {
                        source_id: actor_id,
                        target_id: tid,
                        damage,
                    });
                }
            }
            Primitive::DealMagicalDamageConsumeStatus {
                target,
                multiplier,
                status,
                stat,
                bonus_per_stack,
            } => {
                let key = status_key(status, stat.as_ref());
                let target_indices = resolve_enemy_targets(
                    target,
                    actor_idx,
                    ctx.actor_team,
                    ctx.enemy_team,
                    ctx.rng,
                    ctx.trigger_target_id,
                );
                for tidx in target_indices {
                    let defender_res = ctx.enemy_team[tidx].get_eff_stat(&Stat::RES);
                    let base = (actor_int as i32 - defender_res as i32).max(1) as u32;
                    let consumed_stacks = ctx.enemy_team[tidx].status_stacks(&key);
                    let raw_damage = ((base as f64 * multiplier).max(1.0)) as u32
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
                    });
                    damage_dealt.push(DamageRecord {
                        source_id: actor_id,
                        target_id: tid,
                        damage,
                    });
                }
            }
            Primitive::RestoreHp { target, amount } => {
                let target_indices = resolve_ally_targets(target, actor_idx, ctx.actor_team, ctx.rng);
                for tidx in target_indices {
                    if ctx.actor_team[tidx].is_alive() {
                        ctx.actor_team[tidx].heal(*amount);
                    }
                }
            }
            Primitive::RestoreMp { target, amount } => {
                let target_indices = resolve_ally_targets(target, actor_idx, ctx.actor_team, ctx.rng);
                for tidx in target_indices {
                    if ctx.actor_team[tidx].is_alive() {
                        ctx.actor_team[tidx].restore_mp(*amount);
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
                        let target_indices = resolve_enemy_targets(
                            target,
                            actor_idx,
                            ctx.actor_team,
                            ctx.enemy_team,
                            ctx.rng,
                            ctx.trigger_target_id,
                        );
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
                                }
                            }
                        }
                    } else {
                        let target_indices =
                            resolve_ally_targets(target, actor_idx, ctx.actor_team, ctx.rng);
                        for tidx in target_indices {
                            if ctx.actor_team[tidx].is_alive() {
                                ctx.actor_team[tidx].add_status(
                                    &key,
                                    *stacks,
                                    actor_id,
                                    def,
                                    stat.clone(),
                                );
                            }
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
                if target_is_enemy_side(target) {
                    let target_indices = resolve_enemy_targets(
                        target,
                        actor_idx,
                        ctx.actor_team,
                        ctx.enemy_team,
                        ctx.rng,
                        ctx.trigger_target_id,
                    );
                    for tidx in target_indices {
                        ctx.enemy_team[tidx].remove_status(&key, *stacks);
                    }
                } else {
                    let target_indices = resolve_ally_targets(target, actor_idx, ctx.actor_team, ctx.rng);
                    for tidx in target_indices {
                        ctx.actor_team[tidx].remove_status(&key, *stacks);
                    }
                }
            }
            Primitive::RemoveOneBuff { target } => {
                let target_indices = resolve_enemy_targets(
                    target,
                    actor_idx,
                    ctx.actor_team,
                    ctx.enemy_team,
                    ctx.rng,
                    ctx.trigger_target_id,
                );
                for tidx in target_indices {
                    ctx.enemy_team[tidx].remove_one_buff();
                }
            }
            Primitive::Cleanse { target, amount } => {
                if target_is_enemy_side(target) {
                    let target_indices = resolve_enemy_targets(
                        target,
                        actor_idx,
                        ctx.actor_team,
                        ctx.enemy_team,
                        ctx.rng,
                        ctx.trigger_target_id,
                    );
                    for tidx in target_indices {
                        ctx.enemy_team[tidx].cleanse(*amount);
                    }
                } else {
                    let target_indices =
                        resolve_ally_targets(target, actor_idx, ctx.actor_team, ctx.rng);
                    for tidx in target_indices {
                        ctx.actor_team[tidx].cleanse(*amount);
                    }
                }
            }
            Primitive::Dispel { target, amount } => {
                if target_is_enemy_side(target) {
                    let target_indices = resolve_enemy_targets(
                        target,
                        actor_idx,
                        ctx.actor_team,
                        ctx.enemy_team,
                        ctx.rng,
                        ctx.trigger_target_id,
                    );
                    for tidx in target_indices {
                        ctx.enemy_team[tidx].dispel(*amount);
                    }
                } else {
                    let target_indices =
                        resolve_ally_targets(target, actor_idx, ctx.actor_team, ctx.rng);
                    for tidx in target_indices {
                        ctx.actor_team[tidx].dispel(*amount);
                    }
                }
            }
            Primitive::Retarget {
                target,
                mode,
                filter,
            } => {
                let target_indices = if target_is_enemy_side(target) {
                    resolve_enemy_targets(
                        target,
                        actor_idx,
                        ctx.actor_team,
                        ctx.enemy_team,
                        ctx.rng,
                        ctx.trigger_target_id,
                    )
                } else {
                    Vec::new()
                };
                for tidx in target_indices {
                    if !ctx.enemy_team[tidx].is_alive()
                        || !retarget_filter_matches(&ctx.enemy_team[tidx], filter.as_ref())
                    {
                        continue;
                    }

                    let new_target = match mode {
                        RetargetMode::ToSelf => Some(ctx.actor_team[actor_idx].id()),
                        RetargetMode::ToCompanion => {
                            let mut living_companion_ids: Vec<u32> = ctx.actor_team[actor_idx]
                                .companions()
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
                    };

                    if let Some(new_target_id) = new_target {
                        ctx.enemy_team[tidx].set_target(new_target_id);
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
                    .companions()
                    .iter()
                    .filter_map(|id| ctx.actor_team.iter().position(|c| c.id() == *id && c.is_alive()))
                    .max_by_key(|idx| ctx.actor_team[*idx].get_eff_stat(&Stat::MGT))
                else {
                    continue;
                };

                let attacker_str = ctx.actor_team[companion_idx].get_eff_stat(&Stat::MGT);
                let defender_for = ctx.enemy_team[target_idx].get_eff_stat(&Stat::ARM);
                let raw_damage = (attacker_str as i32 - defender_for as i32).max(1) as u32;
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
                });
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
                try_move_actor(actor_idx, ctx.actor_team, direction, *if_empty, ctx.log, ctx.step);
            }
            Primitive::IfTargetHasStatus {
                target,
                status,
                stat,
                primitives,
            } => {
                let key = status_key(status, stat.as_ref());
                let target_indices = resolve_enemy_targets(
                    target,
                    actor_idx,
                    ctx.actor_team,
                    ctx.enemy_team,
                    ctx.rng,
                    ctx.trigger_target_id,
                );
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
                let target_indices = resolve_enemy_targets(
                    target,
                    actor_idx,
                    ctx.actor_team,
                    ctx.enemy_team,
                    ctx.rng,
                    ctx.trigger_target_id,
                );
                let should_execute = target_indices
                    .iter()
                    .any(|tidx| ctx.enemy_team[*tidx].is_alive() && !ctx.enemy_team[*tidx].has_status(&key));
                if should_execute {
                    let nested_damage =
                        execute_primitives_with_context(ctx, actor_idx, _source_name, primitives);
                    damage_dealt.extend(nested_damage);
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
    let mut ctx = ExecutionContext::new(actor_team, enemy_team, rng, log, step, status_defs)
        .with_trigger_target(trigger_target_id);
    execute_primitives_with_context(&mut ctx, actor_idx, source_name, primitives)
}

fn retarget_mode_label(mode: &RetargetMode) -> &'static str {
    match mode {
        RetargetMode::ToSelf => "to_self",
        RetargetMode::ToCompanion => "to_companion",
        RetargetMode::DefaultRetarget => "default_retarget",
    }
}

fn default_true() -> bool {
    true
}

fn try_move_actor(
    actor_idx: usize,
    actor_team: &mut [CharacterState],
    direction: &MoveDirection,
    if_empty: bool,
    log: &mut BattleLog,
    step: u32,
) {
    let current = actor_team[actor_idx].position().clone();
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
    let occupied = actor_team
        .iter()
        .enumerate()
        .any(|(idx, c)| idx != actor_idx && c.is_alive() && c.position().row == destination.row && c.position().col == destination.col);
    if if_empty && occupied {
        return;
    }

    actor_team[actor_idx].set_position(destination.clone());
    recompute_team_companions(actor_team);
    log.push(BattleEvent::Moved {
        tick_count: step,
        character_id: actor_team[actor_idx].id(),
        character_name: actor_team[actor_idx].base_name().to_string(),
        from_row: current.row,
        from_col: current.col,
        to_row: destination.row,
        to_col: destination.col,
    });
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
