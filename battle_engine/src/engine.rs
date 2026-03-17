//! Core battle simulation engine.

use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::abilities::{execute_ability, execute_primitives, AbilityMap, PassiveDef, PassiveMap, PassiveTrigger};
use crate::damage::calc_basic_attack_damage;
use crate::logger::{BattleEvent, BattleLog};
use crate::models::{CharacterConfig, CharacterState, StatusTick, Stat};
use crate::rules::evaluate_rules;
use crate::statuses::StatusMap;
use crate::targeting::select_target;

const MAX_STEPS: u32 = 1000;
const SPI_REGEN_INTERVAL: u32 = 10;

pub struct BattleState {
    team_a: Vec<CharacterState>,
    team_b: Vec<CharacterState>,
    abilities: AbilityMap,
    passives: PassiveMap,
    status_defs: StatusMap,
    step: u32,
    log: BattleLog,
    rng: StdRng,
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
            .map(|(i, c)| CharacterState::from_config(i as u32, c))
            .collect();

        let n = team_a.len() as u32;
        let team_b: Vec<CharacterState> = team_b_configs
            .iter()
            .enumerate()
            .map(|(i, c)| CharacterState::from_config(n + i as u32, c))
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
        self.log.push(BattleEvent::BattleStart {
            step: 0,
            team_a: self.team_a.iter().map(|c| c.base_name().to_string()).collect(),
            team_b: self.team_b.iter().map(|c| c.base_name().to_string()).collect(),
        });

        self.execute_battle_start_passives();

        loop {
            if self.step_once() {
                break;
            }
        }

