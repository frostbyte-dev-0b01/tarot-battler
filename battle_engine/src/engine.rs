//! Core battle simulation engine.

use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::damage::calc_basic_attack_damage;
use crate::logger::{BattleEvent, BattleLog};
use crate::models::{CharacterConfig, CharacterState, Stat};
use crate::targeting::select_target;

const MAX_STEPS: u32 = 1000;
const SPI_REGEN_INTERVAL: u32 = 10;

pub struct BattleState {
    team_a: Vec<CharacterState>,
    team_b: Vec<CharacterState>,
    step: u32,
    log: BattleLog,
    rng: StdRng,
}

impl BattleState {
    pub fn new(
        team_a_configs: &[CharacterConfig],
        team_b_configs: &[CharacterConfig],
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

        // Assign initial targets
        let mut state = Self {
            team_a,
            team_b,
            step: 0,
            log: BattleLog::new(),
            rng,
        };
        state.assign_all_targets();
        state
    }

    fn assign_all_targets(&mut self) {
        // Team A targets team B
        for attacker in &mut self.team_a {
            if let Some(target_id) = select_target(attacker, &self.team_b, &mut self.rng) {
                attacker.set_target(target_id);
            }
        }

        // Team B targets team A
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

        loop {
            if self.step_once() {
                break;
            }
        }

        self.log
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
                self.execute_turn_a(idx);
            }
        }
        for idx in ready_b {
            if self.team_b[idx].is_alive() {
                self.execute_turn_b(idx);
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

    /// Execute a turn for a team A character attacking team B.
    fn execute_turn_a(&mut self, actor_idx: usize) {
        let actor = &mut self.team_a[actor_idx];
        actor.reset_speed();

        let actor_id = actor.id();
        let actor_name = actor.base_name().to_string();

        // Get or reassign target
        let target_id = match actor.target() {
            Some(tid) => {
                if self.find_in_team_b(tid).map_or(true, |t| !t.is_alive()) {
                    let new_target = select_target(&self.team_a[actor_idx], &self.team_b, &mut self.rng);
                    match new_target {
                        Some(tid) => {
                            self.team_a[actor_idx].set_target(tid);
                            tid
                        }
                        None => return, // no living enemies
                    }
                } else {
                    tid
                }
            }
            None => {
                let new_target = select_target(&self.team_a[actor_idx], &self.team_b, &mut self.rng);
                match new_target {
                    Some(tid) => {
                        self.team_a[actor_idx].set_target(tid);
                        tid
                    }
                    None => return,
                }
            }
        };

        // Calculate damage
        let target_idx = self.team_b.iter().position(|c| c.id() == target_id).unwrap();
        let damage = calc_basic_attack_damage(
            &self.team_a[actor_idx],
            &self.team_b[target_idx],
            &mut self.rng,
        );

        // Apply damage
        let target = &mut self.team_b[target_idx];
        target.take_damage(damage);
        let target_name = target.base_name().to_string();
        let hp_remaining = target.current_hp();

        self.log.push(BattleEvent::BasicAttack {
            step: self.step,
            actor_id,
            actor_name,
            target_id,
            target_name: target_name.clone(),
            damage,
            target_hp_remaining: hp_remaining,
        });

        if !self.team_b[target_idx].is_alive() {
            self.log.push(BattleEvent::Defeat {
                step: self.step,
                character_id: target_id,
                character_name: target_name,
            });
            // Reassign target
            let new_target = select_target(&self.team_a[actor_idx], &self.team_b, &mut self.rng);
            if let Some(tid) = new_target {
                self.team_a[actor_idx].set_target(tid);
            } else {
                self.team_a[actor_idx].clear_target();
            }
        }
    }

    /// Execute a turn for a team B character attacking team A.
    fn execute_turn_b(&mut self, actor_idx: usize) {
        let actor = &mut self.team_b[actor_idx];
        actor.reset_speed();

        let actor_id = actor.id();
        let actor_name = actor.base_name().to_string();

        let target_id = match actor.target() {
            Some(tid) => {
                if self.find_in_team_a(tid).map_or(true, |t| !t.is_alive()) {
                    let new_target = select_target(&self.team_b[actor_idx], &self.team_a, &mut self.rng);
                    match new_target {
                        Some(tid) => {
                            self.team_b[actor_idx].set_target(tid);
                            tid
                        }
                        None => return,
                    }
                } else {
                    tid
                }
            }
            None => {
                let new_target = select_target(&self.team_b[actor_idx], &self.team_a, &mut self.rng);
                match new_target {
                    Some(tid) => {
                        self.team_b[actor_idx].set_target(tid);
                        tid
                    }
                    None => return,
                }
            }
        };

        let target_idx = self.team_a.iter().position(|c| c.id() == target_id).unwrap();
        let damage = calc_basic_attack_damage(
            &self.team_b[actor_idx],
            &self.team_a[target_idx],
            &mut self.rng,
        );

        let target = &mut self.team_a[target_idx];
        target.take_damage(damage);
        let target_name = target.base_name().to_string();
        let hp_remaining = target.current_hp();

        self.log.push(BattleEvent::BasicAttack {
            step: self.step,
            actor_id,
            actor_name,
            target_id,
            target_name: target_name.clone(),
            damage,
            target_hp_remaining: hp_remaining,
        });

        if !self.team_a[target_idx].is_alive() {
            self.log.push(BattleEvent::Defeat {
                step: self.step,
                character_id: target_id,
                character_name: target_name,
            });
            let new_target = select_target(&self.team_b[actor_idx], &self.team_a, &mut self.rng);
            if let Some(tid) = new_target {
                self.team_b[actor_idx].set_target(tid);
            } else {
                self.team_b[actor_idx].clear_target();
            }
        }
    }

    fn find_in_team_a(&self, id: u32) -> Option<&CharacterState> {
        self.team_a.iter().find(|c| c.id() == id)
    }

    fn find_in_team_b(&self, id: u32) -> Option<&CharacterState> {
        self.team_b.iter().find(|c| c.id() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger::BattleEvent;
    use crate::models::{Position, Stat};

    fn make_config(name: &str, row: u8, stats: Vec<(Stat, u32)>) -> CharacterConfig {
        CharacterConfig {
            base_name: name.to_string(),
            passive: String::new(),
            actives: Vec::new(),
            item: None,
            position: Position { row, col: 0 },
            stats: stats.into_iter().collect(),
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
        let log = BattleState::new(&[warrior()], &[mage()], 42).run();
        let events = log.events();
        assert!(events.len() >= 2);
        assert!(matches!(&events[0], BattleEvent::BattleStart { .. }));
        assert!(matches!(events.last().unwrap(), BattleEvent::BattleEnd { .. }));
    }

    #[test]
    fn battle_is_deterministic_with_same_seed() {
        let log1 = BattleState::new(&[warrior()], &[mage()], 123).run().to_json();
        let log2 = BattleState::new(&[warrior()], &[mage()], 123).run().to_json();
        assert_eq!(log1, log2);
    }

    #[test]
    fn battle_has_winner() {
        let log = BattleState::new(&[warrior()], &[mage()], 42).run();
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
        let log = BattleState::new(&[warrior()], &[mage()], 42).run();
        let has_defeat = log.events().iter().any(|e| matches!(e, BattleEvent::Defeat { .. }));
        assert!(has_defeat, "A 1v1 battle should have a Defeat event");
    }

    #[test]
    fn battle_ids_are_unique_across_teams() {
        let battle = BattleState::new(&[warrior(), warrior()], &[mage(), mage()], 0);
        // Team A: ids 0,1. Team B: ids 2,3.
        // Just verify it runs without panic (id collisions would cause logic errors)
        let log = battle.run();
        assert!(log.events().len() > 2);
    }

    #[test]
    fn high_con_tank_survives_longer() {
        // Tank with huge CON vs glass cannon
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
        let log = BattleState::new(&[tank], &[glass], 42).run();
        match log.events().last().unwrap() {
            BattleEvent::BattleEnd { winner, .. } => {
                // Tank should win — glass cannon only has 6 HP
                assert_eq!(winner, "team_a");
            }
            _ => panic!("Expected BattleEnd"),
        }
    }

    #[test]
    fn draw_safety_triggers_at_max_steps() {
        // Two characters that deal minimum damage (1) with massive HP — will hit step limit
        let tanky = make_config("Tanky", 0, vec![
            (Stat::CON, 200), (Stat::STR, 1), (Stat::INT, 1),
            (Stat::FOR, 50), (Stat::WIS, 50), (Stat::DEX, 30),
            (Stat::SPI, 5),
        ]);
        let log = BattleState::new(&[tanky.clone()], &[tanky], 0).run();
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
            99,
        ).run();
        let events = log.events();
        // Should have a definitive end
        assert!(matches!(events.last().unwrap(), BattleEvent::BattleEnd { .. }));
        // Should have multiple defeats (at least losing team fully defeated)
        let defeat_count = events.iter().filter(|e| matches!(e, BattleEvent::Defeat { .. })).count();
        assert!(defeat_count >= 2, "3v3 should have at least 2 defeats, got {}", defeat_count);
    }

    #[test]
    fn row_protection_prevents_back_row_targeting() {
        // Team B: tanky front row + squishy back row
        // Team A attacks team B, so we check that team A (id=0) never targets SquishyBack
        // before Front is defeated.
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
            42,
        ).run();

        // Before Front is defeated, team A's attacks (actor_id=0) must only target Front
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
        // 3v1: team A should win easily
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
            42,
        ).run();
        match log.events().last().unwrap() {
            BattleEvent::BattleEnd { winner, .. } => assert_eq!(winner, "team_a"),
            _ => panic!("Expected BattleEnd"),
        }
    }

    #[test]
    fn multi_row_formation_two_rows() {
        // Team A: 2 front, 1 back. Team B: 1 front, 2 back.
        // Team B's back row should be safe until front falls.
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
            7,
        ).run();
        let events = log.events();
        assert!(matches!(events.last().unwrap(), BattleEvent::BattleEnd { .. }));
        // Verify battle involved multiple characters acting
        let unique_actors: std::collections::HashSet<u32> = events.iter().filter_map(|e| match e {
            BattleEvent::BasicAttack { actor_id, .. } => Some(*actor_id),
            _ => None,
        }).collect();
        assert!(unique_actors.len() >= 3, "Expected at least 3 unique actors, got {}", unique_actors.len());
    }

    #[test]
    fn dead_characters_do_not_act() {
        // Run a multi-character battle and verify no defeated character acts after death
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
}
