//! Target selection based on offensive/defensive type matching and row constraints.

use rand::Rng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::models::{CharacterState, Stat};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffensiveType {
    Physical,
    Magical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefensiveType {
    Physical,
    Magical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusSelectionMode {
    Preferred,
    Disoriented,
}

/// Compute offensive type from effective MGT vs MAG. Tie → random.
pub fn compute_offensive_type(character: &CharacterState, rng: &mut StdRng) -> OffensiveType {
    let str_val = character.get_eff_stat(&Stat::MGT);
    let int_val = character.get_eff_stat(&Stat::MAG);
    if str_val > int_val {
        OffensiveType::Physical
    } else if int_val > str_val {
        OffensiveType::Magical
    } else if rng.gen_bool(0.5) {
        OffensiveType::Physical
    } else {
        OffensiveType::Magical
    }
}

/// Compute defensive type from effective ARM vs RES. Tie → random.
pub fn compute_defensive_type(character: &CharacterState, rng: &mut StdRng) -> DefensiveType {
    let for_val = character.get_eff_stat(&Stat::ARM);
    let wis_val = character.get_eff_stat(&Stat::RES);
    if for_val > wis_val {
        DefensiveType::Physical
    } else if wis_val > for_val {
        DefensiveType::Magical
    } else if rng.gen_bool(0.5) {
        DefensiveType::Physical
    } else {
        DefensiveType::Magical
    }
}

/// Select a target from the enemy list based on type matching and row constraints.
///
/// 1. Find frontmost occupied row among living enemies
/// 2. Physical attackers prefer magical defenders (weak to physical), and vice versa
/// 3. If no matching weakness in front row, pick randomly from all front-row enemies
pub fn select_target(
    attacker: &CharacterState,
    enemies: &[CharacterState],
    rng: &mut StdRng,
) -> Option<u32> {
    select_target_with_mode(attacker, enemies, rng, FocusSelectionMode::Preferred)
}

pub fn select_disoriented_target(
    attacker: &CharacterState,
    enemies: &[CharacterState],
    rng: &mut StdRng,
) -> Option<u32> {
    select_target_with_mode(attacker, enemies, rng, FocusSelectionMode::Disoriented)
}

fn select_target_with_mode(
    attacker: &CharacterState,
    enemies: &[CharacterState],
    rng: &mut StdRng,
    mode: FocusSelectionMode,
) -> Option<u32> {
    let living: Vec<&CharacterState> = enemies.iter().filter(|e| e.is_alive()).collect();
    if living.is_empty() {
        return None;
    }

    // Find frontmost occupied row
    let front_row = living.iter().map(|e| e.position().row).min().unwrap();
    let front_row_enemies: Vec<&CharacterState> = living
        .into_iter()
        .filter(|e| e.position().row == front_row)
        .collect();

    let off_type = compute_offensive_type(attacker, rng);

    // Physical attackers target magical defenders (weak to physical), and vice versa
    let preferred_defense = match off_type {
        OffensiveType::Physical => DefensiveType::Magical,
        OffensiveType::Magical => DefensiveType::Physical,
    };
    let selected_defense = match mode {
        FocusSelectionMode::Preferred => preferred_defense,
        FocusSelectionMode::Disoriented => match preferred_defense {
            DefensiveType::Physical => DefensiveType::Magical,
            DefensiveType::Magical => DefensiveType::Physical,
        },
    };

    let matched: Vec<u32> = front_row_enemies
        .iter()
        .filter(|e| compute_defensive_type(e, rng) == selected_defense)
        .map(|e| e.id())
        .collect();

    if !matched.is_empty() {
        Some(*matched.choose(rng).unwrap())
    } else {
        let ids: Vec<u32> = front_row_enemies.iter().map(|e| e.id()).collect();
        Some(*ids.choose(rng).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CharacterConfig, Position};
    use rand::SeedableRng;

    fn make_char(id: u32, row: u8, stats: Vec<(Stat, u32)>) -> CharacterState {
        let config = CharacterConfig {
            id: None,
            base_name: format!("Char{}", id),
            display_name: None,
            passive: String::new(),
            actives: Vec::new(),
            aspect_passive: None,
            aspect: None,
            position: Position { row, col: 0 },
            stats: stats.into_iter().collect(),
            rules: Vec::new(),
        };
        CharacterState::from_config(id, &config)
    }

    #[test]
    fn offensive_type_physical_when_str_higher() {
        let c = make_char(0, 0, vec![(Stat::MGT, 15), (Stat::MAG, 5)]);
        let mut rng = StdRng::seed_from_u64(0);
        assert_eq!(
            compute_offensive_type(&c, &mut rng),
            OffensiveType::Physical
        );
    }

    #[test]
    fn offensive_type_magical_when_int_higher() {
        let c = make_char(0, 0, vec![(Stat::MGT, 5), (Stat::MAG, 15)]);
        let mut rng = StdRng::seed_from_u64(0);
        assert_eq!(compute_offensive_type(&c, &mut rng), OffensiveType::Magical);
    }

    #[test]
    fn defensive_type_physical_when_for_higher() {
        let c = make_char(0, 0, vec![(Stat::ARM, 12), (Stat::RES, 5)]);
        let mut rng = StdRng::seed_from_u64(0);
        assert_eq!(
            compute_defensive_type(&c, &mut rng),
            DefensiveType::Physical
        );
    }

    #[test]
    fn defensive_type_magical_when_wis_higher() {
        let c = make_char(0, 0, vec![(Stat::ARM, 5), (Stat::RES, 12)]);
        let mut rng = StdRng::seed_from_u64(0);
        assert_eq!(compute_defensive_type(&c, &mut rng), DefensiveType::Magical);
    }

    #[test]
    fn select_target_returns_none_for_empty_enemies() {
        let attacker = make_char(0, 0, vec![(Stat::MGT, 10), (Stat::MAG, 5)]);
        let mut rng = StdRng::seed_from_u64(0);
        assert_eq!(select_target(&attacker, &[], &mut rng), None);
    }

    #[test]
    fn select_target_returns_none_when_all_dead() {
        let attacker = make_char(0, 0, vec![(Stat::MGT, 10), (Stat::MAG, 5)]);
        let mut enemy = make_char(1, 0, vec![(Stat::VIT, 5)]);
        enemy.take_damage(100);
        let mut rng = StdRng::seed_from_u64(0);
        assert_eq!(select_target(&attacker, &[enemy], &mut rng), None);
    }

    #[test]
    fn select_target_picks_from_frontmost_row() {
        // Physical attacker (MGT > MAG)
        let attacker = make_char(0, 0, vec![(Stat::MGT, 15), (Stat::MAG, 5)]);
        // Enemy in row 1 and row 2 — should only consider row 1
        let front = make_char(
            10,
            1,
            vec![(Stat::VIT, 10), (Stat::ARM, 5), (Stat::RES, 12)],
        );
        let back = make_char(
            11,
            2,
            vec![(Stat::VIT, 10), (Stat::ARM, 5), (Stat::RES, 12)],
        );
        let mut rng = StdRng::seed_from_u64(0);
        let target = select_target(&attacker, &[front, back], &mut rng);
        assert_eq!(target, Some(10)); // must be front row
    }

    #[test]
    fn select_target_skips_dead_in_front_row() {
        let attacker = make_char(0, 0, vec![(Stat::MGT, 15), (Stat::MAG, 5)]);
        let mut front = make_char(10, 0, vec![(Stat::VIT, 5)]);
        front.take_damage(100); // dead
        let back = make_char(11, 1, vec![(Stat::VIT, 10), (Stat::ARM, 5), (Stat::RES, 5)]);
        let mut rng = StdRng::seed_from_u64(0);
        let target = select_target(&attacker, &[front, back], &mut rng);
        assert_eq!(target, Some(11)); // front is dead, targets back
    }

    #[test]
    fn select_target_prefers_weakness_match() {
        // Physical attacker should prefer magical defenders (weak to physical)
        let attacker = make_char(0, 0, vec![(Stat::MGT, 15), (Stat::MAG, 5)]);
        // Physical defender (ARM > RES)
        let phys_def = make_char(
            10,
            0,
            vec![(Stat::VIT, 10), (Stat::ARM, 15), (Stat::RES, 3)],
        );
        // Magical defender (RES > ARM) — weak to physical
        let mag_def = make_char(
            11,
            0,
            vec![(Stat::VIT, 10), (Stat::ARM, 3), (Stat::RES, 15)],
        );

        // Run many times to verify preference (with different seeds)
        let mut targeted_mag = 0;
        for seed in 0..50 {
            let mut rng = StdRng::seed_from_u64(seed);
            let target = select_target(&attacker, &[phys_def.clone(), mag_def.clone()], &mut rng);
            if target == Some(11) {
                targeted_mag += 1;
            }
        }
        // Physical attacker should prefer magical defender most of the time
        assert!(
            targeted_mag > 25,
            "Expected to prefer magical defender, got {}/50",
            targeted_mag
        );
    }

    #[test]
    fn select_disoriented_target_prefers_wrong_defense_profile() {
        let attacker = make_char(0, 0, vec![(Stat::MGT, 15), (Stat::MAG, 5)]);
        let phys_def = make_char(
            10,
            0,
            vec![(Stat::VIT, 10), (Stat::ARM, 15), (Stat::RES, 3)],
        );
        let mag_def = make_char(
            11,
            0,
            vec![(Stat::VIT, 10), (Stat::ARM, 3), (Stat::RES, 15)],
        );

        let mut targeted_phys = 0;
        for seed in 0..50 {
            let mut rng = StdRng::seed_from_u64(seed);
            let target = select_disoriented_target(
                &attacker,
                &[phys_def.clone(), mag_def.clone()],
                &mut rng,
            );
            if target == Some(10) {
                targeted_phys += 1;
            }
        }
        assert!(
            targeted_phys > 25,
            "Expected to prefer physical defender when disoriented, got {}/50",
            targeted_phys
        );
    }
}
