//! Damage calculation functions.

use rand::rngs::StdRng;

use crate::models::{CharacterState, Stat};
use crate::targeting::{OffensiveType, compute_offensive_type};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CharacterConfig, Position};
    use rand::SeedableRng;

    fn make_char(id: u32, stats: Vec<(Stat, u32)>) -> CharacterState {
        let config = CharacterConfig {
            base_name: "Test".to_string(),
            passive: String::new(),
            actives: Vec::new(),
            item: None,
            position: Position { row: 0, col: 0 },
            stats: stats.into_iter().collect(),
            rules: Vec::new(),
        };
        CharacterState::from_config(id, &config)
    }

    #[test]
    fn physical_damage_str_minus_for() {
        let attacker = make_char(0, vec![(Stat::STR, 15)]);
        let defender = make_char(1, vec![(Stat::FOR, 8)]);
        assert_eq!(calc_physical_damage(&attacker, &defender), 7);
    }

    #[test]
    fn physical_damage_minimum_one() {
        let attacker = make_char(0, vec![(Stat::STR, 3)]);
        let defender = make_char(1, vec![(Stat::FOR, 20)]);
        assert_eq!(calc_physical_damage(&attacker, &defender), 1);
    }

    #[test]
    fn magical_damage_int_minus_wis() {
        let attacker = make_char(0, vec![(Stat::INT, 16)]);
        let defender = make_char(1, vec![(Stat::WIS, 5)]);
        assert_eq!(calc_magical_damage(&attacker, &defender), 11);
    }

    #[test]
    fn magical_damage_minimum_one() {
        let attacker = make_char(0, vec![(Stat::INT, 2)]);
        let defender = make_char(1, vec![(Stat::WIS, 15)]);
        assert_eq!(calc_magical_damage(&attacker, &defender), 1);
    }

    #[test]
    fn basic_attack_uses_physical_when_str_higher() {
        let attacker = make_char(0, vec![(Stat::STR, 15), (Stat::INT, 5)]);
        let defender = make_char(1, vec![(Stat::FOR, 8), (Stat::WIS, 3)]);
        let mut rng = StdRng::seed_from_u64(0);
        // STR > INT so physical: 15 - 8 = 7
        assert_eq!(calc_basic_attack_damage(&attacker, &defender, &mut rng), 7);
    }

    #[test]
    fn basic_attack_uses_magical_when_int_higher() {
        let attacker = make_char(0, vec![(Stat::STR, 4), (Stat::INT, 16)]);
        let defender = make_char(1, vec![(Stat::FOR, 8), (Stat::WIS, 5)]);
        let mut rng = StdRng::seed_from_u64(0);
        // INT > STR so magical: 16 - 5 = 11
        assert_eq!(calc_basic_attack_damage(&attacker, &defender, &mut rng), 11);
    }
}
