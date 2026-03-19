//! Turn-execution helpers for the battle engine.

use rand::rngs::StdRng;

use crate::abilities::{AbilityMap, AbilityDef, DamageRecord, execute_ability};
use crate::damage::calc_basic_attack_damage;
use crate::logger::{BattleEvent, BattleLog};
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
}

impl<'a> TurnRuntime<'a> {
    pub fn new(
        abilities: &'a AbilityMap,
        status_defs: &'a StatusMap,
        rng: &'a mut StdRng,
        log: &'a mut BattleLog,
        step: u32,
    ) -> Self {
        Self {
            abilities,
            status_defs,
            rng,
            log,
            step,
        }
    }
}

pub(crate) struct TurnStart {
    pub actor_id: u32,
    pub actor_name: String,
}

pub(crate) fn log_turn_start(
    runtime: &mut TurnRuntime<'_>,
    actor_team: &[CharacterState],
    actor_idx: usize,
) -> TurnStart {
    let actor_id = actor_team[actor_idx].id();
    let actor_name = actor_team[actor_idx].base_name().to_string();

    runtime.log.push(BattleEvent::TurnStart {
        tick_count: runtime.step,
        actor_id,
        actor_name: actor_name.clone(),
        current_hp: actor_team[actor_idx].current_hp(),
        current_mp: actor_team[actor_idx].current_mp(),
    });

    TurnStart {
        actor_id,
        actor_name,
    }
}

pub(crate) fn log_turn_skipped(
    runtime: &mut TurnRuntime<'_>,
    actor_id: u32,
    actor_name: String,
) {
    runtime.log.push(BattleEvent::TurnSkipped {
        tick_count: runtime.step,
        character_id: actor_id,
        character_name: actor_name,
        reason: "incapacitated".to_string(),
    });
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
    let damage_dealt = execute_ability(
        actor_idx,
        ability_name,
        ability_def,
        actor_team,
        enemy_team,
        runtime.rng,
        runtime.log,
        runtime.step,
        runtime.status_defs,
    );

    (event_start, damage_dealt)
}

pub(crate) fn execute_basic_attack_action(
    runtime: &mut TurnRuntime<'_>,
    actor_idx: usize,
    actor_id: u32,
    actor_name: String,
    target_id: u32,
    actor_team: &mut [CharacterState],
    enemy_team: &mut [CharacterState],
) -> Vec<DamageRecord> {
    let target_idx = enemy_team.iter().position(|c| c.id() == target_id).unwrap();
    let damage = calc_basic_attack_damage(&actor_team[actor_idx], &enemy_team[target_idx], runtime.rng);
    let damage = enemy_team[target_idx].take_hit(damage);
    let target_name = enemy_team[target_idx].base_name().to_string();
    let hp_remaining = enemy_team[target_idx].current_hp();

    runtime.log.push(BattleEvent::BasicAttack {
        tick_count: runtime.step,
        actor_id,
        actor_name,
        target_id,
        target_name,
        damage,
        target_hp_remaining: hp_remaining,
    });

    vec![DamageRecord {
        source_id: actor_id,
        target_id,
        damage,
    }]
}

pub(crate) fn finish_turn(
    runtime: &mut TurnRuntime<'_>,
    actor_idx: usize,
    actor_team: &mut [CharacterState],
) {
    if !actor_team[actor_idx].is_alive() {
        return;
    }

    let regen = actor_team[actor_idx].get_base_stat(&Stat::WIL) / 2;
    let actor_id = actor_team[actor_idx].id();
    let actor_name = actor_team[actor_idx].base_name().to_string();
    actor_team[actor_idx].restore_mp(regen);
    if regen > 0 {
        runtime.log.push(BattleEvent::ResourceChanged {
            tick_count: runtime.step,
            actor_id,
            actor_name,
            resource: "mp".to_string(),
            delta: regen as i32,
            value_after: actor_team[actor_idx].current_mp(),
            reason: "turn_regen".to_string(),
        });
    }
    actor_team[actor_idx].reset_speed();
}
