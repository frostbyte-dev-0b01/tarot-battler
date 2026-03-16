use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Debug, Clone, serde::Deserialize)]
pub enum Stat {
    CON,
    STR,
    INT,
    FOR,
    WIS,
    DEX,
    SPI,
    FOC,
    RES,
}

#[derive(serde::Deserialize)]
pub struct Position {
    pub row: u8,
    pub col: u8,
}

#[derive(serde::Deserialize)]
pub struct CharacterConfig {
    pub character_id: String,
    pub passive: String,
    pub actives: Vec<String>,
    pub item: String,
    pub position: Position,
    // rules: ??? Maybe leave these out for now
    pub stats: HashMap<Stat, u32>,
}

pub struct CharacterState {
    base_stats: HashMap<Stat, u32>,
    curr_hp: u32,
    curr_spi: u32,
    spd_counter: u32,
    spd_max: u32,
}

impl CharacterState {
    pub fn get_stat(&self, stat: &Stat) -> u32 {
        self.base_stats.get(stat).copied().unwrap_or(0)
    }

    pub fn current_hp(&self) -> u32 {
        self.curr_hp
    }
}
