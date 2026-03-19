//! Shared target resolution helpers for ability primitives.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::abilities::{
    AbilityTarget, PositionalCondition, RetargetFilter, SimpleAbilityTarget, TargetCategory,
    TargetSelector,
};
use crate::models::{CharacterState, Stat};
use crate::statuses::status_key;

pub(crate) fn resolve_enemy_targets(
    target: &AbilityTarget,
    actor_idx: usize,
    actor_team: &[CharacterState],
    enemy_team: &[CharacterState],
    rng: &mut StdRng,
    trigger_target_id: Option<u32>,
) -> Vec<usize> {
    match target {
        AbilityTarget::Simple(SimpleAbilityTarget::CurrentTarget) => {
            if let Some(target_id) = actor_team[actor_idx].target()
                && let Some(idx) = enemy_team.iter().position(|c| c.id() == target_id)
            {
                return vec![idx];
            }
            Vec::new()
        }
        AbilityTarget::Simple(SimpleAbilityTarget::CurrentTargetAndCompanions) => {
            let Some(target_id) = actor_team[actor_idx].target() else {
                return Vec::new();
            };
            let Some(target_idx) = enemy_team.iter().position(|c| c.id() == target_id) else {
                return Vec::new();
            };

            let companion_ids = enemy_team[target_idx].companions().to_vec();
            let mut targets = vec![target_idx];
            targets.extend(
                companion_ids
                    .iter()
                    .filter_map(|id| enemy_team.iter().position(|c| c.id() == *id && c.is_alive())),
            );
            targets.sort_unstable();
            targets.dedup();
            targets
        }
        AbilityTarget::Simple(SimpleAbilityTarget::TriggerTarget) => {
            let Some(target_id) = trigger_target_id else {
                return Vec::new();
            };
            enemy_team
                .iter()
                .position(|c| c.id() == target_id && c.is_alive())
                .into_iter()
                .collect()
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
            match spec.selector.as_ref() {
                Some(selector) => {
                    select_single_target(&mut candidates, enemy_team, Some(selector), rng)
                        .into_iter()
                        .collect()
                }
                None => candidates,
            }
        }
        AbilityTarget::Simple(_) | AbilityTarget::Detailed(_) => Vec::new(),
    }
}

pub(crate) fn resolve_ally_targets(
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
            match spec.selector.as_ref() {
                Some(selector) => {
                    select_single_target(&mut candidates, actor_team, Some(selector), rng)
                        .into_iter()
                        .collect()
                }
                None => candidates,
            }
        }
        AbilityTarget::Detailed(spec) if matches!(spec.category, TargetCategory::Companion) => {
            let comp_ids = actor_team[actor_idx].companions().to_vec();
            let mut candidates =
                ally_candidates(actor_idx, actor_team, Some(&comp_ids), spec.position.as_ref());
            match spec.selector.as_ref() {
                Some(selector) => {
                    select_single_target(&mut candidates, actor_team, Some(selector), rng)
                        .into_iter()
                        .collect()
                }
                None => candidates,
            }
        }
        AbilityTarget::Simple(_) | AbilityTarget::Detailed(_) => Vec::new(),
    }
}

pub(crate) fn target_is_enemy_side(target: &AbilityTarget) -> bool {
    matches!(
        target,
        AbilityTarget::Simple(SimpleAbilityTarget::CurrentTarget)
            | AbilityTarget::Simple(SimpleAbilityTarget::CurrentTargetAndCompanions)
            | AbilityTarget::Simple(SimpleAbilityTarget::TriggerTarget)
            | AbilityTarget::Simple(SimpleAbilityTarget::FrontRow)
            | AbilityTarget::Simple(SimpleAbilityTarget::AllEnemies)
            | AbilityTarget::Detailed(crate::abilities::TargetSpec {
                category: TargetCategory::Enemy,
                ..
            })
    )
}

pub(crate) fn retarget_filter_matches(
    target: &CharacterState,
    filter: Option<&RetargetFilter>,
) -> bool {
    match filter {
        None => true,
        Some(RetargetFilter::PhysicalAttackers) => {
            target.get_eff_stat(&Stat::MGT) > target.get_eff_stat(&Stat::MAG)
        }
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
            *i != actor_idx && c.is_alive() && allowed_ids.is_none_or(|ids| ids.contains(&c.id()))
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
    candidates: &mut [usize],
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
        TargetSelector::HighestHp => extrema_indices(candidates, |idx| team[*idx].current_hp(), true),
        TargetSelector::LowestHp => extrema_indices(candidates, |idx| team[*idx].current_hp(), false),
        TargetSelector::HighestMp => extrema_indices(candidates, |idx| team[*idx].current_mp(), true),
        TargetSelector::LowestMp => extrema_indices(candidates, |idx| team[*idx].current_mp(), false),
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
        TargetSelector::Random => candidates.to_owned(),
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
