//! Damage and death aftermath helpers for the battle engine.

use std::collections::HashSet;

use crate::abilities::{DamageRecord, PassiveTrigger};
use crate::logger::BattleEvent;

use super::BattleState;

impl BattleState {
    /// Resolve deaths caused by passive damage while respecting the passive re-entry guard.
    pub(crate) fn resolve_defeats_from_damage(
        &mut self,
        damage_dealt: &[DamageRecord],
        team_is_a: bool,
    ) {
        let mut seen = HashSet::new();
        for record in damage_dealt {
            if !seen.insert(record.target_id) {
                continue;
            }

            let idx_opt = if team_is_a {
                self.team_a
                    .iter()
                    .position(|c| c.id() == record.target_id && !c.is_alive())
            } else {
                self.team_b
                    .iter()
                    .position(|c| c.id() == record.target_id && !c.is_alive())
            };

            if let Some(idx) = idx_opt {
                self.resolve_character_death(idx, team_is_a);
            }
        }
    }

    /// Process damage results: check defeats, fire on_kill/on_deal_damage/on_take_damage
    /// passives, and apply damage reflect.
    pub(crate) fn process_damage_results(
        &mut self,
        _actor_idx: usize,
        is_team_a: bool,
        damage_dealt: &[DamageRecord],
    ) {
        if damage_dealt.is_empty() {
            return;
        }

        let mut damaged_enemy_indices: HashSet<usize> = HashSet::new();
        let mut defeated_enemy_indices: HashSet<usize> = HashSet::new();
        let mut kills_by_source: Vec<(u32, usize)> = Vec::new();
        let mut reflect_sources: Vec<(u32, u32, String, u32)> = Vec::new();

        {
            let (actor_team, enemy_team) =
                Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
            for record in damage_dealt {
                if let Some(eidx) = enemy_team.iter().position(|c| c.id() == record.target_id) {
                    if record.damage > 0 {
                        damaged_enemy_indices.insert(eidx);
                    }
                    if !enemy_team[eidx].is_alive() {
                        defeated_enemy_indices.insert(eidx);
                        kills_by_source.push((record.source_id, eidx));
                    }
                    let reflect = enemy_team[eidx].damage_reflect_amount();
                    if reflect > 0
                        && actor_team
                            .iter()
                            .any(|c| c.id() == record.source_id && c.is_alive())
                    {
                        reflect_sources.push((
                            record.source_id,
                            enemy_team[eidx].id(),
                            enemy_team[eidx].base_name().to_string(),
                            reflect,
                        ));
                    }
                }
            }
        }

        for (source_id, reflector_id, reflector_name, reflect) in &reflect_sources {
            let source_idx_opt = if is_team_a {
                self.team_a.iter().position(|c| c.id() == *source_id)
            } else {
                self.team_b.iter().position(|c| c.id() == *source_id)
            };
            let Some(source_idx) = source_idx_opt else {
                continue;
            };
            let reflect = {
                let (actor_team, _) =
                    Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
                if !actor_team[source_idx].is_alive() {
                    continue;
                }
                actor_team[source_idx].take_hit(*reflect)
            };
            let target_name = if is_team_a {
                self.team_a[source_idx].base_name().to_string()
            } else {
                self.team_b[source_idx].base_name().to_string()
            };
            let target_hp_remaining = if is_team_a {
                self.team_a[source_idx].current_hp()
            } else {
                self.team_b[source_idx].current_hp()
            };
            self.log.push(BattleEvent::DamageReflect {
                tick_count: self.step,
                reflector_id: *reflector_id,
                reflector_name: reflector_name.clone(),
                target_id: *source_id,
                target_name,
                damage: reflect,
                target_hp_remaining,
            });
            self.capture_latest_replay_snapshot();
            let source_alive = if is_team_a {
                self.team_a[source_idx].is_alive()
            } else {
                self.team_b[source_idx].is_alive()
            };
            if !source_alive {
                self.resolve_character_death(source_idx, is_team_a);
            }
        }

        for record in damage_dealt.iter().filter(|record| record.damage > 0) {
            let (actor_team, _) = Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
            if let Some(source_idx) = actor_team
                .iter()
                .position(|c| c.id() == record.source_id && c.is_alive())
            {
                self.try_fire_passive_with_target(
                    source_idx,
                    &PassiveTrigger::OnDealDamage,
                    is_team_a,
                    Some(record.target_id),
                );
            }
        }

        for record in damage_dealt.iter().filter(|record| record.damage > 0) {
            let owner_indices: Vec<usize> = {
                let (actor_team, _) =
                    Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
                actor_team
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| {
                        c.is_alive()
                            && c.id() != record.source_id
                            && c.target() == Some(record.target_id)
                    })
                    .map(|(idx, _)| idx)
                    .collect()
            };

            for owner_idx in owner_indices {
                self.try_fire_passive(owner_idx, &PassiveTrigger::OnAllyDamageMyTarget, is_team_a);
            }

            let row_owner_indices: Vec<usize> = {
                let (actor_team, _) =
                    Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
                let Some(source_idx) = actor_team
                    .iter()
                    .position(|c| c.id() == record.source_id && c.is_alive())
                else {
                    continue;
                };
                let source_row = actor_team[source_idx].position().row;
                actor_team
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| {
                        c.is_alive()
                            && c.id() != record.source_id
                            && c.position().row == source_row
                            && c.target() == Some(record.target_id)
                    })
                    .map(|(idx, _)| idx)
                    .collect()
            };

