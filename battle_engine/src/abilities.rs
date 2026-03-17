//! Ability definitions and execution.

use std::collections::HashMap;

use rand::rngs::StdRng;

use crate::logger::{BattleEvent, BattleLog};
use crate::models::{CharacterState, Stat};
use crate::statuses::{status_key, StatusMap};

/// Who the ability primitive targets.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AbilityTarget {
    CurrentTarget,
    #[serde(rename = "self")]
    SelfChar,
    Companions,
    AllEnemies,
    AllAllies,
}

/// A single primitive effect composing an ability.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Primitive {
    DealPhysicalDamage {
        target: AbilityTarget,
        multiplier: f64,
    },
    DealMagicalDamage {
        target: AbilityTarget,
        multiplier: f64,
    },
    RestoreHp {
        target: AbilityTarget,
        amount: u32,
    },
    RestoreSpi {
        target: AbilityTarget,
        amount: u32,
    },
    ApplyStatus {
        target: AbilityTarget,
        status: String,
        #[serde(default)]
        stat: Option<Stat>,
        stacks: u32,
    },
    RemoveStatus {
        target: AbilityTarget,
        status: String,
        #[serde(default)]
        stat: Option<Stat>,
        stacks: u32,
    },
}

/// A complete ability definition.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AbilityDef {
    pub spi_cost: u32,
    pub primitives: Vec<Primitive>,
}

pub type AbilityMap = HashMap<String, AbilityDef>;

/// When a triggered passive fires.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PassiveTrigger {
    OnBattleStart,
    OnDeath,
    OnKill,
    OnDealDamage,
    OnTakeDamage,
    OnTurnStart,
}

/// A passive ability definition — either a triggered effect or a permanent trait.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PassiveDef {
    Triggered {
        trigger: PassiveTrigger,
        primitives: Vec<Primitive>,
    },
    Trait {
        effect: crate::models::TraitEffect,
    },
}

pub type PassiveMap = HashMap<String, PassiveDef>;

/// Execute an ability's primitives.
///
/// Returns a list of (target_id, damage) pairs for defeat checking by the caller.
pub fn execute_ability(
    actor_idx: usize,
    ability_name: &str,
    ability: &AbilityDef,
    actor_team: &mut [CharacterState],
    enemy_team: &mut [CharacterState],
    rng: &mut StdRng,
    log: &mut BattleLog,
    step: u32,
    status_defs: &StatusMap,
) -> Vec<(u32, u32)> {
    let actor_id = actor_team[actor_idx].id();
    let actor_name = actor_team[actor_idx].base_name().to_string();

    log.push(BattleEvent::AbilityUsed {
        step,
        actor_id,
        actor_name,
        ability_name: ability_name.to_string(),
        spi_cost: ability.spi_cost,
    });

    execute_primitives(
        actor_idx, ability_name, &ability.primitives,
        actor_team, enemy_team, rng, log, step, status_defs,
    )
}

