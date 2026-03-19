//! Status ticking and status-trigger helpers for the battle engine.

use crate::abilities::PassiveTrigger;
use crate::logger::BattleEvent;
use crate::models::StatusTick;

use super::BattleState;

impl BattleState {
    pub(crate) fn process_status_application_events(&mut self, event_start: usize, is_team_a: bool) {
        let omen_applications: Vec<(u32, u32)> = self
            .log
            .events_from(event_start)
            .iter()
            .filter_map(|event| match event {
                BattleEvent::StatusApplied {
                    actor_id,
                    target_id,
                    status_name,
                    ..
                } if status_name == "Omen" => Some((*actor_id, *target_id)),
                _ => None,
            })
            .collect();

        for (source_id, target_id) in omen_applications {
            let owner_indices: Vec<usize> = {
                let (actor_team, _) = Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
                actor_team
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.is_alive() && c.id() != source_id)
                    .map(|(idx, _)| idx)
                    .collect()
            };

            for owner_idx in owner_indices {
                self.try_fire_passive_with_target(
                    owner_idx,
                    &PassiveTrigger::OnAllyApplyOmen,
                    is_team_a,
                    Some(target_id),
                );
            }
        }
    }

    /// Tick statuses on a character and log any DoT/HoT results.
    pub(crate) fn tick_and_log_statuses(&mut self, idx: usize, is_team_a: bool) {
        let step = self.step;
        let log = &mut self.log;
        let (team, _) = Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
        let ticks = team[idx].tick_statuses();
        for tick in ticks {
            match tick {
                StatusTick::DamageDealt { name, damage } => {
                    log.push(BattleEvent::StatusDamage {
                        tick_count: step,
                        character_id: team[idx].id(),
                        character_name: team[idx].base_name().to_string(),
                        status_name: name,
                        damage,
                        hp_remaining: team[idx].current_hp(),
                    });
                }
                StatusTick::HealApplied { name, amount } => {
                    log.push(BattleEvent::StatusHeal {
                        tick_count: step,
                        character_id: team[idx].id(),
                        character_name: team[idx].base_name().to_string(),
                        status_name: name,
                        amount,
                        hp_remaining: team[idx].current_hp(),
                    });
                }
            }
        }
        if !team[idx].is_alive() {
            self.resolve_character_death(idx, is_team_a);
        }
    }
}
