//! Core data types for the battle engine: characters, stats, effects, and grid positions.

use std::collections::HashMap;

use crate::statuses::{
    StackType, StatusBehavior, StatusDef, StatusGroup, StatusInstance, opposite_key, status_key,
};

/// Maximum stacks of a stat-mod status (Empower/Weaken) per stat. Permanence
/// rewards setup without letting a single stat run away.
pub const MAX_STAT_MOD_STACKS: u32 = 8;

/// Universal mana cap. Mana is "pips": every character starts at 0, caps at
/// MAX_MP, and gains MP_PER_ATTACK per basic attack. There is no mana stat.
pub const MAX_MP: u32 = 5;
/// Mana pips gained from a single basic attack.
pub const MP_PER_ATTACK: u32 = 1;

/// The current character attributes.
#[derive(Hash, Eq, PartialEq, Debug, Clone, serde::Deserialize, serde::Serialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum Stat {
    #[serde(rename = "vit")]
    VIT, // Max HP = 3 * VIT
    #[serde(rename = "mgt")]
    MGT, // Base physical damage
    #[serde(rename = "mag")]
    MAG, // Base magical damage
    #[serde(rename = "arm")]
    ARM, // Physical resistance
    #[serde(rename = "res")]
    RES, // Magical resistance
    #[serde(rename = "spd")]
    SPD, // Determines how often to act
}

/// Cell on the battle grid (rows 0-2, cols 0-2).
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Position {
    pub row: u8,
    pub col: u8,
}

impl Position {
    pub fn is_valid(&self) -> bool {
        self.row < 3 && self.col < 3
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
    /// Any living ally (the whole team, position-independent).
    AnyAlly,
    /// The living ally with the lowest current HP.
    LowestAlly,
    /// Any living enemy.
    AnyEnemy,
    /// The living enemy with the lowest current HP.
    LowestEnemy,
    World,
}

/// What value the condition reads.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryValue {
    Stat(Stat),
    Hp,
    Mp,
    SelfRow,
    SelfCompanionCount,
    TargetCompanionCount,
    HasStatus(String),
    StatusStacks(String),
    HasCondition(String),
    ConditionStacks(String),
    TickCount,
    AllyCount,
    EnemyCount,
    UseCount,
    TurnsSinceUse,
    /// Number of living enemies whose current focus is this character.
    FocusedByCount,
}

/// Comparison operator for conditions.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Comparator {
    Gte,
    Lte,
    Eq,
}

/// A single condition that must be met for a rule to fire.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Condition {
    pub subject: ConditionSubject,
    pub value: QueryValue,
    #[serde(rename = "op")]
    pub comparator: Comparator,
    pub threshold: u32,
}

/// An ordered rule: if all conditions are met (AND), use this ability.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Rule {
    pub ability: String,
    #[serde(rename = "when")]
    pub conditions: Vec<Condition>,
    /// When true, the rule fires if ANY condition holds (default: all must hold).
    #[serde(default)]
    pub match_any: bool,
}

/// Static character definition loaded from JSON (archetype + loadout).
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CharacterConfig {
    #[serde(default)]
    pub id: Option<String>,
    pub base_name: String,
    #[serde(default)]
    pub display_name: Option<String>,
    pub passive: String,
    #[serde(default)]
    pub aspect_passive: Option<String>,
    pub actives: Vec<String>,
    #[serde(default, alias = "item")]
    pub aspect: Option<String>,
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
    /// Dynamic stat bonus from a row aura passive.
    AuraStatMod { stat: Stat, amount: i32 },
}

