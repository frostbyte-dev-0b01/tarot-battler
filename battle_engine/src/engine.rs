//! Core battle simulation engine.

#[path = "damage_resolution.rs"]
mod damage_resolution;
#[path = "status_resolution.rs"]
mod status_resolution;

use std::collections::HashSet;

use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::abilities::{AbilityMap, DamageRecord, PassiveMap, PassiveTrigger};
use crate::logger::{BattleEvent, BattleLog};
use crate::models::{CharacterConfig, CharacterState, ConditionKind};
use crate::passive_system::{self, PassiveRuntime};
use crate::statuses::StatusMap;
use crate::targeting::select_target;
use crate::turns::{self, TurnRuntime};

const MAX_STEPS: u32 = 1000;
pub struct BattleState {
    team_a: Vec<CharacterState>,
    team_b: Vec<CharacterState>,
    abilities: AbilityMap,
    passives: PassiveMap,
    status_defs: StatusMap,
    step: u32,
    log: BattleLog,
    rng: StdRng,
    in_passive_phase: bool,
    passive_fired_this_tick: HashSet<(u32, String, u32)>,
}

impl BattleState {
    pub fn new(
        team_a_configs: &[CharacterConfig],
        team_b_configs: &[CharacterConfig],
        abilities: AbilityMap,
        passives: PassiveMap,
        status_defs: StatusMap,
        seed: u64,
    ) -> Self {
        let rng = StdRng::seed_from_u64(seed);

        let team_a: Vec<CharacterState> = team_a_configs
            .iter()
            .enumerate()
            .map(|(i, c)| {
                CharacterState::from_config_with_identity(
                    i as u32,
                    replay_character_id(c, "team_a", i),
                    replay_display_name(c),
                    c,
                )
                
            })
            .collect();

        let n = team_a.len() as u32;
        let team_b: Vec<CharacterState> = team_b_configs
            .iter()
            .enumerate()
            .map(|(i, c)| {
                CharacterState::from_config_with_identity(
                    n + i as u32,
                    replay_character_id(c, "team_b", i),
                    replay_display_name(c),
                    c,
                )
                
            })
            .collect();

        let mut state = Self {
            team_a,
            team_b,
            abilities,
            passives,
            status_defs,
            step: 0,
            log: BattleLog::new(),
            rng,
            in_passive_phase: false,
            passive_fired_this_tick: HashSet::new(),
        };
        state.assign_companions();
        state.assign_all_targets();
        state
    }

    /// Assigns companions based on cardinal adjacency within each team.
    /// Set once at battle start and never reassigned.
    fn assign_companions(&mut self) {
        Self::assign_team_companions(&mut self.team_a);
        Self::assign_team_companions(&mut self.team_b);
    }

    fn assign_team_companions(team: &mut [CharacterState]) {
        let positions: Vec<(u32, crate::models::Position)> = team
            .iter()
            .map(|c| (c.id(), c.position().clone()))
            .collect();

        for c in team.iter_mut() {
            let comps: Vec<u32> = positions
                .iter()
                .filter(|(id, pos)| *id != c.id() && c.position().is_adjacent(pos))
                .map(|(id, _)| *id)
                .collect();
            c.set_companions(comps);
        }
    }

    fn assign_all_targets(&mut self) {
        for attacker in &mut self.team_a {
            if let Some(target_id) = select_target(attacker, &self.team_b, &mut self.rng) {
                attacker.set_target(target_id);
            }
        }
        for attacker in &mut self.team_b {
            if let Some(target_id) = select_target(attacker, &self.team_a, &mut self.rng) {
                attacker.set_target(target_id);
            }
        }
    }

    pub fn run(mut self) -> BattleLog {
        self.log.capture_initial_snapshot(&self.team_a, &self.team_b);
        self.log.push(BattleEvent::BattleStart {
            tick_count: 0,
            team_a: self
                .team_a
                .iter()
                .map(|c| {
                    format!(
                        "{} (r{}, c{})",
                        c.base_name(),
                        c.position().row,
                        c.position().col
                    )
                })
                .collect(),
            team_b: self
                .team_b
                .iter()
                .map(|c| {
                    format!(
                        "{} (r{}, c{})",
                        c.base_name(),
                        c.position().row,
                        c.position().col
                    )
                })
                .collect(),
        });
        self.capture_latest_replay_snapshot();

        self.execute_battle_start_passives();

        loop {
            if self.step_once() {
                break;
            }
        }

        self.log
    }

    pub(crate) fn capture_latest_replay_snapshot(&mut self) {
        self.log.capture_latest_snapshot(&self.team_a, &self.team_b);
    }

    pub(crate) fn refresh_latest_replay_snapshot(&mut self) {
        self.log.refresh_latest_snapshot(&self.team_a, &self.team_b);
    }

    /// Fire on_battle_start passives and apply permanent traits for all characters.
    fn execute_battle_start_passives(&mut self) {
        let team_a_passives = passive_system::collect_passive_names(&self.team_a);
        let team_b_passives = passive_system::collect_passive_names(&self.team_b);
        let trigger = PassiveTrigger::OnBattleStart;

        for (idx, passive_name) in team_a_passives {
            let mut runtime = PassiveRuntime::new(
                &self.passives,
                &self.status_defs,
                &mut self.rng,
                &mut self.log,
                0,
                true,
                &mut self.in_passive_phase,
                &mut self.passive_fired_this_tick,
            );
            if let Some(passive_def) = passive_system::load_passive(&runtime, &passive_name) {
                let damage_dealt = passive_system::fire_passive_if_matches(
                    idx,
                    &passive_name,
                    &passive_def,
                    &trigger,
                    &mut runtime,
                    &mut self.team_a,
                    &mut self.team_b,
                    None,
                );
                self.resolve_defeats_from_damage(&damage_dealt, false);
            }
        }

        for (idx, passive_name) in team_b_passives {
            let mut runtime = PassiveRuntime::new(
                &self.passives,
                &self.status_defs,
                &mut self.rng,
                &mut self.log,
                0,
                false,
                &mut self.in_passive_phase,
                &mut self.passive_fired_this_tick,
            );
            if let Some(passive_def) = passive_system::load_passive(&runtime, &passive_name) {
                let damage_dealt = passive_system::fire_passive_if_matches(
                    idx,
                    &passive_name,
                    &passive_def,
                    &trigger,
                    &mut runtime,
                    &mut self.team_b,
                    &mut self.team_a,
                    None,
                );
                self.resolve_defeats_from_damage(&damage_dealt, true);
            }
        }

        self.refresh_auras();
    }

    fn refresh_auras(&mut self) {
        Self::clear_team_auras(&mut self.team_a);
        Self::clear_team_auras(&mut self.team_b);

        passive_system::apply_row_auras(&mut self.team_a, &self.passives);
        passive_system::apply_row_auras(&mut self.team_b, &self.passives);
    }

    fn clear_team_auras(team: &mut [CharacterState]) {
        for character in team {
            character.clear_aura_traits();
        }
    }

    /// Look up a character's passive and fire it if it matches the trigger.
    /// Sets in_passive_phase during execution. Returns damage dealt.
    fn try_fire_passive(
        &mut self,
        char_idx: usize,
        trigger: &PassiveTrigger,
        actor_team_is_a: bool,
    ) -> Vec<DamageRecord> {
        self.try_fire_passive_with_target(char_idx, trigger, actor_team_is_a, None)
    }

    fn try_fire_passive_with_target(
        &mut self,
        char_idx: usize,
        trigger: &PassiveTrigger,
        actor_team_is_a: bool,
        trigger_target_id: Option<u32>,
    ) -> Vec<DamageRecord> {
        let (actor_team, enemy_team) = if actor_team_is_a {
            (
                &mut self.team_a as &mut [CharacterState],
                &mut self.team_b as &mut [CharacterState],
            )
        } else {
            (
                &mut self.team_b as &mut [CharacterState],
                &mut self.team_a as &mut [CharacterState],
            )
        };

        let passive_name = actor_team[char_idx].passive().to_string();
        if passive_name.is_empty() {
            return Vec::new();
        }

        let mut runtime = PassiveRuntime::new(
            &self.passives,
            &self.status_defs,
            &mut self.rng,
            &mut self.log,
            self.step,
            actor_team_is_a,
            &mut self.in_passive_phase,
            &mut self.passive_fired_this_tick,
        );
        let passive_def = match passive_system::load_passive(&runtime, &passive_name) {
            Some(def) => def,
            None => return Vec::new(),
        };
        if !passive_system::begin_passive_trigger(
            &mut runtime,
            actor_team,
            char_idx,
            &passive_name,
            &passive_def,
        ) {
            return Vec::new();
        }

        let damage_dealt = passive_system::fire_passive_if_matches(
            char_idx,
            &passive_name,
            &passive_def,
            trigger,
            &mut runtime,
            actor_team,
            enemy_team,
            trigger_target_id,
        );
        passive_system::end_passive_trigger(&mut runtime);
        self.resolve_defeats_from_damage(&damage_dealt, !actor_team_is_a);
        damage_dealt
    }

    /// Advance one step. Returns true if the battle is over.
    fn step_once(&mut self) -> bool {
        self.step += 1;

        if self.step > MAX_STEPS {
            self.log.push(BattleEvent::BattleEnd {
                tick_count: self.step,
                winner: "draw".to_string(),
            });
            self.capture_latest_replay_snapshot();
            return true;
        }

        // Tick speed for all living characters, collect those ready to act
        let mut ready_a: Vec<usize> = Vec::new();
        for (i, c) in self.team_a.iter_mut().enumerate() {
            if c.is_alive() && c.tick_speed() {
                ready_a.push(i);
            }
        }
        let mut ready_b: Vec<usize> = Vec::new();
        for (i, c) in self.team_b.iter_mut().enumerate() {
            if c.is_alive() && c.tick_speed() {
                ready_b.push(i);
            }
        }

        // Execute turns for ready characters (re-check alive — may have died this step)
        for idx in ready_a {
            if self.team_a[idx].is_alive() {
                self.execute_turn(idx, true);
            }
        }
        for idx in ready_b {
            if self.team_b[idx].is_alive() {
                self.execute_turn(idx, false);
            }
        }

        // Check win conditions
        let a_alive = self.team_a.iter().any(|c| c.is_alive());
        let b_alive = self.team_b.iter().any(|c| c.is_alive());

        if !a_alive && !b_alive {
            self.log.push(BattleEvent::BattleEnd {
                tick_count: self.step,
                winner: "draw".to_string(),
            });
            self.capture_latest_replay_snapshot();
            true
        } else if !b_alive {
            self.log.push(BattleEvent::BattleEnd {
                tick_count: self.step,
                winner: "team_a".to_string(),
            });
            self.capture_latest_replay_snapshot();
            true
        } else if !a_alive {
            self.log.push(BattleEvent::BattleEnd {
                tick_count: self.step,
                winner: "team_b".to_string(),
            });
            self.capture_latest_replay_snapshot();
            true
        } else {
            false
        }
    }

