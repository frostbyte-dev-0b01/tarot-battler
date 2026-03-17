//! Core data types for the battle engine: characters, stats, effects, and grid positions.

use std::collections::HashMap;

use crate::statuses::{StackType, StatusBehavior, StatusDef, StatusInstance, opposite_key};

/// The current character attributes.
#[derive(Hash, Eq, PartialEq, Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum Stat {
    CON, // Max HP = 2 * CON
    STR, // Base physical damage
    INT, // Base magical damage
    FOR, // Physical resistance
    WIS, // Magical resistance
    DEX, // Determines how often to act
    SPI, // Spirit stat: max MP and MP regen
}

/// Cell on the battle grid (rows 0-2, cols 0-3).
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Position {
    pub row: u8,
    pub col: u8,
}

impl Position {
    pub fn is_valid(&self) -> bool {
        self.row < 3 && self.col < 4
    }

    /// Cardinal adjacency (up/down/left/right, no diagonals).
    pub fn is_adjacent(&self, other: &Position) -> bool {
        let dr = (self.row as i8 - other.row as i8).abs();
        let dc = (self.col as i8 - other.col as i8).abs();
        (dr == 1 && dc == 0) || (dr == 0 && dc == 1)
    }
}

/// Who the condition checks against.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionSubject {
    #[serde(rename = "self")]
    SelfChar,
    Target,
    Companion,
    Ally,
}

/// What value the condition reads.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryValue {
    Stat(Stat),
    Hp,
    Mp,
    UseCount,
    TurnsSinceUse,
    // TODO: StatusStacks(String) and HasStatus(String) for rule conditions
    // e.g., { "value": { "status_stacks": "Bleed" }, "comparator": "gte", "threshold": 3 }
}

/// Comparison operator for conditions.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Comparator {
    Gte,
    Lte,
}

/// A single condition that must be met for a rule to fire.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Condition {
    pub subject: ConditionSubject,
    pub value: QueryValue,
    pub comparator: Comparator,
    pub threshold: u32,
}

/// An ordered rule: if all conditions are met (AND), use this ability.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Rule {
    pub ability: String,
    pub conditions: Vec<Condition>,
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
    #[serde(default)]
    pub rules: Vec<Rule>,
}

/// A permanent trait that modifies engine behavior. Applied at battle start
/// from a character's passive and stored on CharacterState for the battle's duration.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TraitEffect {
    /// Abilities cost `amount` less MP (minimum 1).
    MpCostReduction { amount: u32 },
    /// First `count` debuffs applied to this character are negated.
    DebuffResistance { count: u32 },
    /// Attackers take `amount` damage when they hit this character.
    DamageReflect { amount: u32 },
}

/// What happened when statuses ticked.
#[derive(Debug, Clone)]
pub enum StatusTick {
    DamageDealt { name: String, damage: u32 },
    HealApplied { name: String, amount: u32 },
}

/// Mutable runtime state of a character during battle. Created from a [`CharacterConfig`].
#[derive(Clone)]
pub struct CharacterState {
    id: u32,
    base_name: String,
    passive: String,
    actives: Vec<String>,
    base_stats: HashMap<Stat, u32>,
    position: Position,
    curr_hp: u32,
    curr_mp: u32,
    spd_counter: u32,
    spd_max: u32,
    target: Option<u32>,
    companions: Vec<u32>,
    statuses: HashMap<String, StatusInstance>,
    traits: Vec<TraitEffect>,
    rules: Vec<Rule>,
    defeat_resolved: bool,
    actor_turn_count: u32,
    ability_use_counts: HashMap<String, u32>,
    ability_last_used_turn: HashMap<String, u32>,
}

