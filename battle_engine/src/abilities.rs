//! Ability definitions and execution.

use std::collections::HashMap;

use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::logger::{BattleEvent, BattleLog};
use crate::models::{CharacterState, Stat};
use crate::statuses::{StatusMap, status_key};
use crate::targeting::select_target;

/// Simple target categories used by existing sample data.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SimpleAbilityTarget {
    CurrentTarget,
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
    DealMagicalDamage {
        target: AbilityTarget,
        multiplier: f64,
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
pub enum PassiveTrigger {
    OnBattleStart,
    OnDeath,
    OnKill,
    OnDealDamage,
    OnTakeDamage,
    OnTurnStart,
}

/// A passive ability definition — either a triggered effect or a permanent trait.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PassiveDef {
    Triggered {
        trigger: PassiveTrigger,
        primitives: Vec<Primitive>,
    },
    Trait {
        effect: crate::models::TraitEffect,
    },
}

pub type PassiveMap = HashMap<String, PassiveDef>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DamageRecord {
    pub source_id: u32,
    pub target_id: u32,
    pub damage: u32,
}

/// Execute an ability's primitives.
///
/// Returns a list of damage records for defeat checking by the caller.
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
    let actor_id = actor_team[actor_idx].id();
    let actor_name = actor_team[actor_idx].base_name().to_string();

    log.push(BattleEvent::AbilityUsed {
        tick_count: step,
        actor_id,
        actor_name,
        ability_name: ability_name.to_string(),
        mp_cost: ability.mp_cost,
    });

    execute_primitives(
        actor_idx,
        ability_name,
        &ability.primitives,
        actor_team,
        enemy_team,
        rng,
        log,
        step,
        status_defs,
    )
}