        self.log
    }

    /// Fire on_battle_start passives and apply permanent traits for all characters.
    fn execute_battle_start_passives(&mut self) {
        // Collect passive info first to avoid borrow issues
        let team_a_passives: Vec<(usize, String)> = self.team_a.iter().enumerate()
            .filter(|(_, c)| !c.passive().is_empty())
            .map(|(i, c)| (i, c.passive().to_string()))
            .collect();
        let team_b_passives: Vec<(usize, String)> = self.team_b.iter().enumerate()
            .filter(|(_, c)| !c.passive().is_empty())
            .map(|(i, c)| (i, c.passive().to_string()))
            .collect();

        for (idx, passive_name) in team_a_passives {
            if let Some(passive_def) = self.passives.get(&passive_name).cloned() {
                Self::apply_passive(
                    idx, &passive_name, &passive_def,
                    &mut self.team_a, &mut self.team_b,
                    &mut self.rng, &mut self.log, &self.status_defs,
                );
            }
        }

        for (idx, passive_name) in team_b_passives {
            if let Some(passive_def) = self.passives.get(&passive_name).cloned() {
                Self::apply_passive(
                    idx, &passive_name, &passive_def,
                    &mut self.team_b, &mut self.team_a,
                    &mut self.rng, &mut self.log, &self.status_defs,
                );
            }
        }
    }

    fn apply_passive(
        idx: usize,
        passive_name: &str,
        passive_def: &PassiveDef,
        actor_team: &mut [CharacterState],
        enemy_team: &mut [CharacterState],
        rng: &mut StdRng,
        log: &mut BattleLog,
        status_defs: &StatusMap,
    ) {
        let char_id = actor_team[idx].id();
        let char_name = actor_team[idx].base_name().to_string();

        match passive_def {
            PassiveDef::Triggered { trigger, primitives } => {
                if matches!(trigger, PassiveTrigger::OnBattleStart) {
                    log.push(BattleEvent::PassiveTriggered {
                        step: 0,
                        character_id: char_id,
                        character_name: char_name,
                        passive_name: passive_name.to_string(),
                    });
                    execute_primitives(
                        idx, passive_name, primitives,
                        actor_team, enemy_team,
                        rng, log, 0, status_defs,
                    );
                }
            }
            PassiveDef::Trait { effect } => {
                log.push(BattleEvent::PassiveTriggered {
                    step: 0,
                    character_id: char_id,
                    character_name: char_name,
                    passive_name: passive_name.to_string(),
                });
                actor_team[idx].add_trait(effect.clone());
            }
        }
    }

    /// Advance one step. Returns true if the battle is over.
    fn step_once(&mut self) -> bool {
        self.step += 1;

        if self.step > MAX_STEPS {
            self.log.push(BattleEvent::BattleEnd {
                step: self.step,
                winner: "draw".to_string(),
            });
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

        // SPI regen every N steps
        if self.step % SPI_REGEN_INTERVAL == 0 {
            for c in self.team_a.iter_mut().chain(self.team_b.iter_mut()) {
                if c.is_alive() {
                    let regen = c.get_base_stat(&Stat::SPI) / 2;
                    c.restore_spi(regen);
                }
            }
        }

        // Check win conditions
        let a_alive = self.team_a.iter().any(|c| c.is_alive());
        let b_alive = self.team_b.iter().any(|c| c.is_alive());

        if !a_alive && !b_alive {
            self.log.push(BattleEvent::BattleEnd {
                step: self.step,
                winner: "draw".to_string(),
            });
            true
        } else if !b_alive {
            self.log.push(BattleEvent::BattleEnd {
                step: self.step,
                winner: "team_a".to_string(),
            });
            true
        } else if !a_alive {
            self.log.push(BattleEvent::BattleEnd {
                step: self.step,
                winner: "team_b".to_string(),
            });
            true
        } else {
            false
        }
    }

    /// Execute a turn for a character. `is_team_a` determines which team the actor belongs to.
    fn execute_turn(&mut self, actor_idx: usize, is_team_a: bool) {
        let (actor_team, enemy_team) = if is_team_a {
            (&mut self.team_a as &mut [CharacterState], &mut self.team_b as &mut [CharacterState])
        } else {
            (&mut self.team_b as &mut [CharacterState], &mut self.team_a as &mut [CharacterState])
        };

        actor_team[actor_idx].reset_speed();
        actor_team[actor_idx].increment_turn_count();

        let actor_id = actor_team[actor_idx].id();
        let actor_name = actor_team[actor_idx].base_name().to_string();

        // Incapacitate check: skip action but still tick effects
        if actor_team[actor_idx].is_incapacitated() {
            self.log.push(BattleEvent::TurnSkipped {
                step: self.step,
                character_id: actor_id,
                character_name: actor_name,
                reason: "incapacitated".to_string(),
            });
            Self::tick_and_log_statuses(actor_team, actor_idx, &mut self.log, self.step);
            return;
        }

        // Get or reassign target
        let target_id = match Self::resolve_target(actor_idx, actor_team, enemy_team, &mut self.rng)
        {
            Some(tid) => tid,
            None => return, // no living enemies
        };

        let target_idx = enemy_team.iter().position(|c| c.id() == target_id).unwrap();

        // Evaluate rules to see if an ability should be used
        let target_ref = &enemy_team[target_idx];
        let ability_name = evaluate_rules(
            &actor_team[actor_idx],
            Some(target_ref),
            actor_team,
            &self.abilities,
        );

        if let Some(ref name) = ability_name {
            if let Some(ability_def) = self.abilities.get(name).cloned() {
                // Spend SPI (reduced by trait) and record usage
                let effective_cost = ability_def.spi_cost
                    .saturating_sub(actor_team[actor_idx].spi_cost_reduction())
                    .max(1);
                actor_team[actor_idx].spend_spi(effective_cost);
                actor_team[actor_idx].record_ability_use(name);

                // Execute ability
                let damage_dealt = execute_ability(
                    actor_idx,
                    name,
                    &ability_def,
                    actor_team,
                    enemy_team,
                    &mut self.rng,
                    &mut self.log,
                    self.step,
                    &self.status_defs,
                );

                // Check for defeats and damage reflect from ability damage
                for (tid, _) in &damage_dealt {
                    if let Some(eidx) = enemy_team.iter().position(|c| c.id() == *tid) {
                        if !enemy_team[eidx].is_alive() {
                            let ename = enemy_team[eidx].base_name().to_string();
                            self.log.push(BattleEvent::Defeat {
                                step: self.step,
                                character_id: *tid,
                                character_name: ename,
                            });
                        }
                        // Damage reflect
                        let reflect = enemy_team[eidx].damage_reflect_amount();
                        if reflect > 0 && actor_team[actor_idx].is_alive() {
                            actor_team[actor_idx].take_damage(reflect);
                            let reflector_name = enemy_team[eidx].base_name().to_string();
                            self.log.push(BattleEvent::DamageReflect {
                                step: self.step,
                                reflector_id: *tid,
                                reflector_name,
                                target_id: actor_id,
                                target_name: actor_team[actor_idx].base_name().to_string(),
                                damage: reflect,
                                target_hp_remaining: actor_team[actor_idx].current_hp(),
                            });
                            if !actor_team[actor_idx].is_alive() {
                                self.log.push(BattleEvent::Defeat {
                                    step: self.step,
                                    character_id: actor_id,
                                    character_name: actor_team[actor_idx].base_name().to_string(),
                                });
                            }
                        }
                    }
                }

                // Tick effects after action
                Self::tick_and_log_statuses(actor_team, actor_idx, &mut self.log, self.step);

                // Reassign target if current target is dead
                let current_target = actor_team[actor_idx].target();
                if let Some(ct) = current_target {
                    if enemy_team.iter().find(|c| c.id() == ct).map_or(true, |c| !c.is_alive()) {
                        let new_target = select_target(&actor_team[actor_idx], enemy_team, &mut self.rng);
                        if let Some(tid) = new_target {
                            actor_team[actor_idx].set_target(tid);
                        } else {
                            actor_team[actor_idx].clear_target();
                        }
                    }
                }

                return;
            }
        }

        // Fallback: basic attack
        let damage = calc_basic_attack_damage(
            &actor_team[actor_idx],
            &enemy_team[target_idx],
            &mut self.rng,
        );

        enemy_team[target_idx].take_damage(damage);
        let target_name = enemy_team[target_idx].base_name().to_string();
        let hp_remaining = enemy_team[target_idx].current_hp();

        self.log.push(BattleEvent::BasicAttack {
            step: self.step,
            actor_id,
            actor_name,
            target_id,
            target_name: target_name.clone(),
            damage,
            target_hp_remaining: hp_remaining,
        });

        if !enemy_team[target_idx].is_alive() {
            self.log.push(BattleEvent::Defeat {
                step: self.step,
                character_id: target_id,
                character_name: target_name,
            });
            let new_target = select_target(&actor_team[actor_idx], enemy_team, &mut self.rng);
            if let Some(tid) = new_target {
                actor_team[actor_idx].set_target(tid);
            } else {
                actor_team[actor_idx].clear_target();
            }
        }

        // Damage reflect from basic attack
        let reflect = enemy_team[target_idx].damage_reflect_amount();
        if reflect > 0 && actor_team[actor_idx].is_alive() {
            actor_team[actor_idx].take_damage(reflect);
            let reflector_name = enemy_team[target_idx].base_name().to_string();
            self.log.push(BattleEvent::DamageReflect {
                step: self.step,
                reflector_id: target_id,
                reflector_name,
                target_id: actor_id,
                target_name: actor_team[actor_idx].base_name().to_string(),
                damage: reflect,
                target_hp_remaining: actor_team[actor_idx].current_hp(),
            });
            if !actor_team[actor_idx].is_alive() {
                self.log.push(BattleEvent::Defeat {
                    step: self.step,
                    character_id: actor_id,
                    character_name: actor_team[actor_idx].base_name().to_string(),
                });
            }
        }

        // Tick effects after basic attack
        Self::tick_and_log_statuses(actor_team, actor_idx, &mut self.log, self.step);
    }

    /// Tick statuses on a character and log any DoT/HoT results.
    fn tick_and_log_statuses(
        team: &mut [CharacterState],
        idx: usize,
        log: &mut BattleLog,
        step: u32,
    ) {
        let ticks = team[idx].tick_statuses();
        let char_id = team[idx].id();
        let char_name = team[idx].base_name().to_string();
        for tick in ticks {
            match tick {
                StatusTick::DamageDealt { name, damage } => {
                    log.push(BattleEvent::StatusDamage {
                        step,
                        character_id: char_id,
                        character_name: char_name.clone(),
                        status_name: name,
                        damage,
                        hp_remaining: team[idx].current_hp(),
                    });
                }
                StatusTick::HealApplied { name, amount } => {
                    log.push(BattleEvent::StatusHeal {
                        step,
                        character_id: char_id,
                        character_name: char_name.clone(),
                        status_name: name,
                        amount,
                        hp_remaining: team[idx].current_hp(),
                    });
                }
            }
        }
        // Check if actor died from status damage
        if !team[idx].is_alive() {
            log.push(BattleEvent::Defeat {
                step,
                character_id: char_id,
                character_name: char_name,
            });
        }
    }

    /// Resolve the actor's target, reassigning if needed. Returns None if no enemies alive.
    fn resolve_target(
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
                .map_or(true, |t| !t.is_alive()),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abilities::{AbilityDef, Primitive, AbilityTarget, PassiveMap};
    use crate::logger::BattleEvent;
    use crate::models::{Comparator, Condition, ConditionSubject, Position, QueryValue, Rule, Stat};
    use crate::statuses::StatusMap;
    use std::collections::HashMap;

    fn empty_abilities() -> AbilityMap {
        HashMap::new()
    }

    fn empty_passives() -> PassiveMap {
        HashMap::new()
    }

    fn empty_statuses() -> StatusMap {
        HashMap::new()
    }

    fn make_config(name: &str, row: u8, stats: Vec<(Stat, u32)>) -> CharacterConfig {
        make_config_at(name, row, 0, stats)
    }

    fn make_config_at(name: &str, row: u8, col: u8, stats: Vec<(Stat, u32)>) -> CharacterConfig {
        CharacterConfig {
            base_name: name.to_string(),
            passive: String::new(),
            actives: Vec::new(),
            item: None,
            position: Position { row, col },
            stats: stats.into_iter().collect(),
            rules: Vec::new(),
        }
    }

    fn warrior() -> CharacterConfig {
        make_config("Warrior", 0, vec![
            (Stat::CON, 12), (Stat::STR, 15), (Stat::INT, 4),
            (Stat::FOR, 10), (Stat::WIS, 5), (Stat::DEX, 8),
            (Stat::SPI, 6), (Stat::FOC, 5), (Stat::RES, 5),
        ])
    }

    fn mage() -> CharacterConfig {
        make_config("Mage", 0, vec![
            (Stat::CON, 8), (Stat::STR, 4), (Stat::INT, 16),
            (Stat::FOR, 5), (Stat::WIS, 12), (Stat::DEX, 10),
            (Stat::SPI, 10), (Stat::FOC, 8), (Stat::RES, 7),
        ])
    }

    #[test]
    fn battle_produces_start_and_end_events() {
        let log = BattleState::new(&[warrior()], &[mage()], empty_abilities(), empty_passives(), empty_statuses(), 42).run();
        let events = log.events();
        assert!(events.len() >= 2);
        assert!(matches!(&events[0], BattleEvent::BattleStart { .. }));
        assert!(matches!(events.last().unwrap(), BattleEvent::BattleEnd { .. }));
    }

    #[test]
    fn battle_is_deterministic_with_same_seed() {
        let log1 = BattleState::new(&[warrior()], &[mage()], empty_abilities(), empty_passives(), empty_statuses(), 123).run().to_json();
        let log2 = BattleState::new(&[warrior()], &[mage()], empty_abilities(), empty_passives(), empty_statuses(), 123).run().to_json();
        assert_eq!(log1, log2);
    }

    #[test]
    fn battle_has_winner() {
        let log = BattleState::new(&[warrior()], &[mage()], empty_abilities(), empty_passives(), empty_statuses(), 42).run();
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
        let log = BattleState::new(&[warrior()], &[mage()], empty_abilities(), empty_passives(), empty_statuses(), 42).run();
        let has_defeat = log.events().iter().any(|e| matches!(e, BattleEvent::Defeat { .. }));
        assert!(has_defeat, "A 1v1 battle should have a Defeat event");
    }

    #[test]
    fn battle_ids_are_unique_across_teams() {
        let battle = BattleState::new(&[warrior(), warrior()], &[mage(), mage()], empty_abilities(), empty_passives(), empty_statuses(), 0);
        let log = battle.run();
        assert!(log.events().len() > 2);
    }

    #[test]
    fn high_con_tank_survives_longer() {
        let tank = make_config("Tank", 0, vec![
            (Stat::CON, 50), (Stat::STR, 8), (Stat::INT, 4),
            (Stat::FOR, 10), (Stat::WIS, 10), (Stat::DEX, 5),
            (Stat::SPI, 5),
        ]);
        let glass = make_config("Glass", 0, vec![
            (Stat::CON, 3), (Stat::STR, 20), (Stat::INT, 4),
            (Stat::FOR, 2), (Stat::WIS, 2), (Stat::DEX, 5),
            (Stat::SPI, 5),
        ]);
        let log = BattleState::new(&[tank], &[glass], empty_abilities(), empty_passives(), empty_statuses(), 42).run();
        match log.events().last().unwrap() {
            BattleEvent::BattleEnd { winner, .. } => {
                assert_eq!(winner, "team_a");
            }
            _ => panic!("Expected BattleEnd"),
        }
    }

    #[test]
    fn draw_safety_triggers_at_max_steps() {
        let tanky = make_config("Tanky", 0, vec![
            (Stat::CON, 200), (Stat::STR, 1), (Stat::INT, 1),
            (Stat::FOR, 50), (Stat::WIS, 50), (Stat::DEX, 30),
            (Stat::SPI, 5),
        ]);
        let log = BattleState::new(&[tanky.clone()], &[tanky], empty_abilities(), empty_passives(), empty_statuses(), 0).run();
        match log.events().last().unwrap() {
            BattleEvent::BattleEnd { winner, .. } => {
                assert_eq!(winner, "draw");
            }
            _ => panic!("Expected BattleEnd"),
        }
    }

    #[test]
    fn three_v_three_completes() {
        let front1 = make_config("Front1", 0, vec![
            (Stat::CON, 10), (Stat::STR, 8), (Stat::INT, 3),
            (Stat::FOR, 5), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 4),
        ]);
        let front2 = make_config("Front2", 0, vec![
            (Stat::CON, 8), (Stat::STR, 6), (Stat::INT, 4),
            (Stat::FOR, 4), (Stat::WIS, 4), (Stat::DEX, 6), (Stat::SPI, 5),
        ]);
        let back = make_config("Back", 1, vec![
            (Stat::CON, 6), (Stat::STR, 3), (Stat::INT, 10),
            (Stat::FOR, 2), (Stat::WIS, 8), (Stat::DEX, 4), (Stat::SPI, 7),
        ]);
        let log = BattleState::new(
            &[front1.clone(), back.clone()],
            &[front2.clone(), back.clone(), front1.clone()],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            99,
        ).run();
        let events = log.events();
        assert!(matches!(events.last().unwrap(), BattleEvent::BattleEnd { .. }));
        let defeat_count = events.iter().filter(|e| matches!(e, BattleEvent::Defeat { .. })).count();
        assert!(defeat_count >= 2, "3v3 should have at least 2 defeats, got {}", defeat_count);
    }

    #[test]
    fn row_protection_prevents_back_row_targeting() {
        let front = make_config("Front", 0, vec![
            (Stat::CON, 15), (Stat::STR, 6), (Stat::INT, 3),
            (Stat::FOR, 5), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 4),
        ]);
        let squishy_back = make_config("SquishyBack", 1, vec![
            (Stat::CON, 3), (Stat::STR, 3), (Stat::INT, 10),
            (Stat::FOR, 1), (Stat::WIS, 8), (Stat::DEX, 4), (Stat::SPI, 7),
        ]);
        let attacker = make_config("Attacker", 0, vec![
            (Stat::CON, 12), (Stat::STR, 10), (Stat::INT, 3),
            (Stat::FOR, 5), (Stat::WIS, 3), (Stat::DEX, 6), (Stat::SPI, 4),
        ]);
        let log = BattleState::new(
            &[attacker],
            &[front, squishy_back],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            42,
        ).run();

        let mut front_defeated = false;
        for event in log.events() {
            match event {
                BattleEvent::Defeat { character_name, .. } if character_name == "Front" => {
                    front_defeated = true;
                }
                BattleEvent::BasicAttack { actor_id, target_name, .. } if *actor_id == 0 => {
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
        let fighter = make_config("Fighter", 0, vec![
            (Stat::CON, 10), (Stat::STR, 8), (Stat::INT, 3),
            (Stat::FOR, 5), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 4),
        ]);
        let lone = make_config("Lone", 0, vec![
            (Stat::CON, 8), (Stat::STR, 6), (Stat::INT, 3),
            (Stat::FOR, 3), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 4),
        ]);
        let log = BattleState::new(
            &[fighter.clone(), fighter.clone(), fighter],
            &[lone],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            42,
        ).run();
        match log.events().last().unwrap() {
            BattleEvent::BattleEnd { winner, .. } => assert_eq!(winner, "team_a"),
            _ => panic!("Expected BattleEnd"),
        }
    }

    #[test]
    fn multi_row_formation_two_rows() {
        let tanky = make_config("Tank", 0, vec![
            (Stat::CON, 15), (Stat::STR, 5), (Stat::INT, 3),
            (Stat::FOR, 8), (Stat::WIS, 5), (Stat::DEX, 4), (Stat::SPI, 4),
        ]);
        let dps = make_config("DPS", 1, vec![
            (Stat::CON, 6), (Stat::STR, 12), (Stat::INT, 3),
            (Stat::FOR, 3), (Stat::WIS, 2), (Stat::DEX, 6), (Stat::SPI, 4),
        ]);
        let log = BattleState::new(
            &[tanky.clone(), dps.clone()],
            &[tanky, dps.clone(), dps],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            7,
        ).run();
        let events = log.events();
        assert!(matches!(events.last().unwrap(), BattleEvent::BattleEnd { .. }));
        let unique_actors: std::collections::HashSet<u32> = events.iter().filter_map(|e| match e {
            BattleEvent::BasicAttack { actor_id, .. } => Some(*actor_id),
            _ => None,
        }).collect();
        assert!(unique_actors.len() >= 3, "Expected at least 3 unique actors, got {}", unique_actors.len());
    }

    #[test]
    fn dead_characters_do_not_act() {
        let front = make_config("Front", 0, vec![
            (Stat::CON, 8), (Stat::STR, 8), (Stat::INT, 3),
            (Stat::FOR, 4), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 4),
        ]);
        let back = make_config("Back", 1, vec![
            (Stat::CON, 6), (Stat::STR, 3), (Stat::INT, 8),
            (Stat::FOR, 2), (Stat::WIS, 6), (Stat::DEX, 5), (Stat::SPI, 6),
        ]);
        let log = BattleState::new(
            &[front.clone(), back.clone(), front.clone()],
            &[front, back.clone(), back],
            empty_abilities(),
            empty_passives(),
            empty_statuses(),
            42,
        ).run();

        let mut defeated_ids: std::collections::HashSet<u32> = std::collections::HashSet::new();
        for event in log.events() {
            match event {
                BattleEvent::BasicAttack { actor_id, .. } => {
                    assert!(
                        !defeated_ids.contains(actor_id),
                        "Defeated character {} acted after death", actor_id
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
            (Stat::CON, 10), (Stat::STR, 6), (Stat::INT, 4),
            (Stat::FOR, 4), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 5),
        ]
    }

    #[test]
    fn companions_assigned_by_cardinal_adjacency() {
        let a = make_config_at("A", 0, 0, simple_stats());
        let b = make_config_at("B", 0, 1, simple_stats());
        let c = make_config_at("C", 1, 0, simple_stats());
        let d = make_config_at("D", 0, 2, simple_stats());

        let dummy = make_config_at("Enemy", 0, 0, simple_stats());
        let battle = BattleState::new(&[a, b, c, d], &[dummy], empty_abilities(), empty_passives(), empty_statuses(), 0);

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
        let battle = BattleState::new(&[a], &[b], empty_abilities(), empty_passives(), empty_statuses(), 0);

        assert!(battle.team_a[0].companions().is_empty());
        assert!(battle.team_b[0].companions().is_empty());
    }

    #[test]
    fn isolated_character_has_no_companions() {
        let loner = make_config_at("Loner", 0, 0, simple_stats());
        let far = make_config_at("Far", 2, 2, simple_stats());
        let enemy = make_config_at("Enemy", 0, 0, simple_stats());
        let battle = BattleState::new(&[loner, far], &[enemy], empty_abilities(), empty_passives(), empty_statuses(), 0);

        assert!(battle.team_a[0].companions().is_empty());
        assert!(battle.team_a[1].companions().is_empty());
    }

    // --- Ability integration tests ---

    fn crush_ability() -> AbilityDef {
        AbilityDef {
            spi_cost: 2,
            primitives: vec![Primitive::DealPhysicalDamage {
                target: AbilityTarget::CurrentTarget,
                multiplier: 1.5,
            }],
        }
    }

    fn embolden_ability() -> AbilityDef {
        AbilityDef {
            spi_cost: 3,
            primitives: vec![Primitive::RestoreSpi {
                target: AbilityTarget::Companions,
                amount: 1,
            }],
        }
    }

    fn test_abilities() -> AbilityMap {
        let mut map = HashMap::new();
        map.insert("Crush".to_string(), crush_ability());
        map.insert("Embolden".to_string(), embolden_ability());
        map
    }

    fn emperor_config() -> CharacterConfig {
        let mut config = make_config_at("The Emperor", 0, 0, vec![
            (Stat::CON, 10), (Stat::STR, 6), (Stat::INT, 4),
            (Stat::FOR, 3), (Stat::WIS, 2), (Stat::DEX, 4), (Stat::SPI, 5),
            (Stat::FOC, 5), (Stat::RES, 3),
        ]);
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
                    value: QueryValue::Spi,
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
        let enemy = make_config("Enemy", 0, vec![
            (Stat::CON, 15), (Stat::STR, 4), (Stat::INT, 3),
            (Stat::FOR, 3), (Stat::WIS, 3), (Stat::DEX, 4), (Stat::SPI, 4),
        ]);

        let log = BattleState::new(&[emperor], &[enemy], test_abilities(), empty_passives(), empty_statuses(), 42).run();
        let events = log.events();

        // Should have at least one AbilityUsed for Crush
        let crush_count = events.iter().filter(|e| matches!(e,
            BattleEvent::AbilityUsed { ability_name, .. } if ability_name == "Crush"
        )).count();
        assert!(crush_count > 0, "Emperor should use Crush at least once");
    }

    #[test]
    fn emperor_falls_back_to_basic_attack_when_spi_exhausted() {
        // Emperor with only 2 SPI — can Crush once, then basic attacks
        let mut emperor = emperor_config();
        emperor.stats.insert(Stat::SPI, 2);

        let enemy = make_config("Enemy", 0, vec![
            (Stat::CON, 30), (Stat::STR, 4), (Stat::INT, 3),
            (Stat::FOR, 3), (Stat::WIS, 3), (Stat::DEX, 4), (Stat::SPI, 4),
        ]);

        let log = BattleState::new(&[emperor], &[enemy], test_abilities(), empty_passives(), empty_statuses(), 42).run();
        let events = log.events();

        let crush_count = events.iter().filter(|e| matches!(e,
            BattleEvent::AbilityUsed { ability_name, .. } if ability_name == "Crush"
        )).count();
        let basic_count = events.iter().filter(|e| matches!(e,
            BattleEvent::BasicAttack { actor_id, .. } if *actor_id == 0
        )).count();

        assert!(crush_count >= 1, "Should use Crush at least once");
        assert!(basic_count >= 1, "Should fall back to basic attack when SPI runs out");
    }

    #[test]
    fn characters_without_rules_only_basic_attack() {
        // Two characters with no rules — should only produce BasicAttack events
        let a = make_config("A", 0, vec![
            (Stat::CON, 10), (Stat::STR, 6), (Stat::INT, 4),
            (Stat::FOR, 3), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 5),
        ]);
        let b = make_config("B", 0, vec![
            (Stat::CON, 10), (Stat::STR, 6), (Stat::INT, 4),
            (Stat::FOR, 3), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 5),
        ]);

        let log = BattleState::new(&[a], &[b], test_abilities(), empty_passives(), empty_statuses(), 42).run();
        let has_ability = log.events().iter().any(|e| matches!(e, BattleEvent::AbilityUsed { .. }));
        assert!(!has_ability, "Characters without rules should not use abilities");
    }

    // --- Effect ticking tests ---

    #[test]
    fn status_damage_produces_events() {
        use crate::statuses::{StatusDef, StatusBehavior, StackType};

        let mut attacker = make_config("Attacker", 0, vec![
            (Stat::CON, 20), (Stat::STR, 6), (Stat::INT, 3),
            (Stat::FOR, 5), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 5),
        ]);
        attacker.rules = vec![Rule {
            ability: "Poison".to_string(),
            conditions: Vec::new(),
        }];
        let enemy = make_config("Enemy", 0, vec![
            (Stat::CON, 30), (Stat::STR, 4), (Stat::INT, 3),
            (Stat::FOR, 3), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 4),
        ]);

        let mut abilities = HashMap::new();
        abilities.insert("Poison".to_string(), AbilityDef {
            spi_cost: 1,
            primitives: vec![Primitive::ApplyStatus {
                target: AbilityTarget::CurrentTarget,
                status: "Poison".to_string(),
                stat: None,
                stacks: 3,
            }],
        });

        let mut statuses: StatusMap = HashMap::new();
        statuses.insert("Poison".to_string(), StatusDef {
            behavior: StatusBehavior::DamagePerStack { value: 2 },
            stack_type: StackType::TickDown,
            opposes: None,
        });

        let log = BattleState::new(&[attacker], &[enemy], abilities, empty_passives(), statuses, 42).run();
        let status_dmg_count = log.events().iter().filter(|e| matches!(e, BattleEvent::StatusDamage { .. })).count();
        assert!(status_dmg_count > 0, "Should have StatusDamage events from Poison");
    }

    #[test]
    fn incapacitated_character_skips_turn() {
        use crate::statuses::{StatusDef, StatusBehavior, StackType};

        let a = make_config("A", 0, vec![
            (Stat::CON, 50), (Stat::STR, 6), (Stat::INT, 3),
            (Stat::FOR, 5), (Stat::WIS, 3), (Stat::DEX, 3), (Stat::SPI, 5),
        ]);
        let b = make_config("B", 0, vec![
            (Stat::CON, 50), (Stat::STR, 6), (Stat::INT, 3),
            (Stat::FOR, 5), (Stat::WIS, 3), (Stat::DEX, 3), (Stat::SPI, 5),
        ]);

        let stun_def = StatusDef {
            behavior: StatusBehavior::SkipTurn,
            stack_type: StackType::NoStack,
            opposes: None,
        };

        let mut battle = BattleState::new(&[a], &[b], empty_abilities(), empty_passives(), empty_statuses(), 42);
        battle.team_a[0].add_status("Stun", 2, 99, &stun_def, None);

        let log = battle.run();
        let skip_count = log.events().iter().filter(|e| matches!(e, BattleEvent::TurnSkipped { .. })).count();
        assert!(skip_count > 0, "Stunned character should have TurnSkipped events");
    }

    #[test]
    fn on_battle_start_passive_fires() {
        use crate::abilities::{PassiveDef, PassiveTrigger};
        use crate::statuses::{StatusDef, StatusBehavior, StackType};

        let mut char_config = make_config("Warrior", 0, vec![
            (Stat::CON, 10), (Stat::STR, 6), (Stat::INT, 3),
            (Stat::FOR, 5), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 5),
        ]);
        char_config.passive = "TestPassive".to_string();

        let enemy = make_config("Enemy", 0, vec![
            (Stat::CON, 10), (Stat::STR, 4), (Stat::INT, 3),
            (Stat::FOR, 3), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 4),
        ]);

        let mut passives: PassiveMap = HashMap::new();
        passives.insert("TestPassive".to_string(), PassiveDef::Triggered {
            trigger: PassiveTrigger::OnBattleStart,
            primitives: vec![Primitive::ApplyStatus {
                target: AbilityTarget::SelfChar,
                status: "Empower".to_string(),
                stat: Some(Stat::STR),
                stacks: 5,
            }],
        });

        let mut statuses: StatusMap = HashMap::new();
        statuses.insert("Empower".to_string(), StatusDef {
            behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
            stack_type: StackType::Permanent,
            opposes: None,
        });

        let log = BattleState::new(&[char_config], &[enemy], empty_abilities(), passives, statuses, 42).run();
        let has_passive = log.events().iter().any(|e| matches!(e,
            BattleEvent::PassiveTriggered { passive_name, .. } if passive_name == "TestPassive"
        ));
        assert!(has_passive, "Should have PassiveTriggered event at battle start");
    }

    #[test]
    fn passive_buff_affects_combat() {
        use crate::abilities::{PassiveDef, PassiveTrigger};
        use crate::statuses::{StatusDef, StatusBehavior, StackType};

        let mut warrior = make_config("Warrior", 0, vec![
            (Stat::CON, 10), (Stat::STR, 6), (Stat::INT, 3),
            (Stat::FOR, 5), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 5),
        ]);
        warrior.passive = "PowerUp".to_string();

        let enemy = make_config("Enemy", 0, vec![
            (Stat::CON, 10), (Stat::STR, 4), (Stat::INT, 3),
            (Stat::FOR, 3), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 4),
        ]);

        let mut passives: PassiveMap = HashMap::new();
        passives.insert("PowerUp".to_string(), PassiveDef::Triggered {
            trigger: PassiveTrigger::OnBattleStart,
            primitives: vec![Primitive::ApplyStatus {
                target: AbilityTarget::SelfChar,
                status: "Empower".to_string(),
                stat: Some(Stat::STR),
                stacks: 100,
            }],
        });

        let mut statuses: StatusMap = HashMap::new();
        statuses.insert("Empower".to_string(), StatusDef {
            behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
            stack_type: StackType::Permanent,
            opposes: None,
        });

        // With huge STR buff, warrior should win easily
        let log = BattleState::new(&[warrior], &[enemy], empty_abilities(), passives, statuses, 42).run();
        match log.events().last().unwrap() {
            BattleEvent::BattleEnd { winner, .. } => assert_eq!(winner, "team_a"),
            _ => panic!("Expected BattleEnd"),
        }
    }

    #[test]
    fn unknown_passive_does_not_crash() {
        let mut char_config = make_config("Warrior", 0, vec![
            (Stat::CON, 10), (Stat::STR, 6), (Stat::INT, 3),
            (Stat::FOR, 5), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 5),
        ]);
        char_config.passive = "NonexistentPassive".to_string();

        let enemy = make_config("Enemy", 0, vec![
            (Stat::CON, 10), (Stat::STR, 4), (Stat::INT, 3),
            (Stat::FOR, 3), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 4),
        ]);

        // Should complete without panicking
        let log = BattleState::new(&[char_config], &[enemy], empty_abilities(), empty_passives(), empty_statuses(), 42).run();
        assert!(matches!(log.events().last().unwrap(), BattleEvent::BattleEnd { .. }));
    }

    // --- Permanent trait tests ---

    #[test]
    fn trait_passive_applied_at_battle_start() {
        use crate::abilities::PassiveDef;
        use crate::models::TraitEffect;

        let mut char_config = make_config("Warrior", 0, vec![
            (Stat::CON, 10), (Stat::STR, 6), (Stat::INT, 3),
            (Stat::FOR, 5), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 5),
        ]);
        char_config.passive = "Thorns".to_string();

        let enemy = make_config("Enemy", 0, vec![
            (Stat::CON, 10), (Stat::STR, 4), (Stat::INT, 3),
            (Stat::FOR, 3), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 4),
        ]);

        let mut passives: PassiveMap = HashMap::new();
        passives.insert("Thorns".to_string(), PassiveDef::Trait {
            effect: TraitEffect::DamageReflect { amount: 2 },
        });

        let log = BattleState::new(&[char_config], &[enemy], empty_abilities(), passives, empty_statuses(), 42).run();
        let has_passive = log.events().iter().any(|e| matches!(e,
            BattleEvent::PassiveTriggered { passive_name, .. } if passive_name == "Thorns"
        ));
        assert!(has_passive, "Trait passive should log PassiveTriggered");
    }

    #[test]
    fn damage_reflect_hurts_attacker() {
        use crate::abilities::PassiveDef;
        use crate::models::TraitEffect;

        // Defender has DamageReflect, attacker should take reflect damage
        let attacker = make_config("Attacker", 0, vec![
            (Stat::CON, 20), (Stat::STR, 6), (Stat::INT, 3),
            (Stat::FOR, 5), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 4),
        ]);

        let mut defender = make_config("Defender", 0, vec![
            (Stat::CON, 20), (Stat::STR, 4), (Stat::INT, 3),
            (Stat::FOR, 3), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 4),
        ]);
        defender.passive = "Thorns".to_string();

        let mut passives: PassiveMap = HashMap::new();
        passives.insert("Thorns".to_string(), PassiveDef::Trait {
            effect: TraitEffect::DamageReflect { amount: 2 },
        });

        let log = BattleState::new(&[attacker], &[defender], empty_abilities(), passives, empty_statuses(), 42).run();
        let has_reflect = log.events().iter().any(|e| matches!(e, BattleEvent::DamageReflect { .. }));
        assert!(has_reflect, "Should have DamageReflect events");
    }

    #[test]
    fn damage_reflect_can_kill_attacker() {
        use crate::abilities::PassiveDef;
        use crate::models::TraitEffect;

        // Attacker has very low HP, defender has high reflect
        let attacker = make_config("Attacker", 0, vec![
            (Stat::CON, 2), (Stat::STR, 6), (Stat::INT, 3),
            (Stat::FOR, 5), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 4),
        ]);

        let mut defender = make_config("Defender", 0, vec![
            (Stat::CON, 50), (Stat::STR, 4), (Stat::INT, 3),
            (Stat::FOR, 3), (Stat::WIS, 3), (Stat::DEX, 5), (Stat::SPI, 4),
        ]);
        defender.passive = "Thorns".to_string();

        let mut passives: PassiveMap = HashMap::new();
        passives.insert("Thorns".to_string(), PassiveDef::Trait {
            effect: TraitEffect::DamageReflect { amount: 50 },
        });

        let log = BattleState::new(&[attacker], &[defender], empty_abilities(), passives, empty_statuses(), 42).run();

        // Attacker should die from reflect
        let attacker_defeated = log.events().iter().any(|e| matches!(e,
            BattleEvent::Defeat { character_name, .. } if character_name == "Attacker"
        ));
        assert!(attacker_defeated, "Attacker should die from reflect damage");

        // Defender should win
        match log.events().last().unwrap() {
            BattleEvent::BattleEnd { winner, .. } => assert_eq!(winner, "team_b"),
            _ => panic!("Expected BattleEnd"),
        }
    }

    #[test]
    fn spi_cost_reduction_in_battle() {
        use crate::abilities::PassiveDef;
        use crate::models::TraitEffect;

        // Emperor with SPI cost reduction — should be able to use Crush more
        let mut emperor = emperor_config();
        emperor.passive = "Thrift".to_string();
        emperor.stats.insert(Stat::SPI, 3); // barely enough for Crush(2) without reduction

        let enemy = make_config("Enemy", 0, vec![
            (Stat::CON, 30), (Stat::STR, 4), (Stat::INT, 3),
            (Stat::FOR, 3), (Stat::WIS, 3), (Stat::DEX, 4), (Stat::SPI, 4),
        ]);

        let mut passives: PassiveMap = HashMap::new();
        passives.insert("Thrift".to_string(), PassiveDef::Trait {
            effect: TraitEffect::SpiCostReduction { amount: 1 },
        });

        let log = BattleState::new(&[emperor], &[enemy], test_abilities(), passives, empty_statuses(), 42).run();

        // With reduction, Crush costs 1 instead of 2, so should use it more times
        let crush_count = log.events().iter().filter(|e| matches!(e,
            BattleEvent::AbilityUsed { ability_name, .. } if ability_name == "Crush"
        )).count();
        assert!(crush_count >= 2, "Should use Crush at least twice with cost reduction, got {}", crush_count);
    }
}
