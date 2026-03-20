//! Turn-execution helpers for the battle engine.

use rand::rngs::StdRng;

use crate::abilities::{AbilityDef, AbilityMap, DamageRecord, execute_ability_for_side};
use crate::logger::BattleLog;
use crate::models::{CharacterState, Stat};
use crate::rules::{WorldState, evaluate_rules};
use crate::statuses::StatusMap;
use crate::targeting::select_target;

pub(crate) struct TurnRuntime<'a> {
    pub abilities: &'a AbilityMap,
    pub status_defs: &'a StatusMap,
    pub rng: &'a mut StdRng,
    pub log: &'a mut BattleLog,
    pub step: u32,
    pub actor_team_is_a: bool,
}

impl<'a> TurnRuntime<'a> {
    pub fn new(
        abilities: &'a AbilityMap,
        status_defs: &'a StatusMap,
        rng: &'a mut StdRng,
        log: &'a mut BattleLog,
        step: u32,
        actor_team_is_a: bool,
    ) -> Self {
        Self {
            abilities,
            status_defs,
            rng,
            log,
            step,
            actor_team_is_a,
        }
    }
}

pub(crate) fn log_turn_start(
    runtime: &mut TurnRuntime<'_>,
    actor_team: &[CharacterState],
    enemy_team: &[CharacterState],
    actor_idx: usize,
) {
    let actor = &actor_team[actor_idx];
    runtime.log.push_turn_start(runtime.step, actor);
    capture_runtime_snapshot(runtime, actor_team, enemy_team);
}

pub(crate) fn log_turn_skipped(
    runtime: &mut TurnRuntime<'_>,
    actor: &CharacterState,
    reason: &str,
    actor_team: &[CharacterState],
    enemy_team: &[CharacterState],
) {
    runtime.log.push_turn_skipped(runtime.step, actor, reason);
    capture_runtime_snapshot(runtime, actor_team, enemy_team);
}

pub(crate) fn resolve_target(
    actor_idx: usize,
    actor_team: &mut [CharacterState],
    enemy_team: &[CharacterState],
    rng: &mut StdRng,
) -> Option<u32> {
    let current = actor_team[actor_idx].target();
    let needs_new = match current {
        Some(tid) => enemy_team
            .iter()
            .find(|c| c.id() == tid)
            .is_none_or(|t| !t.is_alive()),
        None => true,
    };

    if needs_new {
        let new_target = select_target(&actor_team[actor_idx], enemy_team, rng);
        match new_target {
            Some(tid) => {
                actor_team[actor_idx].set_target(tid);
                Some(tid)
            }
            None => None,
        }
    } else {
        current
    }
}

pub(crate) fn choose_ability(
    runtime: &TurnRuntime<'_>,
    actor_idx: usize,
    actor_team: &[CharacterState],
    enemy_team: &[CharacterState],
    target_id: u32,
) -> Option<(String, AbilityDef)> {
    let target_idx = enemy_team.iter().position(|c| c.id() == target_id)?;
    let target_ref = &enemy_team[target_idx];
    let world = WorldState {
        tick_count: runtime.step,
        ally_count: actor_team.iter().filter(|c| c.is_alive()).count() as u32,
        enemy_count: enemy_team.iter().filter(|c| c.is_alive()).count() as u32,
    };
    let ability_name = evaluate_rules(
        &actor_team[actor_idx],
        Some(target_ref),
        actor_team,
        world,
        runtime.abilities,
    )?;
    let ability_def = runtime.abilities.get(&ability_name)?.clone();
    Some((ability_name, ability_def))
}

pub(crate) fn execute_ability_action(
    runtime: &mut TurnRuntime<'_>,
    actor_idx: usize,
    actor_team: &mut [CharacterState],
    enemy_team: &mut [CharacterState],
    ability_name: &str,
    ability_def: &AbilityDef,
) -> (usize, Vec<DamageRecord>) {
    let effective_cost = ability_def
        .mp_cost
        .saturating_sub(actor_team[actor_idx].mp_cost_reduction())
        .max(1);
    actor_team[actor_idx].spend_mp(effective_cost);
    actor_team[actor_idx].record_ability_use(ability_name);

    let event_start = runtime.log.len();
    let damage_dealt = execute_ability_for_side(
        actor_idx,
        ability_name,
        ability_def,
        actor_team,
        enemy_team,
        runtime.actor_team_is_a,
        runtime.rng,
        runtime.log,
        runtime.step,
        runtime.status_defs,
    );

    (event_start, damage_dealt)
}

pub(crate) fn execute_rest_action(
    runtime: &mut TurnRuntime<'_>,
    actor_idx: usize,
    actor_team: &mut [CharacterState],
    enemy_team: &[CharacterState],
) {
    let restored = actor_team[actor_idx].get_base_stat(&Stat::WIL) / 2;
    actor_team[actor_idx].restore_mp(restored);
    runtime.log.push_rest(runtime.step, &actor_team[actor_idx], restored);
    capture_runtime_snapshot(runtime, actor_team, enemy_team);
}

pub(crate) fn finish_turn(
    _runtime: &mut TurnRuntime<'_>,
    actor_idx: usize,
    actor_team: &mut [CharacterState],
) {
    actor_team[actor_idx].decay_statuses_end_of_turn();
    actor_team[actor_idx].decay_conditions_end_of_turn();
    if !actor_team[actor_idx].is_alive() {
        return;
    }
    actor_team[actor_idx].reset_speed();
}

fn capture_runtime_snapshot(
    runtime: &mut TurnRuntime<'_>,
    actor_team: &[CharacterState],
    enemy_team: &[CharacterState],
) {
    if runtime.actor_team_is_a {
        runtime.log.capture_latest_snapshot(actor_team, enemy_team);
    } else {
        runtime.log.capture_latest_snapshot(enemy_team, actor_team);
    }
}
