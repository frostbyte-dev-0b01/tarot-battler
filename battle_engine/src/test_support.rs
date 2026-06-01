//! Shared test fixtures for battle-engine modules.

use std::collections::HashMap;

use crate::abilities::{
    AbilityDef, AbilityMap, PassiveMap, PowerTier, Primitive, SimpleAbilityTarget,
};
use crate::engine::BattleState;
use crate::models::{CharacterConfig, CharacterState, Condition, Position, Rule, Stat};
use crate::statuses::{StackType, StatusBehavior, StatusDef, StatusMap};

pub fn empty_abilities() -> AbilityMap {
    let mut map = HashMap::new();
    map.insert(
        "Strike".to_string(),
        AbilityDef {
            mp_cost: 1,
            primitives: vec![Primitive::DealPhysicalDamage {
                target: SimpleAbilityTarget::CurrentTarget.into(),
                power: PowerTier::Light,
                double_empower_stat: None,
            }],
        },
    );
    map
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
            group: None,
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
            group: None,
            opposes: None,
        },
    );
    map.insert(
        "Regen".to_string(),
        StatusDef {
            behavior: StatusBehavior::HealPerStack { value: 2 },
            stack_type: StackType::TickDown,
            group: None,
            opposes: None,
        },
    );
    map.insert(
        "Omen".to_string(),
        StatusDef {
            behavior: StatusBehavior::DamagePerStack { value: 1 },
            stack_type: StackType::TickDown,
            group: None,
            opposes: None,
        },
    );
    map.insert(
        "Empower".to_string(),
        StatusDef {
            behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
            stack_type: StackType::TickDown,
            group: None,
            opposes: Some("Weaken".to_string()),
        },
    );
    map.insert(
        "Weaken".to_string(),
        StatusDef {
            behavior: StatusBehavior::StatModPerStack { magnitude: -1 },
            stack_type: StackType::TickDown,
            group: None,
            opposes: Some("Empower".to_string()),
        },
    );
    map.insert(
        "Ward".to_string(),
        StatusDef {
            behavior: StatusBehavior::Ward,
            stack_type: StackType::Permanent,
            group: None,
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
        actives: vec!["Strike".to_string()],
        aspect_passive: None,
        aspect: None,
        position: Position { row, col },
        stats: stats.into_iter().collect(),
        rules: vec![Rule {
            ability: "Strike".to_string(),
            conditions: Vec::<Condition>::new(),

            match_any: false,
        }],
    }
}

pub fn make_char(id: u32, stats: Vec<(Stat, u32)>) -> CharacterState {
    with_full_mp(CharacterState::from_config(
        id,
        &make_config_at(&format!("Char{}", id), 0, 0, stats),
    ))
}

/// Characters now start a battle at 0 MP; most unit tests assume MP is available
/// for the behavior under test, so test helpers top characters up to full MP.
fn with_full_mp(mut character: CharacterState) -> CharacterState {
    character.restore_mp(crate::models::MAX_MP);
    character
}

pub fn build_battle(
    team_a: &[CharacterConfig],
    team_b: &[CharacterConfig],
    abilities: AbilityMap,
    passives: PassiveMap,
    statuses: StatusMap,
) -> BattleState {
    BattleState::new(team_a, team_b, abilities, passives, statuses, 42)
}

pub fn make_adjacent_char(id: u32, row: u8, col: u8, stats: Vec<(Stat, u32)>) -> CharacterState {
    with_full_mp(CharacterState::from_config(
        id,
        &make_config_at(&format!("Char{}", id), row, col, stats),
    ))
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
        ],
    )
}