            for owner_idx in row_owner_indices {
                self.try_fire_passive_with_target(
                    owner_idx,
                    &PassiveTrigger::OnRowAllyDamageMyTarget,
                    is_team_a,
                    Some(record.source_id),
                );
            }
        }

        for eidx in damaged_enemy_indices {
            let (enemy_alive, target_id, below_half, companion_owner_indices) = {
                let (_, enemy_team) =
                    Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
                let enemy_alive = enemy_team[eidx].is_alive();
                let target_id = enemy_team[eidx].id();
                let below_half = enemy_team[eidx].current_hp() * 2
                    <= enemy_team[eidx].get_base_stat(&crate::models::Stat::VIT) * 3;
                let companion_owner_indices = if enemy_alive && below_half {
                    enemy_team
                        .iter()
                        .enumerate()
                        .filter(|(idx, character)| {
                            *idx != eidx
                                && character.is_alive()
                                && character.effective_companion_ids().contains(&target_id)
                        })
                        .map(|(idx, _)| idx)
                        .collect()
                } else {
                    Vec::new()
                };
                (enemy_alive, target_id, below_half, companion_owner_indices)
            };
            if enemy_alive {
                self.try_fire_passive(eidx, &PassiveTrigger::OnTakeDamage, !is_team_a);
            }
            if enemy_alive && below_half {
                self.try_fire_passive(eidx, &PassiveTrigger::OnSelfBelowHalfHp, !is_team_a);
                for owner_idx in companion_owner_indices {
                    self.try_fire_passive_with_target(
                        owner_idx,
                        &PassiveTrigger::OnCompanionBelowHalfHp,
                        !is_team_a,
                        Some(target_id),
                    );
                }
            }
        }

        for (source_id, eidx) in kills_by_source {
            let (actor_team, _) = Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
            if let Some(source_idx) = actor_team
                .iter()
                .position(|c| c.id() == source_id && c.is_alive())
            {
                self.try_fire_passive(source_idx, &PassiveTrigger::OnKill, is_team_a);
            }
            self.resolve_character_death(eidx, !is_team_a);
        }

        for eidx in defeated_enemy_indices {
            self.resolve_character_death(eidx, !is_team_a);
        }
    }

    /// Run death-side effects for a character exactly when they reach 0 HP.
    pub(crate) fn resolve_character_death(&mut self, char_idx: usize, is_team_a: bool) {
        let (team, _) = Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
        if team[char_idx].is_alive() || team[char_idx].is_defeat_resolved() {
            return;
        }

        self.try_fire_passive(char_idx, &PassiveTrigger::OnDeath, is_team_a);

        {
            let (team, _) = Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
            team[char_idx].mark_defeat_resolved();
        }
        let actor = if is_team_a {
            &self.team_a[char_idx]
        } else {
            &self.team_b[char_idx]
        };
        self.log.push_defeat(self.step, actor);
        self.capture_latest_replay_snapshot();
        self.refresh_auras();
    }
}
