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
#[derive(Clone)]
pub struct CharacterState {
    id: u32,
    base_name: String,
    base_stats: HashMap<Stat, u32>,
    position: Position,
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
            position: config.position.clone(),
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

    pub fn position(&self) -> &Position {
        &self.position
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

    /// Resets speed counter after acting. Each action adds +2 to the reset value
    /// (first reset: DEX+2, second: DEX+4, etc.), softening high-DEX dominance.
    pub fn reset_speed(&mut self) {
        self.spd_max += 2;
        self.spd_counter = self.spd_max;
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
    /// Permanent effects (duration 0) are kept indefinitely.
    pub fn tick_effects(&mut self) {
        self.effects.retain_mut(|e| {
            if e.duration == 0 {
                return true; // permanent, keep
            }
            e.duration -= 1;
            e.duration > 0
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(stats: Vec<(Stat, u32)>) -> CharacterConfig {
        CharacterConfig {
            base_name: "Test".to_string(),
            passive: String::new(),
            actives: Vec::new(),
            item: None,
            position: Position { row: 0, col: 0 },
            stats: stats.into_iter().collect(),
        }
    }

    #[test]
    fn from_config_sets_hp_to_twice_con() {
        let config = make_config(vec![(Stat::CON, 10), (Stat::DEX, 5), (Stat::SPI, 3)]);
        let state = CharacterState::from_config(0, &config);
        assert_eq!(state.current_hp(), 20);
    }

    #[test]
    fn from_config_copies_position() {
        let mut config = make_config(vec![(Stat::CON, 5)]);
        config.position = Position { row: 2, col: 3 };
        let state = CharacterState::from_config(0, &config);
        assert_eq!(state.position().row, 2);
        assert_eq!(state.position().col, 3);
    }

    #[test]
    fn take_damage_saturates_at_zero() {
        let config = make_config(vec![(Stat::CON, 5)]);
        let mut state = CharacterState::from_config(0, &config);
        assert_eq!(state.current_hp(), 10);
        state.take_damage(100);
        assert_eq!(state.current_hp(), 0);
        assert!(!state.is_alive());
    }

    #[test]
    fn heal_caps_at_max_hp() {
        let config = make_config(vec![(Stat::CON, 5)]);
        let mut state = CharacterState::from_config(0, &config);
        state.take_damage(3);
        assert_eq!(state.current_hp(), 7);
        state.heal(100);
        assert_eq!(state.current_hp(), 10);
    }

    #[test]
    fn spend_spi_fails_when_insufficient() {
        let config = make_config(vec![(Stat::SPI, 5)]);
        let mut state = CharacterState::from_config(0, &config);
        assert!(state.spend_spi(3));
        assert_eq!(state.current_spi(), 2);
        assert!(!state.spend_spi(3));
        assert_eq!(state.current_spi(), 2); // unchanged
    }

    #[test]
    fn restore_spi_caps_at_base() {
        let config = make_config(vec![(Stat::SPI, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.spend_spi(8);
        state.restore_spi(100);
        assert_eq!(state.current_spi(), 10);
    }

    #[test]
    fn speed_system_ticks_and_escalates() {
        let config = make_config(vec![(Stat::DEX, 3)]);
        let mut state = CharacterState::from_config(0, &config);
        // DEX=3: ticks 3->2, 2->1, 1->0 (ready)
        assert!(!state.tick_speed());
        assert!(!state.tick_speed());
        assert!(state.tick_speed());
        // After reset: spd_max = 3+2=5, counter=5
        state.reset_speed();
        for _ in 0..4 {
            assert!(!state.tick_speed());
        }
        assert!(state.tick_speed()); // 5th tick
        // Second reset: spd_max = 5+2=7
        state.reset_speed();
        for _ in 0..6 {
            assert!(!state.tick_speed());
        }
        assert!(state.tick_speed()); // 7th tick
    }

    #[test]
    fn effective_stat_includes_modifiers() {
        let config = make_config(vec![(Stat::STR, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        assert_eq!(state.get_eff_stat(&Stat::STR), 10);

        state.add_effect(Effect {
            name: "buff".to_string(),
            effect_type: EffectType::StatModifier { stat: Stat::STR, magnitude: 5 },
            duration: 3,
            source_id: 99,
        });
        assert_eq!(state.get_eff_stat(&Stat::STR), 15);
    }

    #[test]
    fn effective_stat_floors_at_zero() {
        let config = make_config(vec![(Stat::STR, 3)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_effect(Effect {
            name: "debuff".to_string(),
            effect_type: EffectType::StatModifier { stat: Stat::STR, magnitude: -10 },
            duration: 2,
            source_id: 99,
        });
        assert_eq!(state.get_eff_stat(&Stat::STR), 0);
    }

    #[test]
    fn add_effect_rejects_pool_stat_modifiers() {
        let config = make_config(vec![(Stat::CON, 10), (Stat::DEX, 5), (Stat::SPI, 5)]);
        let mut state = CharacterState::from_config(0, &config);

        for stat in [Stat::CON, Stat::DEX, Stat::SPI] {
            let result = state.add_effect(Effect {
                name: "bad".to_string(),
                effect_type: EffectType::StatModifier { stat, magnitude: 5 },
                duration: 1,
                source_id: 99,
            });
            assert!(!result);
        }
        assert!(state.effects().is_empty());
    }

    #[test]
    fn add_effect_allows_non_pool_stat_modifiers() {
        let config = make_config(vec![(Stat::STR, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        let result = state.add_effect(Effect {
            name: "buff".to_string(),
            effect_type: EffectType::StatModifier { stat: Stat::STR, magnitude: 3 },
            duration: 2,
            source_id: 99,
        });
        assert!(result);
        assert_eq!(state.effects().len(), 1);
    }

    #[test]
    fn add_effect_allows_non_stat_modifier_types() {
        let config = make_config(vec![(Stat::CON, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        assert!(state.add_effect(Effect {
            name: "dot".to_string(),
            effect_type: EffectType::DamageOverTime { damage: 5 },
            duration: 3,
            source_id: 99,
        }));
        assert!(state.add_effect(Effect {
            name: "hot".to_string(),
            effect_type: EffectType::HealOverTime { amount: 3 },
            duration: 2,
            source_id: 99,
        }));
        assert!(state.add_effect(Effect {
            name: "stun".to_string(),
            effect_type: EffectType::Incapacitate,
            duration: 1,
            source_id: 99,
        }));
        assert_eq!(state.effects().len(), 3);
    }

    #[test]
    fn tick_effects_preserves_permanent_effects() {
        let config = make_config(vec![(Stat::STR, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_effect(Effect {
            name: "permanent".to_string(),
            effect_type: EffectType::StatModifier { stat: Stat::STR, magnitude: 5 },
            duration: 0, // permanent
            source_id: 99,
        });
        state.tick_effects();
        state.tick_effects();
        assert_eq!(state.effects().len(), 1);
        assert_eq!(state.get_eff_stat(&Stat::STR), 15);
    }

    #[test]
    fn tick_effects_removes_expired() {
        let config = make_config(vec![(Stat::STR, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_effect(Effect {
            name: "short".to_string(),
            effect_type: EffectType::StatModifier { stat: Stat::STR, magnitude: 5 },
            duration: 2,
            source_id: 99,
        });
        state.tick_effects(); // duration 2 -> 1
        assert_eq!(state.effects().len(), 1);
        state.tick_effects(); // duration 1 -> 0, removed
        assert_eq!(state.effects().len(), 0);
        assert_eq!(state.get_eff_stat(&Stat::STR), 10);
    }

    #[test]
    fn remove_effects_by_source() {
        let config = make_config(vec![(Stat::STR, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_effect(Effect {
            name: "a".to_string(),
            effect_type: EffectType::StatModifier { stat: Stat::STR, magnitude: 3 },
            duration: 5,
            source_id: 1,
        });
        state.add_effect(Effect {
            name: "b".to_string(),
            effect_type: EffectType::StatModifier { stat: Stat::FOR, magnitude: 2 },
            duration: 5,
            source_id: 2,
        });
        state.remove_effects_by_source(1);
        assert_eq!(state.effects().len(), 1);
        assert_eq!(state.get_eff_stat(&Stat::STR), 10); // modifier removed
    }

    #[test]
    fn position_validity() {
        assert!(Position { row: 0, col: 0 }.is_valid());
        assert!(Position { row: 3, col: 3 }.is_valid());
        assert!(!Position { row: 4, col: 0 }.is_valid());
        assert!(!Position { row: 0, col: 4 }.is_valid());
    }
}
