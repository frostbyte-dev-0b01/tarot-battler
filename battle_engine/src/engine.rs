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

        // Execute turns for ready characters
        for idx in ready_a {
            self.execute_turn_a(idx);
        }
        for idx in ready_b {
            self.execute_turn_b(idx);
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