/// Execute a list of primitives (shared by abilities and passives).
///
/// Returns a list of (target_id, damage) pairs for defeat checking.
pub fn execute_primitives(
    actor_idx: usize,
    source_name: &str,
    primitives: &[Primitive],
    actor_team: &mut [CharacterState],
    enemy_team: &mut [CharacterState],
    _rng: &mut StdRng,
    log: &mut BattleLog,
    step: u32,
    status_defs: &StatusMap,
) -> Vec<(u32, u32)> {
    let mut damage_dealt: Vec<(u32, u32)> = Vec::new();

    let actor_id = actor_team[actor_idx].id();

    // Pre-compute actor offensive stats for damage calculation
    let actor_str = actor_team[actor_idx].get_eff_stat(&Stat::STR);
    let actor_int = actor_team[actor_idx].get_eff_stat(&Stat::INT);

    for primitive in primitives {
        match primitive {
            Primitive::DealPhysicalDamage { target, multiplier } => {
                let target_indices = resolve_enemy_targets(target, actor_idx, actor_team, enemy_team);
                for tidx in target_indices {
                    let defender_for = enemy_team[tidx].get_eff_stat(&Stat::FOR);
                    let base = (actor_str as i32 - defender_for as i32).max(1) as u32;
                    let damage = ((base as f64 * multiplier).max(1.0)) as u32;
                    enemy_team[tidx].take_damage(damage);
                    let tid = enemy_team[tidx].id();
                    let tname = enemy_team[tidx].base_name().to_string();
                    let hp = enemy_team[tidx].current_hp();
                    log.push(BattleEvent::AbilityDamage {
                        step,
                        actor_id,
                        target_id: tid,
                        target_name: tname,
                        damage,
                        target_hp_remaining: hp,
                    });
                    damage_dealt.push((tid, damage));
                }
            }
            Primitive::DealMagicalDamage { target, multiplier } => {
                let target_indices = resolve_enemy_targets(target, actor_idx, actor_team, enemy_team);
                for tidx in target_indices {
                    let defender_wis = enemy_team[tidx].get_eff_stat(&Stat::WIS);
                    let base = (actor_int as i32 - defender_wis as i32).max(1) as u32;
                    let damage = ((base as f64 * multiplier).max(1.0)) as u32;
                    enemy_team[tidx].take_damage(damage);
                    let tid = enemy_team[tidx].id();
                    let tname = enemy_team[tidx].base_name().to_string();
                    let hp = enemy_team[tidx].current_hp();
                    log.push(BattleEvent::AbilityDamage {
                        step,
                        actor_id,
                        target_id: tid,
                        target_name: tname,
                        damage,
                        target_hp_remaining: hp,
                    });
                    damage_dealt.push((tid, damage));
                }
            }
            Primitive::RestoreHp { target, amount } => {
                let target_indices = resolve_ally_targets(target, actor_idx, actor_team);
                for tidx in target_indices {
                    if actor_team[tidx].is_alive() {
                        actor_team[tidx].heal(*amount);
                    }
                }
            }
            Primitive::RestoreSpi { target, amount } => {
                let target_indices = resolve_ally_targets(target, actor_idx, actor_team);
                for tidx in target_indices {
                    if actor_team[tidx].is_alive() {
                        actor_team[tidx].restore_spi(*amount);
                    }
                }
            }
            Primitive::ApplyStatus {
                target,
                status,
                stat,
                stacks,
            } => {
                if let Some(def) = status_defs.get(status) {
                    let key = status_key(status, stat.as_ref());
                    // Determine targeting based on behavior:
                    // stat-mod with negative magnitude or damage → enemy
                    // stat-mod with positive magnitude or heal/skip → ally
                    let targets_enemy = match &def.behavior {
                        crate::statuses::StatusBehavior::DamagePerStack { .. } => true,
                        crate::statuses::StatusBehavior::StatModPerStack { magnitude } => *magnitude < 0,
                        _ => false,
                    };

                    if targets_enemy {
                        let target_indices = resolve_enemy_targets(target, actor_idx, actor_team, enemy_team);
                        for tidx in target_indices {
                            if enemy_team[tidx].is_alive() {
                                enemy_team[tidx].add_status(&key, *stacks, actor_id, def, stat.clone());
                            }
                        }
                    } else {
                        let target_indices = resolve_ally_targets(target, actor_idx, actor_team);
                        for tidx in target_indices {
                            if actor_team[tidx].is_alive() {
                                actor_team[tidx].add_status(&key, *stacks, actor_id, def, stat.clone());
                            }
                        }
                    }
                }
            }
            Primitive::RemoveStatus {
                target,
                status,
                stat,
                stacks,
            } => {
                let key = status_key(status, stat.as_ref());
                // RemoveStatus can target either team — use ally targets
                let target_indices = resolve_ally_targets(target, actor_idx, actor_team);
                for tidx in target_indices {
                    actor_team[tidx].remove_status(&key, *stacks);
                }
            }
        }
    }

    damage_dealt
}

/// Resolve an AbilityTarget to indices into enemy_team.
fn resolve_enemy_targets(
    target: &AbilityTarget,
    actor_idx: usize,
    actor_team: &[CharacterState],
    enemy_team: &[CharacterState],
) -> Vec<usize> {
    match target {
        AbilityTarget::CurrentTarget => {
            if let Some(target_id) = actor_team[actor_idx].target() {
                if let Some(idx) = enemy_team.iter().position(|c| c.id() == target_id) {
                    return vec![idx];
                }
            }
            Vec::new()
        }
        AbilityTarget::AllEnemies => {
            enemy_team.iter().enumerate()
                .filter(|(_, c)| c.is_alive())
                .map(|(i, _)| i)
                .collect()
        }
        AbilityTarget::SelfChar | AbilityTarget::Companions | AbilityTarget::AllAllies => Vec::new(),
    }
}

