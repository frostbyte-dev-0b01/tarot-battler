//! Damage calculation functions.

use rand::rngs::StdRng;

use crate::models::{CharacterState, Stat};
use crate::targeting::{compute_offensive_type, OffensiveType};

/// Physical damage: max(STR - FOR, 1)
pub fn calc_physical_damage(attacker: &CharacterState, defender: &CharacterState) -> u32 {
    let str_val = attacker.get_eff_stat(&Stat::STR) as i32;
    let for_val = defender.get_eff_stat(&Stat::FOR) as i32;
    (str_val - for_val).max(1) as u32
}

/// Magical damage: max(INT - WIS, 1)
pub fn calc_magical_damage(attacker: &CharacterState, defender: &CharacterState) -> u32 {
    let int_val = attacker.get_eff_stat(&Stat::INT) as i32;
    let wis_val = defender.get_eff_stat(&Stat::WIS) as i32;
    (int_val - wis_val).max(1) as u32
}

/// Determines basic attack damage based on attacker's offensive type.
pub fn calc_basic_attack_damage(
    attacker: &CharacterState,
    defender: &CharacterState,
    rng: &mut StdRng,
) -> u32 {
    match compute_offensive_type(attacker, rng) {
        OffensiveType::Physical => calc_physical_damage(attacker, defender),
        OffensiveType::Magical => calc_magical_damage(attacker, defender),
    }
}
