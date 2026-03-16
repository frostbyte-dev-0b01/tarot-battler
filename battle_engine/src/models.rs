//! Core data types for the battle engine: characters, stats, effects, and grid positions.

use std::collections::HashMap;

/// The nine character attributes.
#[derive(Hash, Eq, PartialEq, Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum Stat {
    CON, // Max HP = 2 * CON
    STR, // Base physical damage
    INT, // Base magical damage
    FOR, // Physical resistance
    WIS, // Magical resistance
    DEX, // Determines how often to act
    SPI, // Max Spirit pool and Spirit regen
    FOC, // Debuff chance modifier
    RES, // Debuff resist chance modifier
}

/// Cell on the 4x4 battle grid (rows 0-3, cols 0-3).
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Position {
    pub row: u8,
    pub col: u8,
}

impl Position {
    pub fn is_valid(&self) -> bool {
        self.row < 4 && self.col < 4
    }
}

/// Static character definition loaded from JSON (archetype + loadout).
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CharacterConfig {
    pub base_name: String,
    pub passive: String,
    pub actives: Vec<String>,
    pub item: Option<String>,
    pub position: Position,
    pub stats: HashMap<Stat, u32>,
}

/// Determines how an [`Effect`] modifies a character each turn.
#[derive(Debug, Clone)]
pub enum EffectType {
    StatModifier { stat: Stat, magnitude: i32 },
    DamageOverTime { damage: u32 },
    HealOverTime { amount: u32 },
    Incapacitate,
}

/// A temporary modifier attached to a character. Duration of 0 means permanent until explicitly removed.
#[derive(Debug, Clone)]
pub struct Effect {
    pub name: String,
    pub effect_type: EffectType,
    pub duration: u32,
    pub source_id: u32,
}

/// Mutable runtime state of a character during battle. Created from a [`CharacterConfig`].
pub struct CharacterState {
    id: u32,
    base_name: String,
    base_stats: HashMap<Stat, u32>,
    curr_hp: u32,
    curr_spi: u32,
    spd_counter: u32,
    spd_max: u32,
    target: Option<u32>,
    effects: Vec<Effect>,
}

impl CharacterState {
    pub fn from_config(id: u32, config: &CharacterConfig) -> Self {
        let hp = config.stats.get(&Stat::CON).copied().unwrap_or(0) * 2;
        let spi = config.stats.get(&Stat::SPI).copied().unwrap_or(0);
        let dex = config.stats.get(&Stat::DEX).copied().unwrap_or(0);
        Self {
            id,
            base_name: config.base_name.clone(),
            base_stats: config.stats.clone(),
            curr_hp: hp,
            curr_spi: spi,
            spd_counter: dex,
            spd_max: dex,
            target: None,
            effects: Vec::new(),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn base_name(&self) -> &str {
        &self.base_name
    }

    pub fn get_base_stat(&self, stat: &Stat) -> u32 {
        self.base_stats.get(stat).copied().unwrap_or(0)
    }

    /// Returns the effective stat value (base + sum of StatModifier effects).
    pub fn get_eff_stat(&self, stat: &Stat) -> u32 {
        let base = self.get_base_stat(stat) as i32;
        let modifier: i32 = self
            .effects
            .iter()
            .filter_map(|e| match &e.effect_type {
                EffectType::StatModifier { stat: s, magnitude } if s == stat => Some(magnitude),
                _ => None,
            })
            .sum();
        (base + modifier).max(0) as u32
    }

    pub fn current_hp(&self) -> u32 {
        self.curr_hp
    }

    pub fn is_alive(&self) -> bool {
        self.curr_hp > 0
    }

    pub fn take_damage(&mut self, amount: u32) {
        self.curr_hp = self.curr_hp.saturating_sub(amount);
    }

    /// Heals up to max HP (2 * CON).
    pub fn heal(&mut self, amount: u32) {
        let max_hp = self.get_base_stat(&Stat::CON) * 2;
        self.curr_hp = (self.curr_hp + amount).min(max_hp);
    }

    pub fn current_spi(&self) -> u32 {
        self.curr_spi
    }

    /// Returns false and does nothing if SPI is insufficient.
    pub fn spend_spi(&mut self, cost: u32) -> bool {
        if self.curr_spi >= cost {
            self.curr_spi -= cost;
            true
        } else {
            false
        }
    }

    /// Restores up to base SPI.
    pub fn restore_spi(&mut self, amount: u32) {
        let max_spi = self.get_base_stat(&Stat::SPI);
        self.curr_spi = (self.curr_spi + amount).min(max_spi);
    }

    /// Decrements speed counter. Returns true when the character is ready to act.
    pub fn tick_speed(&mut self) -> bool {
        self.spd_counter = self.spd_counter.saturating_sub(1);
        self.spd_counter == 0
    }

    /// Resets speed counter after acting (DEX + 2 per design).
    pub fn reset_speed(&mut self) {
        self.spd_counter = self.spd_max + 2;
    }

    pub fn target(&self) -> Option<u32> {
        self.target
    }

    pub fn set_target(&mut self, target_id: u32) {
        self.target = Some(target_id);
    }

    pub fn clear_target(&mut self) {
        self.target = None;
    }

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    /// Rejects StatModifier effects targeting pool stats (CON, DEX, SPI).
    pub fn add_effect(&mut self, effect: Effect) -> bool {
        if let EffectType::StatModifier { ref stat, .. } = effect.effect_type {
            if matches!(stat, Stat::CON | Stat::DEX | Stat::SPI) {
                return false;
            }
        }
        self.effects.push(effect);
        true
    }

    pub fn remove_effects_by_source(&mut self, source_id: u32) {
        self.effects.retain(|e| e.source_id != source_id);
    }

    /// Decrements effect durations and removes expired ones.
    pub fn tick_effects(&mut self) {
        for effect in &mut self.effects {
            effect.duration = effect.duration.saturating_sub(1);
        }
        self.effects.retain(|e| e.duration > 0);
    }
}