/// Resolve an AbilityTarget to indices into actor_team.
fn resolve_ally_targets(
    target: &AbilityTarget,
    actor_idx: usize,
    actor_team: &[CharacterState],
) -> Vec<usize> {
    match target {
        AbilityTarget::SelfChar => vec![actor_idx],
        AbilityTarget::Companions => {
            let comp_ids = actor_team[actor_idx].companions().to_vec();
            comp_ids
                .iter()
                .filter_map(|id| actor_team.iter().position(|c| c.id() == *id))
                .collect()
        }
        AbilityTarget::AllAllies => {
            actor_team.iter().enumerate()
                .filter(|(i, c)| *i != actor_idx && c.is_alive())
                .map(|(i, _)| i)
                .collect()
        }
        AbilityTarget::CurrentTarget | AbilityTarget::AllEnemies => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CharacterConfig, Position};
    use crate::statuses::{StatusDef, StatusBehavior, StackType};
    use rand::SeedableRng;

    fn make_char(id: u32, stats: Vec<(Stat, u32)>) -> CharacterState {
        let config = CharacterConfig {
            base_name: format!("Char{}", id),
            passive: String::new(),
            actives: Vec::new(),
            item: None,
            position: Position { row: 0, col: 0 },
            stats: stats.into_iter().collect(),
            rules: Vec::new(),
        };
        CharacterState::from_config(id, &config)
    }

    fn make_adjacent_char(id: u32, row: u8, col: u8, stats: Vec<(Stat, u32)>) -> CharacterState {
        let config = CharacterConfig {
            base_name: format!("Char{}", id),
            passive: String::new(),
            actives: Vec::new(),
            item: None,
            position: Position { row, col },
            stats: stats.into_iter().collect(),
            rules: Vec::new(),
        };
        CharacterState::from_config(id, &config)
    }

    fn empty_statuses() -> StatusMap {
        HashMap::new()
    }

    fn test_statuses() -> StatusMap {
        let mut map = HashMap::new();
        map.insert("Bleed".to_string(), StatusDef {
            behavior: StatusBehavior::DamagePerStack { value: 1 },
            stack_type: StackType::TickDown,
            opposes: None,
        });
        map.insert("Regen".to_string(), StatusDef {
            behavior: StatusBehavior::HealPerStack { value: 2 },
            stack_type: StackType::TickDown,
            opposes: None,
        });
        map.insert("Empower".to_string(), StatusDef {
            behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
            stack_type: StackType::TickDown,
            opposes: Some("Weaken".to_string()),
        });
        map.insert("Weaken".to_string(), StatusDef {
            behavior: StatusBehavior::StatModPerStack { magnitude: -1 },
            stack_type: StackType::TickDown,
            opposes: Some("Empower".to_string()),
        });
        map
    }

    #[test]
    fn deal_physical_damage_with_multiplier() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let mut actor_team = vec![make_char(0, vec![(Stat::STR, 10), (Stat::CON, 10), (Stat::SPI, 5)])];
        let mut enemy_team = vec![make_char(1, vec![(Stat::FOR, 4), (Stat::CON, 20)])];
        actor_team[0].set_target(1);

        let ability = AbilityDef {
            spi_cost: 2,
            primitives: vec![Primitive::DealPhysicalDamage {
                target: AbilityTarget::CurrentTarget,
                multiplier: 1.5,
            }],
        };

        let dealt = execute_ability(0, "Crush", &ability, &mut actor_team, &mut enemy_team, &mut rng, &mut log, 1, &empty_statuses());
        assert_eq!(dealt.len(), 1);
        assert_eq!(dealt[0].0, 1);
        assert_eq!(dealt[0].1, 9);
        assert_eq!(enemy_team[0].current_hp(), 40 - 9);
    }

    #[test]
    fn restore_spi_to_companions() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();

        let stats = vec![(Stat::CON, 10), (Stat::SPI, 5), (Stat::STR, 5)];
        let mut actor_team = vec![
            make_adjacent_char(0, 0, 0, stats.clone()),
            make_adjacent_char(1, 0, 1, stats.clone()),
            make_adjacent_char(2, 0, 2, stats.clone()),
        ];
        actor_team[0].set_companions(vec![1]);
        actor_team[1].spend_spi(4);
        assert_eq!(actor_team[1].current_spi(), 1);

        let ability = AbilityDef {
            spi_cost: 3,
            primitives: vec![Primitive::RestoreSpi {
                target: AbilityTarget::Companions,
                amount: 2,
            }],
        };

        let mut enemy_team = vec![make_char(10, vec![(Stat::CON, 5)])];
        execute_ability(0, "Embolden", &ability, &mut actor_team, &mut enemy_team, &mut rng, &mut log, 1, &empty_statuses());
        assert_eq!(actor_team[1].current_spi(), 3);
        assert_eq!(actor_team[2].current_spi(), 5);
    }

    #[test]
    fn apply_status_bleed_on_enemy() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let statuses = test_statuses();
        let mut actor_team = vec![make_char(0, vec![(Stat::CON, 10), (Stat::SPI, 5)])];
        let mut enemy_team = vec![make_char(1, vec![(Stat::CON, 10)])];
        actor_team[0].set_target(1);

        let ability = AbilityDef {
            spi_cost: 2,
            primitives: vec![Primitive::ApplyStatus {
                target: AbilityTarget::CurrentTarget,
                status: "Bleed".to_string(),
                stat: None,
                stacks: 3,
            }],
        };

        execute_ability(0, "Slash", &ability, &mut actor_team, &mut enemy_team, &mut rng, &mut log, 1, &statuses);
        assert_eq!(enemy_team[0].status_stacks("Bleed"), 3);
    }

    #[test]
    fn apply_status_empower_on_ally() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let statuses = test_statuses();
        let mut actor_team = vec![
            make_char(0, vec![(Stat::CON, 10), (Stat::SPI, 5), (Stat::STR, 8)]),
        ];
        let mut enemy_team = vec![make_char(10, vec![(Stat::CON, 5)])];

        let ability = AbilityDef {
            spi_cost: 2,
            primitives: vec![Primitive::ApplyStatus {
                target: AbilityTarget::SelfChar,
                status: "Empower".to_string(),
                stat: Some(Stat::STR),
                stacks: 3,
            }],
        };

        execute_ability(0, "Rage", &ability, &mut actor_team, &mut enemy_team, &mut rng, &mut log, 1, &statuses);
        assert_eq!(actor_team[0].get_eff_stat(&Stat::STR), 11); // 8 + 3
    }

    #[test]
    fn apply_status_weaken_on_enemy() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let statuses = test_statuses();
        let mut actor_team = vec![make_char(0, vec![(Stat::CON, 10), (Stat::SPI, 5)])];
        let mut enemy_team = vec![make_char(1, vec![(Stat::CON, 10), (Stat::STR, 10)])];
        actor_team[0].set_target(1);

        let ability = AbilityDef {
            spi_cost: 1,
            primitives: vec![Primitive::ApplyStatus {
                target: AbilityTarget::CurrentTarget,
                status: "Weaken".to_string(),
                stat: Some(Stat::STR),
                stacks: 2,
            }],
        };

        execute_ability(0, "Curse", &ability, &mut actor_team, &mut enemy_team, &mut rng, &mut log, 1, &statuses);
        assert_eq!(enemy_team[0].get_eff_stat(&Stat::STR), 8); // 10 - 2
    }

    #[test]
    fn all_enemies_target_resolves_to_all_living() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let mut actor_team = vec![make_char(0, vec![(Stat::CON, 10), (Stat::SPI, 5), (Stat::STR, 10)])];
        let mut enemy_team = vec![
            make_char(1, vec![(Stat::CON, 10), (Stat::FOR, 3)]),
            make_char(2, vec![(Stat::CON, 10), (Stat::FOR, 3)]),
            make_char(3, vec![(Stat::CON, 10), (Stat::FOR, 3)]),
        ];
        enemy_team[1].take_damage(100);

        let ability = AbilityDef {
            spi_cost: 1,
            primitives: vec![Primitive::DealPhysicalDamage {
                target: AbilityTarget::AllEnemies,
                multiplier: 1.0,
            }],
        };

        let dealt = execute_ability(0, "Sweep", &ability, &mut actor_team, &mut enemy_team, &mut rng, &mut log, 1, &empty_statuses());
        assert_eq!(dealt.len(), 2);
        assert!(dealt.iter().any(|(id, _)| *id == 1));
        assert!(dealt.iter().any(|(id, _)| *id == 3));
    }

    #[test]
    fn all_allies_target_excludes_self() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        let mut actor_team = vec![
            make_char(0, vec![(Stat::CON, 10), (Stat::SPI, 5)]),
            make_char(1, vec![(Stat::CON, 10), (Stat::SPI, 3)]),
            make_char(2, vec![(Stat::CON, 10), (Stat::SPI, 3)]),
        ];
        actor_team[1].spend_spi(2);
        actor_team[2].spend_spi(2);
        let mut enemy_team = vec![make_char(10, vec![(Stat::CON, 5)])];

        let ability = AbilityDef {
            spi_cost: 1,
            primitives: vec![Primitive::RestoreSpi {
                target: AbilityTarget::AllAllies,
                amount: 5,
            }],
        };

        execute_ability(0, "Rally", &ability, &mut actor_team, &mut enemy_team, &mut rng, &mut log, 1, &empty_statuses());
        assert_eq!(actor_team[0].current_spi(), 5);
        assert_eq!(actor_team[1].current_spi(), 3);
        assert_eq!(actor_team[2].current_spi(), 3);
    }
}
