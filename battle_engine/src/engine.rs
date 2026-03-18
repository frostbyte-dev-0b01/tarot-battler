//! Core battle simulation engine.

use std::collections::HashSet;

use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::abilities::{
    AbilityMap, DamageRecord, PassiveDef, PassiveMap, PassiveTrigger, execute_ability,
    execute_primitives,
};
use crate::damage::calc_basic_attack_damage;
use crate::logger::{BattleEvent, BattleLog};
use crate::models::{CharacterConfig, CharacterState, Stat, StatusTick};
use crate::rules::{WorldState, evaluate_rules};
use crate::statuses::StatusMap;
use crate::targeting::select_target;

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
            in_passive_phase: false,
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
        let team_a_passives: Vec<(usize, String)> = self
            .team_a
            .iter()
            .enumerate()
            .filter(|(_, c)| !c.passive().is_empty())
            .map(|(i, c)| (i, c.passive().to_string()))
            .collect();
        let team_b_passives: Vec<(usize, String)> = self
            .team_b
            .iter()
            .enumerate()
            .filter(|(_, c)| !c.passive().is_empty())
            .map(|(i, c)| (i, c.passive().to_string()))
            .collect();

        let trigger = PassiveTrigger::OnBattleStart;

        for (idx, passive_name) in team_a_passives {
            if let Some(passive_def) = self.passives.get(&passive_name).cloned() {
                let damage_dealt = Self::fire_passive_if_matches(
                    idx,
                    &passive_name,
                    &passive_def,
                    &trigger,
                    &mut self.team_a,
                    &mut self.team_b,
                    &mut self.rng,
                    &mut self.log,
                    0,
                    &self.status_defs,
                );
                self.resolve_defeats_from_damage(&damage_dealt, false);
            }
        }

        for (idx, passive_name) in team_b_passives {
            if let Some(passive_def) = self.passives.get(&passive_name).cloned() {
                let damage_dealt = Self::fire_passive_if_matches(
                    idx,
                    &passive_name,
                    &passive_def,
                    &trigger,
                    &mut self.team_b,
                    &mut self.team_a,
                    &mut self.rng,
                    &mut self.log,
                    0,
                    &self.status_defs,
                );
                self.resolve_defeats_from_damage(&damage_dealt, true);
            }
        }
    }

    /// Fire a triggered passive if it matches the expected trigger. Returns damage dealt.
    fn fire_passive_if_matches(
        idx: usize,
        passive_name: &str,
        passive_def: &PassiveDef,
        expected: &PassiveTrigger,
        actor_team: &mut [CharacterState],
        enemy_team: &mut [CharacterState],
        rng: &mut StdRng,
        log: &mut BattleLog,
        step: u32,
        status_defs: &StatusMap,
    ) -> Vec<DamageRecord> {
        match passive_def {
            PassiveDef::Triggered {
                trigger,
                primitives,
            } if std::mem::discriminant(trigger) == std::mem::discriminant(expected) => {
                let char_id = actor_team[idx].id();
                let char_name = actor_team[idx].base_name().to_string();
                log.push(BattleEvent::PassiveTriggered {
                    tick_count: step,
                    character_id: char_id,
                    character_name: char_name,
                    passive_name: passive_name.to_string(),
                });
                execute_primitives(
                    idx,
                    passive_name,
                    primitives,
                    actor_team,
                    enemy_team,
                    rng,
                    log,
                    step,
                    status_defs,
                )
            }
            PassiveDef::Trait { effect } if matches!(expected, PassiveTrigger::OnBattleStart) => {
                let char_id = actor_team[idx].id();
                let char_name = actor_team[idx].base_name().to_string();
                log.push(BattleEvent::PassiveTriggered {
                    tick_count: step,
                    character_id: char_id,
                    character_name: char_name,
                    passive_name: passive_name.to_string(),
                });
                actor_team[idx].add_trait(effect.clone());
                Vec::new()
            }
            _ => Vec::new(),
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
        if self.in_passive_phase {
            return Vec::new();
        }

        let passive_name = {
            let (actor_team, _) = if actor_team_is_a {
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
            actor_team[char_idx].passive().to_string()
        };
        if passive_name.is_empty() {
            return Vec::new();
        }

        let passive_def = match self.passives.get(&passive_name).cloned() {
            Some(def) => def,
            None => return Vec::new(),
        };

        self.in_passive_phase = true;
        let damage_dealt = {
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

            Self::fire_passive_if_matches(
                char_idx,
                &passive_name,
                &passive_def,
                trigger,
                actor_team,
                enemy_team,
                &mut self.rng,
                &mut self.log,
                self.step,
                &self.status_defs,
            )
        };

        self.resolve_defeats_from_damage(&damage_dealt, !actor_team_is_a);
        self.in_passive_phase = false;
        damage_dealt
    }

    /// Resolve deaths caused by passive damage while respecting the passive re-entry guard.
    fn resolve_defeats_from_damage(&mut self, damage_dealt: &[DamageRecord], team_is_a: bool) {
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

    /// Advance one step. Returns true if the battle is over.
    fn step_once(&mut self) -> bool {
        self.step += 1;

        if self.step > MAX_STEPS {
            self.log.push(BattleEvent::BattleEnd {
                tick_count: self.step,
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

        // Check win conditions
        let a_alive = self.team_a.iter().any(|c| c.is_alive());
        let b_alive = self.team_b.iter().any(|c| c.is_alive());

        if !a_alive && !b_alive {
            self.log.push(BattleEvent::BattleEnd {
                tick_count: self.step,
                winner: "draw".to_string(),
            });
            true
        } else if !b_alive {
            self.log.push(BattleEvent::BattleEnd {
                tick_count: self.step,
                winner: "team_a".to_string(),
            });
            true
        } else if !a_alive {
            self.log.push(BattleEvent::BattleEnd {
                tick_count: self.step,
                winner: "team_b".to_string(),
            });
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

        let (actor_team, _) = Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
        if !actor_team[actor_idx].is_alive() {
            return;
        }

        let actor_id = actor_team[actor_idx].id();
        let actor_name = actor_team[actor_idx].base_name().to_string();

        self.log.push(BattleEvent::TurnStart {
            tick_count: self.step,
            actor_id,
            actor_name: actor_name.clone(),
            current_hp: actor_team[actor_idx].current_hp(),
            current_mp: actor_team[actor_idx].current_mp(),
        });

        // Incapacitate check happens after start-of-turn passives and status ticks.
        if actor_team[actor_idx].is_incapacitated() {
            self.log.push(BattleEvent::TurnSkipped {
                tick_count: self.step,
                character_id: actor_id,
                character_name: actor_name,
                reason: "incapacitated".to_string(),
            });
            actor_team[actor_idx].consume_skip_turn_statuses();
            self.finish_turn(actor_idx, is_team_a);
            return;
        }

        // Get or reassign target
        let (actor_team, enemy_team) =
            Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
        let target_id = match Self::resolve_target(actor_idx, actor_team, enemy_team, &mut self.rng)
        {
            Some(tid) => tid,
            None => {
                self.finish_turn(actor_idx, is_team_a);
                return;
            }
        };

        let target_idx = enemy_team.iter().position(|c| c.id() == target_id).unwrap();

        // Evaluate rules to see if an ability should be used
        let target_ref = &enemy_team[target_idx];
        let world = WorldState {
            tick_count: self.step,
            ally_count: actor_team.iter().filter(|c| c.is_alive()).count() as u32,
            enemy_count: enemy_team.iter().filter(|c| c.is_alive()).count() as u32,
        };
        let ability_name = evaluate_rules(
            &actor_team[actor_idx],
            Some(target_ref),
            actor_team,
            world,
            &self.abilities,
        );

        if let Some(ref name) = ability_name {
            if let Some(ability_def) = self.abilities.get(name).cloned() {
                let (actor_team, enemy_team) =
                    Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
                // Spend MP (reduced by trait) and record usage
                let effective_cost = ability_def
                    .mp_cost
                    .saturating_sub(actor_team[actor_idx].mp_cost_reduction())
                    .max(1);
                actor_team[actor_idx].spend_mp(effective_cost);
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

                // Process damage results: defeats, reflect, and passive triggers
                self.process_damage_results(actor_idx, is_team_a, &damage_dealt);

                // Reassign target if current target is dead
                let (actor_team, enemy_team) =
                    Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
                Self::reassign_target_if_dead(actor_idx, actor_team, enemy_team, &mut self.rng);

                self.finish_turn(actor_idx, is_team_a);
                return;
            }
        }

        // Fallback: basic attack
        let (actor_team, enemy_team) =
            Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
        let damage = calc_basic_attack_damage(
            &actor_team[actor_idx],
            &enemy_team[target_idx],
            &mut self.rng,
        );

        let damage = enemy_team[target_idx].take_hit(damage);
        let target_name = enemy_team[target_idx].base_name().to_string();
        let hp_remaining = enemy_team[target_idx].current_hp();

        self.log.push(BattleEvent::BasicAttack {
            tick_count: self.step,
            actor_id,
            actor_name,
            target_id,
            target_name: target_name.clone(),
            damage,
            target_hp_remaining: hp_remaining,
        });

        // Process damage: defeat, reflect, passive triggers
        let damage_dealt = vec![DamageRecord {
            source_id: actor_id,
            target_id,
            damage,
        }];
        self.process_damage_results(actor_idx, is_team_a, &damage_dealt);

        // Reassign target if dead
        let (actor_team, enemy_team) =
            Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
        Self::reassign_target_if_dead(actor_idx, actor_team, enemy_team, &mut self.rng);

        self.finish_turn(actor_idx, is_team_a);
    }

    fn finish_turn(&mut self, actor_idx: usize, is_team_a: bool) {
        let (actor_team, _) = Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
        if !actor_team[actor_idx].is_alive() {
            return;
        }

        let regen = actor_team[actor_idx].get_base_stat(&Stat::SPI) / 2;
        let actor_id = actor_team[actor_idx].id();
        let actor_name = actor_team[actor_idx].base_name().to_string();
        actor_team[actor_idx].restore_mp(regen);
        if regen > 0 {
            self.log.push(BattleEvent::ResourceChanged {
                tick_count: self.step,
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
        if let Some(ct) = actor_team[actor_idx].target() {
            if enemy_team
                .iter()
                .find(|c| c.id() == ct)
                .map_or(true, |c| !c.is_alive())
            {
                if let Some(tid) = select_target(&actor_team[actor_idx], enemy_team, rng) {
                    actor_team[actor_idx].set_target(tid);
                } else {
                    actor_team[actor_idx].clear_target();
                }
            }
        }
    }

    /// Process damage results: check defeats, fire on_kill/on_death/on_deal_damage/on_take_damage
    /// passives, and apply damage reflect.
    fn process_damage_results(
        &mut self,
        _actor_idx: usize,
        is_team_a: bool,
        damage_dealt: &[DamageRecord],
    ) {
        if damage_dealt.is_empty() {
            return;
        }

        let mut any_damage_by_source: HashSet<u32> = HashSet::new();
        let mut damaged_enemy_indices: HashSet<usize> = HashSet::new();
        let mut defeated_enemy_indices: HashSet<usize> = HashSet::new();
        let mut kills_by_source: Vec<(u32, usize)> = Vec::new();
        let mut reflect_sources: Vec<(u32, u32, String, u32)> = Vec::new();

        {
            let (actor_team, enemy_team) =
                Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
            for record in damage_dealt {
                if record.damage > 0 {
                    any_damage_by_source.insert(record.source_id);
                }
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
            let (actor_team, _) = Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
            let Some(source_idx) = actor_team.iter().position(|c| c.id() == *source_id) else {
                continue;
            };
            if !actor_team[source_idx].is_alive() {
                continue;
            }
            let reflect = actor_team[source_idx].take_hit(*reflect);
            self.log.push(BattleEvent::DamageReflect {
                tick_count: self.step,
                reflector_id: *reflector_id,
                reflector_name: reflector_name.clone(),
                target_id: *source_id,
                target_name: actor_team[source_idx].base_name().to_string(),
                damage: reflect,
                target_hp_remaining: actor_team[source_idx].current_hp(),
            });
            if !actor_team[source_idx].is_alive() {
                self.resolve_character_death(source_idx, is_team_a);
            }
        }

        for source_id in any_damage_by_source {
            let (actor_team, _) = Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
            if let Some(source_idx) = actor_team.iter().position(|c| c.id() == source_id && c.is_alive())
            {
                self.try_fire_passive(source_idx, &PassiveTrigger::OnDealDamage, is_team_a);
            }
        }

        for eidx in damaged_enemy_indices {
            let (_, enemy_team) = Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
            if enemy_team[eidx].is_alive() {
                self.try_fire_passive(eidx, &PassiveTrigger::OnTakeDamage, !is_team_a);
            }
        }

        for (source_id, eidx) in kills_by_source {
            let (actor_team, _) = Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
            if let Some(source_idx) = actor_team.iter().position(|c| c.id() == source_id && c.is_alive())
            {
                self.try_fire_passive(source_idx, &PassiveTrigger::OnKill, is_team_a);
            }
            self.resolve_character_death(eidx, !is_team_a);
        }

        for eidx in defeated_enemy_indices {
            self.resolve_character_death(eidx, !is_team_a);
        }
    }

    /// Tick statuses on a character and log any DoT/HoT results.
    fn tick_and_log_statuses(&mut self, idx: usize, is_team_a: bool) {
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
        // Check if actor died from status damage
        if !team[idx].is_alive() {
            self.resolve_character_death(idx, is_team_a);
        }
    }

    /// Run death-side effects for a character exactly when they reach 0 HP.
    fn resolve_character_death(&mut self, char_idx: usize, is_team_a: bool) {
        let (team, _) = Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
        if team[char_idx].is_alive() || team[char_idx].is_defeat_resolved() {
            return;
        }

        self.try_fire_passive(char_idx, &PassiveTrigger::OnDeath, is_team_a);

        let (team, _) = Self::teams_mut(&mut self.team_a, &mut self.team_b, is_team_a);
        team[char_idx].mark_defeat_resolved();
        self.log.push(BattleEvent::Defeat {
            tick_count: self.step,
            character_id: team[char_idx].id(),
            character_name: team[char_idx].base_name().to_string(),
        });
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
    use crate::abilities::{AbilityDef, PassiveMap, Primitive, SimpleAbilityTarget};
    use crate::logger::BattleEvent;
    use crate::models::{
        Comparator, Condition, ConditionSubject, Position, QueryValue, Rule, Stat,
    };
    use crate::statuses::{StackType, StatusBehavior, StatusDef, StatusMap};
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

    fn ward_statuses() -> StatusMap {
        let mut statuses = HashMap::new();
        statuses.insert(
            "Ward".to_string(),
            StatusDef {
                behavior: StatusBehavior::Ward,
                stack_type: StackType::Permanent,
                opposes: None,
            },
        );
        statuses
    }

    fn make_config(name: &str, row: u8, stats: Vec<(Stat, u32)>) -> CharacterConfig {
        make_config_at(name, row, 0, stats)
    }

    fn make_config_at(name: &str, row: u8, col: u8, stats: Vec<(Stat, u32)>) -> CharacterConfig {
        CharacterConfig {
            id: None,
            base_name: name.to_string(),
            display_name: None,
            passive: String::new(),
            actives: Vec::new(),
            item: None,
            position: Position { row, col },
            stats: stats.into_iter().collect(),
            rules: Vec::new(),
        }
    }

    fn warrior() -> CharacterConfig {
        make_config(
            "Warrior",
            0,
            vec![
                (Stat::CON, 12),
                (Stat::STR, 15),
                (Stat::INT, 4),
                (Stat::FOR, 10),
                (Stat::WIS, 5),
                (Stat::DEX, 8),
                (Stat::SPI, 6),
            ],
        )
    }

    fn mage() -> CharacterConfig {
        make_config(
            "Mage",
            0,
            vec![
                (Stat::CON, 8),
                (Stat::STR, 4),
                (Stat::INT, 16),
                (Stat::FOR, 5),
                (Stat::WIS, 12),
                (Stat::DEX, 10),
                (Stat::SPI, 10),
            ],
        )
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
                (Stat::CON, 50),
                (Stat::STR, 8),
                (Stat::INT, 4),
                (Stat::FOR, 10),
                (Stat::WIS, 10),
                (Stat::DEX, 5),
                (Stat::SPI, 5),
            ],
        );
        let glass = make_config(
            "Glass",
            0,
            vec![
                (Stat::CON, 3),
                (Stat::STR, 20),
                (Stat::INT, 4),
                (Stat::FOR, 2),
                (Stat::WIS, 2),
                (Stat::DEX, 5),
                (Stat::SPI, 5),
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
                (Stat::CON, 200),
                (Stat::STR, 1),
                (Stat::INT, 1),
                (Stat::FOR, 50),
                (Stat::WIS, 50),
                (Stat::DEX, 30),
                (Stat::SPI, 5),
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
                (Stat::CON, 10),
                (Stat::STR, 8),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );
        let front2 = make_config(
            "Front2",
            0,
            vec![
                (Stat::CON, 8),
                (Stat::STR, 6),
                (Stat::INT, 4),
                (Stat::FOR, 4),
                (Stat::WIS, 4),
                (Stat::DEX, 6),
                (Stat::SPI, 5),
            ],
        );
        let back = make_config(
            "Back",
            1,
            vec![
                (Stat::CON, 6),
                (Stat::STR, 3),
                (Stat::INT, 10),
                (Stat::FOR, 2),
                (Stat::WIS, 8),
                (Stat::DEX, 4),
                (Stat::SPI, 7),
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
                (Stat::CON, 15),
                (Stat::STR, 6),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );
        let squishy_back = make_config(
            "SquishyBack",
            1,
            vec![
                (Stat::CON, 3),
                (Stat::STR, 3),
                (Stat::INT, 10),
                (Stat::FOR, 1),
                (Stat::WIS, 8),
                (Stat::DEX, 4),
                (Stat::SPI, 7),
            ],
        );
        let attacker = make_config(
            "Attacker",
            0,
            vec![
                (Stat::CON, 12),
                (Stat::STR, 10),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 6),
                (Stat::SPI, 4),
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
                (Stat::CON, 10),
                (Stat::STR, 8),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );
        let lone = make_config(
            "Lone",
            0,
            vec![
                (Stat::CON, 8),
                (Stat::STR, 6),
                (Stat::INT, 3),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
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
                (Stat::CON, 15),
                (Stat::STR, 5),
                (Stat::INT, 3),
                (Stat::FOR, 8),
                (Stat::WIS, 5),
                (Stat::DEX, 4),
                (Stat::SPI, 4),
            ],
        );
        let dps = make_config(
            "DPS",
            1,
            vec![
                (Stat::CON, 6),
                (Stat::STR, 12),
                (Stat::INT, 3),
                (Stat::FOR, 3),
                (Stat::WIS, 2),
                (Stat::DEX, 6),
                (Stat::SPI, 4),
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
                BattleEvent::BasicAttack { actor_id, .. } => Some(*actor_id),
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
                (Stat::CON, 8),
                (Stat::STR, 8),
                (Stat::INT, 3),
                (Stat::FOR, 4),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );
        let back = make_config(
            "Back",
            1,
            vec![
                (Stat::CON, 6),
                (Stat::STR, 3),
                (Stat::INT, 8),
                (Stat::FOR, 2),
                (Stat::WIS, 6),
                (Stat::DEX, 5),
                (Stat::SPI, 6),
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
            (Stat::CON, 10),
            (Stat::STR, 6),
            (Stat::INT, 4),
            (Stat::FOR, 4),
            (Stat::WIS, 3),
            (Stat::DEX, 5),
            (Stat::SPI, 5),
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
                (Stat::CON, 10),
                (Stat::STR, 4),
                (Stat::INT, 4),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 10),
                (Stat::SPI, 4),
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
                (Stat::CON, 10),
                (Stat::STR, 4),
                (Stat::INT, 4),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 1),
                (Stat::SPI, 5),
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

        assert_eq!(battle.team_a[0].current_mp(), 4);
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
        let mut map = HashMap::new();
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
                (Stat::CON, 10),
                (Stat::STR, 6),
                (Stat::INT, 4),
                (Stat::FOR, 3),
                (Stat::WIS, 2),
                (Stat::DEX, 4),
                (Stat::SPI, 5),
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
                (Stat::CON, 15),
                (Stat::STR, 4),
                (Stat::INT, 3),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 4),
                (Stat::SPI, 4),
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
    fn emperor_falls_back_to_basic_attack_when_spi_exhausted() {
        // Emperor with only 2 SPI — can Crush once, then basic attacks
        let mut emperor = emperor_config();
        emperor.stats.insert(Stat::SPI, 2);

        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::CON, 30),
                (Stat::STR, 4),
                (Stat::INT, 3),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 4),
                (Stat::SPI, 4),
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
        let basic_count = events
            .iter()
            .filter(|e| {
                matches!(e,
                    BattleEvent::BasicAttack { actor_id, .. } if *actor_id == 0
                )
            })
            .count();

        assert!(crush_count >= 1, "Should use Crush at least once");
        assert!(
            basic_count >= 1,
            "Should fall back to basic attack when SPI runs out"
        );
    }

    #[test]
    fn characters_without_rules_only_basic_attack() {
        // Two characters with no rules — should only produce BasicAttack events
        let a = make_config(
            "A",
            0,
            vec![
                (Stat::CON, 10),
                (Stat::STR, 6),
                (Stat::INT, 4),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 5),
            ],
        );
        let b = make_config(
            "B",
            0,
            vec![
                (Stat::CON, 10),
                (Stat::STR, 6),
                (Stat::INT, 4),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 5),
            ],
        );

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
        assert!(
            !has_ability,
            "Characters without rules should not use abilities"
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
                (Stat::CON, 20),
                (Stat::STR, 6),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 5),
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
                (Stat::CON, 30),
                (Stat::STR, 4),
                (Stat::INT, 3),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
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
                (Stat::CON, 50),
                (Stat::STR, 6),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 3),
                (Stat::SPI, 5),
            ],
        );
        let b = make_config(
            "B",
            0,
            vec![
                (Stat::CON, 50),
                (Stat::STR, 6),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 3),
                (Stat::SPI, 5),
            ],
        );

        let stun_def = StatusDef {
            behavior: StatusBehavior::SkipTurn,
            stack_type: StackType::NoStack,
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
    fn stunned_turn_still_ticks_statuses_and_regens_mp() {
        use crate::statuses::{StackType, StatusBehavior, StatusDef};

        let actor = make_config(
            "Actor",
            0,
            vec![
                (Stat::CON, 20),
                (Stat::STR, 6),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 10),
                (Stat::SPI, 4),
            ],
        );
        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::CON, 20),
                (Stat::STR, 4),
                (Stat::INT, 3),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 1),
                (Stat::SPI, 4),
            ],
        );

        let stun_def = StatusDef {
            behavior: StatusBehavior::SkipTurn,
            stack_type: StackType::NoStack,
            opposes: None,
        };
        let bleed_def = StatusDef {
            behavior: StatusBehavior::DamagePerStack { value: 1 },
            stack_type: StackType::TickDown,
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

        assert_eq!(battle.team_a[0].current_hp(), 39);
        assert_eq!(battle.team_a[0].current_mp(), 2);
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
                (Stat::CON, 20),
                (Stat::STR, 6),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 10),
                (Stat::SPI, 4),
            ],
        );
        actor.passive = "Meditation".to_string();

        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::CON, 20),
                (Stat::STR, 4),
                (Stat::INT, 3),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 1),
                (Stat::SPI, 4),
            ],
        );

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Meditation".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnTurnStart,
                primitives: vec![Primitive::RestoreMp {
                    target: SimpleAbilityTarget::SelfChar.into(),
                    amount: 1,
                }],
            },
        );

        let bleed_def = StatusDef {
            behavior: StatusBehavior::DamagePerStack { value: 1 },
            stack_type: StackType::TickDown,
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
                (Stat::CON, 10),
                (Stat::STR, 6),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 5),
            ],
        );
        char_config.passive = "TestPassive".to_string();

        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::CON, 10),
                (Stat::STR, 4),
                (Stat::INT, 3),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "TestPassive".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnBattleStart,
                primitives: vec![Primitive::ApplyStatus {
                    target: SimpleAbilityTarget::SelfChar.into(),
                    status: "Empower".to_string(),
                    stat: Some(Stat::STR),
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
                (Stat::CON, 10),
                (Stat::STR, 6),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 5),
            ],
        );
        warrior.passive = "PowerUp".to_string();

        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::CON, 10),
                (Stat::STR, 4),
                (Stat::INT, 3),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "PowerUp".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnBattleStart,
                primitives: vec![Primitive::ApplyStatus {
                    target: SimpleAbilityTarget::SelfChar.into(),
                    status: "Empower".to_string(),
                    stat: Some(Stat::STR),
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
                opposes: None,
            },
        );

        // With huge STR buff, warrior should win easily
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
                (Stat::CON, 10),
                (Stat::STR, 6),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 5),
            ],
        );
        char_config.passive = "NonexistentPassive".to_string();

        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::CON, 10),
                (Stat::STR, 4),
                (Stat::INT, 3),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
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
                (Stat::CON, 10),
                (Stat::STR, 6),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 5),
            ],
        );
        char_config.passive = "Thorns".to_string();

        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::CON, 10),
                (Stat::STR, 4),
                (Stat::INT, 3),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
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
                (Stat::CON, 20),
                (Stat::STR, 6),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );

        let mut defender = make_config(
            "Defender",
            0,
            vec![
                (Stat::CON, 20),
                (Stat::STR, 4),
                (Stat::INT, 3),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
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
                (Stat::CON, 2),
                (Stat::STR, 6),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );

        let mut defender = make_config(
            "Defender",
            0,
            vec![
                (Stat::CON, 50),
                (Stat::STR, 4),
                (Stat::INT, 3),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
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
                (Stat::CON, 2),
                (Stat::STR, 6),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );
        attacker.passive = "Collapse".to_string();

        let mut defender = make_config(
            "Defender",
            0,
            vec![
                (Stat::CON, 50),
                (Stat::STR, 4),
                (Stat::INT, 3),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );
        defender.passive = "Thorns".to_string();

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Collapse".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnDeath,
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
                (Stat::CON, 10),
                (Stat::STR, 6),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );
        attacker.passive = "Barrier".to_string();

        let mut defender = make_config(
            "Defender",
            0,
            vec![
                (Stat::CON, 20),
                (Stat::STR, 4),
                (Stat::INT, 3),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );
        defender.passive = "Thorns".to_string();

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Barrier".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnBattleStart,
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

        assert_eq!(reflect_event, (0, 20));
    }

    #[test]
    fn spi_cost_reduction_in_battle() {
        use crate::abilities::PassiveDef;
        use crate::models::TraitEffect;

        // Emperor with SPI cost reduction — should be able to use Crush more
        let mut emperor = emperor_config();
        emperor.passive = "Thrift".to_string();
        emperor.stats.insert(Stat::SPI, 3); // barely enough for Crush(2) without reduction

        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::CON, 30),
                (Stat::STR, 4),
                (Stat::INT, 3),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 4),
                (Stat::SPI, 4),
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
                (Stat::CON, 20),
                (Stat::STR, 6),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 5),
            ],
        );
        char_config.passive = "Meditation".to_string();

        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::CON, 20),
                (Stat::STR, 4),
                (Stat::INT, 3),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Meditation".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnTurnStart,
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
                (Stat::CON, 20),
                (Stat::STR, 8),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );
        attacker.passive = "Momentum".to_string();

        let enemy = make_config(
            "Enemy",
            0,
            vec![
                (Stat::CON, 30),
                (Stat::STR, 4),
                (Stat::INT, 3),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Momentum".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnDealDamage,
                primitives: vec![Primitive::ApplyStatus {
                    target: SimpleAbilityTarget::SelfChar.into(),
                    status: "Empower".to_string(),
                    stat: Some(Stat::STR),
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
    fn on_take_damage_passive_fires() {
        use crate::abilities::{PassiveDef, PassiveTrigger};
        use crate::statuses::{StackType, StatusBehavior, StatusDef};

        let attacker = make_config(
            "Attacker",
            0,
            vec![
                (Stat::CON, 20),
                (Stat::STR, 8),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );

        let mut defender = make_config(
            "Defender",
            0,
            vec![
                (Stat::CON, 30),
                (Stat::STR, 4),
                (Stat::INT, 3),
                (Stat::FOR, 3),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );
        defender.passive = "Vengeance".to_string();

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Vengeance".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnTakeDamage,
                primitives: vec![Primitive::ApplyStatus {
                    target: SimpleAbilityTarget::SelfChar.into(),
                    status: "Empower".to_string(),
                    stat: Some(Stat::STR),
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
                (Stat::CON, 20),
                (Stat::STR, 15),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );
        killer.passive = "Reaper".to_string();

        let weak_enemy = make_config(
            "Weak",
            0,
            vec![
                (Stat::CON, 3),
                (Stat::STR, 4),
                (Stat::INT, 3),
                (Stat::FOR, 2),
                (Stat::WIS, 2),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Reaper".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnKill,
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
                (Stat::CON, 30),
                (Stat::STR, 15),
                (Stat::INT, 3),
                (Stat::FOR, 10),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );

        let mut dying = make_config(
            "Dying",
            0,
            vec![
                (Stat::CON, 3),
                (Stat::STR, 4),
                (Stat::INT, 3),
                (Stat::FOR, 2),
                (Stat::WIS, 2),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );
        dying.passive = "Collapse".to_string();

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Collapse".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnDeath,
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
                (Stat::CON, 30),
                (Stat::STR, 15),
                (Stat::INT, 3),
                (Stat::FOR, 10),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );

        let mut dying = make_config(
            "Dying",
            0,
            vec![
                (Stat::CON, 2),
                (Stat::STR, 4),
                (Stat::INT, 3),
                (Stat::FOR, 2),
                (Stat::WIS, 2),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );
        dying.passive = "Collapse".to_string();

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Collapse".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnDeath,
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
                (Stat::CON, 20),
                (Stat::STR, 6),
                (Stat::INT, 3),
                (Stat::FOR, 4),
                (Stat::WIS, 3),
                (Stat::DEX, 4),
                (Stat::SPI, 4),
            ],
        );

        let mut dying = make_config(
            "Dying",
            0,
            vec![
                (Stat::CON, 2),
                (Stat::STR, 4),
                (Stat::INT, 3),
                (Stat::FOR, 2),
                (Stat::WIS, 2),
                (Stat::DEX, 4),
                (Stat::SPI, 4),
            ],
        );
        dying.passive = "Collapse".to_string();

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Collapse".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnDeath,
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
    fn passive_reentry_guard_prevents_cascading() {
        use crate::abilities::{PassiveDef, PassiveTrigger};

        // Both sides have on_deal_damage passives that deal damage —
        // should not cascade infinitely
        let mut a = make_config(
            "A",
            0,
            vec![
                (Stat::CON, 20),
                (Stat::STR, 8),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );
        a.passive = "Splash".to_string();

        let mut b = make_config(
            "B",
            0,
            vec![
                (Stat::CON, 20),
                (Stat::STR, 8),
                (Stat::INT, 3),
                (Stat::FOR, 5),
                (Stat::WIS, 3),
                (Stat::DEX, 5),
                (Stat::SPI, 4),
            ],
        );
        b.passive = "Splash".to_string();

        let mut passives: PassiveMap = HashMap::new();
        passives.insert(
            "Splash".to_string(),
            PassiveDef::Triggered {
                trigger: PassiveTrigger::OnDealDamage,
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
