//! Shared test fixtures for battle-engine modules.

use std::collections::HashMap;

use crate::abilities::{AbilityMap, PassiveMap};
use crate::models::{CharacterConfig, CharacterState, Position, Stat};
use crate::statuses::{StackType, StatusBehavior, StatusDef, StatusMap};

pub fn empty_abilities() -> AbilityMap {
    HashMap::new()
}

pub fn empty_passives() -> PassiveMap {
    HashMap::new()
}

pub fn empty_statuses() -> StatusMap {
    HashMap::new()
}

pub fn ward_statuses() -> StatusMap {
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

pub fn test_statuses() -> StatusMap {
    let mut map = HashMap::new();
    map.insert(
        "Bleed".to_string(),
        StatusDef {
            behavior: StatusBehavior::DamagePerStack { value: 1 },
            stack_type: StackType::TickDown,
            opposes: None,
        },
    );
    map.insert(
        "Regen".to_string(),
        StatusDef {
            behavior: StatusBehavior::HealPerStack { value: 2 },
            stack_type: StackType::TickDown,
            opposes: None,
        },
    );
    map.insert(
        "Omen".to_string(),
        StatusDef {
            behavior: StatusBehavior::DamagePerStack { value: 1 },
            stack_type: StackType::TickDown,
            opposes: None,
        },
    );
    map.insert(
        "Empower".to_string(),
        StatusDef {
            behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
            stack_type: StackType::TickDown,
            opposes: Some("Weaken".to_string()),
        },
    );
    map.insert(
        "Weaken".to_string(),
        StatusDef {
            behavior: StatusBehavior::StatModPerStack { magnitude: -1 },
            stack_type: StackType::TickDown,
            opposes: Some("Empower".to_string()),
        },
    );
    map.insert(
        "Ward".to_string(),
        StatusDef {
            behavior: StatusBehavior::Ward,
            stack_type: StackType::Permanent,
            opposes: None,
        },
    );
    map
}

pub fn make_config(name: &str, row: u8, stats: Vec<(Stat, u32)>) -> CharacterConfig {
    make_config_at(name, row, 0, stats)
}

pub fn make_config_at(name: &str, row: u8, col: u8, stats: Vec<(Stat, u32)>) -> CharacterConfig {
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

pub fn make_char(id: u32, stats: Vec<(Stat, u32)>) -> CharacterState {
    CharacterState::from_config(id, &make_config_at(&format!("Char{}", id), 0, 0, stats))
}

pub fn make_adjacent_char(
    id: u32,
    row: u8,
    col: u8,
    stats: Vec<(Stat, u32)>,
) -> CharacterState {
    CharacterState::from_config(id, &make_config_at(&format!("Char{}", id), row, col, stats))
}

pub fn warrior() -> CharacterConfig {
    make_config(
        "Warrior",
        0,
        vec![
            (Stat::VIT, 12),
            (Stat::MGT, 15),
            (Stat::MAG, 4),
            (Stat::ARM, 10),
            (Stat::RES, 5),
            (Stat::SPD, 8),
            (Stat::WIL, 6),
        ],
    )
}

pub fn mage() -> CharacterConfig {
    make_config(
        "Mage",
        0,
        vec![
            (Stat::VIT, 8),
            (Stat::MGT, 4),
            (Stat::MAG, 16),
            (Stat::ARM, 5),
            (Stat::RES, 12),
            (Stat::SPD, 10),
            (Stat::WIL, 10),
        ],
    )
}