/// Execute a list of primitives (shared by abilities and passives).
///
/// Returns a list of damage records for defeat checking.
pub fn execute_primitives(
    actor_idx: usize,
    _source_name: &str,
    primitives: &[Primitive],
    actor_team: &mut [CharacterState],
    enemy_team: &mut [CharacterState],
    _rng: &mut StdRng,
    log: &mut BattleLog,
    step: u32,
    status_defs: &StatusMap,
) -> Vec<DamageRecord> {
    let mut damage_dealt: Vec<DamageRecord> = Vec::new();

    let actor_id = actor_team[actor_idx].id();

    // Pre-compute actor offensive stats for damage calculation
    let actor_int = actor_team[actor_idx].get_eff_stat(&Stat::INT);

    for primitive in primitives {
        match primitive {
            Primitive::DealPhysicalDamage {
                target,
                multiplier,
                double_empower_stat,
            } => {
                let actor_str = match double_empower_stat {
                    Some(Stat::STR) => actor_team[actor_idx].get_eff_stat_with_doubled_empower(&Stat::STR),
                    _ => actor_team[actor_idx].get_eff_stat(&Stat::STR),
                };
                let target_indices =
                    resolve_enemy_targets(target, actor_idx, actor_team, enemy_team, _rng);
                for tidx in target_indices {
                    let defender_for = enemy_team[tidx].get_eff_stat(&Stat::FOR);
                    let base = (actor_str as i32 - defender_for as i32).max(1) as u32;
                    let raw_damage = ((base as f64 * multiplier).max(1.0)) as u32;
                    let damage = enemy_team[tidx].take_hit(raw_damage);
                    let tid = enemy_team[tidx].id();
                    let tname = enemy_team[tidx].base_name().to_string();
                    let hp = enemy_team[tidx].current_hp();
                    log.push(BattleEvent::AbilityDamage {
                        tick_count: step,
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
                let target_indices =
                    resolve_enemy_targets(target, actor_idx, actor_team, enemy_team, _rng);
                for tidx in target_indices {
                    let defender_wis = enemy_team[tidx].get_eff_stat(&Stat::WIS);
                    let base = (actor_int as i32 - defender_wis as i32).max(1) as u32;
                    let raw_damage = ((base as f64 * multiplier).max(1.0)) as u32;
                    let damage = enemy_team[tidx].take_hit(raw_damage);
                    let tid = enemy_team[tidx].id();
                    let tname = enemy_team[tidx].base_name().to_string();
                    let hp = enemy_team[tidx].current_hp();
                    log.push(BattleEvent::AbilityDamage {
                        tick_count: step,
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
                let target_indices = resolve_ally_targets(target, actor_idx, actor_team, _rng);
                for tidx in target_indices {
                    if actor_team[tidx].is_alive() {
                        actor_team[tidx].heal(*amount);
                    }
                }
            }
            Primitive::RestoreMp { target, amount } => {
                let target_indices = resolve_ally_targets(target, actor_idx, actor_team, _rng);
                for tidx in target_indices {
                    if actor_team[tidx].is_alive() {
                        actor_team[tidx].restore_mp(*amount);
                    }
                }
            }
            Primitive::ApplyStatus {
                target,
                status,
                stat,
                stacks,
            } => {
                if let Some(def) = status_defs.get(status) {
                    let key = status_key(status, stat.as_ref());
                    if target_is_enemy_side(target) {
                        let target_indices =
                            resolve_enemy_targets(target, actor_idx, actor_team, enemy_team, _rng);
                        for tidx in target_indices {
                            if enemy_team[tidx].is_alive() {
                                enemy_team[tidx].add_status(
                                    &key,
                                    *stacks,
                                    actor_id,
                                    def,
                                    stat.clone(),
                                );
                            }
                        }
                    } else {
                        let target_indices =
                            resolve_ally_targets(target, actor_idx, actor_team, _rng);
                        for tidx in target_indices {
                            if actor_team[tidx].is_alive() {
                                actor_team[tidx].add_status(
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
                    let target_indices =
                        resolve_enemy_targets(target, actor_idx, actor_team, enemy_team, _rng);
                    for tidx in target_indices {
                        enemy_team[tidx].remove_status(&key, *stacks);
                    }
                } else {
                    let target_indices = resolve_ally_targets(target, actor_idx, actor_team, _rng);
                    for tidx in target_indices {
                        actor_team[tidx].remove_status(&key, *stacks);
                    }
                }
            }
            Primitive::Retarget {
                target,
                mode,
                filter,
            } => {
                let target_indices = if target_is_enemy_side(target) {
                    resolve_enemy_targets(target, actor_idx, actor_team, enemy_team, _rng)
                } else {
                    Vec::new()
                };
                for tidx in target_indices {
                    if !enemy_team[tidx].is_alive()
                        || !retarget_filter_matches(&enemy_team[tidx], filter.as_ref())
                    {
                        continue;
                    }

                    let new_target = match mode {
                        RetargetMode::ToSelf => Some(actor_team[actor_idx].id()),
                        RetargetMode::ToCompanion => {
                            let mut living_companion_ids: Vec<u32> = actor_team[actor_idx]
                                .companions()
                                .iter()
                                .filter_map(|id| {
                                    actor_team
                                        .iter()
                                        .find(|c| c.id() == *id && c.is_alive())
                                        .map(|c| c.id())
                                })
                                .collect();
                            living_companion_ids.shuffle(_rng);
                            living_companion_ids.into_iter().next()
                        }
                        RetargetMode::DefaultRetarget => {
                            enemy_team[tidx].clear_target();
                            select_target(&enemy_team[tidx], actor_team, _rng)
                        }
                    };

                    if let Some(new_target_id) = new_target {
                        enemy_team[tidx].set_target(new_target_id);
                    }

                    let new_target_name = new_target.and_then(|target_id| {
                        actor_team
                            .iter()
                            .find(|c| c.id() == target_id)
                            .map(|c| c.base_name().to_string())
                    });
                    log.push(BattleEvent::Retargeted {
                        tick_count: step,
                        character_id: enemy_team[tidx].id(),
                        character_name: enemy_team[tidx].base_name().to_string(),
                        new_target_id: enemy_team[tidx].target(),
                        new_target_name,
                        mode: retarget_mode_label(mode).to_string(),
                    });
                }
            }
            Primitive::CommandAttack => {
                let target_id = match actor_team[actor_idx].target() {
                    Some(target_id) => target_id,
                    None => continue,
                };
                let Some(target_idx) = enemy_team.iter().position(|c| c.id() == target_id) else {
                    continue;
                };
                let Some(companion_idx) = actor_team[actor_idx]
                    .companions()
                    .iter()
                    .filter_map(|id| actor_team.iter().position(|c| c.id() == *id && c.is_alive()))
                    .max_by_key(|idx| actor_team[*idx].get_eff_stat(&Stat::STR))
                else {
                    continue;
                };

                let attacker_str = actor_team[companion_idx].get_eff_stat(&Stat::STR);
                let defender_for = enemy_team[target_idx].get_eff_stat(&Stat::FOR);
                let raw_damage = (attacker_str as i32 - defender_for as i32).max(1) as u32;
                let damage = enemy_team[target_idx].take_hit(raw_damage);
                let source_id = actor_team[companion_idx].id();
                let source_name = actor_team[companion_idx].base_name().to_string();
                let hp = enemy_team[target_idx].current_hp();
                log.push(BattleEvent::BasicAttack {
                    tick_count: step,
                    actor_id: source_id,
                    actor_name: source_name,
                    target_id,
                    target_name: enemy_team[target_idx].base_name().to_string(),
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
                try_move_actor(actor_idx, actor_team, direction, *if_empty, log, step);
            }
        }
    }

    damage_dealt
}

/// Resolve an AbilityTarget to indices into enemy_team.
fn resolve_enemy_targets(
    target: &AbilityTarget,
    actor_idx: usize,
    actor_team: &[CharacterState],
    enemy_team: &[CharacterState],
    rng: &mut StdRng,
) -> Vec<usize> {
    match target {
        AbilityTarget::Simple(SimpleAbilityTarget::CurrentTarget) => {
            if let Some(target_id) = actor_team[actor_idx].target() {
                if let Some(idx) = enemy_team.iter().position(|c| c.id() == target_id) {
                    return vec![idx];
                }
            }
            Vec::new()
        }
        AbilityTarget::Simple(SimpleAbilityTarget::FrontRow) => front_row_enemy_indices(enemy_team),
        AbilityTarget::Simple(SimpleAbilityTarget::AllEnemies) => enemy_team
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_alive())
            .map(|(i, _)| i)
            .collect(),
        AbilityTarget::Detailed(spec) if matches!(spec.category, TargetCategory::Enemy) => {
            let mut candidates = enemy_candidates(
                actor_idx,
                actor_team,
                enemy_team,
                spec.position.as_ref(),
                spec.bypass_row_protection,
            );
            select_single_target(&mut candidates, enemy_team, spec.selector.as_ref(), rng)
                .into_iter()
                .collect()
        }
        AbilityTarget::Simple(_) | AbilityTarget::Detailed(_) => Vec::new(),
    }
}

/// Resolve an AbilityTarget to indices into actor_team.
fn resolve_ally_targets(
    target: &AbilityTarget,
    actor_idx: usize,
    actor_team: &[CharacterState],
    rng: &mut StdRng,
) -> Vec<usize> {
    match target {
        AbilityTarget::Simple(SimpleAbilityTarget::SelfChar) => vec![actor_idx],
        AbilityTarget::Simple(SimpleAbilityTarget::Companions) => {
            let comp_ids = actor_team[actor_idx].companions().to_vec();
            comp_ids
                .iter()
                .filter_map(|id| actor_team.iter().position(|c| c.id() == *id))
                .collect()
        }
        AbilityTarget::Simple(SimpleAbilityTarget::AllAllies) => actor_team
            .iter()
            .enumerate()
            .filter(|(i, c)| *i != actor_idx && c.is_alive())
            .map(|(i, _)| i)
            .collect(),
        AbilityTarget::Detailed(spec) if matches!(spec.category, TargetCategory::Ally) => {
            let mut candidates = ally_candidates(actor_idx, actor_team, None, spec.position.as_ref());
            select_single_target(&mut candidates, actor_team, spec.selector.as_ref(), rng)
                .into_iter()
                .collect()
        }
        AbilityTarget::Detailed(spec) if matches!(spec.category, TargetCategory::Companion) => {
            let comp_ids = actor_team[actor_idx].companions().to_vec();
            let mut candidates =
                ally_candidates(actor_idx, actor_team, Some(&comp_ids), spec.position.as_ref());
            select_single_target(&mut candidates, actor_team, spec.selector.as_ref(), rng)
                .into_iter()
                .collect()
        }
        AbilityTarget::Simple(_) | AbilityTarget::Detailed(_) => Vec::new(),
    }
}

fn ally_candidates(
    actor_idx: usize,
    actor_team: &[CharacterState],
    allowed_ids: Option<&[u32]>,
    position: Option<&PositionalCondition>,
) -> Vec<usize> {
    let mut candidates: Vec<usize> = actor_team
        .iter()
        .enumerate()
        .filter(|(i, c)| {
            *i != actor_idx
                && c.is_alive()
                && allowed_ids.map_or(true, |ids| ids.contains(&c.id()))
        })
        .map(|(i, _)| i)
        .collect();

    if let Some(position) = position {
        candidates = filter_ally_positions(candidates, actor_idx, actor_team, position);
    }

    candidates
}

fn filter_ally_positions(
    candidates: Vec<usize>,
    actor_idx: usize,
    actor_team: &[CharacterState],
    position: &PositionalCondition,
) -> Vec<usize> {
    if candidates.is_empty() {
        return candidates;
    }

    match position {
        PositionalCondition::Frontmost => {
            let row = candidates
                .iter()
                .map(|idx| actor_team[*idx].position().row)
                .min()
                .unwrap();
            candidates
                .into_iter()
                .filter(|idx| actor_team[*idx].position().row == row)
                .collect()
        }
        PositionalCondition::Backmost => {
            let row = candidates
                .iter()
                .map(|idx| actor_team[*idx].position().row)
                .max()
                .unwrap();
            candidates
                .into_iter()
                .filter(|idx| actor_team[*idx].position().row == row)
                .collect()
        }
        PositionalCondition::SameRow => {
            let row = actor_team[actor_idx].position().row;
            candidates
                .into_iter()
                .filter(|idx| actor_team[*idx].position().row == row)
                .collect()
        }
        PositionalCondition::SameColumn => {
            let col = actor_team[actor_idx].position().col;
            candidates
                .into_iter()
                .filter(|idx| actor_team[*idx].position().col == col)
                .collect()
        }
    }
}

fn target_is_enemy_side(target: &AbilityTarget) -> bool {
    matches!(
        target,
        AbilityTarget::Simple(SimpleAbilityTarget::CurrentTarget)
            | AbilityTarget::Simple(SimpleAbilityTarget::FrontRow)
            | AbilityTarget::Simple(SimpleAbilityTarget::AllEnemies)
            | AbilityTarget::Detailed(TargetSpec {
                category: TargetCategory::Enemy,
                ..
            })
    )
}

fn retarget_filter_matches(
    target: &CharacterState,
    filter: Option<&RetargetFilter>,
) -> bool {
    match filter {
        None => true,
        Some(RetargetFilter::PhysicalAttackers) => {
            target.get_eff_stat(&Stat::STR) > target.get_eff_stat(&Stat::INT)
        }
    }
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

fn front_row_enemy_indices(enemy_team: &[CharacterState]) -> Vec<usize> {
    let Some(front_row) = enemy_team
        .iter()
        .filter(|c| c.is_alive())
        .map(|c| c.position().row)
        .min()
    else {
        return Vec::new();
    };

    enemy_team
        .iter()
        .enumerate()
        .filter(|(_, c)| c.is_alive() && c.position().row == front_row)
        .map(|(i, _)| i)
        .collect()
}

fn enemy_candidates(
    actor_idx: usize,
    actor_team: &[CharacterState],
    enemy_team: &[CharacterState],
    position: Option<&PositionalCondition>,
    bypass_row_protection: bool,
) -> Vec<usize> {
    let mut candidates: Vec<usize> = if bypass_row_protection {
        enemy_team
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_alive())
            .map(|(i, _)| i)
            .collect()
    } else {
        front_row_enemy_indices(enemy_team)
    };

    if let Some(position) = position {
        candidates = filter_by_position(candidates, actor_idx, actor_team, enemy_team, position);
    }

    candidates
}

fn filter_by_position(
    candidates: Vec<usize>,
    actor_idx: usize,
    actor_team: &[CharacterState],
    enemy_team: &[CharacterState],
    position: &PositionalCondition,
) -> Vec<usize> {
    if candidates.is_empty() {
        return candidates;
    }

    match position {
        PositionalCondition::Frontmost => {
            let row = candidates
                .iter()
                .map(|idx| enemy_team[*idx].position().row)
                .min()
                .unwrap();
            candidates
                .into_iter()
                .filter(|idx| enemy_team[*idx].position().row == row)
                .collect()
        }
        PositionalCondition::Backmost => {
            let row = candidates
                .iter()
                .map(|idx| enemy_team[*idx].position().row)
                .max()
                .unwrap();
            candidates
                .into_iter()
                .filter(|idx| enemy_team[*idx].position().row == row)
                .collect()
        }
        PositionalCondition::SameRow => {
            let row = actor_team[actor_idx].position().row;
            candidates
                .into_iter()
                .filter(|idx| enemy_team[*idx].position().row == row)
                .collect()
        }
        PositionalCondition::SameColumn => {
            let col = actor_team[actor_idx].position().col;
            candidates
                .into_iter()
                .filter(|idx| enemy_team[*idx].position().col == col)
                .collect()
        }
    }
}

fn select_single_target(
    candidates: &mut Vec<usize>,
    team: &[CharacterState],
    selector: Option<&TargetSelector>,
    rng: &mut StdRng,
) -> Option<usize> {
    if candidates.is_empty() {
        return None;
    }

    let selector = selector.unwrap_or(&TargetSelector::Random);
    let chosen = match selector {
        TargetSelector::HighestStat { stat } => {
            extrema_indices(candidates, |idx| team[*idx].get_eff_stat(stat), true)
        }
        TargetSelector::LowestStat { stat } => {
            extrema_indices(candidates, |idx| team[*idx].get_eff_stat(stat), false)
        }
        TargetSelector::HighestHp => {
            extrema_indices(candidates, |idx| team[*idx].current_hp(), true)
        }
        TargetSelector::LowestHp => {
            extrema_indices(candidates, |idx| team[*idx].current_hp(), false)
        }
        TargetSelector::HighestMp => {
            extrema_indices(candidates, |idx| team[*idx].current_mp(), true)
        }
        TargetSelector::LowestMp => {
            extrema_indices(candidates, |idx| team[*idx].current_mp(), false)
        }
        TargetSelector::MostStacks { status, stat } => {
            let key = status_key(status, stat.as_ref());
            extrema_indices(candidates, |idx| team[*idx].status_stacks(&key), true)
        }
        TargetSelector::FewestStacks { status, stat } => {
            let key = status_key(status, stat.as_ref());
            extrema_indices(candidates, |idx| team[*idx].status_stacks(&key), false)
        }
        TargetSelector::HasStatus { status, stat } => {
            let key = status_key(status, stat.as_ref());
            candidates
                .iter()
                .copied()
                .filter(|idx| team[*idx].has_status(&key))
                .collect()
        }
        TargetSelector::LacksStatus { status, stat } => {
            let key = status_key(status, stat.as_ref());
            candidates
                .iter()
                .copied()
                .filter(|idx| !team[*idx].has_status(&key))
                .collect()
        }
        TargetSelector::Random => candidates.clone(),
    };

    chosen.choose(rng).copied()
}

fn extrema_indices(
    candidates: &[usize],
    value_fn: impl Fn(&usize) -> u32,
    want_max: bool,
) -> Vec<usize> {
    let Some(best) = candidates
        .iter()
        .map(&value_fn)
        .reduce(|a, b| if want_max { a.max(b) } else { a.min(b) })
    else {
        return Vec::new();
    };

    candidates
        .iter()
        .copied()
        .filter(|idx| value_fn(idx) == best)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CharacterConfig, Position};
    use crate::statuses::{StackType, StatusBehavior, StatusDef};
    use rand::SeedableRng;

    fn make_char(id: u32, stats: Vec<(Stat, u32)>) -> CharacterState {
        let config = CharacterConfig {
            id: None,
            base_name: format!("Char{}", id),
            display_name: None,
            passive: String::new(),
            actives: Vec::new(),
            item: None,
            position: Position { row: 0, col: 0 },
            stats: stats.into_iter().collect(),
            rules: Vec::new(),
        };
        CharacterState::from_config(id, &config)
    }

    fn make_adjacent_char(id: u32, row: u8, col: u8, stats: Vec<(Stat, u32)>) -> CharacterState {
        let config = CharacterConfig {
            id: None,
            base_name: format!("Char{}", id),
            display_name: None,
            passive: String::new(),
            actives: Vec::new(),
            item: None,
            position: Position { row, col },
            stats: stats.into_iter().collect(),
            rules: Vec::new(),
        };
        CharacterState::from_config(id, &config)
    }

    fn empty_statuses() -> StatusMap {
        HashMap::new()
    }

    fn test_statuses() -> StatusMap {
        let mut map = HashMap::new();
        map.insert(
            "Bleed".to_string(),
            StatusDef {
                behavior: StatusBehavior::DamagePerStack { value: 1 },
                stack_type: StackType::TickDown,
                opposes: None,
            },
        );
        map.insert(
            "Regen".to_string(),
            StatusDef {
                behavior: StatusBehavior::HealPerStack { value: 2 },
                stack_type: StackType::TickDown,
                opposes: None,
            },
        );
        map.insert(
            "Empower".to_string(),
            StatusDef {
                behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
                stack_type: StackType::TickDown,
                opposes: Some("Weaken".to_string()),
            },
        );
        map.insert(
            "Weaken".to_string(),
            StatusDef {
                behavior: StatusBehavior::StatModPerStack { magnitude: -1 },
                stack_type: StackType::TickDown,
                opposes: Some("Empower".to_string()),
            },
        );
        map.insert(
            "Ward".to_string(),
            StatusDef {
                behavior: StatusBehavior::Ward,
                stack_type: StackType::Permanent,
                opposes: None,
            },
        );
        map
    }

    #[test]
    fn deal_physical_damage_with_multiplier() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let mut actor_team = vec![make_char(
            0,
            vec![(Stat::STR, 10), (Stat::CON, 10), (Stat::SPI, 5)],
        )];
        let mut enemy_team = vec![make_char(1, vec![(Stat::FOR, 4), (Stat::CON, 20)])];
        actor_team[0].set_target(1);

        let ability = AbilityDef {
            mp_cost: 2,
            primitives: vec![Primitive::DealPhysicalDamage {
                target: SimpleAbilityTarget::CurrentTarget.into(),
                multiplier: 1.5,
                double_empower_stat: None,
            }],
        };

        let dealt = execute_ability(
            0,
            "Crush",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &empty_statuses(),
        );
        assert_eq!(dealt.len(), 1);
        assert_eq!(dealt[0].target_id, 1);
        assert_eq!(dealt[0].damage, 9);
        assert_eq!(enemy_team[0].current_hp(), 40 - 9);
    }

    #[test]
    fn restore_mp_to_companions() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();

        let stats = vec![(Stat::CON, 10), (Stat::SPI, 5), (Stat::STR, 5)];
        let mut actor_team = vec![
            make_adjacent_char(0, 0, 0, stats.clone()),
            make_adjacent_char(1, 0, 1, stats.clone()),
            make_adjacent_char(2, 0, 2, stats.clone()),
        ];
        actor_team[0].set_companions(vec![1]);
        actor_team[1].spend_mp(4);
        assert_eq!(actor_team[1].current_mp(), 1);

        let ability = AbilityDef {
            mp_cost: 3,
            primitives: vec![Primitive::RestoreMp {
                target: SimpleAbilityTarget::Companions.into(),
                amount: 2,
            }],
        };

        let mut enemy_team = vec![make_char(10, vec![(Stat::CON, 5)])];
        execute_ability(
            0,
            "Embolden",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &empty_statuses(),
        );
        assert_eq!(actor_team[1].current_mp(), 3);
        assert_eq!(actor_team[2].current_mp(), 5);
    }

    #[test]
    fn apply_status_bleed_on_enemy() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let statuses = test_statuses();
        let mut actor_team = vec![make_char(0, vec![(Stat::CON, 10), (Stat::SPI, 5)])];
        let mut enemy_team = vec![make_char(1, vec![(Stat::CON, 10)])];
        actor_team[0].set_target(1);

        let ability = AbilityDef {
            mp_cost: 2,
            primitives: vec![Primitive::ApplyStatus {
                target: SimpleAbilityTarget::CurrentTarget.into(),
                status: "Bleed".to_string(),
                stat: None,
                stacks: 3,
            }],
        };

        execute_ability(
            0,
            "Slash",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &statuses,
        );
        assert_eq!(enemy_team[0].status_stacks("Bleed"), 3);
    }

    #[test]
    fn apply_status_empower_on_ally() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let statuses = test_statuses();
        let mut actor_team = vec![make_char(
            0,
            vec![(Stat::CON, 10), (Stat::SPI, 5), (Stat::STR, 8)],
        )];
        let mut enemy_team = vec![make_char(10, vec![(Stat::CON, 5)])];

        let ability = AbilityDef {
            mp_cost: 2,
            primitives: vec![Primitive::ApplyStatus {
                target: SimpleAbilityTarget::SelfChar.into(),
                status: "Empower".to_string(),
                stat: Some(Stat::STR),
                stacks: 3,
            }],
        };

        execute_ability(
            0,
            "Rage",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &statuses,
        );
        assert_eq!(actor_team[0].get_eff_stat(&Stat::STR), 11); // 8 + 3
    }

    #[test]
    fn apply_status_weaken_on_enemy() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let statuses = test_statuses();
        let mut actor_team = vec![make_char(0, vec![(Stat::CON, 10), (Stat::SPI, 5)])];
        let mut enemy_team = vec![make_char(1, vec![(Stat::CON, 10), (Stat::STR, 10)])];
        actor_team[0].set_target(1);

        let ability = AbilityDef {
            mp_cost: 1,
            primitives: vec![Primitive::ApplyStatus {
                target: SimpleAbilityTarget::CurrentTarget.into(),
                status: "Weaken".to_string(),
                stat: Some(Stat::STR),
                stacks: 2,
            }],
        };

        execute_ability(
            0,
            "Curse",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &statuses,
        );
        assert_eq!(enemy_team[0].get_eff_stat(&Stat::STR), 8); // 10 - 2
    }

    #[test]
    fn remove_status_clears_self_status() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let statuses = test_statuses();
        let mut actor_team = vec![make_char(0, vec![(Stat::CON, 10), (Stat::SPI, 5)])];
        let mut enemy_team = vec![make_char(1, vec![(Stat::CON, 10)])];

        let bleed = statuses.get("Bleed").unwrap();
        actor_team[0].add_status("Bleed", 2, 99, bleed, None);

        let ability = AbilityDef {
            mp_cost: 1,
            primitives: vec![Primitive::RemoveStatus {
                target: SimpleAbilityTarget::SelfChar.into(),
                status: "Bleed".to_string(),
                stat: None,
                stacks: 1,
            }],
        };

        execute_ability(
            0,
            "Cleanse",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &statuses,
        );
        assert_eq!(actor_team[0].status_stacks("Bleed"), 1);
    }

    #[test]
    fn remove_status_clears_current_target_status() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let statuses = test_statuses();
        let mut actor_team = vec![make_char(0, vec![(Stat::CON, 10), (Stat::SPI, 5)])];
        let mut enemy_team = vec![make_char(1, vec![(Stat::CON, 10)])];
        actor_team[0].set_target(1);

        let bleed = statuses.get("Bleed").unwrap();
        enemy_team[0].add_status("Bleed", 3, 99, bleed, None);

        let ability = AbilityDef {
            mp_cost: 1,
            primitives: vec![Primitive::RemoveStatus {
                target: SimpleAbilityTarget::CurrentTarget.into(),
                status: "Bleed".to_string(),
                stat: None,
                stacks: 2,
            }],
        };

        execute_ability(
            0,
            "Dispel",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &statuses,
        );
        assert_eq!(enemy_team[0].status_stacks("Bleed"), 1);
    }

    #[test]
    fn remove_status_clears_all_enemy_statuses() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let statuses = test_statuses();
        let mut actor_team = vec![make_char(0, vec![(Stat::CON, 10), (Stat::SPI, 5)])];
        let mut enemy_team = vec![
            make_char(1, vec![(Stat::CON, 10)]),
            make_char(2, vec![(Stat::CON, 10)]),
        ];

        let bleed = statuses.get("Bleed").unwrap();
        enemy_team[0].add_status("Bleed", 1, 99, bleed, None);
        enemy_team[1].add_status("Bleed", 1, 99, bleed, None);

        let ability = AbilityDef {
            mp_cost: 1,
            primitives: vec![Primitive::RemoveStatus {
                target: SimpleAbilityTarget::AllEnemies.into(),
                status: "Bleed".to_string(),
                stat: None,
                stacks: 1,
            }],
        };

        execute_ability(
            0,
            "MassDispel",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &statuses,
        );
        assert_eq!(enemy_team[0].status_stacks("Bleed"), 0);
        assert_eq!(enemy_team[1].status_stacks("Bleed"), 0);
    }

    #[test]
    fn all_enemies_target_resolves_to_all_living() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let mut actor_team = vec![make_char(
            0,
            vec![(Stat::CON, 10), (Stat::SPI, 5), (Stat::STR, 10)],
        )];
        let mut enemy_team = vec![
            make_char(1, vec![(Stat::CON, 10), (Stat::FOR, 3)]),
            make_char(2, vec![(Stat::CON, 10), (Stat::FOR, 3)]),
            make_char(3, vec![(Stat::CON, 10), (Stat::FOR, 3)]),
        ];
        enemy_team[1].take_damage(100);

        let ability = AbilityDef {
            mp_cost: 1,
            primitives: vec![Primitive::DealPhysicalDamage {
                target: SimpleAbilityTarget::AllEnemies.into(),
                multiplier: 1.0,
                double_empower_stat: None,
            }],
        };

        let dealt = execute_ability(
            0,
            "Sweep",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &empty_statuses(),
        );
        assert_eq!(dealt.len(), 2);
        assert!(dealt.iter().any(|record| record.target_id == 1));
        assert!(dealt.iter().any(|record| record.target_id == 3));
    }

    #[test]
    fn all_allies_target_excludes_self() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let mut actor_team = vec![
            make_char(0, vec![(Stat::CON, 10), (Stat::SPI, 5)]),
            make_char(1, vec![(Stat::CON, 10), (Stat::SPI, 3)]),
            make_char(2, vec![(Stat::CON, 10), (Stat::SPI, 3)]),
        ];
        actor_team[1].spend_mp(2);
        actor_team[2].spend_mp(2);
        let mut enemy_team = vec![make_char(10, vec![(Stat::CON, 5)])];

        let ability = AbilityDef {
            mp_cost: 1,
            primitives: vec![Primitive::RestoreMp {
                target: SimpleAbilityTarget::AllAllies.into(),
                amount: 5,
            }],
        };

        execute_ability(
            0,
            "Rally",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &empty_statuses(),
        );
        assert_eq!(actor_team[0].current_mp(), 5);
        assert_eq!(actor_team[1].current_mp(), 3);
        assert_eq!(actor_team[2].current_mp(), 3);
    }

    #[test]
    fn ally_selector_targets_lowest_hp_ally() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let mut actor_team = vec![
            make_char(0, vec![(Stat::CON, 10), (Stat::SPI, 5)]),
            make_char(1, vec![(Stat::CON, 10)]),
            make_char(2, vec![(Stat::CON, 10)]),
        ];
        actor_team[1].take_damage(3);
        actor_team[2].take_damage(8);
        let mut enemy_team = vec![make_char(10, vec![(Stat::CON, 5)])];

        let ability = AbilityDef {
            mp_cost: 1,
            primitives: vec![Primitive::RestoreHp {
                target: AbilityTarget::Detailed(TargetSpec {
                    category: TargetCategory::Ally,
                    selector: Some(TargetSelector::LowestHp),
                    position: None,
                    bypass_row_protection: false,
                }),
                amount: 4,
            }],
        };

        execute_ability(
            0,
            "Rescue",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &empty_statuses(),
        );
        assert_eq!(actor_team[1].current_hp(), 17);
        assert_eq!(actor_team[2].current_hp(), 16);
    }

    #[test]
    fn enemy_selector_can_target_backmost_with_row_bypass() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let mut actor_team = vec![make_adjacent_char(
            0,
            0,
            0,
            vec![(Stat::STR, 10), (Stat::CON, 10), (Stat::SPI, 5)],
        )];
        let mut enemy_team = vec![
            make_adjacent_char(1, 0, 0, vec![(Stat::FOR, 3), (Stat::CON, 10)]),
            make_adjacent_char(2, 2, 0, vec![(Stat::FOR, 3), (Stat::CON, 10)]),
        ];

        let ability = AbilityDef {
            mp_cost: 1,
            primitives: vec![Primitive::DealPhysicalDamage {
                target: AbilityTarget::Detailed(TargetSpec {
                    category: TargetCategory::Enemy,
                    selector: Some(TargetSelector::Random),
                    position: Some(PositionalCondition::Backmost),
                    bypass_row_protection: true,
                }),
                multiplier: 1.0,
                double_empower_stat: None,
            }],
        };

        let dealt = execute_ability(
            0,
            "Snipe",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &empty_statuses(),
        );

        assert_eq!(dealt.len(), 1);
        assert_eq!(dealt[0].target_id, 2);
    }

    #[test]
    fn enemy_selector_can_target_same_column_enemy() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let mut actor_team = vec![make_adjacent_char(
            0,
            0,
            1,
            vec![(Stat::STR, 10), (Stat::CON, 10), (Stat::SPI, 5)],
        )];
        let mut enemy_team = vec![
            make_adjacent_char(1, 0, 0, vec![(Stat::FOR, 3), (Stat::CON, 10)]),
            make_adjacent_char(2, 0, 1, vec![(Stat::FOR, 3), (Stat::CON, 10)]),
        ];

        let ability = AbilityDef {
            mp_cost: 1,
            primitives: vec![Primitive::DealPhysicalDamage {
                target: AbilityTarget::Detailed(TargetSpec {
                    category: TargetCategory::Enemy,
                    selector: Some(TargetSelector::Random),
                    position: Some(PositionalCondition::SameColumn),
                    bypass_row_protection: false,
                }),
                multiplier: 1.0,
                double_empower_stat: None,
            }],
        };

        let dealt = execute_ability(
            0,
            "LineStrike",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &empty_statuses(),
        );

        assert_eq!(dealt.len(), 1);
        assert_eq!(dealt[0].target_id, 2);
    }

    #[test]
    fn ally_selector_can_target_same_row_ally() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let mut actor_team = vec![
            make_adjacent_char(0, 1, 1, vec![(Stat::CON, 10), (Stat::SPI, 6)]),
            make_adjacent_char(1, 1, 2, vec![(Stat::CON, 10), (Stat::SPI, 4)]),
            make_adjacent_char(2, 2, 1, vec![(Stat::CON, 10), (Stat::SPI, 2)]),
        ];
        actor_team[1].spend_mp(3);
        actor_team[2].spend_mp(2);
        let mut enemy_team = vec![make_char(10, vec![(Stat::CON, 5)])];

        let ability = AbilityDef {
            mp_cost: 1,
            primitives: vec![Primitive::RestoreMp {
                target: AbilityTarget::Detailed(TargetSpec {
                    category: TargetCategory::Ally,
                    selector: Some(TargetSelector::LowestMp),
                    position: Some(PositionalCondition::SameRow),
                    bypass_row_protection: false,
                }),
                amount: 2,
            }],
        };

        execute_ability(
            0,
            "RowBlessing",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &empty_statuses(),
        );

        assert_eq!(actor_team[1].current_mp(), 3);
        assert_eq!(actor_team[2].current_mp(), 0);
    }

    #[test]
    fn companion_selector_can_target_same_row_companion() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let mut actor_team = vec![
            make_adjacent_char(0, 1, 1, vec![(Stat::CON, 10)]),
            make_adjacent_char(1, 1, 2, vec![(Stat::CON, 10)]),
            make_adjacent_char(2, 2, 1, vec![(Stat::CON, 10)]),
        ];
        actor_team[0].set_companions(vec![1, 2]);
        let mut enemy_team = vec![make_char(10, vec![(Stat::CON, 5)])];

        let mut statuses = empty_statuses();
        statuses.insert(
            "Empower".to_string(),
            StatusDef {
                behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
                stack_type: StackType::TickDown,
                opposes: Some("Weaken".to_string()),
            },
        );

        let ability = AbilityDef {
            mp_cost: 1,
            primitives: vec![Primitive::ApplyStatus {
                target: AbilityTarget::Detailed(TargetSpec {
                    category: TargetCategory::Companion,
                    selector: Some(TargetSelector::Random),
                    position: Some(PositionalCondition::SameRow),
                    bypass_row_protection: false,
                }),
                status: "Empower".to_string(),
                stat: Some(Stat::STR),
                stacks: 1,
            }],
        };

        execute_ability(
            0,
            "RowCommand",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &statuses,
        );

        let key = status_key("Empower", Some(&Stat::STR));
        assert_eq!(actor_team[1].status_stacks(&key), 1);
        assert_eq!(actor_team[2].status_stacks(&key), 0);
    }

    #[test]
    fn retarget_to_self_only_affects_physical_attackers() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let mut actor_team = vec![
            make_adjacent_char(0, 0, 0, vec![(Stat::CON, 10)]),
            make_adjacent_char(1, 0, 1, vec![(Stat::CON, 10)]),
        ];
        actor_team[0].set_companions(vec![1]);
        let mut enemy_team = vec![
            make_adjacent_char(10, 0, 0, vec![(Stat::STR, 7), (Stat::INT, 2), (Stat::CON, 10)]),
            make_adjacent_char(11, 0, 1, vec![(Stat::STR, 2), (Stat::INT, 7), (Stat::CON, 10)]),
        ];
        enemy_team[0].set_target(1);
        enemy_team[1].set_target(1);

        let ability = AbilityDef {
            mp_cost: 1,
            primitives: vec![Primitive::Retarget {
                target: SimpleAbilityTarget::AllEnemies.into(),
                mode: RetargetMode::ToSelf,
                filter: Some(RetargetFilter::PhysicalAttackers),
            }],
        };

        execute_ability(
            0,
            "Taunt",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &empty_statuses(),
        );

        assert_eq!(enemy_team[0].target(), Some(0));
        assert_eq!(enemy_team[1].target(), Some(1));
    }

    #[test]
    fn retarget_to_companion_chooses_living_companion() {
        let mut rng = StdRng::seed_from_u64(2);
        let mut log = BattleLog::new();
        let mut actor_team = vec![
            make_adjacent_char(0, 0, 0, vec![(Stat::CON, 10)]),
            make_adjacent_char(1, 0, 1, vec![(Stat::CON, 10)]),
            make_adjacent_char(2, 1, 0, vec![(Stat::CON, 10)]),
        ];
        actor_team[0].set_companions(vec![1, 2]);
        actor_team[2].take_damage(100);
        let mut enemy_team = vec![make_adjacent_char(
            10,
            0,
            2,
            vec![(Stat::STR, 7), (Stat::CON, 10)],
        )];
        enemy_team[0].set_target(0);

        let ability = AbilityDef {
            mp_cost: 1,
            primitives: vec![Primitive::Retarget {
                target: SimpleAbilityTarget::AllEnemies.into(),
                mode: RetargetMode::ToCompanion,
                filter: None,
            }],
        };

        execute_ability(
            0,
            "DecoyOrders",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &empty_statuses(),
        );

        assert_eq!(enemy_team[0].target(), Some(1));
    }

    #[test]
    fn default_retarget_uses_normal_target_selection() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let mut actor_team = vec![
            make_adjacent_char(0, 0, 0, vec![(Stat::FOR, 9), (Stat::WIS, 2), (Stat::CON, 10)]),
            make_adjacent_char(1, 0, 1, vec![(Stat::FOR, 2), (Stat::WIS, 9), (Stat::CON, 10)]),
        ];
        let mut enemy_team = vec![make_adjacent_char(
            10,
            0,
            2,
            vec![(Stat::STR, 8), (Stat::INT, 3), (Stat::CON, 10)],
        )];
        enemy_team[0].set_target(0);

        let ability = AbilityDef {
            mp_cost: 1,
            primitives: vec![Primitive::Retarget {
                target: SimpleAbilityTarget::AllEnemies.into(),
                mode: RetargetMode::DefaultRetarget,
                filter: None,
            }],
        };

        execute_ability(
            0,
            "Confuse",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &empty_statuses(),
        );

        assert_eq!(enemy_team[0].target(), Some(1));
    }

    #[test]
    fn command_attack_uses_highest_str_living_companion() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let mut actor_team = vec![
            make_adjacent_char(0, 0, 0, vec![(Stat::CON, 10)]),
            make_adjacent_char(1, 0, 1, vec![(Stat::STR, 8), (Stat::CON, 10)]),
            make_adjacent_char(2, 1, 0, vec![(Stat::STR, 5), (Stat::CON, 10)]),
        ];
        actor_team[0].set_companions(vec![1, 2]);
        actor_team[0].set_target(10);
        let mut enemy_team = vec![make_adjacent_char(
            10,
            0,
            2,
            vec![(Stat::FOR, 3), (Stat::CON, 10)],
        )];

        let ability = AbilityDef {
            mp_cost: 1,
            primitives: vec![Primitive::CommandAttack],
        };

        let dealt = execute_ability(
            0,
            "Command",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &empty_statuses(),
        );

        assert_eq!(dealt.len(), 1);
        assert_eq!(dealt[0].source_id, 1);
        assert_eq!(dealt[0].target_id, 10);
        assert_eq!(enemy_team[0].current_hp(), 15);
    }

    #[test]
    fn ward_negates_physical_ability_damage() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let statuses = test_statuses();
        let mut actor_team = vec![make_char(
            0,
            vec![(Stat::STR, 10), (Stat::CON, 10), (Stat::SPI, 5)],
        )];
        let mut enemy_team = vec![make_char(1, vec![(Stat::FOR, 4), (Stat::CON, 20)])];
        enemy_team[0].add_status("Ward", 1, 99, statuses.get("Ward").unwrap(), None);
        actor_team[0].set_target(1);

        let ability = AbilityDef {
            mp_cost: 2,
            primitives: vec![Primitive::DealPhysicalDamage {
                target: SimpleAbilityTarget::CurrentTarget.into(),
                multiplier: 1.5,
                double_empower_stat: None,
            }],
        };

        let dealt = execute_ability(
            0,
            "Crush",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &statuses,
        );

        assert_eq!(dealt[0].damage, 0);
        assert_eq!(enemy_team[0].current_hp(), 40);
        assert_eq!(enemy_team[0].status_stacks("Ward"), 0);
    }

    #[test]
    fn ward_negates_magical_ability_damage() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let statuses = test_statuses();
        let mut actor_team = vec![make_char(
            0,
            vec![(Stat::INT, 10), (Stat::CON, 10), (Stat::SPI, 5)],
        )];
        let mut enemy_team = vec![make_char(1, vec![(Stat::WIS, 4), (Stat::CON, 20)])];
        enemy_team[0].add_status("Ward", 1, 99, statuses.get("Ward").unwrap(), None);
        actor_team[0].set_target(1);

        let ability = AbilityDef {
            mp_cost: 2,
            primitives: vec![Primitive::DealMagicalDamage {
                target: SimpleAbilityTarget::CurrentTarget.into(),
                multiplier: 1.5,
            }],
        };

        let dealt = execute_ability(
            0,
            "Smite",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &statuses,
        );

        assert_eq!(dealt[0].damage, 0);
        assert_eq!(enemy_team[0].current_hp(), 40);
        assert_eq!(enemy_team[0].status_stacks("Ward"), 0);
    }

    #[test]
    fn doubled_empower_stat_increases_only_the_flagged_attack() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let statuses = test_statuses();
        let mut actor_team = vec![make_char(
            0,
            vec![(Stat::STR, 10), (Stat::INT, 8), (Stat::CON, 10), (Stat::SPI, 5)],
        )];
        let mut enemy_team = vec![make_char(1, vec![(Stat::FOR, 4), (Stat::WIS, 4), (Stat::CON, 20)])];
        actor_team[0].set_target(1);
        actor_team[0].add_status(
            &status_key("Empower", Some(&Stat::STR)),
            2,
            99,
            statuses.get("Empower").unwrap(),
            Some(Stat::STR),
        );

        let normal = AbilityDef {
            mp_cost: 2,
            primitives: vec![Primitive::DealPhysicalDamage {
                target: SimpleAbilityTarget::CurrentTarget.into(),
                multiplier: 1.0,
                double_empower_stat: None,
            }],
        };
        let payoff = AbilityDef {
            mp_cost: 2,
            primitives: vec![Primitive::DealPhysicalDamage {
                target: SimpleAbilityTarget::CurrentTarget.into(),
                multiplier: 1.0,
                double_empower_stat: Some(Stat::STR),
            }],
        };

        let normal_dealt = execute_ability(
            0,
            "Charge",
            &normal,
            &mut actor_team.clone(),
            &mut enemy_team.clone(),
            &mut rng,
            &mut log,
            1,
            &statuses,
        );

        let payoff_dealt = execute_ability(
            0,
            "Breakthrough",
            &payoff,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &statuses,
        );

        assert_eq!(normal_dealt[0].damage, 8);
        assert_eq!(payoff_dealt[0].damage, 10);
    }

    #[test]
    fn move_forward_updates_position_and_companions() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let mut actor_team = vec![
            make_adjacent_char(0, 1, 1, vec![(Stat::CON, 10)]),
            make_adjacent_char(1, 1, 2, vec![(Stat::CON, 10)]),
            make_adjacent_char(2, 0, 0, vec![(Stat::CON, 10)]),
        ];
        actor_team[0].set_companions(vec![1]);
        actor_team[1].set_companions(vec![0]);
        actor_team[2].set_companions(vec![]);
        let mut enemy_team = vec![make_adjacent_char(10, 0, 3, vec![(Stat::CON, 10)])];

        let ability = AbilityDef {
            mp_cost: 1,
            primitives: vec![Primitive::Move {
                direction: MoveDirection::Forward,
                if_empty: true,
            }],
        };

        execute_ability(
            0,
            "Charge",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &empty_statuses(),
        );

        assert_eq!(actor_team[0].position().row, 0);
        assert_eq!(actor_team[0].position().col, 1);
        assert_eq!(actor_team[0].companions(), &[2]);
        assert!(actor_team[1].companions().is_empty());
        assert_eq!(actor_team[2].companions(), &[0]);

        let moved_event = log.events().iter().find_map(|event| match event {
            BattleEvent::Moved {
                character_id,
                from_row,
                from_col,
                to_row,
                to_col,
                ..
            } => Some((*character_id, *from_row, *from_col, *to_row, *to_col)),
            _ => None,
        });
        assert_eq!(moved_event, Some((0, 1, 1, 0, 1)));
    }

    #[test]
    fn move_backward_requires_empty_destination_when_flagged() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let mut actor_team = vec![
            make_adjacent_char(0, 1, 1, vec![(Stat::CON, 10)]),
            make_adjacent_char(1, 2, 1, vec![(Stat::CON, 10)]),
        ];
        actor_team[0].set_companions(vec![]);
        actor_team[1].set_companions(vec![]);
        let mut enemy_team = vec![make_adjacent_char(10, 0, 0, vec![(Stat::CON, 10)])];

        let ability = AbilityDef {
            mp_cost: 1,
            primitives: vec![Primitive::Move {
                direction: MoveDirection::Backward,
                if_empty: true,
            }],
        };

        execute_ability(
            0,
            "Withdraw",
            &ability,
            &mut actor_team,
            &mut enemy_team,
            &mut rng,
            &mut log,
            1,
            &empty_statuses(),
        );

        assert_eq!(actor_team[0].position().row, 1);
        assert!(log
            .events()
            .iter()
            .all(|event| !matches!(event, BattleEvent::Moved { .. })));
    }
}