    /// Execute a turn for a character. `is_team_a` determines which team the actor belongs to.
    fn execute_turn(&mut self, actor_idx: usize, is_team_a: bool) {
        {
            let (actor_team, _) = Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
            actor_team[actor_idx].increment_turn_count();
        }

        // on_turn_start fires before status ticks and action resolution.
        self.try_fire_passive(actor_idx, &PassiveTrigger::OnTurnStart, is_team_a);

        // Start-of-turn statuses tick even when the actor is stunned.
        self.tick_and_log_statuses(actor_idx, is_team_a);

        let actor_alive = if is_team_a {
            self.team_a[actor_idx].is_alive()
        } else {
            self.team_b[actor_idx].is_alive()
        };
        if !actor_alive {
            return;
        }

        {
            let mut runtime = TurnRuntime::new(
                &self.abilities,
                &self.status_defs,
                &mut self.rng,
                &mut self.log,
                self.step,
                is_team_a,
            );
            let (actor_team, enemy_team) = if is_team_a {
                (&self.team_a[..], &self.team_b[..])
            } else {
                (&self.team_b[..], &self.team_a[..])
            };
            turns::log_turn_start(&mut runtime, actor_team, enemy_team, actor_idx);
        }

        // Incapacitate check happens after start-of-turn passives and status ticks.
        let actor_incapacitated = if is_team_a {
            self.team_a[actor_idx].is_incapacitated()
        } else {
            self.team_b[actor_idx].is_incapacitated()
        };
        if actor_incapacitated {
            let mut runtime = TurnRuntime::new(
                &self.abilities,
                &self.status_defs,
                &mut self.rng,
                &mut self.log,
                self.step,
                is_team_a,
            );
            let (actor_team, enemy_team) = if is_team_a {
                (&self.team_a[..], &self.team_b[..])
            } else {
                (&self.team_b[..], &self.team_a[..])
            };
            let reason = if actor_team[actor_idx].has_condition(ConditionKind::Stunned) {
                "stunned"
            } else {
                "incapacitated"
            };
            turns::log_turn_skipped(
                &mut runtime,
                &actor_team[actor_idx],
                reason,
                actor_team,
                enemy_team,
            );
            let (actor_team, _) = Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
            actor_team[actor_idx].consume_skip_turn_statuses();
            self.finish_turn(actor_idx, is_team_a);
            return;
        }

        // Get or reassign target
        let (actor_team, enemy_team) =
            Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
        let target_id = {
            let runtime = TurnRuntime::new(
                &self.abilities,
                &self.status_defs,
                &mut self.rng,
                &mut self.log,
                self.step,
                is_team_a,
            );
            match turns::resolve_target(actor_idx, actor_team, enemy_team, runtime.rng) {
                Some(tid) => tid,
                None => {
                    self.finish_turn(actor_idx, is_team_a);
                    return;
                }
            }
        };

        let ability_choice = {
            let runtime = TurnRuntime::new(
                &self.abilities,
                &self.status_defs,
                &mut self.rng,
                &mut self.log,
                self.step,
                is_team_a,
            );
            turns::choose_ability(&runtime, actor_idx, actor_team, enemy_team, target_id)
        };

        if let Some((ability_name, ability_def)) = ability_choice {
            let (actor_team, enemy_team) =
                Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
            let (event_start, damage_dealt) = {
                let mut runtime = TurnRuntime::new(
                    &self.abilities,
                    &self.status_defs,
                    &mut self.rng,
                    &mut self.log,
                    self.step,
                    is_team_a,
                );
                turns::execute_ability_action(
                    &mut runtime,
                    actor_idx,
                    actor_team,
                    enemy_team,
                    &ability_name,
                    &ability_def,
                )
            };

            self.process_status_application_events(event_start, is_team_a);

            // Process damage results: defeats, reflect, and passive triggers
            self.process_damage_results(actor_idx, is_team_a, &damage_dealt);

            // Reassign target if current target is dead
            let (actor_team, enemy_team) =
                Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
            Self::reassign_target_if_dead(actor_idx, actor_team, enemy_team, &mut self.rng);

            self.finish_turn(actor_idx, is_team_a);
            return;
        }

        // Fallback: Rest
        let (actor_team, enemy_team) = Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
        let mut runtime = TurnRuntime::new(
            &self.abilities,
            &self.status_defs,
            &mut self.rng,
            &mut self.log,
            self.step,
            is_team_a,
        );
        turns::execute_rest_action(&mut runtime, actor_idx, actor_team, enemy_team);
        self.finish_turn(actor_idx, is_team_a);
    }

    fn finish_turn(&mut self, actor_idx: usize, is_team_a: bool) {
        self.refresh_auras();
        let (actor_team, _) = Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
        let mut runtime = TurnRuntime::new(
            &self.abilities,
            &self.status_defs,
            &mut self.rng,
            &mut self.log,
            self.step,
            is_team_a,
        );
        turns::finish_turn(&mut runtime, actor_idx, actor_team);
        self.refresh_latest_replay_snapshot();
    }

    /// Split teams into (actor_team, enemy_team) based on which team the actor is on.
    fn teams_mut<'a>(
        team_a: &'a mut Vec<CharacterState>,
        team_b: &'a mut Vec<CharacterState>,
        is_team_a: bool,
    ) -> (&'a mut [CharacterState], &'a mut [CharacterState]) {
        if is_team_a {
            (team_a.as_mut_slice(), team_b.as_mut_slice())
        } else {
            (team_b.as_mut_slice(), team_a.as_mut_slice())
        }
    }

    /// Reassign target if current target is dead.
    fn reassign_target_if_dead(
        actor_idx: usize,
        actor_team: &mut [CharacterState],
        enemy_team: &[CharacterState],
        rng: &mut StdRng,
    ) {
        if let Some(ct) = actor_team[actor_idx].target()
            && enemy_team
                .iter()
                .find(|c| c.id() == ct)
                .is_none_or(|c| !c.is_alive())
        {
            if let Some(tid) = select_target(&actor_team[actor_idx], enemy_team, rng) {
                actor_team[actor_idx].set_target(tid);
            } else {
                actor_team[actor_idx].clear_target();
            }
        }
    }

}

fn replay_character_id(config: &CharacterConfig, team_key: &str, index: usize) -> String {
    config
        .id
        .clone()
        .filter(|id| !id.trim().is_empty())
        .unwrap_or_else(|| format!("{team_key}_{index}"))
}

