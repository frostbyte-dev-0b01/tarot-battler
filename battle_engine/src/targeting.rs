//! Target selection based on offensive/defensive type matching and row constraints.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::Rng;

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

/// Compute offensive type from effective STR vs INT. Tie → random.
pub fn compute_offensive_type(character: &CharacterState, rng: &mut StdRng) -> OffensiveType {
    let str_val = character.get_eff_stat(&Stat::STR);
    let int_val = character.get_eff_stat(&Stat::INT);
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

/// Compute defensive type from effective FOR vs WIS. Tie → random.
pub fn compute_defensive_type(character: &CharacterState, rng: &mut StdRng) -> DefensiveType {
    let for_val = character.get_eff_stat(&Stat::FOR);
    let wis_val = character.get_eff_stat(&Stat::WIS);
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
    let preferred_weakness = match off_type {
        OffensiveType::Physical => DefensiveType::Magical,
        OffensiveType::Magical => DefensiveType::Physical,
    };

    let matched: Vec<u32> = front_row_enemies
        .iter()
        .filter(|e| compute_defensive_type(e, rng) == preferred_weakness)
        .map(|e| e.id())
        .collect();

    if !matched.is_empty() {
        Some(*matched.choose(rng).unwrap())
    } else {
        let ids: Vec<u32> = front_row_enemies.iter().map(|e| e.id()).collect();
        Some(*ids.choose(rng).unwrap())
    }
}
