//! Ability definitions and execution.

use std::collections::HashMap;

use rand::rngs::StdRng;

use crate::logger::{BattleEvent, BattleLog};
use crate::models::{CharacterState, Effect, EffectType, Stat};

/// Who the ability primitive targets.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AbilityTarget {
    CurrentTarget,
    #[serde(rename = "self")]
    SelfChar,
    Companions,
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
    ApplyBuff {
        target: AbilityTarget,
        stat: Stat,
        magnitude: i32,
        duration: u32,
    },
    ApplyDebuff {
        target: AbilityTarget,
        stat: Stat,
        magnitude: i32,
        duration: u32,
    },
}

/// A complete ability definition.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AbilityDef {
    pub spi_cost: u32,
    pub primitives: Vec<Primitive>,
}

pub type AbilityMap = HashMap<String, AbilityDef>;

/// Execute an ability's primitives.
///
/// `actor_idx` indexes into `actor_team`. Damage and debuffs always target `enemy_team`.
/// Heals, restores, and buffs always target `actor_team`.
///
/// Returns a list of (target_id, damage) pairs for defeat checking by the caller.
pub fn execute_ability(
    actor_idx: usize,
    ability_name: &str,
    ability: &AbilityDef,
    actor_team: &mut [CharacterState],
    enemy_team: &mut [CharacterState],
    _rng: &mut StdRng,
    log: &mut BattleLog,
    step: u32,
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

    let mut damage_dealt: Vec<(u32, u32)> = Vec::new();

    // Pre-compute actor offensive stats for damage calculation
    let actor_str = actor_team[actor_idx].get_eff_stat(&Stat::STR);
    let actor_int = actor_team[actor_idx].get_eff_stat(&Stat::INT);

    for primitive in &ability.primitives {
        match primitive {
            Primitive::DealPhysicalDamage { target, multiplier } => {
                // Damage always hits enemy_team
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
                // Damage always hits enemy_team
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
                // Heals always target actor_team
                let target_indices = resolve_ally_targets(target, actor_idx, actor_team);
                for tidx in target_indices {
                    if actor_team[tidx].is_alive() {
                        actor_team[tidx].heal(*amount);
                    }
                }
            }
            Primitive::RestoreSpi { target, amount } => {
                // Restores always target actor_team
                let target_indices = resolve_ally_targets(target, actor_idx, actor_team);
                for tidx in target_indices {
                    if actor_team[tidx].is_alive() {
                        actor_team[tidx].restore_spi(*amount);
                    }
                }
            }
            Primitive::ApplyBuff {
                target,
                stat,
                magnitude,
                duration,
            } => {
                // Buffs always target actor_team
                let target_indices = resolve_ally_targets(target, actor_idx, actor_team);
                for tidx in target_indices {
                    if actor_team[tidx].is_alive() {
                        actor_team[tidx].add_effect(Effect {
                            name: ability_name.to_string(),
                            effect_type: EffectType::StatModifier {
                                stat: stat.clone(),
                                magnitude: magnitude.abs(),
                            },
                            duration: *duration,
                            source_id: actor_id,
                        });
                    }
                }
            }
            Primitive::ApplyDebuff {
                target,
                stat,
                magnitude,
                duration,
            } => {
                // Debuffs always target enemy_team
                let target_indices = resolve_enemy_targets(target, actor_idx, actor_team, enemy_team);
                for tidx in target_indices {
                    if enemy_team[tidx].is_alive() {
                        enemy_team[tidx].add_effect(Effect {
                            name: ability_name.to_string(),
                            effect_type: EffectType::StatModifier {
                                stat: stat.clone(),
                                magnitude: -(magnitude.abs()),
                            },
                            duration: *duration,
                            source_id: actor_id,
                        });
                    }
                }
            }
        }
    }

    damage_dealt
}

/// Resolve an AbilityTarget to indices into enemy_team.
/// `CurrentTarget` resolves the actor's current target. `SelfChar`/`Companions`
/// are not valid for enemy-targeting primitives and return empty.
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
        // SelfChar/Companions don't make sense for enemy-targeting primitives
        AbilityTarget::SelfChar | AbilityTarget::Companions => Vec::new(),
    }
}

/// Resolve an AbilityTarget to indices into actor_team.
/// `SelfChar` returns the actor. `Companions` returns adjacent allies.
/// `CurrentTarget` is not valid for ally-targeting primitives and returns empty.
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
        // CurrentTarget doesn't make sense for ally-targeting primitives
        AbilityTarget::CurrentTarget => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CharacterConfig, Position};
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

    #[test]
    fn deal_physical_damage_with_multiplier() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut log = BattleLog::new();
        // Attacker: STR=10, Defender: FOR=4 → base damage = 6, * 1.5 = 9
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

        let dealt = execute_ability(0, "Crush", &ability, &mut actor_team, &mut enemy_team, &mut rng, &mut log, 1);
        assert_eq!(dealt.len(), 1);
        assert_eq!(dealt[0].0, 1); // target id
        assert_eq!(dealt[0].1, 9); // 6 * 1.5 = 9
        assert_eq!(enemy_team[0].current_hp(), 40 - 9); // CON=20 → HP=40
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
        // Set companions: char 0 is adjacent to char 1
        actor_team[0].set_companions(vec![1]);

        // Spend SPI on companion so we can see restore
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
        execute_ability(0, "Embolden", &ability, &mut actor_team, &mut enemy_team, &mut rng, &mut log, 1);
        assert_eq!(actor_team[1].current_spi(), 3); // 1 + 2
        // Char 2 is NOT a companion, should be unchanged
        assert_eq!(actor_team[2].current_spi(), 5);
    }
}