fn replay_display_name(config: &CharacterConfig) -> String {
    config
        .display_name
        .clone()
        .unwrap_or_else(|| config.base_name.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abilities::{
        AbilityDef, AuraStatEffect, PassiveDef, PassiveMap, Primitive, SimpleAbilityTarget,
        execute_ability,
    };
    use crate::logger::BattleEvent;
    use crate::models::{
        Comparator, Condition, ConditionSubject, Position, QueryValue, Rule, Stat,
    };
    use crate::statuses::{StackType, StatusBehavior, StatusDef};
    use crate::test_support::{
        build_battle, empty_abilities, empty_passives, empty_statuses, mage, make_config,
        make_config_at, ward_statuses, warrior,
    };
    use std::collections::HashMap;

    #[test]
    fn row_aura_applies_to_same_row_allies() {
        let mut emperor = make_config_at("Emperor", 0, 0, vec![(Stat::MGT, 10), (Stat::VIT, 10)]);
        emperor.passive = "Imperial Formation".to_string();
        let ally = make_config_at("Ally", 0, 1, vec![(Stat::MGT, 8), (Stat::VIT, 10)]);
        let other = make_config_at("Other", 1, 1, vec![(Stat::MGT, 8), (Stat::VIT, 10)]);

        let mut passives = PassiveMap::new();
        passives.insert(
            "Imperial Formation".to_string(),
            PassiveDef::RowAura {
                effects: vec![AuraStatEffect {
                    stat: Stat::MGT,
                    amount: 1,
                }],
            },
        );

        let battle = build_battle(
            &[emperor, ally, other],
            &[mage()],
            empty_abilities(),
            passives,
            empty_statuses(),
        );
        let mut battle = battle;
        battle.refresh_auras();

        assert_eq!(battle.team_a[1].get_eff_stat(&Stat::MGT), 9);
        assert_eq!(battle.team_a[2].get_eff_stat(&Stat::MGT), 8);
    }

    #[test]
    fn row_aura_recomputes_after_movement_and_death() {
        let mut emperor = make_config_at("Emperor", 0, 0, vec![(Stat::MGT, 10), (Stat::VIT, 10)]);
        emperor.passive = "Imperial Formation".to_string();
        let ally = make_config_at("Ally", 0, 1, vec![(Stat::MGT, 8), (Stat::VIT, 10)]);
        let mover = make_config_at("Mover", 1, 1, vec![(Stat::MGT, 8), (Stat::VIT, 10)]);

        let mut passives = PassiveMap::new();
        passives.insert(
            "Imperial Formation".to_string(),
            PassiveDef::RowAura {
                effects: vec![AuraStatEffect {
                    stat: Stat::MGT,
                    amount: 1,
                }],
            },
        );

        let mut battle = build_battle(
            &[emperor, ally, mover],
            &[mage()],
            empty_abilities(),
            passives,
            empty_statuses(),
        );
        battle.refresh_auras();
        assert_eq!(battle.team_a[1].get_eff_stat(&Stat::MGT), 9);
        assert_eq!(battle.team_a[2].get_eff_stat(&Stat::MGT), 8);

        battle.team_a[2].set_position(Position { row: 0, col: 1 });
        battle.refresh_auras();
        assert_eq!(battle.team_a[2].get_eff_stat(&Stat::MGT), 9);

        battle.team_a[0].take_damage(999);
        battle.resolve_character_death(0, true);
        assert_eq!(battle.team_a[1].get_eff_stat(&Stat::MGT), 8);
        assert_eq!(battle.team_a[2].get_eff_stat(&Stat::MGT), 8);
    }

    #[test]
    fn on_ally_damage_my_target_fires_for_matching_target() {
        let striker = make_config_at(
            "Striker",
            0,
            0,
            vec![(Stat::MGT, 10), (Stat::VIT, 10), (Stat::WIL, 5)],
        );
        let mut chariot = make_config_at(
            "Chariot",
            0,
            1,
            vec![(Stat::MGT, 8), (Stat::VIT, 10), (Stat::WIL, 5)],
        );
        chariot.passive = "Pursuit".to_string();
        let enemy = make_config_at("Target", 0, 0, vec![(Stat::ARM, 3), (Stat::VIT, 10)]);

        let mut passives = PassiveMap::new();
        passives.insert(
            "Pursuit".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnAllyDamageMyTarget,
                once_per_tick: false,
                primitives: vec![Primitive::ApplyStatus {
                    target: SimpleAbilityTarget::SelfChar.into(),
                    status: "Empower".to_string(),
                    stat: Some(Stat::MGT),
                    stacks: 1,
                }],
            },
        );
        let statuses = [(
            "Empower".to_string(),
            StatusDef {
                behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
                stack_type: StackType::TickDown,
                group: None,
                opposes: Some("Weaken".to_string()),
            },
        )]
        .into_iter()
        .collect();

        let mut battle = build_battle(
            &[striker, chariot],
            &[enemy],
            empty_abilities(),
            passives,
            statuses,
        );
        battle.team_a[0].set_target(battle.team_b[0].id());
        battle.team_a[1].set_target(battle.team_b[0].id());

        battle.process_damage_results(
            0,
            true,
            &[DamageRecord {
                source_id: battle.team_a[0].id(),
                target_id: battle.team_b[0].id(),
                damage: 3,
            }],
        );

        assert_eq!(
            battle.team_a[1].status_stacks(&crate::statuses::status_key("Empower", Some(&Stat::MGT))),
            1
        );
    }

    #[test]
    fn on_ally_damage_my_target_does_not_fire_for_self_or_other_targets() {
        let mut chariot = make_config_at(
            "Chariot",
            0,
            1,
            vec![(Stat::MGT, 8), (Stat::VIT, 10), (Stat::WIL, 5)],
        );
        chariot.passive = "Pursuit".to_string();
        let ally = make_config_at(
            "Ally",
            0,
            0,
            vec![(Stat::MGT, 10), (Stat::VIT, 10), (Stat::WIL, 5)],
        );
        let enemy_a = make_config_at("EnemyA", 0, 0, vec![(Stat::ARM, 3), (Stat::VIT, 10)]);
        let enemy_b = make_config_at("EnemyB", 0, 1, vec![(Stat::ARM, 3), (Stat::VIT, 10)]);

        let mut passives = PassiveMap::new();
        passives.insert(
            "Pursuit".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnAllyDamageMyTarget,
                once_per_tick: false,
                primitives: vec![Primitive::ApplyStatus {
                    target: SimpleAbilityTarget::SelfChar.into(),
                    status: "Empower".to_string(),
                    stat: Some(Stat::MGT),
                    stacks: 1,
                }],
            },
        );
        let statuses = [(
            "Empower".to_string(),
            StatusDef {
                behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
                stack_type: StackType::TickDown,
                group: None,
                opposes: Some("Weaken".to_string()),
            },
        )]
        .into_iter()
        .collect();

        let mut battle = build_battle(
            &[ally, chariot],
            &[enemy_a, enemy_b],
            empty_abilities(),
            passives,
            statuses,
        );
        battle.team_a[1].set_target(battle.team_b[0].id());

        battle.process_damage_results(
            1,
            true,
            &[DamageRecord {
                source_id: battle.team_a[1].id(),
                target_id: battle.team_b[0].id(),
                damage: 3,
            }],
        );
        assert_eq!(
            battle.team_a[1].status_stacks(&crate::statuses::status_key("Empower", Some(&Stat::MGT))),
            0
        );

        battle.process_damage_results(
            0,
            true,
            &[DamageRecord {
                source_id: battle.team_a[0].id(),
                target_id: battle.team_b[1].id(),
                damage: 3,
            }],
        );
        assert_eq!(
            battle.team_a[1].status_stacks(&crate::statuses::status_key("Empower", Some(&Stat::MGT))),
            0
        );
    }

    #[test]
    fn on_ally_apply_omen_applies_extra_omen_to_same_target() {
        let mut moon = make_config_at(
            "Moon",
            0,
            0,
            vec![(Stat::MAG, 8), (Stat::VIT, 10), (Stat::WIL, 5)],
        );
        moon.actives = vec!["Hex".to_string()];
        let mut magician = make_config_at(
            "Magician",
            0,
            1,
            vec![(Stat::MAG, 6), (Stat::VIT, 10), (Stat::WIL, 5)],
        );
        magician.passive = "Catalyst".to_string();
        let enemy = make_config_at("Target", 0, 0, vec![(Stat::RES, 3), (Stat::VIT, 10)]);

        let mut abilities = AbilityMap::new();
        abilities.insert(
            "Hex".to_string(),
            crate::abilities::AbilityDef {
                mp_cost: 2,
                primitives: vec![Primitive::ApplyStatus {
                    target: SimpleAbilityTarget::CurrentTarget.into(),
                    status: "Omen".to_string(),
                    stat: None,
                    stacks: 2,
                }],
            },
        );

        let mut passives = PassiveMap::new();
        passives.insert(
            "Catalyst".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnAllyApplyOmen,
                once_per_tick: false,
                primitives: vec![Primitive::ApplyStatus {
                    target: SimpleAbilityTarget::TriggerTarget.into(),
                    status: "Omen".to_string(),
                    stat: None,
                    stacks: 1,
                }],
            },
        );

        let statuses = [(
            "Omen".to_string(),
            StatusDef {
                behavior: StatusBehavior::DamagePerStack { value: 1 },
                stack_type: StackType::TickDown,
                group: Some(crate::statuses::StatusGroup::Fate),
                opposes: None,
            },
        )]
        .into_iter()
        .collect();

        let mut battle = BattleState::new(&[moon, magician], &[enemy], abilities, passives, statuses, 42);
        battle.team_a[0].set_target(battle.team_b[0].id());

        let event_start = battle.log.len();
        execute_ability(
            0,
            "Hex",
            battle.abilities.get("Hex").unwrap(),
            &mut battle.team_a,
            &mut battle.team_b,
            &mut battle.rng,
            &mut battle.log,
            1,
            &battle.status_defs,
        );
        battle.process_status_application_events(event_start, true);

        assert_eq!(battle.team_b[0].status_stacks("Omen"), 3);
    }

    #[test]
    fn once_per_tick_passive_only_fires_once_for_multiple_omen_applications() {
        let mut moon = make_config_at(
            "Moon",
            0,
            0,
            vec![(Stat::MAG, 8), (Stat::VIT, 10), (Stat::WIL, 5)],
        );
        moon.actives = vec!["DoubleHex".to_string()];
        let mut magician = make_config_at(
            "Magician",
            0,
            1,
            vec![(Stat::MAG, 6), (Stat::VIT, 10), (Stat::WIL, 5)],
        );
        magician.passive = "Catalyst".to_string();
        let enemy = make_config_at("Target", 0, 0, vec![(Stat::RES, 3), (Stat::VIT, 10)]);

        let mut abilities = AbilityMap::new();
        abilities.insert(
            "DoubleHex".to_string(),
            crate::abilities::AbilityDef {
                mp_cost: 2,
                primitives: vec![
                    Primitive::ApplyStatus {
                        target: SimpleAbilityTarget::CurrentTarget.into(),
                        status: "Omen".to_string(),
                        stat: None,
                        stacks: 1,
                    },
                    Primitive::ApplyStatus {
                        target: SimpleAbilityTarget::CurrentTarget.into(),
                        status: "Omen".to_string(),
                        stat: None,
                        stacks: 1,
                    },
                ],
            },
        );

        let mut passives = PassiveMap::new();
        passives.insert(
            "Catalyst".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnAllyApplyOmen,
                once_per_tick: true,
                primitives: vec![Primitive::ApplyStatus {
                    target: SimpleAbilityTarget::TriggerTarget.into(),
                    status: "Omen".to_string(),
                    stat: None,
                    stacks: 1,
                }],
            },
        );

        let statuses = [(
            "Omen".to_string(),
            StatusDef {
                behavior: StatusBehavior::DamagePerStack { value: 1 },
                stack_type: StackType::TickDown,
                group: Some(crate::statuses::StatusGroup::Fate),
                opposes: None,
            },
        )]
        .into_iter()
        .collect();

        let mut battle = BattleState::new(&[moon, magician], &[enemy], abilities, passives, statuses, 42);
        battle.team_a[0].set_target(battle.team_b[0].id());

        let event_start = battle.log.len();
        execute_ability(
            0,
            "DoubleHex",
            battle.abilities.get("DoubleHex").unwrap(),
            &mut battle.team_a,
            &mut battle.team_b,
            &mut battle.rng,
            &mut battle.log,
            1,
            &battle.status_defs,
        );
        battle.process_status_application_events(event_start, true);

        assert_eq!(battle.team_b[0].status_stacks("Omen"), 3);
    }

    #[test]
    fn battle_produces_start_and_end_events() {
        let log = BattleState::new(
            &[warrior()],
            &[mage()],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            42,
        )
        .run();
        let events = log.events();
        assert!(events.len() >= 2);
        assert!(matches!(&events[0], BattleEvent::BattleStart { .. }));
        assert!(matches!(
            events.last().unwrap(),
            BattleEvent::BattleEnd { .. }
        ));
    }

    #[test]
    fn battle_is_deterministic_with_same_seed() {
        let log1 = BattleState::new(
            &[warrior()],
            &[mage()],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            123,
        )
        .run()
        .to_json();
        let log2 = BattleState::new(
            &[warrior()],
            &[mage()],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            123,
        )
        .run()
        .to_json();
        assert_eq!(log1, log2);
    }

    #[test]
    fn battle_has_winner() {
        let log = BattleState::new(
            &[warrior()],
            &[mage()],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            42,
        )
        .run();
        let events = log.events();
        match events.last().unwrap() {
            BattleEvent::BattleEnd { winner, .. } => {
                assert!(winner == "team_a" || winner == "team_b" || winner == "draw");
            }
            _ => panic!("Last event should be BattleEnd"),
        }
    }

    #[test]
    fn battle_contains_defeat_event() {
        let log = BattleState::new(
            &[warrior()],
            &[mage()],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            42,
        )
        .run();
        let has_defeat = log
            .events()
            .iter()
            .any(|e| matches!(e, BattleEvent::Defeat { .. }));
        assert!(has_defeat, "A 1v1 battle should have a Defeat event");
    }

    #[test]
    fn battle_ids_are_unique_across_teams() {
        let battle = BattleState::new(
            &[warrior(), warrior()],
            &[mage(), mage()],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            0,
        );
        let log = battle.run();
        assert!(log.events().len() > 2);
    }

    #[test]
    fn high_con_tank_survives_longer() {
        let tank = make_config(
            "Tank",
            0,
            vec![
                (Stat::VIT, 50),
                (Stat::MGT, 8),
                (Stat::MAG, 4),
                (Stat::ARM, 10),
                (Stat::RES, 10),
                (Stat::SPD, 5),
                (Stat::WIL, 5),
            ],
        );
        let glass = make_config(
            "Glass",
            0,
            vec![
                (Stat::VIT, 3),
                (Stat::MGT, 20),
                (Stat::MAG, 4),
                (Stat::ARM, 2),
                (Stat::RES, 2),
                (Stat::SPD, 5),
                (Stat::WIL, 5),
            ],
        );
        let log = BattleState::new(
            &[tank],
            &[glass],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            42,
        )
        .run();
        match log.events().last().unwrap() {
            BattleEvent::BattleEnd { winner, .. } => {
                assert_eq!(winner, "team_a");
            }
            _ => panic!("Expected BattleEnd"),
        }
    }

    #[test]
    fn draw_safety_triggers_at_max_steps() {
        let tanky = make_config(
            "Tanky",
            0,
            vec![
                (Stat::VIT, 200),
                (Stat::MGT, 1),
                (Stat::MAG, 1),
                (Stat::ARM, 50),
                (Stat::RES, 50),
                (Stat::SPD, 30),
                (Stat::WIL, 5),
            ],
        );
        let log = BattleState::new(
            &[tanky.clone()],
            &[tanky],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            0,
        )
        .run();
        match log.events().last().unwrap() {
            BattleEvent::BattleEnd { winner, .. } => {
                assert_eq!(winner, "draw");
            }
            _ => panic!("Expected BattleEnd"),
        }
    }

    #[test]
    fn three_v_three_completes() {
        let front1 = make_config(
            "Front1",
            0,
            vec![
                (Stat::VIT, 10),
                (Stat::MGT, 8),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );
        let front2 = make_config(
            "Front2",
            0,
            vec![
                (Stat::VIT, 8),
                (Stat::MGT, 6),
                (Stat::MAG, 4),
                (Stat::ARM, 4),
                (Stat::RES, 4),
                (Stat::SPD, 6),
                (Stat::WIL, 5),
            ],
        );
        let back = make_config(
            "Back",
            1,
            vec![
                (Stat::VIT, 6),
                (Stat::MGT, 3),
                (Stat::MAG, 10),
                (Stat::ARM, 2),
                (Stat::RES, 8),
                (Stat::SPD, 4),
                (Stat::WIL, 7),
            ],
        );
        let log = BattleState::new(
            &[front1.clone(), back.clone()],
            &[front2.clone(), back.clone(), front1.clone()],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            99,
        )
        .run();
        let events = log.events();
        assert!(matches!(
            events.last().unwrap(),
            BattleEvent::BattleEnd { .. }
        ));
        let defeat_count = events
            .iter()
            .filter(|e| matches!(e, BattleEvent::Defeat { .. }))
            .count();
        assert!(
            defeat_count >= 2,
            "3v3 should have at least 2 defeats, got {}",
            defeat_count
        );
    }

    #[test]
    fn row_protection_prevents_back_row_targeting() {
        let front = make_config(
            "Front",
            0,
            vec![
                (Stat::VIT, 15),
                (Stat::MGT, 6),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );
        let squishy_back = make_config(
            "SquishyBack",
            1,
            vec![
                (Stat::VIT, 3),
                (Stat::MGT, 3),
                (Stat::MAG, 10),
                (Stat::ARM, 1),
                (Stat::RES, 8),
                (Stat::SPD, 4),
                (Stat::WIL, 7),
            ],
        );
        let attacker = make_config(
            "Attacker",
            0,
            vec![
                (Stat::VIT, 12),
                (Stat::MGT, 10),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 6),
                (Stat::WIL, 4),
            ],
        );
        let log = BattleState::new(
            &[attacker],
            &[front, squishy_back],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            42,
        )
        .run();

        let mut front_defeated = false;
        for event in log.events() {
            match event {
                BattleEvent::Defeat { character_name, .. } if character_name == "Front" => {
                    front_defeated = true;
                }
                BattleEvent::BasicAttack {
                    actor_id,
                    target_name,
                    ..
                } if *actor_id == 0 => {
                    if !front_defeated {
                        assert_eq!(
                            target_name, "Front",
                            "Back row targeted before front row defeated"
                        );
                    }
                }
                _ => {}
            }
        }
    }

    #[test]
    fn all_enemies_defeated_means_victory() {
        let fighter = make_config(
            "Fighter",
            0,
            vec![
                (Stat::VIT, 10),
                (Stat::MGT, 8),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );
        let lone = make_config(
            "Lone",
            0,
            vec![
                (Stat::VIT, 8),
                (Stat::MGT, 6),
                (Stat::MAG, 3),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );
        let log = BattleState::new(
            &[fighter.clone(), fighter.clone(), fighter],
            &[lone],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            42,
        )
        .run();
        match log.events().last().unwrap() {
            BattleEvent::BattleEnd { winner, .. } => assert_eq!(winner, "team_a"),
            _ => panic!("Expected BattleEnd"),
        }
    }

    #[test]
    fn multi_row_formation_two_rows() {
        let tanky = make_config(
            "Tank",
            0,
            vec![
                (Stat::VIT, 15),
                (Stat::MGT, 5),
                (Stat::MAG, 3),
                (Stat::ARM, 8),
                (Stat::RES, 5),
                (Stat::SPD, 4),
                (Stat::WIL, 4),
            ],
        );
        let dps = make_config(
            "DPS",
            1,
            vec![
                (Stat::VIT, 6),
                (Stat::MGT, 12),
                (Stat::MAG, 3),
                (Stat::ARM, 3),
                (Stat::RES, 2),
                (Stat::SPD, 6),
                (Stat::WIL, 4),
            ],
        );
        let log = BattleState::new(
            &[tanky.clone(), dps.clone()],
            &[tanky, dps.clone(), dps],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            7,
        )
        .run();
        let events = log.events();
        assert!(matches!(
            events.last().unwrap(),
            BattleEvent::BattleEnd { .. }
        ));
        let unique_actors: std::collections::HashSet<u32> = events
            .iter()
            .filter_map(|e| match e {
                BattleEvent::AbilityUsed {
                    actor_id,
                    ability_name,
                    ..
                } if ability_name == "Strike" => Some(*actor_id),
                _ => None,
            })
            .collect();
        assert!(
            unique_actors.len() >= 3,
            "Expected at least 3 unique actors, got {}",
            unique_actors.len()
        );
    }

    #[test]
    fn dead_characters_do_not_act() {
        let front = make_config(
            "Front",
            0,
            vec![
                (Stat::VIT, 8),
                (Stat::MGT, 8),
                (Stat::MAG, 3),
                (Stat::ARM, 4),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );
        let back = make_config(
            "Back",
            1,
            vec![
                (Stat::VIT, 6),
                (Stat::MGT, 3),
                (Stat::MAG, 8),
                (Stat::ARM, 2),
                (Stat::RES, 6),
                (Stat::SPD, 5),
                (Stat::WIL, 6),
            ],
        );
        let log = BattleState::new(
            &[front.clone(), back.clone(), front.clone()],
            &[front, back.clone(), back],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            42,
        )
        .run();

        let mut defeated_ids: std::collections::HashSet<u32> = std::collections::HashSet::new();
        for event in log.events() {
            match event {
                BattleEvent::BasicAttack { actor_id, .. } => {
                    assert!(
                        !defeated_ids.contains(actor_id),
                        "Defeated character {} acted after death",
                        actor_id
                    );
                }
                BattleEvent::Defeat { character_id, .. } => {
                    defeated_ids.insert(*character_id);
                }
                _ => {}
            }
        }
    }

    fn simple_stats() -> Vec<(Stat, u32)> {
        vec![
            (Stat::VIT, 10),
            (Stat::MGT, 6),
            (Stat::MAG, 4),
            (Stat::ARM, 4),
            (Stat::RES, 3),
            (Stat::SPD, 5),
            (Stat::WIL, 5),
        ]
    }

    #[test]
    fn companions_assigned_by_cardinal_adjacency() {
        let a = make_config_at("A", 0, 0, simple_stats());
        let b = make_config_at("B", 0, 1, simple_stats());
        let c = make_config_at("C", 1, 0, simple_stats());
        let d = make_config_at("D", 0, 2, simple_stats());

        let dummy = make_config_at("Enemy", 0, 0, simple_stats());
        let battle = BattleState::new(
            &[a, b, c, d],
            &[dummy],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            0,
        );

        let comps_a = battle.team_a[0].companions();
        assert!(comps_a.contains(&1), "A should have B as companion");
        assert!(comps_a.contains(&2), "A should have C as companion");
        assert!(!comps_a.contains(&3), "A should NOT have D as companion");
        assert_eq!(comps_a.len(), 2);

        let comps_b = battle.team_a[1].companions();
        assert!(comps_b.contains(&0), "B should have A as companion");
        assert!(comps_b.contains(&3), "B should have D as companion");
        assert!(!comps_b.contains(&2), "B should NOT have C (diagonal)");
        assert_eq!(comps_b.len(), 2);

        let comps_c = battle.team_a[2].companions();
        assert_eq!(comps_c, &[0], "C should only have A as companion");

        let comps_d = battle.team_a[3].companions();
        assert_eq!(comps_d, &[1], "D should only have B as companion");
    }

    #[test]
    fn companions_only_within_same_team() {
        let a = make_config_at("TeamA", 0, 0, simple_stats());
        let b = make_config_at("TeamB", 0, 1, simple_stats());
        let battle = BattleState::new(
            &[a],
            &[b],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            0,
        );

        assert!(battle.team_a[0].companions().is_empty());
        assert!(battle.team_b[0].companions().is_empty());
    }

    #[test]
    fn isolated_character_has_no_companions() {
        let loner = make_config_at("Loner", 0, 0, simple_stats());
        let far = make_config_at("Far", 2, 2, simple_stats());
        let enemy = make_config_at("Enemy", 0, 0, simple_stats());
        let battle = BattleState::new(
            &[loner, far],
            &[enemy],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            0,
        );

        assert!(battle.team_a[0].companions().is_empty());
        assert!(battle.team_a[1].companions().is_empty());
    }

    #[test]
    fn companion_rule_does_not_imply_companion_targeting() {
        let mut actor = make_config_at(
            "Support",
            0,
            0,
            vec![
                (Stat::VIT, 10),
                (Stat::MGT, 4),
                (Stat::MAG, 4),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 10),
                (Stat::WIL, 4),
            ],
        );
        actor.actives = vec!["Withdraw".to_string()];
        actor.rules = vec![Rule {
            ability: "Withdraw".to_string(),
            conditions: vec![Condition {
                subject: ConditionSubject::Companion,
                value: QueryValue::Mp,
                comparator: Comparator::Lte,
                threshold: 1,
            }],
        }];

        let companion = make_config_at(
            "Companion",
            0,
            1,
            vec![
                (Stat::VIT, 10),
                (Stat::MGT, 4),
                (Stat::MAG, 4),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 1),
                (Stat::WIL, 5),
            ],
        );
        let enemy = make_config_at("Enemy", 0, 0, simple_stats());

        let mut abilities = HashMap::new();
        abilities.insert(
            "Withdraw".to_string(),
            AbilityDef {
                mp_cost: 1,
                primitives: vec![Primitive::RestoreMp {
                    target: SimpleAbilityTarget::SelfChar.into(),
                    amount: 3,
                }],
            },
        );

        let mut battle = BattleState::new(
            &[actor, companion],
            &[enemy],
            abilities,
            empty_passives(),
            empty_statuses(),
            42,
        );
        battle.team_a[0].spend_mp(3);
        battle.team_a[1].spend_mp(4);

        battle.step_once();

        assert_eq!(battle.team_a[0].current_mp(), 3);
        assert_eq!(battle.team_a[1].current_mp(), 1);
    }

    // --- Ability integration tests ---

    fn crush_ability() -> AbilityDef {
        AbilityDef {
            mp_cost: 2,
            primitives: vec![Primitive::DealPhysicalDamage {
                target: SimpleAbilityTarget::CurrentTarget.into(),
                multiplier: 1.5,
                double_empower_stat: None,
            }],
        }
    }

    fn embolden_ability() -> AbilityDef {
        AbilityDef {
            mp_cost: 3,
            primitives: vec![Primitive::RestoreMp {
                target: SimpleAbilityTarget::Companions.into(),
                amount: 1,
            }],
        }
    }

    fn test_abilities() -> AbilityMap {
        let mut map = empty_abilities();
        map.insert("Crush".to_string(), crush_ability());
        map.insert("Embolden".to_string(), embolden_ability());
        map
    }

    fn emperor_config() -> CharacterConfig {
        let mut config = make_config_at(
            "The Emperor",
            0,
            0,
            vec![
                (Stat::VIT, 10),
                (Stat::MGT, 6),
                (Stat::MAG, 4),
                (Stat::ARM, 3),
                (Stat::RES, 2),
                (Stat::SPD, 4),
                (Stat::WIL, 5),
            ],
        );
        config.actives = vec!["Crush".to_string(), "Embolden".to_string()];
        config.rules = vec![
            Rule {
                ability: "Crush".to_string(),
                conditions: vec![Condition {
                    subject: ConditionSubject::Target,
                    value: QueryValue::Hp,
                    comparator: Comparator::Lte,
                    threshold: 3,
                }],
            },
            Rule {
                ability: "Embolden".to_string(),
                conditions: vec![Condition {
                    subject: ConditionSubject::Companion,
                    value: QueryValue::Mp,
                    comparator: Comparator::Lte,
                    threshold: 1,
                }],
            },
            Rule {
                ability: "Crush".to_string(),
                conditions: Vec::new(), // always
            },
        ];
        config
    }

    #[test]
    fn emperor_uses_crush_always_rule() {
        // Emperor with Crush always-rule vs a simple enemy
        let emperor = emperor_config();
        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::VIT, 15),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 4),
                (Stat::WIL, 4),
            ],
        );

        let log = BattleState::new(
            &[emperor],
            &[enemy],
            test_abilities(),
            empty_passives(),
            empty_statuses(),
            42,
        )
        .run();
        let events = log.events();

        // Should have at least one AbilityUsed for Crush
        let crush_count = events
            .iter()
            .filter(|e| {
                matches!(e,
                    BattleEvent::AbilityUsed { ability_name, .. } if ability_name == "Crush"
                )
            })
            .count();
        assert!(crush_count > 0, "Emperor should use Crush at least once");
    }

    #[test]
    fn emperor_rests_when_mp_exhausted() {
        // Emperor with only 2 max MP from WIL — can Crush once, then Rest
        let mut emperor = emperor_config();
        emperor.stats.insert(Stat::WIL, 2);

        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::VIT, 30),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 4),
                (Stat::WIL, 4),
            ],
        );

        let log = BattleState::new(
            &[emperor],
            &[enemy],
            test_abilities(),
            empty_passives(),
            empty_statuses(),
            42,
        )
        .run();
        let events = log.events();

        let crush_count = events
            .iter()
            .filter(|e| {
                matches!(e,
                    BattleEvent::AbilityUsed { ability_name, .. } if ability_name == "Crush"
                )
            })
            .count();
        let rest_count = events
            .iter()
            .filter(|e| {
                matches!(e,
                    BattleEvent::Rest { actor_id, .. } if *actor_id == 0
                )
            })
            .count();

        assert!(crush_count >= 1, "Should use Crush at least once");
        assert!(
            rest_count >= 1,
            "Should fall back to Rest when MP runs out"
        );
    }

    #[test]
    fn characters_without_rules_rest() {
        // Two characters with no rules — should Rest instead of attacking.
        let mut a = make_config(
            "A",
            0,
            vec![
                (Stat::VIT, 10),
                (Stat::MGT, 6),
                (Stat::MAG, 4),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 5),
            ],
        );
        a.actives.clear();
        a.rules.clear();
        let mut b = make_config(
            "B",
            0,
            vec![
                (Stat::VIT, 10),
                (Stat::MGT, 6),
                (Stat::MAG, 4),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 5),
            ],
        );
        b.actives.clear();
        b.rules.clear();

        let log = BattleState::new(
            &[a],
            &[b],
            test_abilities(),
            empty_passives(),
            empty_statuses(),
            42,
        )
        .run();
        let has_ability = log
            .events()
            .iter()
            .any(|e| matches!(e, BattleEvent::AbilityUsed { .. }));
        let has_rest = log.events().iter().any(|e| matches!(e, BattleEvent::Rest { .. }));
        let has_basic = log
            .events()
            .iter()
            .any(|e| matches!(e, BattleEvent::BasicAttack { .. }));
        assert!(
            !has_ability,
            "Characters without rules should not use abilities"
        );
        assert!(has_rest, "Characters without rules should Rest");
        assert!(
            !has_basic,
            "Characters without rules should not produce fallback basic attacks"
        );
    }

    // --- Effect ticking tests ---

    #[test]
    fn status_damage_produces_events() {
        use crate::statuses::{StackType, StatusBehavior, StatusDef};

        let mut attacker = make_config(
            "Attacker",
            0,
            vec![
                (Stat::VIT, 20),
                (Stat::MGT, 6),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 5),
            ],
        );
        attacker.actives = vec!["Poison".to_string()];
        attacker.rules = vec![Rule {
            ability: "Poison".to_string(),
            conditions: Vec::new(),
        }];
        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::VIT, 30),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );

        let mut abilities = HashMap::new();
        abilities.insert(
            "Poison".to_string(),
            AbilityDef {
                mp_cost: 1,
                primitives: vec![Primitive::ApplyStatus {
                    target: SimpleAbilityTarget::CurrentTarget.into(),
                    status: "Poison".to_string(),
                    stat: None,
                    stacks: 3,
                }],
            },
        );

        let mut statuses: StatusMap = HashMap::new();
        statuses.insert(
            "Poison".to_string(),
            StatusDef {
                behavior: StatusBehavior::DamagePerStack { value: 2 },
                stack_type: StackType::TickDown,
                group: None,
                opposes: None,
            },
        );

        let log = BattleState::new(
            &[attacker],
            &[enemy],
            abilities,
            empty_passives(),
            statuses,
            42,
        )
        .run();
        let status_dmg_count = log
            .events()
            .iter()
            .filter(|e| matches!(e, BattleEvent::StatusDamage { .. }))
            .count();
        assert!(
            status_dmg_count > 0,
            "Should have StatusDamage events from Poison"
        );
    }

    #[test]
    fn incapacitated_character_skips_turn() {
        use crate::statuses::{StackType, StatusBehavior, StatusDef};

        let a = make_config(
            "A",
            0,
            vec![
                (Stat::VIT, 50),
                (Stat::MGT, 6),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 3),
                (Stat::WIL, 5),
            ],
        );
        let b = make_config(
            "B",
            0,
            vec![
                (Stat::VIT, 50),
                (Stat::MGT, 6),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 3),
                (Stat::WIL, 5),
            ],
        );

        let stun_def = StatusDef {
            behavior: StatusBehavior::SkipTurn,
            stack_type: StackType::NoStack,
            group: None,
            opposes: None,
        };

        let mut battle = BattleState::new(
            &[a],
            &[b],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            42,
        );
        battle.team_a[0].add_status("Stun", 2, 99, &stun_def, None);

        let log = battle.run();
        let skip_count = log
            .events()
            .iter()
            .filter(|e| matches!(e, BattleEvent::TurnSkipped { .. }))
            .count();
        assert!(
            skip_count > 0,
            "Stunned character should have TurnSkipped events"
        );
    }

    #[test]
    fn stunned_turn_still_ticks_statuses_without_restoring_mp() {
        use crate::statuses::{StackType, StatusBehavior, StatusDef};

        let actor = make_config(
            "Actor",
            0,
            vec![
                (Stat::VIT, 20),
                (Stat::MGT, 6),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 10),
                (Stat::WIL, 4),
            ],
        );
        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::VIT, 20),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 1),
                (Stat::WIL, 4),
            ],
        );

        let stun_def = StatusDef {
            behavior: StatusBehavior::SkipTurn,
            stack_type: StackType::NoStack,
            group: None,
            opposes: None,
        };
        let bleed_def = StatusDef {
            behavior: StatusBehavior::DamagePerStack { value: 1 },
            stack_type: StackType::TickDown,
            group: None,
            opposes: None,
        };

        let mut battle = BattleState::new(
            &[actor],
            &[enemy],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            42,
        );
        battle.team_a[0].add_status("Stun", 1, 99, &stun_def, None);
        battle.team_a[0].add_status("Bleed", 1, 99, &bleed_def, None);
        battle.team_a[0].spend_mp(4);

        battle.step_once();

        assert_eq!(battle.team_a[0].current_hp(), 59);
        assert_eq!(battle.team_a[0].current_mp(), 0);
        assert!(!battle.team_a[0].is_incapacitated());

        let has_skip = battle.log.events().iter().any(|e| {
            matches!(
                e,
                BattleEvent::TurnSkipped { character_name, .. } if character_name == "Actor"
            )
        });
        let has_status_damage = battle.log.events().iter().any(|e| {
            matches!(
                e,
                BattleEvent::StatusDamage { character_name, .. } if character_name == "Actor"
            )
        });
        assert!(has_skip);
        assert!(has_status_damage);
    }

    #[test]
    fn on_turn_start_passive_logs_before_status_tick() {
        use crate::abilities::{PassiveDef, PassiveTrigger};
        use crate::statuses::{StackType, StatusBehavior, StatusDef};

        let mut actor = make_config(
            "Actor",
            0,
            vec![
                (Stat::VIT, 20),
                (Stat::MGT, 6),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 10),
                (Stat::WIL, 4),
            ],
        );
        actor.passive = "Meditation".to_string();

        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::VIT, 20),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 1),
                (Stat::WIL, 4),
            ],
        );

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Meditation".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnTurnStart,
                once_per_tick: false,
                primitives: vec![Primitive::RestoreMp {
                    target: SimpleAbilityTarget::SelfChar.into(),
                    amount: 1,
                }],
            },
        );

        let bleed_def = StatusDef {
            behavior: StatusBehavior::DamagePerStack { value: 1 },
            stack_type: StackType::TickDown,
            group: None,
            opposes: None,
        };

        let mut battle = BattleState::new(
            &[actor],
            &[enemy],
            empty_abilities(),
            passives,
            empty_statuses(),
            42,
        );
        battle.team_a[0].add_status("Bleed", 1, 99, &bleed_def, None);

        battle.step_once();

        let passive_idx = battle
            .log
            .events()
            .iter()
            .position(|e| {
                matches!(
                    e,
                    BattleEvent::PassiveTriggered { passive_name, .. } if passive_name == "Meditation"
                )
            })
            .expect("passive should trigger");
        let tick_idx = battle
            .log
            .events()
            .iter()
            .position(|e| {
                matches!(
                    e,
                    BattleEvent::StatusDamage { character_name, .. } if character_name == "Actor"
                )
            })
            .expect("status tick should log");

        assert!(passive_idx < tick_idx);
    }

    #[test]
    fn on_battle_start_passive_fires() {
        use crate::abilities::{PassiveDef, PassiveTrigger};
        use crate::statuses::{StackType, StatusBehavior, StatusDef};

        let mut char_config = make_config(
            "Warrior",
            0,
            vec![
                (Stat::VIT, 10),
                (Stat::MGT, 6),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 5),
            ],
        );
        char_config.passive = "TestPassive".to_string();

        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::VIT, 10),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "TestPassive".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnBattleStart,
                once_per_tick: false,
                primitives: vec![Primitive::ApplyStatus {
                    target: SimpleAbilityTarget::SelfChar.into(),
                    status: "Empower".to_string(),
                    stat: Some(Stat::MGT),
                    stacks: 5,
                }],
            },
        );

        let mut statuses: StatusMap = HashMap::new();
        statuses.insert(
            "Empower".to_string(),
            StatusDef {
                behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
                stack_type: StackType::Permanent,
                group: None,
                opposes: None,
            },
        );

        let log = BattleState::new(
            &[char_config],
            &[enemy],
            empty_abilities(),
            passives,
            statuses,
            42,
        )
        .run();
        let has_passive = log.events().iter().any(|e| {
            matches!(e,
                BattleEvent::PassiveTriggered { passive_name, .. } if passive_name == "TestPassive"
            )
        });
        assert!(
            has_passive,
            "Should have PassiveTriggered event at battle start"
        );
    }

    #[test]
    fn passive_buff_affects_combat() {
        use crate::abilities::{PassiveDef, PassiveTrigger};
        use crate::statuses::{StackType, StatusBehavior, StatusDef};

        let mut warrior = make_config(
            "Warrior",
            0,
            vec![
                (Stat::VIT, 10),
                (Stat::MGT, 6),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 5),
            ],
        );
        warrior.passive = "PowerUp".to_string();

        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::VIT, 10),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "PowerUp".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnBattleStart,
                once_per_tick: false,
                primitives: vec![Primitive::ApplyStatus {
                    target: SimpleAbilityTarget::SelfChar.into(),
                    status: "Empower".to_string(),
                    stat: Some(Stat::MGT),
                    stacks: 100,
                }],
            },
        );

        let mut statuses: StatusMap = HashMap::new();
        statuses.insert(
            "Empower".to_string(),
            StatusDef {
                behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
                stack_type: StackType::Permanent,
                group: None,
                opposes: None,
            },
        );

        // With huge MGT buff, warrior should win easily
        let log = BattleState::new(
            &[warrior],
            &[enemy],
            empty_abilities(),
            passives,
            statuses,
            42,
        )
        .run();
        match log.events().last().unwrap() {
            BattleEvent::BattleEnd { winner, .. } => assert_eq!(winner, "team_a"),
            _ => panic!("Expected BattleEnd"),
        }
    }

    #[test]
    fn unknown_passive_does_not_crash() {
        let mut char_config = make_config(
            "Warrior",
            0,
            vec![
                (Stat::VIT, 10),
                (Stat::MGT, 6),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 5),
            ],
        );
        char_config.passive = "NonexistentPassive".to_string();

        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::VIT, 10),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );

        // Should complete without panicking
        let log = BattleState::new(
            &[char_config],
            &[enemy],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            42,
        )
        .run();
        assert!(matches!(
            log.events().last().unwrap(),
            BattleEvent::BattleEnd { .. }
        ));
    }

    // --- Permanent trait tests ---

    #[test]
    fn trait_passive_applied_at_battle_start() {
        use crate::abilities::PassiveDef;
        use crate::models::TraitEffect;

        let mut char_config = make_config(
            "Warrior",
            0,
            vec![
                (Stat::VIT, 10),
                (Stat::MGT, 6),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 5),
            ],
        );
        char_config.passive = "Thorns".to_string();

        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::VIT, 10),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Thorns".to_string(),
            PassiveDef::Trait {
                effect: TraitEffect::DamageReflect { amount: 2 },
            },
        );

        let log = BattleState::new(
            &[char_config],
            &[enemy],
            empty_abilities(),
            passives,
            empty_statuses(),
            42,
        )
        .run();
        let has_passive = log.events().iter().any(|e| {
            matches!(e,
                BattleEvent::PassiveTriggered { passive_name, .. } if passive_name == "Thorns"
            )
        });
        assert!(has_passive, "Trait passive should log PassiveTriggered");
    }

    #[test]
    fn damage_reflect_hurts_attacker() {
        use crate::abilities::PassiveDef;
        use crate::models::TraitEffect;

        // Defender has DamageReflect, attacker should take reflect damage
        let attacker = make_config(
            "Attacker",
            0,
            vec![
                (Stat::VIT, 20),
                (Stat::MGT, 6),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );

        let mut defender = make_config(
            "Defender",
            0,
            vec![
                (Stat::VIT, 20),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );
        defender.passive = "Thorns".to_string();

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Thorns".to_string(),
            PassiveDef::Trait {
                effect: TraitEffect::DamageReflect { amount: 2 },
            },
        );

        let log = BattleState::new(
            &[attacker],
            &[defender],
            empty_abilities(),
            passives,
            empty_statuses(),
            42,
        )
        .run();
        let has_reflect = log
            .events()
            .iter()
            .any(|e| matches!(e, BattleEvent::DamageReflect { .. }));
        assert!(has_reflect, "Should have DamageReflect events");
    }

    #[test]
    fn damage_reflect_can_kill_attacker() {
        use crate::abilities::PassiveDef;
        use crate::models::TraitEffect;

        // Attacker has very low HP, defender has high reflect
        let attacker = make_config(
            "Attacker",
            0,
            vec![
                (Stat::VIT, 2),
                (Stat::MGT, 6),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );

        let mut defender = make_config(
            "Defender",
            0,
            vec![
                (Stat::VIT, 50),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );
        defender.passive = "Thorns".to_string();

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Thorns".to_string(),
            PassiveDef::Trait {
                effect: TraitEffect::DamageReflect { amount: 50 },
            },
        );

        let log = BattleState::new(
            &[attacker],
            &[defender],
            empty_abilities(),
            passives,
            empty_statuses(),
            42,
        )
        .run();

        // Attacker should die from reflect
        let attacker_defeated = log.events().iter().any(|e| {
            matches!(e,
                BattleEvent::Defeat { character_name, .. } if character_name == "Attacker"
            )
        });
        assert!(attacker_defeated, "Attacker should die from reflect damage");

        // Defender should win
        match log.events().last().unwrap() {
            BattleEvent::BattleEnd { winner, .. } => assert_eq!(winner, "team_b"),
            _ => panic!("Expected BattleEnd"),
        }
    }

    #[test]
    fn on_death_passive_fires_when_killed_by_reflect() {
        use crate::abilities::{PassiveDef, PassiveTrigger};
        use crate::models::TraitEffect;

        let mut attacker = make_config(
            "Attacker",
            0,
            vec![
                (Stat::VIT, 2),
                (Stat::MGT, 6),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );
        attacker.passive = "Collapse".to_string();

        let mut defender = make_config(
            "Defender",
            0,
            vec![
                (Stat::VIT, 50),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );
        defender.passive = "Thorns".to_string();

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Collapse".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnDeath,
                once_per_tick: false,
                primitives: vec![Primitive::DealPhysicalDamage {
                    target: SimpleAbilityTarget::AllEnemies.into(),
                    multiplier: 1.0,
                    double_empower_stat: None,
                }],
            },
        );
        passives.insert(
            "Thorns".to_string(),
            PassiveDef::Trait {
                effect: TraitEffect::DamageReflect { amount: 50 },
            },
        );

        let log = BattleState::new(
            &[attacker],
            &[defender],
            empty_abilities(),
            passives,
            empty_statuses(),
            42,
        )
        .run();
        let attacker_defeat_idx = log
            .events()
            .iter()
            .position(|e| {
                matches!(e,
                    BattleEvent::Defeat { character_name, .. } if character_name == "Attacker"
                )
            })
            .expect("attacker should be defeated");
        let collapse_idx = log
            .events()
            .iter()
            .position(|e| {
                matches!(e,
                    BattleEvent::PassiveTriggered { passive_name, .. } if passive_name == "Collapse"
                )
            })
            .expect("on_death passive should trigger on reflect death");

        assert!(
            collapse_idx < attacker_defeat_idx,
            "on_death should resolve before Defeat is logged"
        );
    }

    #[test]
    fn ward_negates_reflect_damage_and_is_consumed() {
        use crate::abilities::{PassiveDef, PassiveTrigger};
        use crate::models::TraitEffect;

        let mut attacker = make_config(
            "Attacker",
            0,
            vec![
                (Stat::VIT, 10),
                (Stat::MGT, 6),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );
        attacker.passive = "Barrier".to_string();

        let mut defender = make_config(
            "Defender",
            0,
            vec![
                (Stat::VIT, 20),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );
        defender.passive = "Thorns".to_string();

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Barrier".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnBattleStart,
                once_per_tick: false,
                primitives: vec![Primitive::ApplyStatus {
                    target: SimpleAbilityTarget::SelfChar.into(),
                    status: "Ward".to_string(),
                    stat: None,
                    stacks: 1,
                }],
            },
        );
        passives.insert(
            "Thorns".to_string(),
            PassiveDef::Trait {
                effect: TraitEffect::DamageReflect { amount: 2 },
            },
        );

        let log = BattleState::new(
            &[attacker],
            &[defender],
            empty_abilities(),
            passives,
            ward_statuses(),
            42,
        )
        .run();

        let reflect_event = log
            .events()
            .iter()
            .find_map(|event| match event {
                BattleEvent::DamageReflect {
                    target_name,
                    damage,
                    target_hp_remaining,
                    ..
                } if target_name == "Attacker" => Some((*damage, *target_hp_remaining)),
                _ => None,
            })
            .expect("attacker should receive reflect event");

        assert_eq!(reflect_event, (0, 30));
    }

    #[test]
    fn spi_cost_reduction_in_battle() {
        use crate::abilities::PassiveDef;
        use crate::models::TraitEffect;

        // Emperor with MP cost reduction — should be able to use Crush more
        let mut emperor = emperor_config();
        emperor.passive = "Thrift".to_string();
        emperor.stats.insert(Stat::WIL, 3); // barely enough MP for Crush(2) without reduction

        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::VIT, 30),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 4),
                (Stat::WIL, 4),
            ],
        );

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Thrift".to_string(),
            PassiveDef::Trait {
                effect: TraitEffect::MpCostReduction { amount: 1 },
            },
        );

        let log = BattleState::new(
            &[emperor],
            &[enemy],
            test_abilities(),
            passives,
            empty_statuses(),
            42,
        )
        .run();

        // With reduction, Crush costs 1 instead of 2, so should use it more times
        let crush_count = log
            .events()
            .iter()
            .filter(|e| {
                matches!(e,
                    BattleEvent::AbilityUsed { ability_name, .. } if ability_name == "Crush"
                )
            })
            .count();
        assert!(
            crush_count >= 2,
            "Should use Crush at least twice with cost reduction, got {}",
            crush_count
        );
    }

    // --- Passive trigger tests ---

    #[test]
    fn on_turn_start_passive_fires_each_turn() {
        use crate::abilities::{PassiveDef, PassiveTrigger};

        let mut char_config = make_config(
            "Meditator",
            0,
            vec![
                (Stat::VIT, 20),
                (Stat::MGT, 6),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 5),
            ],
        );
        char_config.passive = "Meditation".to_string();

        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::VIT, 20),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Meditation".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnTurnStart,
                once_per_tick: false,
                primitives: vec![Primitive::RestoreMp {
                    target: SimpleAbilityTarget::SelfChar.into(),
                    amount: 1,
                }],
            },
        );

        let log = BattleState::new(
            &[char_config],
            &[enemy],
            empty_abilities(),
            passives,
            empty_statuses(),
            42,
        )
        .run();
        let passive_count = log.events().iter().filter(|e| matches!(e,
            BattleEvent::PassiveTriggered { passive_name, .. } if passive_name == "Meditation"
        )).count();
        assert!(
            passive_count >= 2,
            "on_turn_start should fire multiple times, got {}",
            passive_count
        );
    }

    #[test]
    fn on_deal_damage_passive_fires() {
        use crate::abilities::{PassiveDef, PassiveTrigger};
        use crate::statuses::{StackType, StatusBehavior, StatusDef};

        let mut attacker = make_config(
            "Attacker",
            0,
            vec![
                (Stat::VIT, 20),
                (Stat::MGT, 8),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );
        attacker.passive = "Momentum".to_string();

        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::VIT, 30),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Momentum".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnDealDamage,
                once_per_tick: false,
                primitives: vec![Primitive::ApplyStatus {
                    target: SimpleAbilityTarget::SelfChar.into(),
                    status: "Empower".to_string(),
                    stat: Some(Stat::MGT),
                    stacks: 1,
                }],
            },
        );

        let mut statuses: StatusMap = HashMap::new();
        statuses.insert(
            "Empower".to_string(),
            StatusDef {
                behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
                stack_type: StackType::Permanent,
                group: None,
                opposes: None,
            },
        );

        let log = BattleState::new(
            &[attacker],
            &[enemy],
            empty_abilities(),
            passives,
            statuses,
            42,
        )
        .run();
        let passive_count = log
            .events()
            .iter()
            .filter(|e| {
                matches!(e,
                    BattleEvent::PassiveTriggered { passive_name, .. } if passive_name == "Momentum"
                )
            })
            .count();
        assert!(
            passive_count >= 1,
            "on_deal_damage should fire when dealing damage"
        );
    }

    #[test]
    fn on_deal_damage_passes_target_context_to_trigger_target() {
        use crate::abilities::{PassiveDef, PassiveTrigger};
        use crate::statuses::{StackType, StatusBehavior, StatusDef};

        let mut attacker = make_config(
            "Moon",
            0,
            vec![
                (Stat::VIT, 20),
                (Stat::MGT, 3),
                (Stat::MAG, 8),
                (Stat::ARM, 3),
                (Stat::RES, 4),
                (Stat::SPD, 5),
                (Stat::WIL, 5),
            ],
        );
        attacker.passive = "Foreboding".to_string();
        attacker.actives = vec!["Eclipse".to_string()];
        attacker.rules = vec![Rule {
            ability: "Eclipse".to_string(),
            conditions: vec![],
        }];

        let mut enemy_a = make_config(
            "Enemy A",
            0,
            vec![
                (Stat::VIT, 20),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 2),
                (Stat::RES, 2),
                (Stat::SPD, 4),
                (Stat::WIL, 4),
            ],
        );
        enemy_a.position = Position { row: 0, col: 0 };
        let mut enemy_b = make_config(
            "Enemy B",
            0,
            vec![
                (Stat::VIT, 20),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 2),
                (Stat::RES, 2),
                (Stat::SPD, 4),
                (Stat::WIL, 4),
            ],
        );
        enemy_b.position = Position { row: 0, col: 1 };

        let mut abilities = AbilityMap::new();
        abilities.insert(
            "Eclipse".to_string(),
            AbilityDef {
                mp_cost: 1,
                primitives: vec![Primitive::DealMagicalDamage {
                    target: SimpleAbilityTarget::CurrentTargetAndCompanions.into(),
                    multiplier: 1.0,
                }],
            },
        );

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Foreboding".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnDealDamage,
                once_per_tick: false,
                primitives: vec![Primitive::ApplyStatus {
                    target: SimpleAbilityTarget::TriggerTarget.into(),
                    status: "Omen".to_string(),
                    stat: None,
                    stacks: 1,
                }],
            },
        );

        let mut statuses: StatusMap = HashMap::new();
        statuses.insert(
            "Omen".to_string(),
            StatusDef {
                behavior: StatusBehavior::DamagePerStack { value: 1 },
                stack_type: StackType::TickDown,
                group: Some(crate::statuses::StatusGroup::Fate),
                opposes: None,
            },
        );

        let mut battle = BattleState::new(&[attacker], &[enemy_a, enemy_b], abilities, passives, statuses, 42);
        battle.team_a[0].restore_mp(5);
        battle.step = 1;
        battle.execute_turn(0, true);

        assert_eq!(battle.team_b[0].status_stacks("Omen"), 1);
        assert_eq!(battle.team_b[1].status_stacks("Omen"), 1);
    }

    #[test]
    fn on_take_damage_passive_fires() {
        use crate::abilities::{PassiveDef, PassiveTrigger};
        use crate::statuses::{StackType, StatusBehavior, StatusDef};

        let attacker = make_config(
            "Attacker",
            0,
            vec![
                (Stat::VIT, 20),
                (Stat::MGT, 8),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );

        let mut defender = make_config(
            "Defender",
            0,
            vec![
                (Stat::VIT, 30),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 3),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );
        defender.passive = "Vengeance".to_string();

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Vengeance".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnTakeDamage,
                once_per_tick: false,
                primitives: vec![Primitive::ApplyStatus {
                    target: SimpleAbilityTarget::SelfChar.into(),
                    status: "Empower".to_string(),
                    stat: Some(Stat::MGT),
                    stacks: 1,
                }],
            },
        );

        let mut statuses: StatusMap = HashMap::new();
        statuses.insert(
            "Empower".to_string(),
            StatusDef {
                behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
                stack_type: StackType::Permanent,
                group: None,
                opposes: None,
            },
        );

        let log = BattleState::new(
            &[attacker],
            &[defender],
            empty_abilities(),
            passives,
            statuses,
            42,
        )
        .run();
        let passive_count = log.events().iter().filter(|e| matches!(e,
            BattleEvent::PassiveTriggered { passive_name, .. } if passive_name == "Vengeance"
        )).count();
        assert!(
            passive_count >= 1,
            "on_take_damage should fire when taking damage"
        );
    }

    #[test]
    fn on_kill_passive_fires() {
        use crate::abilities::{PassiveDef, PassiveTrigger};

        let mut killer = make_config(
            "Killer",
            0,
            vec![
                (Stat::VIT, 20),
                (Stat::MGT, 15),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );
        killer.passive = "Reaper".to_string();

        let weak_enemy = make_config(
            "Weak",
            0,
            vec![
                (Stat::VIT, 3),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 2),
                (Stat::RES, 2),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Reaper".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnKill,
                once_per_tick: false,
                primitives: vec![Primitive::RestoreHp {
                    target: SimpleAbilityTarget::SelfChar.into(),
                    amount: 5,
                }],
            },
        );

        let log = BattleState::new(
            &[killer],
            &[weak_enemy],
            empty_abilities(),
            passives,
            empty_statuses(),
            42,
        )
        .run();
        let passive_count = log
            .events()
            .iter()
            .filter(|e| {
                matches!(e,
                    BattleEvent::PassiveTriggered { passive_name, .. } if passive_name == "Reaper"
                )
            })
            .count();
        assert!(
            passive_count >= 1,
            "on_kill should fire when killing an enemy"
        );
    }

    #[test]
    fn on_death_passive_fires() {
        use crate::abilities::{PassiveDef, PassiveTrigger};

        let strong_enemy = make_config(
            "Strong",
            0,
            vec![
                (Stat::VIT, 30),
                (Stat::MGT, 15),
                (Stat::MAG, 3),
                (Stat::ARM, 10),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );

        let mut dying = make_config(
            "Dying",
            0,
            vec![
                (Stat::VIT, 3),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 2),
                (Stat::RES, 2),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );
        dying.passive = "Collapse".to_string();

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Collapse".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnDeath,
                once_per_tick: false,
                primitives: vec![Primitive::DealPhysicalDamage {
                    target: SimpleAbilityTarget::AllEnemies.into(),
                    multiplier: 1.0,
                    double_empower_stat: None,
                }],
            },
        );

        let log = BattleState::new(
            &[strong_enemy],
            &[dying],
            empty_abilities(),
            passives,
            empty_statuses(),
            42,
        )
        .run();
        let passive_count = log
            .events()
            .iter()
            .filter(|e| {
                matches!(e,
                    BattleEvent::PassiveTriggered { passive_name, .. } if passive_name == "Collapse"
                )
            })
            .count();
        assert!(
            passive_count >= 1,
            "on_death should fire when character dies"
        );
    }

    #[test]
    fn on_death_passive_fires_when_killed_by_status() {
        use crate::abilities::{PassiveDef, PassiveTrigger};
        use crate::statuses::{StackType, StatusBehavior, StatusDef};

        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::VIT, 30),
                (Stat::MGT, 15),
                (Stat::MAG, 3),
                (Stat::ARM, 10),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );

        let mut dying = make_config(
            "Dying",
            0,
            vec![
                (Stat::VIT, 2),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 2),
                (Stat::RES, 2),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );
        dying.passive = "Collapse".to_string();

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Collapse".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnDeath,
                once_per_tick: false,
                primitives: vec![Primitive::DealPhysicalDamage {
                    target: SimpleAbilityTarget::AllEnemies.into(),
                    multiplier: 1.0,
                    double_empower_stat: None,
                }],
            },
        );

        let poison = StatusDef {
            behavior: StatusBehavior::DamagePerStack { value: 2 },
            stack_type: StackType::TickDown,
            group: None,
            opposes: None,
        };
        let mut statuses: StatusMap = HashMap::new();
        statuses.insert("Poison".to_string(), poison.clone());

        let mut battle = BattleState::new(
            &[enemy],
            &[dying],
            empty_abilities(),
            passives,
            statuses,
            42,
        );
        battle.team_b[0].add_status("Poison", 1, 99, &poison, None);

        let log = battle.run();
        let dying_defeat_idx = log
            .events()
            .iter()
            .position(|e| {
                matches!(e,
                    BattleEvent::Defeat { character_name, .. } if character_name == "Dying"
                )
            })
            .expect("dying unit should be defeated by poison");
        let collapse_idx = log
            .events()
            .iter()
            .position(|e| {
                matches!(e,
                    BattleEvent::PassiveTriggered { passive_name, .. } if passive_name == "Collapse"
                )
            })
            .expect("on_death passive should trigger on status death");

        assert!(
            collapse_idx < dying_defeat_idx,
            "on_death should resolve before Defeat is logged"
        );
    }

    #[test]
    fn resolve_character_death_only_logs_defeat_once() {
        use crate::abilities::{PassiveDef, PassiveTrigger};

        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::VIT, 20),
                (Stat::MGT, 6),
                (Stat::MAG, 3),
                (Stat::ARM, 4),
                (Stat::RES, 3),
                (Stat::SPD, 4),
                (Stat::WIL, 4),
            ],
        );

        let mut dying = make_config(
            "Dying",
            0,
            vec![
                (Stat::VIT, 2),
                (Stat::MGT, 4),
                (Stat::MAG, 3),
                (Stat::ARM, 2),
                (Stat::RES, 2),
                (Stat::SPD, 4),
                (Stat::WIL, 4),
            ],
        );
        dying.passive = "Collapse".to_string();

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Collapse".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnDeath,
                once_per_tick: false,
                primitives: vec![Primitive::DealPhysicalDamage {
                    target: SimpleAbilityTarget::AllEnemies.into(),
                    multiplier: 1.0,
                    double_empower_stat: None,
                }],
            },
        );

        let mut battle = BattleState::new(
            &[enemy],
            &[dying],
            empty_abilities(),
            passives,
            empty_statuses(),
            0,
        );
        battle.team_b[0].take_damage(999);

        battle.resolve_character_death(0, false);
        battle.resolve_character_death(0, false);

        let defeat_count = battle
            .log
            .events()
            .iter()
            .filter(|e| {
                matches!(e,
                    BattleEvent::Defeat { character_name, .. } if character_name == "Dying"
                )
            })
            .count();
        let passive_count = battle
            .log
            .events()
            .iter()
            .filter(|e| {
                matches!(e,
                    BattleEvent::PassiveTriggered { passive_name, .. } if passive_name == "Collapse"
                )
            })
            .count();

        assert_eq!(defeat_count, 1, "Defeat should only be logged once");
        assert_eq!(passive_count, 1, "on_death should only resolve once");
    }

    #[test]
    fn replay_snapshots_track_each_logged_event() {
        let mut striker = warrior();
        striker.rules = vec![Rule {
            ability: "Strike".to_string(),
            conditions: vec![],
        }];
        let enemy = mage();

        let abilities = [(
            "Strike".to_string(),
            AbilityDef {
                mp_cost: 2,
                primitives: vec![Primitive::DealPhysicalDamage {
                    target: SimpleAbilityTarget::CurrentTarget.into(),
                    multiplier: 1.0,
                    double_empower_stat: None,
                }],
            },
        )]
        .into_iter()
        .collect();

        let battle = build_battle(
            &[striker],
            &[enemy],
            abilities,
            empty_passives(),
            empty_statuses(),
        );
        let log = battle.run();

        assert_eq!(log.snapshots().len(), log.events().len() + 1);
        assert_eq!(log.snapshots()[0].event_index, -1);
        assert_eq!(log.snapshots()[0].tick, 0);

        let last_snapshot = log.snapshots().last().expect("snapshot log should not be empty");
        assert_eq!(last_snapshot.event_index as usize, log.events().len() - 1);
        assert_eq!(
            last_snapshot.tick,
            log.events()
                .last()
                .expect("event log should not be empty")
                .tick_count()
        );

        let damage_was_captured = log
            .snapshots()
            .iter()
            .flat_map(|snapshot| snapshot.team_a.iter().chain(snapshot.team_b.iter()))
            .any(|character| character.current_hp < character.max_hp);
        assert!(damage_was_captured, "snapshots should capture changing HP state");
    }

    #[test]
    fn passive_reentry_guard_prevents_cascading() {
        use crate::abilities::{PassiveDef, PassiveTrigger};

        // Both sides have on_deal_damage passives that deal damage —
        // should not cascade infinitely
        let mut a = make_config(
            "A",
            0,
            vec![
                (Stat::VIT, 20),
                (Stat::MGT, 8),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );
        a.passive = "Splash".to_string();

        let mut b = make_config(
            "B",
            0,
            vec![
                (Stat::VIT, 20),
                (Stat::MGT, 8),
                (Stat::MAG, 3),
                (Stat::ARM, 5),
                (Stat::RES, 3),
                (Stat::SPD, 5),
                (Stat::WIL, 4),
            ],
        );
        b.passive = "Splash".to_string();

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Splash".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnDealDamage,
                once_per_tick: false,
                primitives: vec![Primitive::DealPhysicalDamage {
                    target: SimpleAbilityTarget::AllEnemies.into(),
                    multiplier: 0.5,
                    double_empower_stat: None,
                }],
            },
        );

        // Should complete without infinite loop
        let log = BattleState::new(
            &[a],
            &[b],
            empty_abilities(),
            passives,
            empty_statuses(),
            42,
        )
        .run();
        assert!(matches!(
            log.events().last().unwrap(),
            BattleEvent::BattleEnd { .. }
        ));
    }
}