/// What happened when statuses ticked.
#[derive(Debug, Clone)]
pub enum StatusTick {
    DamageDealt {
        name: String,
        damage: u32,
        source_id: u32,
    },
    HealApplied {
        name: String,
        amount: u32,
        source_id: u32,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StatusDecayTiming {
    StartOfTurn,
    EndOfTurn,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StatusDecayMode {
    TickDown,
    Halve,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionKind {
    Stunned,
    Marked,
    Severed,
}

impl ConditionKind {
    pub fn from_key(value: &str) -> Option<Self> {
        match value {
            "Stunned" | "stunned" => Some(Self::Stunned),
            "Marked" | "marked" => Some(Self::Marked),
            "Severed" | "severed" => Some(Self::Severed),
            _ => None,
        }
    }

    pub fn as_key(&self) -> &'static str {
        match self {
            Self::Stunned => "Stunned",
            Self::Marked => "Marked",
            Self::Severed => "Severed",
        }
    }

    pub fn stacks(self) -> bool {
        matches!(self, Self::Severed)
    }

    pub fn decays_end_of_turn(self) -> bool {
        !matches!(self, Self::Marked)
    }
}

#[derive(Debug, Clone)]
pub struct ConditionInstance {
    pub stacks: u32,
    pub source_id: u32,
}

/// Mutable runtime state of a character during battle. Created from a [`CharacterConfig`].
#[derive(Clone)]
pub struct CharacterState {
    id: u32,
    replay_id: String,
    base_name: String,
    display_name: String,
    passive: String,
    aspect_passive: Option<String>,
    bonus_passives: Vec<String>,
    actives: Vec<String>,
    base_stats: HashMap<Stat, u32>,
    position: Position,
    curr_hp: u32,
    curr_mp: u32,
    ticks_until_turn: u32,
    max_ticks: i32,
    pending_haste: u32,
    target: Option<u32>,
    pending_focus_change: Option<(u32, u32)>,
    companions: Vec<u32>,
    statuses: HashMap<String, StatusInstance>,
    conditions: HashMap<ConditionKind, ConditionInstance>,
    traits: Vec<TraitEffect>,
    rules: Vec<Rule>,
    defeat_resolved: bool,
    actor_turn_count: u32,
    ability_use_counts: HashMap<String, u32>,
    ability_last_used_turn: HashMap<String, u32>,
}

impl CharacterState {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn from_config(id: u32, config: &CharacterConfig) -> Self {
        let replay_id = config
            .id
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| format!("character_{id}"));
        let display_name = config
            .display_name
            .clone()
            .unwrap_or_else(|| config.base_name.clone());
        Self::from_config_with_identity(id, replay_id, display_name, config)
    }

    pub fn from_config_with_identity(
        id: u32,
        replay_id: String,
        display_name: String,
        config: &CharacterConfig,
    ) -> Self {
        let hp = config.stats.get(&Stat::VIT).copied().unwrap_or(0) * 3;
        // Characters start at 0 MP and charge it up via basic attacks, so abilities
        // are an earned, occasional spend rather than spammed from turn one.
        let dex = config.stats.get(&Stat::SPD).copied().unwrap_or(0) as i32;
        let max_ticks = 15 - dex;
        Self {
            id,
            replay_id,
            base_name: config.base_name.clone(),
            display_name,
            passive: config.passive.clone(),
            aspect_passive: config.aspect_passive.clone(),
            bonus_passives: Vec::new(),
            actives: config.actives.clone(),
            base_stats: config.stats.clone(),
            position: config.position.clone(),
            curr_hp: hp,
            curr_mp: 0,
            ticks_until_turn: max_ticks.max(1) as u32,
            max_ticks,
            pending_haste: 0,
            target: None,
            pending_focus_change: None,
            companions: Vec::new(),
            statuses: HashMap::new(),
            conditions: HashMap::new(),
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

    pub fn replay_id(&self) -> &str {
        &self.replay_id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn passive(&self) -> &str {
        &self.passive
    }

    pub fn aspect_passive(&self) -> Option<&str> {
        self.aspect_passive.as_deref()
    }

    pub fn passive_names(&self) -> Vec<&str> {
        let mut names = Vec::new();
        if !self.passive.is_empty() {
            names.push(self.passive.as_str());
        }
        if let Some(aspect_passive) = self.aspect_passive()
            && !aspect_passive.is_empty()
        {
            names.push(aspect_passive);
        }
        for passive in &self.bonus_passives {
            if !passive.is_empty() {
                names.push(passive.as_str());
            }
        }
        names
    }

    pub fn gain_bonus_passive(&mut self, passive_name: &str) {
        if passive_name.is_empty() || self.passive_names().contains(&passive_name) {
            return;
        }
        self.bonus_passives.push(passive_name.to_string());
    }

    pub fn actives(&self) -> &[String] {
        &self.actives
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
        if self.has_condition(ConditionKind::Stunned) {
            return true;
        }
        self.statuses
            .values()
            .any(|s| matches!(s.behavior, StatusBehavior::SkipTurn))
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn set_position(&mut self, position: Position) {
        self.position = position;
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
            .sum::<i32>()
            + self
                .traits
                .iter()
                .filter_map(|t| match t {
                    TraitEffect::AuraStatMod {
                        stat: aura_stat,
                        amount,
                    } if aura_stat == stat => Some(*amount),
                    _ => None,
                })
                .sum::<i32>();
        (base + modifier).max(0) as u32
    }

    /// Returns the effective stat value while counting Empower on that stat twice.
    pub fn get_eff_stat_with_doubled_empower(&self, stat: &Stat) -> u32 {
        let base = self.get_base_stat(stat) as i32;
        let empower_key = status_key("Empower", Some(stat));
        let modifier: i32 = self
            .statuses
            .iter()
            .filter_map(|(key, status)| match &status.behavior {
                StatusBehavior::StatModPerStack { magnitude }
                    if status.stat.as_ref() == Some(stat) =>
                {
                    let value = *magnitude * status.stacks as i32;
                    if key == &empower_key && *magnitude > 0 {
                        Some(value * 2)
                    } else {
                        Some(value)
                    }
                }
                _ => None,
            })
            .sum::<i32>()
            + self
                .traits
                .iter()
                .filter_map(|t| match t {
                    TraitEffect::AuraStatMod {
                        stat: aura_stat,
                        amount,
                    } if aura_stat == stat => Some(*amount),
                    _ => None,
                })
                .sum::<i32>();
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

    /// Apply an incoming hit. One Ward stack negates the hit entirely.
    /// Returns the damage actually applied after Ward.
    pub fn take_hit(&mut self, amount: u32) -> u32 {
        if amount == 0 {
            return 0;
        }
        if self.status_stacks("Ward") > 0 {
            self.remove_status("Ward", 1);
            return 0;
        }
        self.take_damage(amount);
        amount
    }

    /// Heals up to max HP (3 * VIT).
    pub fn heal(&mut self, amount: u32) {
        let max_hp = self.get_base_stat(&Stat::VIT) * 3;
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

    /// Restores mana pips up to the universal MAX_MP cap.
    pub fn restore_mp(&mut self, amount: u32) {
        self.curr_mp = (self.curr_mp + amount).min(MAX_MP);
    }

    /// Decrements speed counter. Returns true when the character is ready to act.
    pub fn tick_speed(&mut self) -> bool {
        self.ticks_until_turn = self.ticks_until_turn.saturating_sub(1);
        self.ticks_until_turn == 0
    }

    /// Resets speed after acting. Each turn adds +2 to max_ticks, while the
    /// live countdown is clamped to a minimum of 1.
    pub fn reset_speed(&mut self) {
        self.max_ticks += 2;
        self.ticks_until_turn = self.max_ticks.max(1) as u32;
        if self.pending_haste > 0 {
            self.ticks_until_turn = self
                .ticks_until_turn
                .saturating_sub(self.pending_haste)
                .max(1);
            self.pending_haste = 0;
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn apply_haste(&mut self, amount: u32) {
        if amount == 0 || !self.is_alive() {
            return;
        }
        if self.ticks_until_turn == 0 {
            self.pending_haste = self.pending_haste.saturating_add(amount);
        } else {
            self.ticks_until_turn = self.ticks_until_turn.saturating_sub(amount).max(1);
        }
    }

    pub fn target(&self) -> Option<u32> {
        self.target
    }

    pub fn set_target(&mut self, target_id: u32) {
        self.target = Some(target_id);
    }

    pub fn set_target_tracked(&mut self, target_id: u32, step: u32) -> bool {
        let changed = self.target != Some(target_id);
        self.target = Some(target_id);
        if changed {
            self.pending_focus_change = Some((step, target_id));
        }
        changed
    }

    pub fn clear_target(&mut self) {
        self.target = None;
    }

    #[allow(dead_code)]
    pub fn clear_target_tracked(&mut self) {
        self.target = None;
    }

    pub fn take_pending_focus_change(&mut self, step: u32) -> Option<u32> {
        match self.pending_focus_change {
            Some((changed_step, new_target_id)) if changed_step == step => {
                self.pending_focus_change = None;
                Some(new_target_id)
            }
            _ => None,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn companions(&self) -> &[u32] {
        &self.companions
    }

    pub fn effective_companion_ids(&self) -> Vec<u32> {
        if self.has_condition(ConditionKind::Severed) {
            Vec::new()
        } else {
            self.companions.clone()
        }
    }

    pub fn set_companions(&mut self, ids: Vec<u32>) {
        self.companions = ids;
    }

    pub fn statuses(&self) -> &HashMap<String, StatusInstance> {
        &self.statuses
    }

    pub fn has_status(&self, key: &str) -> bool {
        self.statuses.contains_key(key)
    }

    pub fn status_stacks(&self, key: &str) -> u32 {
        self.statuses.get(key).map_or(0, |s| s.stacks)
    }

    pub fn conditions(&self) -> &HashMap<ConditionKind, ConditionInstance> {
        &self.conditions
    }

    pub fn has_condition(&self, kind: ConditionKind) -> bool {
        self.conditions.contains_key(&kind)
    }

    pub fn condition_stacks(&self, kind: ConditionKind) -> u32 {
        self.conditions
            .get(&kind)
            .map_or(0, |condition| condition.stacks)
    }

    pub fn has_condition_key(&self, key: &str) -> bool {
        ConditionKind::from_key(key).is_some_and(|kind| self.has_condition(kind))
    }

    pub fn condition_stacks_key(&self, key: &str) -> u32 {
        ConditionKind::from_key(key).map_or(0, |kind| self.condition_stacks(kind))
    }

    pub fn add_trait(&mut self, t: TraitEffect) {
        self.traits.push(t);
    }

    pub fn clear_aura_traits(&mut self) {
        self.traits
            .retain(|t| !matches!(t, TraitEffect::AuraStatMod { .. }));
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
            if let TraitEffect::DebuffResistance { count } = t
                && *count > 0
            {
                *count -= 1;
                return true;
            }
        }
        false
    }

    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// Returns the value of a query (effective stat, HP, MP, or status state) for
    /// condition evaluation.
    /// UseCount and TurnsSinceUse are not handled here — they require ability context.
    pub fn query_value(&self, qv: &QueryValue) -> u32 {
        match qv {
            QueryValue::Stat(stat) => self.get_eff_stat(stat),
            // HP is a percentage of max (0–100) so rules port across stat lines.
            QueryValue::Hp => {
                let max = self.get_base_stat(&Stat::VIT) * 3;
                if max == 0 {
                    0
                } else {
                    (self.curr_hp * 100) / max
                }
            }
            QueryValue::Mp => self.curr_mp,
            QueryValue::SelfRow => u32::from(self.position.row),
            QueryValue::HasStatus(key) => u32::from(self.has_status(key)),
            QueryValue::StatusStacks(key) => self.status_stacks(key),
            QueryValue::HasCondition(key) => u32::from(self.has_condition_key(key)),
            QueryValue::ConditionStacks(key) => self.condition_stacks_key(key),
            // These need context the character can't see alone (computed in rules.rs).
            QueryValue::SelfCompanionCount
            | QueryValue::TargetCompanionCount
            | QueryValue::FocusedByCount
            | QueryValue::TickCount
            | QueryValue::AllyCount
            | QueryValue::EnemyCount
            | QueryValue::UseCount
            | QueryValue::TurnsSinceUse => 0,
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
    /// pool stats (VIT, SPD, WIL).
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
            StatusBehavior::Ward => false,
            StatusBehavior::StatModPerStack { magnitude } => *magnitude < 0,
            _ => false,
        };
        if is_debuff && self.try_negate_debuff() {
            return true; // debuff negated
        }

        // Reject stat mods on pool stats
        if matches!(&def.behavior, StatusBehavior::StatModPerStack { .. })
            && let Some(ref s) = stat
            && matches!(s, Stat::VIT | Stat::SPD)
        {
            return false;
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
                            group: resolve_status_group(def, stat.as_ref()),
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
                            group: resolve_status_group(def, stat.as_ref()),
                            stat,
                        },
                    );
                }
            }
        }

        // Cap stat-mod (Empower/Weaken) stacks so permanence rewards setup
        // without becoming uncatchable.
        if matches!(&def.behavior, StatusBehavior::StatModPerStack { .. })
            && let Some(inst) = self.statuses.get_mut(key)
        {
            inst.stacks = inst.stacks.min(MAX_STAT_MOD_STACKS);
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

    pub fn add_condition(&mut self, kind: ConditionKind, stacks: u32, source_id: u32) -> bool {
        if stacks == 0 {
            return false;
        }
        if self.try_negate_debuff() {
            return true;
        }

        if kind.stacks() {
            if let Some(existing) = self.conditions.get_mut(&kind) {
                existing.stacks += stacks;
                existing.source_id = source_id;
            } else {
                self.conditions
                    .insert(kind, ConditionInstance { stacks, source_id });
            }
        } else if let Some(existing) = self.conditions.get_mut(&kind) {
            existing.stacks = 1;
            existing.source_id = source_id;
        } else {
            self.conditions.insert(
                kind,
                ConditionInstance {
                    stacks: 1,
                    source_id,
                },
            );
        }

        true
    }

    pub fn remove_condition(&mut self, kind: ConditionKind, stacks: u32) {
        if let Some(existing) = self.conditions.get_mut(&kind) {
            if existing.stacks <= stacks {
                self.conditions.remove(&kind);
            } else {
                existing.stacks -= stacks;
            }
        }
    }

    pub fn decay_conditions_end_of_turn(&mut self) {
        self.conditions.retain(|kind, condition| {
            if !kind.decays_end_of_turn() {
                return true;
            }
            condition.stacks = condition.stacks.saturating_sub(1);
            condition.stacks > 0
        });
    }

    pub fn remove_one_buff(&mut self) -> bool {
        let selected = self
            .statuses
            .iter()
            .filter(|(_, inst)| match inst.behavior {
                StatusBehavior::HealPerStack { .. } | StatusBehavior::Ward => true,
                StatusBehavior::StatModPerStack { magnitude } => magnitude > 0,
                _ => false,
            })
            .max_by(|(left_key, left), (right_key, right)| {
                left.stacks
                    .cmp(&right.stacks)
                    .then_with(|| left_key.cmp(right_key))
            })
            .map(|(key, inst)| (key.clone(), inst.stacks));

        let Some((key, stacks)) = selected else {
            return false;
        };

        self.remove_status(&key, stacks);
        true
    }

    pub fn cleanse(&mut self, amount: u32, group: Option<StatusGroup>) -> bool {
        self.reduce_matching_timed_effects(amount, EffectPolarity::Debuff, group)
    }

    pub fn dispel(&mut self, amount: u32, group: Option<StatusGroup>) -> bool {
        self.reduce_matching_timed_effects(amount, EffectPolarity::Buff, group)
    }

    fn reduce_matching_timed_effects(
        &mut self,
        amount: u32,
        polarity: EffectPolarity,
        group: Option<StatusGroup>,
    ) -> bool {
        if amount == 0 {
            return false;
        }

        let keys: Vec<String> = self
            .statuses
            .iter()
            .filter(|(_, inst)| matches!(inst.stack_type, StackType::TickDown))
            .filter(|(_, inst)| effect_polarity(inst) == Some(polarity))
            .filter(|(_, inst)| group.is_none_or(|expected| inst.group == Some(expected)))
            .map(|(key, _)| key.clone())
            .collect();

        let mut changed = false;
        for key in keys {
            let before = self.status_stacks(&key);
            self.remove_status(&key, amount);
            changed |= self.status_stacks(&key) != before;
        }
        changed
    }

    /// Consume one stack of each skip-turn status after it actually prevents an action.
    pub fn consume_skip_turn_statuses(&mut self) {
        self.statuses.retain(|_, inst| match inst.behavior {
            StatusBehavior::SkipTurn => {
                inst.stacks = inst.stacks.saturating_sub(1);
                inst.stacks > 0
            }
            _ => true,
        });
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
                        source_id: inst.source_id,
                    });
                }
                StatusBehavior::HealPerStack { value } => {
                    let heal = value * inst.stacks;
                    total_heal += heal;
                    ticks.push(StatusTick::HealApplied {
                        name: key.clone(),
                        amount: heal,
                        source_id: inst.source_id,
                    });
                }
                // StatModPerStack, SkipTurn, and Ward don't produce tick events
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

        self.decay_statuses(StatusDecayTiming::StartOfTurn);

        ticks
    }

    pub fn decay_statuses_end_of_turn(&mut self) {
        self.decay_statuses(StatusDecayTiming::EndOfTurn);
    }

    fn decay_statuses(&mut self, timing: StatusDecayTiming) {
        self.statuses.retain(|key, inst| {
            let Some((expected_timing, mode)) = status_decay_rule(key, inst) else {
                return true;
            };
            if expected_timing != timing {
                return true;
            }

            match mode {
                StatusDecayMode::TickDown => {
                    inst.stacks = inst.stacks.saturating_sub(1);
                }
                StatusDecayMode::Halve => {
                    inst.stacks /= 2;
                }
            }
            inst.stacks > 0
        });
    }
}

fn resolve_status_group(def: &StatusDef, stat: Option<&Stat>) -> Option<StatusGroup> {
    def.group.or_else(|| {
        if !matches!(def.behavior, StatusBehavior::StatModPerStack { .. }) {
            return None;
        }
        match stat {
            Some(Stat::MGT | Stat::ARM) => Some(StatusGroup::Body),
            Some(Stat::MAG | Stat::RES) => Some(StatusGroup::Mind),
            _ => None,
        }
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EffectPolarity {
    Buff,
    Debuff,
}

fn effect_polarity(inst: &StatusInstance) -> Option<EffectPolarity> {
    match inst.behavior {
        StatusBehavior::DamagePerStack { .. } => Some(EffectPolarity::Debuff),
        StatusBehavior::HealPerStack { .. } => Some(EffectPolarity::Buff),
        StatusBehavior::StatModPerStack { magnitude } => {
            if magnitude > 0 {
                Some(EffectPolarity::Buff)
            } else if magnitude < 0 {
                Some(EffectPolarity::Debuff)
            } else {
                None
            }
        }
        StatusBehavior::Ward | StatusBehavior::SkipTurn => None,
    }
}

fn status_decay_rule(
    key: &str,
    inst: &StatusInstance,
) -> Option<(StatusDecayTiming, StatusDecayMode)> {
    // Restoration stays on halving decay so sustain is self-limiting.
    if key == "Restoration" {
        return Some((StatusDecayTiming::StartOfTurn, StatusDecayMode::Halve));
    }
    // Lethality (dormant) keeps the halving burst-window family if it returns.
    if key == "Lethality" {
        return Some((StatusDecayTiming::EndOfTurn, StatusDecayMode::Halve));
    }
    // Omen ticks down by 1 (handled by the TickDown stack_type below).
    // Empower/Weaken are permanent (Permanent stack_type below -> no decay).

    match (&inst.stack_type, &inst.behavior) {
        (StackType::TickDown, _) => {
            Some((StatusDecayTiming::StartOfTurn, StatusDecayMode::TickDown))
        }
        (StackType::NoStack, StatusBehavior::SkipTurn) => None,
        (StackType::NoStack, _) => {
            Some((StatusDecayTiming::StartOfTurn, StatusDecayMode::TickDown))
        }
        (StackType::Permanent, _) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::statuses::{StatusDef, status_key};

    fn make_config(stats: Vec<(Stat, u32)>) -> CharacterConfig {
        CharacterConfig {
            id: None,
            base_name: "Test".to_string(),
            display_name: None,
            passive: String::new(),
            actives: Vec::new(),
            aspect_passive: None,
            aspect: None,
            position: Position { row: 0, col: 0 },
            stats: stats.into_iter().collect(),
            rules: Vec::new(),
        }
    }

    fn bleed_def() -> StatusDef {
        StatusDef {
            behavior: StatusBehavior::DamagePerStack { value: 1 },
            stack_type: StackType::TickDown,
            group: None,
            opposes: None,
        }
    }

    fn regen_def() -> StatusDef {
        StatusDef {
            behavior: StatusBehavior::HealPerStack { value: 2 },
            stack_type: StackType::TickDown,
            group: None,
            opposes: None,
        }
    }

    fn omen_def() -> StatusDef {
        StatusDef {
            behavior: StatusBehavior::DamagePerStack { value: 1 },
            stack_type: StackType::TickDown,
            group: Some(StatusGroup::Fate),
            opposes: None,
        }
    }

    fn restoration_def() -> StatusDef {
        StatusDef {
            behavior: StatusBehavior::HealPerStack { value: 1 },
            stack_type: StackType::TickDown,
            group: Some(StatusGroup::Fate),
            opposes: None,
        }
    }

    fn empower_def() -> StatusDef {
        StatusDef {
            behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
            stack_type: StackType::Permanent,
            group: None,
            opposes: Some("Weaken".to_string()),
        }
    }

    fn weaken_def() -> StatusDef {
        StatusDef {
            behavior: StatusBehavior::StatModPerStack { magnitude: -1 },
            stack_type: StackType::Permanent,
            group: None,
            opposes: Some("Empower".to_string()),
        }
    }

    fn stun_def() -> StatusDef {
        StatusDef {
            behavior: StatusBehavior::SkipTurn,
            stack_type: StackType::NoStack,
            group: None,
            opposes: None,
        }
    }

    fn fortify_def() -> StatusDef {
        StatusDef {
            behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
            stack_type: StackType::Permanent,
            group: None,
            opposes: None,
        }
    }

    fn ward_def() -> StatusDef {
        StatusDef {
            behavior: StatusBehavior::Ward,
            stack_type: StackType::Permanent,
            group: None,
            opposes: None,
        }
    }

    fn lethality_def() -> StatusDef {
        StatusDef {
            behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
            stack_type: StackType::TickDown,
            group: Some(StatusGroup::Body),
            opposes: None,
        }
    }

    #[test]
    fn from_config_sets_hp_to_triple_vit() {
        let config = make_config(vec![(Stat::VIT, 10), (Stat::SPD, 5)]);
        let state = CharacterState::from_config(0, &config);
        assert_eq!(state.current_hp(), 30);
    }

    #[test]
    fn from_config_copies_position() {
        let mut config = make_config(vec![(Stat::VIT, 5)]);
        config.position = Position { row: 2, col: 2 };
        let state = CharacterState::from_config(0, &config);
        assert_eq!(state.position().row, 2);
        assert_eq!(state.position().col, 2);
    }

    #[test]
    fn take_damage_saturates_at_zero() {
        let config = make_config(vec![(Stat::VIT, 5)]);
        let mut state = CharacterState::from_config(0, &config);
        assert_eq!(state.current_hp(), 15);
        state.take_damage(100);
        assert_eq!(state.current_hp(), 0);
        assert!(!state.is_alive());
    }

    #[test]
    fn heal_caps_at_max_hp() {
        let config = make_config(vec![(Stat::VIT, 5)]);
        let mut state = CharacterState::from_config(0, &config);
        state.take_damage(3);
        assert_eq!(state.current_hp(), 12);
        state.heal(100);
        assert_eq!(state.current_hp(), 15);
    }

    #[test]
    fn spend_mp_fails_when_insufficient() {
        let config = make_config(vec![]);
        let mut state = CharacterState::from_config(0, &config);
        state.restore_mp(5); // characters now start at 0 MP
        assert!(state.spend_mp(3));
        assert_eq!(state.current_mp(), 2);
        assert!(!state.spend_mp(3));
        assert_eq!(state.current_mp(), 2);
    }

    #[test]
    fn restore_mp_caps_at_max() {
        let config = make_config(vec![]);
        let mut state = CharacterState::from_config(0, &config);
        state.restore_mp(100);
        assert_eq!(state.current_mp(), MAX_MP);
    }

    #[test]
    fn ward_negates_next_hit_and_is_removed() {
        let config = make_config(vec![(Stat::VIT, 5)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_status("Ward", 1, 99, &ward_def(), None);

        assert_eq!(state.take_hit(4), 0);
        assert_eq!(state.current_hp(), 15);
        assert_eq!(state.status_stacks("Ward"), 0);
    }

    #[test]
    fn multiple_wards_consume_one_per_hit() {
        let config = make_config(vec![(Stat::VIT, 5)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_status("Ward", 2, 99, &ward_def(), None);

        assert_eq!(state.take_hit(4), 0);
        assert_eq!(state.status_stacks("Ward"), 1);
        assert_eq!(state.take_hit(3), 0);
        assert_eq!(state.status_stacks("Ward"), 0);
        assert_eq!(state.take_hit(2), 2);
        assert_eq!(state.current_hp(), 13);
    }

    #[test]
    fn ward_does_not_negate_status_tick_damage() {
        let config = make_config(vec![(Stat::VIT, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_status("Ward", 1, 99, &ward_def(), None);
        state.add_status("Bleed", 2, 99, &bleed_def(), None);

        state.tick_statuses();

        assert_eq!(state.current_hp(), 28);
        assert_eq!(state.status_stacks("Ward"), 1);
    }

    #[test]
    fn speed_system_ticks_and_escalates() {
        let config = make_config(vec![(Stat::SPD, 8)]);
        let mut state = CharacterState::from_config(0, &config);
        for _ in 0..6 {
            assert!(!state.tick_speed());
        }
        assert!(state.tick_speed());
        state.reset_speed();
        for _ in 0..8 {
            assert!(!state.tick_speed());
        }
        assert!(state.tick_speed());
        state.reset_speed();
        for _ in 0..10 {
            assert!(!state.tick_speed());
        }
        assert!(state.tick_speed());
    }

    #[test]
    fn speed_system_clamps_high_dex_to_one_tick() {
        let config = make_config(vec![(Stat::SPD, 17)]);
        let mut state = CharacterState::from_config(0, &config);

        assert!(state.tick_speed());
        state.reset_speed();
        assert!(state.tick_speed());
        state.reset_speed();
        assert!(!state.tick_speed());
        assert!(state.tick_speed());
    }

    #[test]
    fn effective_stat_includes_empower() {
        let config = make_config(vec![(Stat::MGT, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        assert_eq!(state.get_eff_stat(&Stat::MGT), 10);

        let key = status_key("Empower", Some(&Stat::MGT));
        state.add_status(&key, 5, 99, &empower_def(), Some(Stat::MGT));
        assert_eq!(state.get_eff_stat(&Stat::MGT), 15);
    }

    #[test]
    fn doubled_empower_only_amplifies_empower_for_requested_stat() {
        let config = make_config(vec![(Stat::MGT, 10), (Stat::MAG, 6)]);
        let mut state = CharacterState::from_config(0, &config);
        let str_key = status_key("Empower", Some(&Stat::MGT));
        let int_key = status_key("Empower", Some(&Stat::MAG));
        state.add_status(&str_key, 2, 99, &empower_def(), Some(Stat::MGT));
        state.add_status(&int_key, 3, 99, &empower_def(), Some(Stat::MAG));

        assert_eq!(state.get_eff_stat(&Stat::MGT), 12);
        assert_eq!(state.get_eff_stat_with_doubled_empower(&Stat::MGT), 14);
        assert_eq!(state.get_eff_stat_with_doubled_empower(&Stat::MAG), 12);
    }

    #[test]
    fn effective_stat_floors_at_zero_with_weaken() {
        let config = make_config(vec![(Stat::MGT, 3)]);
        let mut state = CharacterState::from_config(0, &config);
        let key = status_key("Weaken", Some(&Stat::MGT));
        state.add_status(&key, 10, 99, &weaken_def(), Some(Stat::MGT));
        assert_eq!(state.get_eff_stat(&Stat::MGT), 0);
    }

    #[test]
    fn add_status_rejects_pool_stat_mods() {
        let config = make_config(vec![(Stat::VIT, 10), (Stat::SPD, 5)]);
        let mut state = CharacterState::from_config(0, &config);

        for stat in [Stat::VIT, Stat::SPD] {
            let key = status_key("Empower", Some(&stat));
            let result = state.add_status(&key, 3, 99, &empower_def(), Some(stat));
            assert!(!result);
        }
        assert!(state.statuses().is_empty());
    }

    #[test]
    fn add_status_allows_non_pool_stat_mods() {
        let config = make_config(vec![(Stat::MGT, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        let key = status_key("Empower", Some(&Stat::MGT));
        assert!(state.add_status(&key, 3, 99, &empower_def(), Some(Stat::MGT)));
        assert_eq!(state.statuses().len(), 1);
    }

    #[test]
    fn tick_down_bleed_stacks_decay() {
        let config = make_config(vec![(Stat::VIT, 20)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_status("Bleed", 3, 99, &bleed_def(), None);

        // Turn 1: 3 stacks fire (3 dmg), then 3→2
        let ticks = state.tick_statuses();
        assert_eq!(ticks.len(), 1);
        assert!(matches!(
            &ticks[0],
            StatusTick::DamageDealt { damage: 3, .. }
        ));
        assert_eq!(state.current_hp(), 57); // 60 - 3
        assert_eq!(state.status_stacks("Bleed"), 2);

        // Turn 2: 2 stacks fire (2 dmg), then 2→1
        state.tick_statuses();
        assert_eq!(state.current_hp(), 55);
        assert_eq!(state.status_stacks("Bleed"), 1);

        // Turn 3: 1 stack fires (1 dmg), then 1→0, removed
        state.tick_statuses();
        assert_eq!(state.current_hp(), 54);
        assert!(!state.has_status("Bleed"));
    }

    #[test]
    fn omen_ticks_for_current_stack_count() {
        let config = make_config(vec![(Stat::VIT, 20)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_status("Omen", 4, 99, &omen_def(), None);

        let ticks = state.tick_statuses();
        assert_eq!(ticks.len(), 1);
        assert!(matches!(
            &ticks[0],
            StatusTick::DamageDealt { name, damage, .. } if name == "Omen" && *damage == 4
        ));
        assert_eq!(state.current_hp(), 56);
        // Omen now ticks down by 1 each start of turn (was halving): 4 -> 3.
        assert_eq!(state.status_stacks("Omen"), 3);
    }

    #[test]
    fn omen_can_kill_on_turn_start_tick() {
        let config = make_config(vec![(Stat::VIT, 1)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_status("Omen", 4, 99, &omen_def(), None);

        state.tick_statuses();

        assert_eq!(state.current_hp(), 0);
        assert!(!state.is_alive());
        // Omen now ticks down by 1 each start of turn (was halving): 4 -> 3.
        assert_eq!(state.status_stacks("Omen"), 3);
    }

    #[test]
    fn tick_down_additive_stacking() {
        let config = make_config(vec![(Stat::VIT, 50)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_status("Bleed", 2, 99, &bleed_def(), None);
        state.add_status("Bleed", 3, 99, &bleed_def(), None);
        assert_eq!(state.status_stacks("Bleed"), 5);
    }

    #[test]
    fn no_stack_replaces_with_higher() {
        let config = make_config(vec![(Stat::VIT, 10)]);
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
        let config = make_config(vec![(Stat::MGT, 10)]);
        let mut state = CharacterState::from_config(0, &config);

        let emp_key = status_key("Empower", Some(&Stat::MGT));
        let weak_key = status_key("Weaken", Some(&Stat::MGT));

        state.add_status(&emp_key, 3, 99, &empower_def(), Some(Stat::MGT));
        assert_eq!(state.get_eff_stat(&Stat::MGT), 13);

        // Apply 2 Weaken — cancels 2 Empower, leaving Empower:MGT 1
        state.add_status(&weak_key, 2, 99, &weaken_def(), Some(Stat::MGT));
        assert_eq!(state.status_stacks(&emp_key), 1);
        assert!(!state.has_status(&weak_key));
        assert_eq!(state.get_eff_stat(&Stat::MGT), 11);
    }

    #[test]
    fn empower_weaken_cancellation_overflow() {
        let config = make_config(vec![(Stat::MGT, 10)]);
        let mut state = CharacterState::from_config(0, &config);

        let emp_key = status_key("Empower", Some(&Stat::MGT));
        let weak_key = status_key("Weaken", Some(&Stat::MGT));

        state.add_status(&emp_key, 2, 99, &empower_def(), Some(Stat::MGT));

        // Apply 5 Weaken — cancels 2 Empower, leaves 3 Weaken
        state.add_status(&weak_key, 5, 99, &weaken_def(), Some(Stat::MGT));
        assert!(!state.has_status(&emp_key));
        assert_eq!(state.status_stacks(&weak_key), 3);
        assert_eq!(state.get_eff_stat(&Stat::MGT), 7);
    }

    #[test]
    fn multiple_stat_empower_simultaneously() {
        let config = make_config(vec![(Stat::MGT, 10), (Stat::ARM, 5)]);
        let mut state = CharacterState::from_config(0, &config);

        let str_key = status_key("Empower", Some(&Stat::MGT));
        let for_key = status_key("Empower", Some(&Stat::ARM));

        state.add_status(&str_key, 3, 99, &empower_def(), Some(Stat::MGT));
        state.add_status(&for_key, 2, 99, &empower_def(), Some(Stat::ARM));

        assert_eq!(state.get_eff_stat(&Stat::MGT), 13);
        assert_eq!(state.get_eff_stat(&Stat::ARM), 7);
    }

    #[test]
    fn permanent_status_never_decays() {
        let config = make_config(vec![(Stat::ARM, 5)]);
        let mut state = CharacterState::from_config(0, &config);

        let key = status_key("Fortify", Some(&Stat::ARM));
        state.add_status(&key, 2, 99, &fortify_def(), Some(Stat::ARM));

        state.tick_statuses();
        state.tick_statuses();
        state.tick_statuses();

        assert_eq!(state.status_stacks(&key), 2);
        assert_eq!(state.get_eff_stat(&Stat::ARM), 7);
    }

    #[test]
    fn batch_resolve_bleed_and_regen_survive() {
        // 1 HP, 1 bleed (1 dmg), 3 regen (6 heal). Should survive.
        let config = make_config(vec![(Stat::VIT, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.take_damage(29); // 1 HP
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
        let config = make_config(vec![(Stat::VIT, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        assert!(!state.is_incapacitated());

        state.add_status("Stun", 2, 99, &stun_def(), None);
        assert!(state.is_incapacitated());
    }

    #[test]
    fn stun_is_consumed_when_action_is_skipped() {
        let config = make_config(vec![(Stat::VIT, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_status("Stun", 1, 99, &stun_def(), None);
        assert!(state.is_incapacitated());

        state.tick_statuses(); // should not remove stun on its own
        assert!(state.is_incapacitated());

        state.consume_skip_turn_statuses();
        assert!(!state.is_incapacitated());
    }

    #[test]
    fn empower_persists_and_does_not_decay() {
        let config = make_config(vec![(Stat::MGT, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        let key = status_key("Empower", Some(&Stat::MGT));
        state.add_status(&key, 3, 99, &empower_def(), Some(Stat::MGT));

        // Empower is now Permanent: neither start-of-turn nor end-of-turn decay
        // touches it.
        assert_eq!(state.get_eff_stat(&Stat::MGT), 13);
        state.tick_statuses();
        assert_eq!(state.get_eff_stat(&Stat::MGT), 13);
        state.decay_statuses_end_of_turn();
        assert_eq!(state.get_eff_stat(&Stat::MGT), 13);
        state.tick_statuses();
        state.decay_statuses_end_of_turn();
        assert_eq!(state.status_stacks(&key), 3);
        assert_eq!(state.get_eff_stat(&Stat::MGT), 13);
    }

    #[test]
    fn empower_caps_at_eight_stacks() {
        let config = make_config(vec![(Stat::MGT, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        let key = status_key("Empower", Some(&Stat::MGT));

        // Applying past the cap clamps stacks to MAX_STAT_MOD_STACKS (8).
        state.add_status(&key, 5, 99, &empower_def(), Some(Stat::MGT));
        state.add_status(&key, 5, 99, &empower_def(), Some(Stat::MGT));
        assert_eq!(state.status_stacks(&key), MAX_STAT_MOD_STACKS);
        assert_eq!(state.get_eff_stat(&Stat::MGT), 10 + MAX_STAT_MOD_STACKS);
    }

    #[test]
    fn restoration_halves_on_turn_start() {
        let config = make_config(vec![(Stat::VIT, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.take_damage(15);
        state.add_status("Restoration", 5, 99, &restoration_def(), None);

        let ticks = state.tick_statuses();

        assert_eq!(ticks.len(), 1);
        assert!(matches!(
            &ticks[0],
            StatusTick::HealApplied { name, amount, .. } if name == "Restoration" && *amount == 5
        ));
        assert_eq!(state.current_hp(), 20);
        assert_eq!(state.status_stacks("Restoration"), 2);
    }

    #[test]
    fn lethality_halves_at_end_of_turn() {
        let config = make_config(vec![(Stat::MGT, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_status("Lethality", 5, 99, &lethality_def(), None);

        state.decay_statuses_end_of_turn();
        assert_eq!(state.status_stacks("Lethality"), 2);

        state.decay_statuses_end_of_turn();
        assert_eq!(state.status_stacks("Lethality"), 1);

        state.decay_statuses_end_of_turn();
        assert!(!state.has_status("Lethality"));
    }

    #[test]
    fn remove_status_partial() {
        let config = make_config(vec![(Stat::VIT, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_status("Bleed", 5, 99, &bleed_def(), None);
        state.remove_status("Bleed", 2);
        assert_eq!(state.status_stacks("Bleed"), 3);
    }

    #[test]
    fn remove_status_full() {
        let config = make_config(vec![(Stat::VIT, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_status("Bleed", 3, 99, &bleed_def(), None);
        state.remove_status("Bleed", 5);
        assert!(!state.has_status("Bleed"));
    }

    #[test]
    fn stunned_condition_does_not_stack() {
        let config = make_config(vec![(Stat::VIT, 10)]);
        let mut state = CharacterState::from_config(0, &config);

        assert!(state.add_condition(ConditionKind::Stunned, 1, 99));
        assert_eq!(state.condition_stacks(ConditionKind::Stunned), 1);
        assert!(state.add_condition(ConditionKind::Stunned, 3, 99));
        assert_eq!(state.condition_stacks(ConditionKind::Stunned), 1);
    }

    #[test]
    fn marked_condition_is_non_stacking() {
        let config = make_config(vec![(Stat::VIT, 10)]);
        let mut state = CharacterState::from_config(0, &config);

        assert!(state.add_condition(ConditionKind::Marked, 2, 99));
        assert!(state.add_condition(ConditionKind::Marked, 3, 99));
        assert_eq!(state.condition_stacks(ConditionKind::Marked), 1);
    }

    #[test]
    fn conditions_decay_at_end_of_turn() {
        let config = make_config(vec![(Stat::VIT, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_condition(ConditionKind::Stunned, 1, 99);
        state.add_condition(ConditionKind::Marked, 2, 99);
        state.add_condition(ConditionKind::Severed, 3, 99);

        state.decay_conditions_end_of_turn();

        assert!(!state.has_condition(ConditionKind::Stunned));
        assert_eq!(state.condition_stacks(ConditionKind::Marked), 1);
        assert_eq!(state.condition_stacks(ConditionKind::Severed), 2);
    }

    #[test]
    fn haste_reduces_live_countdown() {
        let config = make_config(vec![(Stat::SPD, 8), (Stat::VIT, 10)]);
        let mut state = CharacterState::from_config(0, &config);

        state.apply_haste(3);

        assert_eq!(state.ticks_until_turn, 4);
    }

    #[test]
    fn haste_applies_after_turn_reset_when_gained_on_turn() {
        let config = make_config(vec![(Stat::SPD, 8), (Stat::VIT, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.ticks_until_turn = 0;

        state.apply_haste(3);
        state.reset_speed();

        assert_eq!(state.ticks_until_turn, 6);
    }

    #[test]
    fn cleanse_respects_status_group_filter() {
        // Weaken is now Permanent (cleanse only reduces TickDown effects), so this
        // exercises the group filter with two TickDown debuffs in distinct groups.
        let body_debuff = StatusDef {
            behavior: StatusBehavior::DamagePerStack { value: 1 },
            stack_type: StackType::TickDown,
            group: Some(StatusGroup::Body),
            opposes: None,
        };
        let mind_debuff = StatusDef {
            behavior: StatusBehavior::DamagePerStack { value: 1 },
            stack_type: StackType::TickDown,
            group: Some(StatusGroup::Mind),
            opposes: None,
        };
        let config = make_config(vec![(Stat::VIT, 10)]);
        let mut state = CharacterState::from_config(0, &config);

        state.add_status("BodyDebuff", 2, 99, &body_debuff, None);
        state.add_status("MindDebuff", 2, 99, &mind_debuff, None);

        assert!(state.cleanse(1, Some(StatusGroup::Body)));
        assert_eq!(state.status_stacks("BodyDebuff"), 1);
        assert_eq!(state.status_stacks("MindDebuff"), 2);
    }

    #[test]
    fn position_validity() {
        assert!(Position { row: 0, col: 0 }.is_valid());
        assert!(Position { row: 2, col: 2 }.is_valid());
        assert!(!Position { row: 3, col: 0 }.is_valid());
        assert!(!Position { row: 0, col: 3 }.is_valid());
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
        let config = make_config(vec![]);
        let mut state = CharacterState::from_config(0, &config);
        assert_eq!(state.mp_cost_reduction(), 0);

        state.add_trait(TraitEffect::MpCostReduction { amount: 2 });
        assert_eq!(state.mp_cost_reduction(), 2);
    }

    #[test]
    fn damage_reflect_sums_amounts() {
        let config = make_config(vec![(Stat::VIT, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        assert_eq!(state.damage_reflect_amount(), 0);

        state.add_trait(TraitEffect::DamageReflect { amount: 3 });
        assert_eq!(state.damage_reflect_amount(), 3);
    }

    #[test]
    fn aura_stat_mod_trait_affects_effective_stat() {
        let config = make_config(vec![(Stat::MGT, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_trait(TraitEffect::AuraStatMod {
            stat: Stat::MGT,
            amount: 2,
        });
        assert_eq!(state.get_eff_stat(&Stat::MGT), 12);
        state.clear_aura_traits();
        assert_eq!(state.get_eff_stat(&Stat::MGT), 10);
    }

    #[test]
    fn debuff_resistance_negates_first_n_debuffs() {
        let config = make_config(vec![(Stat::VIT, 10), (Stat::MGT, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_trait(TraitEffect::DebuffResistance { count: 2 });

        // First debuff: negated
        state.add_status("Bleed", 3, 99, &bleed_def(), None);
        assert!(!state.has_status("Bleed"));

        // Second debuff: negated
        let weak_key = status_key("Weaken", Some(&Stat::MGT));
        state.add_status(&weak_key, 2, 99, &weaken_def(), Some(Stat::MGT));
        assert!(!state.has_status(&weak_key));

        // Third debuff: goes through
        state.add_status("Bleed", 3, 99, &bleed_def(), None);
        assert_eq!(state.status_stacks("Bleed"), 3);
    }

    #[test]
    fn debuff_resistance_allows_buffs_through() {
        let config = make_config(vec![(Stat::VIT, 10), (Stat::MGT, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_trait(TraitEffect::DebuffResistance { count: 1 });

        // Buff should not consume a charge
        let emp_key = status_key("Empower", Some(&Stat::MGT));
        state.add_status(&emp_key, 3, 99, &empower_def(), Some(Stat::MGT));
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
        let config = make_config(vec![(Stat::VIT, 10)]);
        let mut state = CharacterState::from_config(0, &config);
        state.add_trait(TraitEffect::DebuffResistance { count: 1 });

        state.add_status("Stun", 2, 99, &stun_def(), None);
        assert!(!state.is_incapacitated()); // negated
    }
}