impl CharacterState {
    pub fn from_config(id: u32, config: &CharacterConfig) -> Self {
        let hp = config.stats.get(&Stat::CON).copied().unwrap_or(0) * 2;
        let mp = config.stats.get(&Stat::SPI).copied().unwrap_or(0);
        let dex = config.stats.get(&Stat::DEX).copied().unwrap_or(0);
        Self {
            id,
            base_name: config.base_name.clone(),
            passive: config.passive.clone(),
            actives: config.actives.clone(),
            base_stats: config.stats.clone(),
            position: config.position.clone(),
            curr_hp: hp,
            curr_mp: mp,
            spd_counter: dex,
            spd_max: dex,
            target: None,
            companions: Vec::new(),
            statuses: HashMap::new(),
            traits: Vec::new(),
            rules: config.rules.clone(),
            defeat_resolved: false,
            actor_turn_count: 0,
            ability_use_counts: HashMap::new(),
            ability_last_used_turn: HashMap::new(),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn base_name(&self) -> &str {
        &self.base_name
    }

    pub fn passive(&self) -> &str {
        &self.passive
    }

    pub fn has_active(&self, ability_name: &str) -> bool {
        self.actives.iter().any(|active| active == ability_name)
    }

    pub fn is_defeat_resolved(&self) -> bool {
        self.defeat_resolved
    }

    pub fn mark_defeat_resolved(&mut self) {
        self.defeat_resolved = true;
    }

    pub fn is_incapacitated(&self) -> bool {
        self.statuses
            .values()
            .any(|s| matches!(s.behavior, StatusBehavior::SkipTurn))
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn get_base_stat(&self, stat: &Stat) -> u32 {
        self.base_stats.get(stat).copied().unwrap_or(0)
    }

    /// Returns the effective stat value (base + sum of StatModPerStack status effects).
    pub fn get_eff_stat(&self, stat: &Stat) -> u32 {
        let base = self.get_base_stat(stat) as i32;
        let modifier: i32 = self
            .statuses
            .values()
            .filter_map(|s| match &s.behavior {
                StatusBehavior::StatModPerStack { magnitude } if s.stat.as_ref() == Some(stat) => {
                    Some(*magnitude * s.stacks as i32)
                }
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

    pub fn current_mp(&self) -> u32 {
        self.curr_mp
    }

    /// Returns false and does nothing if MP is insufficient.
    pub fn spend_mp(&mut self, cost: u32) -> bool {
        if self.curr_mp >= cost {
            self.curr_mp -= cost;
            true
        } else {
            false
        }
    }

    /// Restores up to base SPI, which determines max MP.
    pub fn restore_mp(&mut self, amount: u32) {
        let max_mp = self.get_base_stat(&Stat::SPI);
        self.curr_mp = (self.curr_mp + amount).min(max_mp);
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

    pub fn companions(&self) -> &[u32] {
        &self.companions
    }

    pub fn set_companions(&mut self, ids: Vec<u32>) {
        self.companions = ids;
    }

    #[cfg(test)]
    pub fn statuses(&self) -> &HashMap<String, StatusInstance> {
        &self.statuses
    }

    #[cfg(test)]
    pub fn has_status(&self, key: &str) -> bool {
        self.statuses.contains_key(key)
    }

    #[cfg(test)]
    pub fn status_stacks(&self, key: &str) -> u32 {
        self.statuses.get(key).map_or(0, |s| s.stacks)
    }

    pub fn add_trait(&mut self, t: TraitEffect) {
        self.traits.push(t);
    }

    pub fn mp_cost_reduction(&self) -> u32 {
        self.traits
            .iter()
            .filter_map(|t| match t {
                TraitEffect::MpCostReduction { amount } => Some(*amount),
                _ => None,
            })
            .sum()
    }

    pub fn damage_reflect_amount(&self) -> u32 {
        self.traits
            .iter()
            .filter_map(|t| match t {
                TraitEffect::DamageReflect { amount } => Some(*amount),
                _ => None,
            })
            .sum()
    }

    /// Try to negate a debuff using DebuffResistance charges. Returns true if negated.
    pub fn try_negate_debuff(&mut self) -> bool {
        for t in &mut self.traits {
            if let TraitEffect::DebuffResistance { count } = t {
                if *count > 0 {
                    *count -= 1;
                    return true;
                }
            }
        }
        false
    }

    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// Returns the value of a query (stat, HP, or MP) for condition evaluation.
    /// UseCount and TurnsSinceUse are not handled here — they require ability context.
    pub fn query_value(&self, qv: &QueryValue) -> u32 {
        match qv {
            QueryValue::Stat(stat) => self.get_eff_stat(stat),
            QueryValue::Hp => self.curr_hp,
            QueryValue::Mp => self.curr_mp,
            QueryValue::UseCount | QueryValue::TurnsSinceUse => 0,
        }
    }

    pub fn increment_turn_count(&mut self) {
        self.actor_turn_count += 1;
    }

    /// Record that the actor used an ability on their current turn.
    pub fn record_ability_use(&mut self, ability_name: &str) {
        *self
            .ability_use_counts
            .entry(ability_name.to_string())
            .or_insert(0) += 1;
        self.ability_last_used_turn
            .insert(ability_name.to_string(), self.actor_turn_count);
    }

    pub fn ability_use_count(&self, ability_name: &str) -> u32 {
        self.ability_use_counts
            .get(ability_name)
            .copied()
            .unwrap_or(0)
    }

    /// Returns actor turns elapsed since this ability was last used.
    /// If never used, returns u32::MAX (always passes >= checks).
    pub fn turns_since_ability_use(&self, ability_name: &str) -> u32 {
        match self.ability_last_used_turn.get(ability_name) {
            Some(&turn) => self.actor_turn_count.saturating_sub(turn),
            None => u32::MAX,
        }
    }

    /// Apply a named status effect. Handles stacking, NoStack replacement,
    /// and Empower/Weaken cancellation. Rejects StatModPerStack targeting
    /// pool stats (CON, DEX, SPI).
    pub fn add_status(
        &mut self,
        key: &str,
        mut stacks: u32,
        source_id: u32,
        def: &StatusDef,
        stat: Option<Stat>,
    ) -> bool {
        // Check debuff resistance
        let is_debuff = match &def.behavior {
            StatusBehavior::DamagePerStack { .. } => true,
            StatusBehavior::SkipTurn => true,
            StatusBehavior::StatModPerStack { magnitude } => *magnitude < 0,
            _ => false,
        };
        if is_debuff && self.try_negate_debuff() {
            return true; // debuff negated
        }

        // Reject stat mods on pool stats
        if matches!(&def.behavior, StatusBehavior::StatModPerStack { .. }) {
            if let Some(ref s) = stat {
                if matches!(s, Stat::CON | Stat::DEX | Stat::SPI) {
                    return false;
                }
            }
        }

        // Handle Empower/Weaken cancellation
        if let Some(ref opposes) = def.opposes {
            let opp_key = opposite_key(key, opposes);
            if let Some(opp) = self.statuses.get_mut(&opp_key) {
                if opp.stacks >= stacks {
                    opp.stacks -= stacks;
                    if opp.stacks == 0 {
                        self.statuses.remove(&opp_key);
                    }
                    return true; // fully cancelled
                } else {
                    stacks -= opp.stacks;
                    self.statuses.remove(&opp_key);
                    // fall through to apply remaining stacks
                }
            }
        }

        match def.stack_type {
            StackType::TickDown | StackType::Permanent => {
                if let Some(existing) = self.statuses.get_mut(key) {
                    existing.stacks += stacks;
                    existing.source_id = source_id;
                } else {
                    self.statuses.insert(
                        key.to_string(),
                        StatusInstance {
                            stacks,
                            source_id,
                            behavior: def.behavior.clone(),
                            stack_type: def.stack_type.clone(),
                            stat,
                        },
                    );
                }
            }
            StackType::NoStack => {
                if let Some(existing) = self.statuses.get_mut(key) {
                    if stacks > existing.stacks {
                        existing.stacks = stacks;
                    }
                    existing.source_id = source_id;
                } else {
                    self.statuses.insert(
                        key.to_string(),
                        StatusInstance {
                            stacks,
                            source_id,
                            behavior: def.behavior.clone(),
                            stack_type: def.stack_type.clone(),
                            stat,
                        },
                    );
                }
            }
        }
        true
    }

    /// Remove stacks of a status. If stacks reaches 0, removes the entry.
    pub fn remove_status(&mut self, key: &str, stacks: u32) {
        if let Some(existing) = self.statuses.get_mut(key) {
            if existing.stacks <= stacks {
                self.statuses.remove(key);
            } else {
                existing.stacks -= stacks;
            }
        }
    }

    /// Tick all statuses: collect damage/heal events, apply net HP change,
    /// then decrement stacks. Order of evaluation never matters — death is
    /// only checked after all effects resolve.
    pub fn tick_statuses(&mut self) -> Vec<StatusTick> {
        let mut ticks = Vec::new();
        let mut total_damage: u32 = 0;
        let mut total_heal: u32 = 0;

        // Collect damage/heal from all statuses
        for (key, inst) in &self.statuses {
            match &inst.behavior {
                StatusBehavior::DamagePerStack { value } => {
                    let dmg = value * inst.stacks;
                    total_damage += dmg;
                    ticks.push(StatusTick::DamageDealt {
                        name: key.clone(),
                        damage: dmg,
                    });
                }
                StatusBehavior::HealPerStack { value } => {
                    let heal = value * inst.stacks;
                    total_heal += heal;
                    ticks.push(StatusTick::HealApplied {
                        name: key.clone(),
                        amount: heal,
                    });
                }
                // StatModPerStack and SkipTurn don't produce tick events
                _ => {}
            }
        }

        // Apply net HP change (batch-resolve: order never matters)
        let net = total_heal as i32 - total_damage as i32;
        if net > 0 {
            self.heal(net as u32);
        } else if net < 0 {
            self.take_damage((-net) as u32);
        }

        // Decrement stacks for TickDown and NoStack
        self.statuses.retain(|_, inst| match inst.stack_type {
            StackType::TickDown | StackType::NoStack => {
                inst.stacks = inst.stacks.saturating_sub(1);
                inst.stacks > 0
            }
            StackType::Permanent => true,
        });

        ticks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::statuses::{StatusDef, status_key};

    fn make_config(stats: Vec<(Stat, u32)>) -> CharacterConfig {
        CharacterConfig {
            base_name: "Test".to_string(),
            passive: String::new(),
            actives: Vec::new(),
            item: None,
            position: Position { row: 0, col: 0 },
            stats: stats.into_iter().collect(),
            rules: Vec::new(),
        }
    }

    fn bleed_def() -> StatusDef {
        StatusDef {
            behavior: StatusBehavior::DamagePerStack { value: 1 },
            stack_type: StackType::TickDown,
            opposes: None,
        }
    }

    fn regen_def() -> StatusDef {
        StatusDef {
            behavior: StatusBehavior::HealPerStack { value: 2 },
            stack_type: StackType::TickDown,
            opposes: None,
        }
    }

    fn empower_def() -> StatusDef {
        StatusDef {
            behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
            stack_type: StackType::TickDown,
            opposes: Some("Weaken".to_string()),
        }
    }

    fn weaken_def() -> StatusDef {
        StatusDef {
            behavior: StatusBehavior::StatModPerStack { magnitude: -1 },
            stack_type: StackType::TickDown,
            opposes: Some("Empower".to_string()),
        }
    }

    fn stun_def() -> StatusDef {
        StatusDef {
            behavior: StatusBehavior::SkipTurn,
            stack_type: StackType::NoStack,
            opposes: None,
        }
    }

    fn fortify_def() -> StatusDef {
        StatusDef {
            behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
            stack_type: StackType::Permanent,
            opposes: None,
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
    fn spend_mp_fails_when_insufficient() {
        let config = make_config(vec![(Stat::SPI, 5)]);
        let mut state = CharacterState::from_config(0, &config);
        assert!(state.spend_mp(3));
        assert_eq!(state.current_mp(), 2);
        assert!(!state.spend_mp(3));
        assert_eq!(state.current_mp(), 2);
    }

    #[test]
    fn restore_mp_caps_at_base() {
        let config = make_config(vec![(Stat::SPI, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.spend_mp(8);
        state.restore_mp(100);
        assert_eq!(state.current_mp(), 10);
    }

    #[test]
    fn speed_system_ticks_and_escalates() {
        let config = make_config(vec![(Stat::DEX, 3)]);
        let mut state = CharacterState::from_config(0, &config);
        assert!(!state.tick_speed());
        assert!(!state.tick_speed());
        assert!(state.tick_speed());
        state.reset_speed();
        for _ in 0..4 {
            assert!(!state.tick_speed());
        }
        assert!(state.tick_speed());
        state.reset_speed();
        for _ in 0..6 {
            assert!(!state.tick_speed());
        }
        assert!(state.tick_speed());
    }

    #[test]
    fn effective_stat_includes_empower() {
        let config = make_config(vec![(Stat::STR, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        assert_eq!(state.get_eff_stat(&Stat::STR), 10);

        let key = status_key("Empower", Some(&Stat::STR));
        state.add_status(&key, 5, 99, &empower_def(), Some(Stat::STR));
        assert_eq!(state.get_eff_stat(&Stat::STR), 15);
    }

    #[test]
    fn effective_stat_floors_at_zero_with_weaken() {
        let config = make_config(vec![(Stat::STR, 3)]);
        let mut state = CharacterState::from_config(0, &config);
        let key = status_key("Weaken", Some(&Stat::STR));
        state.add_status(&key, 10, 99, &weaken_def(), Some(Stat::STR));
        assert_eq!(state.get_eff_stat(&Stat::STR), 0);
    }

    #[test]
    fn add_status_rejects_pool_stat_mods() {
        let config = make_config(vec![(Stat::CON, 10), (Stat::DEX, 5), (Stat::SPI, 5)]);
        let mut state = CharacterState::from_config(0, &config);

        for stat in [Stat::CON, Stat::DEX, Stat::SPI] {
            let key = status_key("Empower", Some(&stat));
            let result = state.add_status(&key, 3, 99, &empower_def(), Some(stat));
            assert!(!result);
        }
        assert!(state.statuses().is_empty());
    }

    #[test]
    fn add_status_allows_non_pool_stat_mods() {
        let config = make_config(vec![(Stat::STR, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        let key = status_key("Empower", Some(&Stat::STR));
        assert!(state.add_status(&key, 3, 99, &empower_def(), Some(Stat::STR)));
        assert_eq!(state.statuses().len(), 1);
    }

    #[test]
    fn tick_down_bleed_stacks_decay() {
        let config = make_config(vec![(Stat::CON, 20)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_status("Bleed", 3, 99, &bleed_def(), None);

        // Turn 1: 3 stacks fire (3 dmg), then 3→2
        let ticks = state.tick_statuses();
        assert_eq!(ticks.len(), 1);
        assert!(matches!(
            &ticks[0],
            StatusTick::DamageDealt { damage: 3, .. }
        ));
        assert_eq!(state.current_hp(), 37); // 40 - 3
        assert_eq!(state.status_stacks("Bleed"), 2);

        // Turn 2: 2 stacks fire (2 dmg), then 2→1
        state.tick_statuses();
        assert_eq!(state.current_hp(), 35);
        assert_eq!(state.status_stacks("Bleed"), 1);

        // Turn 3: 1 stack fires (1 dmg), then 1→0, removed
        state.tick_statuses();
        assert_eq!(state.current_hp(), 34);
        assert!(!state.has_status("Bleed"));
    }

    #[test]
    fn tick_down_additive_stacking() {
        let config = make_config(vec![(Stat::CON, 50)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_status("Bleed", 2, 99, &bleed_def(), None);
        state.add_status("Bleed", 3, 99, &bleed_def(), None);
        assert_eq!(state.status_stacks("Bleed"), 5);
    }

    #[test]
    fn no_stack_replaces_with_higher() {
        let config = make_config(vec![(Stat::CON, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_status("Stun", 2, 99, &stun_def(), None);
        assert_eq!(state.status_stacks("Stun"), 2);

        // Reapply with higher
        state.add_status("Stun", 3, 99, &stun_def(), None);
        assert_eq!(state.status_stacks("Stun"), 3);

        // Reapply with lower — no change
        state.add_status("Stun", 1, 99, &stun_def(), None);
        assert_eq!(state.status_stacks("Stun"), 3);
    }

    #[test]
    fn empower_weaken_cancellation_partial() {
        let config = make_config(vec![(Stat::STR, 10)]);
        let mut state = CharacterState::from_config(0, &config);

        let emp_key = status_key("Empower", Some(&Stat::STR));
        let weak_key = status_key("Weaken", Some(&Stat::STR));

        state.add_status(&emp_key, 3, 99, &empower_def(), Some(Stat::STR));
        assert_eq!(state.get_eff_stat(&Stat::STR), 13);

        // Apply 2 Weaken — cancels 2 Empower, leaving Empower:STR 1
        state.add_status(&weak_key, 2, 99, &weaken_def(), Some(Stat::STR));
        assert_eq!(state.status_stacks(&emp_key), 1);
        assert!(!state.has_status(&weak_key));
        assert_eq!(state.get_eff_stat(&Stat::STR), 11);
    }

    #[test]
    fn empower_weaken_cancellation_overflow() {
        let config = make_config(vec![(Stat::STR, 10)]);
        let mut state = CharacterState::from_config(0, &config);

        let emp_key = status_key("Empower", Some(&Stat::STR));
        let weak_key = status_key("Weaken", Some(&Stat::STR));

        state.add_status(&emp_key, 2, 99, &empower_def(), Some(Stat::STR));

        // Apply 5 Weaken — cancels 2 Empower, leaves 3 Weaken
        state.add_status(&weak_key, 5, 99, &weaken_def(), Some(Stat::STR));
        assert!(!state.has_status(&emp_key));
        assert_eq!(state.status_stacks(&weak_key), 3);
        assert_eq!(state.get_eff_stat(&Stat::STR), 7);
    }

    #[test]
    fn multiple_stat_empower_simultaneously() {
        let config = make_config(vec![(Stat::STR, 10), (Stat::FOR, 5)]);
        let mut state = CharacterState::from_config(0, &config);

        let str_key = status_key("Empower", Some(&Stat::STR));
        let for_key = status_key("Empower", Some(&Stat::FOR));

        state.add_status(&str_key, 3, 99, &empower_def(), Some(Stat::STR));
        state.add_status(&for_key, 2, 99, &empower_def(), Some(Stat::FOR));

        assert_eq!(state.get_eff_stat(&Stat::STR), 13);
        assert_eq!(state.get_eff_stat(&Stat::FOR), 7);
    }

    #[test]
    fn permanent_status_never_decays() {
        let config = make_config(vec![(Stat::FOR, 5)]);
        let mut state = CharacterState::from_config(0, &config);

        let key = status_key("Fortify", Some(&Stat::FOR));
        state.add_status(&key, 2, 99, &fortify_def(), Some(Stat::FOR));

        state.tick_statuses();
        state.tick_statuses();
        state.tick_statuses();

        assert_eq!(state.status_stacks(&key), 2);
        assert_eq!(state.get_eff_stat(&Stat::FOR), 7);
    }

    #[test]
    fn batch_resolve_bleed_and_regen_survive() {
        // 1 HP, 1 bleed (1 dmg), 3 regen (6 heal). Should survive.
        let config = make_config(vec![(Stat::CON, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.take_damage(19); // 1 HP
        assert_eq!(state.current_hp(), 1);

        state.add_status("Bleed", 1, 99, &bleed_def(), None);
        state.add_status("Regen", 3, 99, &regen_def(), None);

        state.tick_statuses();
        // Net: 6 heal - 1 dmg = +5. HP = 1 + 5 = 6
        assert_eq!(state.current_hp(), 6);
        assert!(state.is_alive());
    }

    #[test]
    fn is_incapacitated_with_stun() {
        let config = make_config(vec![(Stat::CON, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        assert!(!state.is_incapacitated());

        state.add_status("Stun", 2, 99, &stun_def(), None);
        assert!(state.is_incapacitated());
    }

    #[test]
    fn stun_expires_after_ticks() {
        let config = make_config(vec![(Stat::CON, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_status("Stun", 1, 99, &stun_def(), None);
        assert!(state.is_incapacitated());

        state.tick_statuses(); // 1→0, removed
        assert!(!state.is_incapacitated());
    }

    #[test]
    fn empower_ticks_down() {
        let config = make_config(vec![(Stat::STR, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        let key = status_key("Empower", Some(&Stat::STR));
        state.add_status(&key, 3, 99, &empower_def(), Some(Stat::STR));

        assert_eq!(state.get_eff_stat(&Stat::STR), 13);
        state.tick_statuses(); // 3→2
        assert_eq!(state.get_eff_stat(&Stat::STR), 12);
        state.tick_statuses(); // 2→1
        assert_eq!(state.get_eff_stat(&Stat::STR), 11);
        state.tick_statuses(); // 1→0, removed
        assert_eq!(state.get_eff_stat(&Stat::STR), 10);
    }

    #[test]
    fn remove_status_partial() {
        let config = make_config(vec![(Stat::CON, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_status("Bleed", 5, 99, &bleed_def(), None);
        state.remove_status("Bleed", 2);
        assert_eq!(state.status_stacks("Bleed"), 3);
    }

    #[test]
    fn remove_status_full() {
        let config = make_config(vec![(Stat::CON, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_status("Bleed", 3, 99, &bleed_def(), None);
        state.remove_status("Bleed", 5);
        assert!(!state.has_status("Bleed"));
    }

    #[test]
    fn position_validity() {
        assert!(Position { row: 0, col: 0 }.is_valid());
        assert!(Position { row: 2, col: 3 }.is_valid());
        assert!(!Position { row: 3, col: 0 }.is_valid());
        assert!(!Position { row: 0, col: 4 }.is_valid());
    }

    #[test]
    fn cardinal_adjacency() {
        let center = Position { row: 1, col: 1 };
        assert!(center.is_adjacent(&Position { row: 0, col: 1 }));
        assert!(center.is_adjacent(&Position { row: 2, col: 1 }));
        assert!(center.is_adjacent(&Position { row: 1, col: 0 }));
        assert!(center.is_adjacent(&Position { row: 1, col: 2 }));
        assert!(!center.is_adjacent(&Position { row: 0, col: 0 }));
        assert!(!center.is_adjacent(&Position { row: 2, col: 2 }));
        assert!(!center.is_adjacent(&Position { row: 1, col: 1 }));
        assert!(!center.is_adjacent(&Position { row: 3, col: 1 }));
    }

    // --- Permanent trait tests ---

    #[test]
    fn mp_cost_reduction_sums_amounts() {
        let config = make_config(vec![(Stat::SPI, 5)]);
        let mut state = CharacterState::from_config(0, &config);
        assert_eq!(state.mp_cost_reduction(), 0);

        state.add_trait(TraitEffect::MpCostReduction { amount: 2 });
        assert_eq!(state.mp_cost_reduction(), 2);
    }

    #[test]
    fn damage_reflect_sums_amounts() {
        let config = make_config(vec![(Stat::CON, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        assert_eq!(state.damage_reflect_amount(), 0);

        state.add_trait(TraitEffect::DamageReflect { amount: 3 });
        assert_eq!(state.damage_reflect_amount(), 3);
    }

    #[test]
    fn debuff_resistance_negates_first_n_debuffs() {
        let config = make_config(vec![(Stat::CON, 10), (Stat::STR, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_trait(TraitEffect::DebuffResistance { count: 2 });

        // First debuff: negated
        state.add_status("Bleed", 3, 99, &bleed_def(), None);
        assert!(!state.has_status("Bleed"));

        // Second debuff: negated
        let weak_key = status_key("Weaken", Some(&Stat::STR));
        state.add_status(&weak_key, 2, 99, &weaken_def(), Some(Stat::STR));
        assert!(!state.has_status(&weak_key));

        // Third debuff: goes through
        state.add_status("Bleed", 3, 99, &bleed_def(), None);
        assert_eq!(state.status_stacks("Bleed"), 3);
    }

    #[test]
    fn debuff_resistance_allows_buffs_through() {
        let config = make_config(vec![(Stat::CON, 10), (Stat::STR, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_trait(TraitEffect::DebuffResistance { count: 1 });

        // Buff should not consume a charge
        let emp_key = status_key("Empower", Some(&Stat::STR));
        state.add_status(&emp_key, 3, 99, &empower_def(), Some(Stat::STR));
        assert_eq!(state.status_stacks(&emp_key), 3);

        // Heal should not consume a charge
        state.add_status("Regen", 2, 99, &regen_def(), None);
        assert_eq!(state.status_stacks("Regen"), 2);

        // Debuff charge still available
        state.add_status("Bleed", 1, 99, &bleed_def(), None);
        assert!(!state.has_status("Bleed")); // negated

        // Now charge is used up
        state.add_status("Bleed", 1, 99, &bleed_def(), None);
        assert_eq!(state.status_stacks("Bleed"), 1); // goes through
    }

    #[test]
    fn debuff_resistance_blocks_stun() {
        let config = make_config(vec![(Stat::CON, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_trait(TraitEffect::DebuffResistance { count: 1 });

        state.add_status("Stun", 2, 99, &stun_def(), None);
        assert!(!state.is_incapacitated()); // negated
    }
}
