//! Passive trigger helpers for the battle engine.

use std::collections::HashSet;

use rand::rngs::StdRng;

use crate::abilities::{
    DamageRecord, ExecutionContext, PassiveDef, PassiveMap, PassiveTrigger,
    execute_primitives_with_context,
};
use crate::logger::BattleLog;
use crate::models::{CharacterState, TraitEffect};
use crate::statuses::StatusMap;

pub(crate) struct PassiveRuntime<'a> {
    pub passives: &'a PassiveMap,
    pub status_defs: &'a StatusMap,
    pub rng: &'a mut StdRng,
    pub log: &'a mut BattleLog,
    pub step: u32,
    pub actor_team_is_a: bool,
    pub in_passive_phase: &'a mut bool,
    pub passive_fired_this_tick: &'a mut HashSet<(u32, String, u32)>,
}

impl<'a> PassiveRuntime<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        passives: &'a PassiveMap,
        status_defs: &'a StatusMap,
        rng: &'a mut StdRng,
        log: &'a mut BattleLog,
        step: u32,
        actor_team_is_a: bool,
        in_passive_phase: &'a mut bool,
        passive_fired_this_tick: &'a mut HashSet<(u32, String, u32)>,
    ) -> Self {
        Self {
            passives,
            status_defs,
            rng,
            log,
            step,
            actor_team_is_a,
            in_passive_phase,
            passive_fired_this_tick,
        }
    }
}

pub(crate) fn collect_passive_names(team: &[CharacterState]) -> Vec<(usize, String)> {
    team.iter()
        .enumerate()
        .filter(|(_, c)| !c.passive().is_empty())
        .map(|(i, c)| (i, c.passive().to_string()))
        .collect()
}

pub(crate) fn load_passive(
    runtime: &PassiveRuntime<'_>,
    passive_name: &str,
) -> Option<PassiveDef> {
    runtime.passives.get(passive_name).cloned()
}

pub(crate) fn begin_passive_trigger(
    runtime: &mut PassiveRuntime<'_>,
    actor_team: &[CharacterState],
    char_idx: usize,
    passive_name: &str,
    passive_def: &PassiveDef,
) -> bool {
    if *runtime.in_passive_phase {
        return false;
    }

    if matches!(
        passive_def,
        PassiveDef::Triggered {
            once_per_tick: true,
            ..
        }
    ) {
        let owner_id = actor_team[char_idx].id();
        let key = (owner_id, passive_name.to_string(), runtime.step);
        if runtime.passive_fired_this_tick.contains(&key) {
            return false;
        }
        runtime.passive_fired_this_tick.insert(key);
    }

    *runtime.in_passive_phase = true;
    true
}

pub(crate) fn end_passive_trigger(runtime: &mut PassiveRuntime<'_>) {
    *runtime.in_passive_phase = false;
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn fire_passive_if_matches(
    idx: usize,
    passive_name: &str,
    passive_def: &PassiveDef,
    expected: &PassiveTrigger,
    runtime: &mut PassiveRuntime<'_>,
    actor_team: &mut [CharacterState],
    enemy_team: &mut [CharacterState],
    trigger_target_id: Option<u32>,
) -> Vec<DamageRecord> {
    match passive_def {
        PassiveDef::Triggered {
            trigger,
            primitives,
            ..
        } if std::mem::discriminant(trigger) == std::mem::discriminant(expected) => {
            runtime
                .log
                .push_passive_triggered(runtime.step, &actor_team[idx], passive_name);
            let mut ctx = ExecutionContext::new(
                actor_team,
                enemy_team,
                runtime.rng,
                runtime.log,
                runtime.step,
                runtime.status_defs,
                runtime.actor_team_is_a,
            )
            .with_trigger_target(trigger_target_id);
            let damage = execute_primitives_with_context(&mut ctx, idx, passive_name, primitives);
            if runtime.actor_team_is_a {
                runtime.log.capture_latest_snapshot(actor_team, enemy_team);
            } else {
                runtime.log.capture_latest_snapshot(enemy_team, actor_team);
            }
            damage
        }
        PassiveDef::Trait { effect } if matches!(expected, PassiveTrigger::OnBattleStart) => {
            runtime
                .log
                .push_passive_triggered(runtime.step, &actor_team[idx], passive_name);
            actor_team[idx].add_trait(effect.clone());
            if runtime.actor_team_is_a {
                runtime.log.capture_latest_snapshot(actor_team, enemy_team);
            } else {
                runtime.log.capture_latest_snapshot(enemy_team, actor_team);
            }
            Vec::new()
        }
        PassiveDef::RowAura { .. } => Vec::new(),
        _ => Vec::new(),
    }
}

pub(crate) fn apply_row_auras(team: &mut [CharacterState], passives: &PassiveMap) {
    let sources: Vec<(usize, Vec<crate::abilities::AuraStatEffect>)> = team
        .iter()
        .enumerate()
        .filter(|(_, character)| character.is_alive())
        .filter_map(|(idx, character)| match passives.get(character.passive()) {
            Some(PassiveDef::RowAura { effects }) => Some((idx, effects.clone())),
            _ => None,
        })
        .collect();

    for (source_idx, effects) in sources {
        let source_row = team[source_idx].position().row;
        for (idx, target) in team.iter_mut().enumerate() {
            if idx == source_idx || !target.is_alive() || target.position().row != source_row {
                continue;
            }
            for effect in &effects {
                target.add_trait(TraitEffect::AuraStatMod {
                    stat: effect.stat.clone(),
                    amount: effect.amount,
                });
            }
        }
    }
}
