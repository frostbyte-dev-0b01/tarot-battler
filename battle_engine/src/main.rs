mod damage;
mod engine;
mod logger;
mod models;
mod targeting;

use std::collections::HashMap;

use models::{CharacterConfig, Position, Stat};

fn main() {
    // STR-heavy physical attacker
    let warrior = CharacterConfig {
        base_name: "The Emperor".to_string(),
        passive: String::new(),
        actives: Vec::new(),
        item: None,
        position: Position { row: 0, col: 0 },
        stats: HashMap::from([
            (Stat::CON, 12),
            (Stat::STR, 15),
            (Stat::INT, 4),
            (Stat::FOR, 10),
            (Stat::WIS, 5),
            (Stat::DEX, 8),
            (Stat::SPI, 6),
            (Stat::FOC, 5),
            (Stat::RES, 5),
        ]),
    };

    // INT-heavy magical attacker
    let mage = CharacterConfig {
        base_name: "The High Priestess".to_string(),
        passive: String::new(),
        actives: Vec::new(),
        item: None,
        position: Position { row: 0, col: 1 },
        stats: HashMap::from([
            (Stat::CON, 8),
            (Stat::STR, 4),
            (Stat::INT, 16),
            (Stat::FOR, 5),
            (Stat::WIS, 12),
            (Stat::DEX, 10),
            (Stat::SPI, 10),
            (Stat::FOC, 8),
            (Stat::RES, 7),
        ]),
    };

    let battle = engine::BattleState::new(&[warrior], &[mage], 42);
    let log = battle.run();
    println!("{}", log.to_json());
}
