//! Battle event logging with JSON serialization.

use std::collections::HashMap;
use std::fmt::Write;

use serde::Serialize;
use serde_json::json;

use crate::models::{CharacterConfig, CharacterState, Stat};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event_type")]
pub enum BattleEvent {
    BattleStart {
        tick_count: u32,
        team_a: Vec<String>,
        team_b: Vec<String>,
    },
    TurnStart {
        tick_count: u32,
        actor_id: u32,
        actor_name: String,
        current_hp: u32,
        current_mp: u32,
    },
    BasicAttack {
        tick_count: u32,
        actor_id: u32,
        actor_name: String,
        target_id: u32,
        target_name: String,
        damage: u32,
        target_hp_remaining: u32,
    },
    #[allow(dead_code)]
    Rest {
        tick_count: u32,
        actor_id: u32,
        actor_name: String,
        mp_restored: u32,
        mp_after: u32,
    },
    AbilityUsed {
        tick_count: u32,
        actor_id: u32,
        actor_name: String,
        ability_name: String,
        mp_cost: u32,
    },
    AbilityDamage {
        tick_count: u32,
        actor_id: u32,
        target_id: u32,
        target_name: String,
        damage: u32,
        target_hp_remaining: u32,
    },
    StatusApplied {
        tick_count: u32,
        actor_id: u32,
        actor_name: String,
        target_id: u32,
        target_name: String,
        status_name: String,
        stacks_added: u32,
        stacks_after: u32,
    },
    Defeat {
        tick_count: u32,
        character_id: u32,
        character_name: String,
    },
    StatusDamage {
        tick_count: u32,
        character_id: u32,
        character_name: String,
        status_name: String,
        damage: u32,
        hp_remaining: u32,
    },
    StatusHeal {
        tick_count: u32,
        character_id: u32,
        character_name: String,
        status_name: String,
        amount: u32,
        hp_remaining: u32,
    },
    TurnSkipped {
        tick_count: u32,
        character_id: u32,
        character_name: String,
        reason: String,
    },
    Retargeted {
        tick_count: u32,
        character_id: u32,
        character_name: String,
        new_target_id: Option<u32>,
        new_target_name: Option<String>,
        mode: String,
    },
    Moved {
        tick_count: u32,
        character_id: u32,
        character_name: String,
        from_row: u8,
        from_col: u8,
        to_row: u8,
        to_col: u8,
    },
    ResourceChanged {
        tick_count: u32,
        actor_id: u32,
        actor_name: String,
        resource: String,
        delta: i32,
        value_after: u32,
        reason: String,
    },
    PassiveTriggered {
        tick_count: u32,
        character_id: u32,
        character_name: String,
        passive_name: String,
    },
    DamageReflect {
        tick_count: u32,
        reflector_id: u32,
        reflector_name: String,
        target_id: u32,
        target_name: String,
        damage: u32,
        target_hp_remaining: u32,
    },
    BattleEnd {
        tick_count: u32,
        winner: String,
    },
}

pub struct BattleLog {
    events: Vec<BattleEvent>,
}

impl BattleLog {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn push(&mut self, event: BattleEvent) {
        self.events.push(event);
    }

    pub fn push_turn_start(&mut self, tick_count: u32, actor: &CharacterState) {
        self.push(BattleEvent::TurnStart {
            tick_count,
            actor_id: actor.id(),
            actor_name: actor.base_name().to_string(),
            current_hp: actor.current_hp(),
            current_mp: actor.current_mp(),
        });
    }

    #[allow(dead_code)]
    pub fn push_rest(&mut self, tick_count: u32, actor: &CharacterState, mp_restored: u32) {
        self.push(BattleEvent::Rest {
            tick_count,
            actor_id: actor.id(),
            actor_name: actor.base_name().to_string(),
            mp_restored,
            mp_after: actor.current_mp(),
        });
    }

    pub fn push_turn_skipped(&mut self, tick_count: u32, actor: &CharacterState, reason: &str) {
        self.push(BattleEvent::TurnSkipped {
            tick_count,
            character_id: actor.id(),
            character_name: actor.base_name().to_string(),
            reason: reason.to_string(),
        });
    }

    pub fn push_passive_triggered(
        &mut self,
        tick_count: u32,
        actor: &CharacterState,
        passive_name: &str,
    ) {
        self.push(BattleEvent::PassiveTriggered {
            tick_count,
            character_id: actor.id(),
            character_name: actor.base_name().to_string(),
            passive_name: passive_name.to_string(),
        });
    }

    pub fn push_defeat(&mut self, tick_count: u32, actor: &CharacterState) {
        self.push(BattleEvent::Defeat {
            tick_count,
            character_id: actor.id(),
            character_name: actor.base_name().to_string(),
        });
    }

    pub fn push_status_damage(
        &mut self,
        tick_count: u32,
        actor: &CharacterState,
        status_name: &str,
        damage: u32,
    ) {
        self.push(BattleEvent::StatusDamage {
            tick_count,
            character_id: actor.id(),
            character_name: actor.base_name().to_string(),
            status_name: status_name.to_string(),
            damage,
            hp_remaining: actor.current_hp(),
        });
    }

    pub fn push_status_heal(
        &mut self,
        tick_count: u32,
        actor: &CharacterState,
        status_name: &str,
        amount: u32,
    ) {
        self.push(BattleEvent::StatusHeal {
            tick_count,
            character_id: actor.id(),
            character_name: actor.base_name().to_string(),
            status_name: status_name.to_string(),
            amount,
            hp_remaining: actor.current_hp(),
        });
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn events_from(&self, start: usize) -> &[BattleEvent] {
        &self.events[start.min(self.events.len())..]
    }

    #[cfg(test)]
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.events).unwrap()
    }

    pub fn to_replay_json(
        &self,
        seed: u64,
        team_a_name: &str,
        team_b_name: &str,
        team_a_configs: &[CharacterConfig],
        team_b_configs: &[CharacterConfig],
    ) -> String {
        let id_map = build_character_id_map(team_a_configs, team_b_configs);
        let team_a_snapshot = build_team_snapshot(team_a_name, team_a_configs, "team_a");
        let team_b_snapshot = build_team_snapshot(team_b_name, team_b_configs, "team_b");
        let mut last_ability_by_actor: HashMap<u32, String> = HashMap::new();
        let mut replay_events = Vec::new();

        for event in &self.events {
            match event {
                BattleEvent::BattleStart { tick_count, .. } => replay_events.push(json!({
                    "tick": tick_count,
                    "type": "battle_start",
                })),
                BattleEvent::TurnStart {
                    tick_count,
                    actor_id,
                    current_hp,
                    current_mp,
                    ..
                } => replay_events.push(json!({
                    "tick": tick_count,
                    "type": "turn_start",
                    "actor_id": stable_id(*actor_id, &id_map),
                    "current_hp": current_hp,
                    "current_mp": current_mp,
                })),
                BattleEvent::BasicAttack {
                    tick_count,
                    actor_id,
                    target_id,
                    damage,
                    target_hp_remaining,
                    ..
                } => {
                    replay_events.push(json!({
                        "tick": tick_count,
                        "type": "basic_attack",
                        "actor_id": stable_id(*actor_id, &id_map),
                        "target_id": stable_id(*target_id, &id_map),
                        "damage_kind": "basic",
                    }));
                    replay_events.push(json!({
                        "tick": tick_count,
                        "type": "damage",
                        "source_id": stable_id(*actor_id, &id_map),
                        "target_id": stable_id(*target_id, &id_map),
                        "amount": damage,
                        "damage_kind": "basic",
                        "source_kind": "basic_attack",
                        "source_name": "Basic Attack",
                        "target_hp_after": target_hp_remaining,
                    }));
                }
                BattleEvent::Rest {
                    tick_count,
                    actor_id,
                    mp_restored,
                    mp_after,
                    ..
                } => replay_events.push(json!({
                    "tick": tick_count,
                    "type": "rest",
                    "actor_id": stable_id(*actor_id, &id_map),
                    "mp_restored": mp_restored,
                    "mp_after": mp_after,
                })),
                BattleEvent::AbilityUsed {
                    tick_count,
                    actor_id,
                    ability_name,
                    mp_cost,
                    ..
                } => {
                    last_ability_by_actor.insert(*actor_id, ability_name.clone());
                    replay_events.push(json!({
                        "tick": tick_count,
                        "type": "ability_used",
                        "actor_id": stable_id(*actor_id, &id_map),
                        "ability": ability_name,
                        "mp_cost": mp_cost,
                    }));
                }
                BattleEvent::AbilityDamage {
                    tick_count,
                    actor_id,
                    target_id,
                    damage,
                    target_hp_remaining,
                    ..
                } => replay_events.push(json!({
                    "tick": tick_count,
                    "type": "damage",
                    "source_id": stable_id(*actor_id, &id_map),
                    "target_id": stable_id(*target_id, &id_map),
                    "amount": damage,
                    "damage_kind": "ability",
                    "source_kind": "ability",
                    "source_name": last_ability_by_actor
                        .get(actor_id)
                        .cloned()
                        .unwrap_or_else(|| "Ability".to_string()),
                    "target_hp_after": target_hp_remaining,
                })),
                BattleEvent::StatusApplied {
                    tick_count,
                    actor_id,
                    target_id,
                    status_name,
                    stacks_added,
                    stacks_after,
                    ..
                } => replay_events.push(json!({
                    "tick": tick_count,
                    "type": "status_applied",
                    "source_id": stable_id(*actor_id, &id_map),
                    "target_id": stable_id(*target_id, &id_map),
                    "status": status_name,
                    "stacks_added": stacks_added,
                    "stacks_after": stacks_after,
                })),
                BattleEvent::Defeat {
                    tick_count,
                    character_id,
                    ..
                } => replay_events.push(json!({
                    "tick": tick_count,
                    "type": "defeat",
                    "actor_id": stable_id(*character_id, &id_map),
                })),
                BattleEvent::StatusDamage {
                    tick_count,
                    character_id,
                    status_name,
                    damage,
                    hp_remaining,
                    ..
                } => replay_events.push(json!({
                    "tick": tick_count,
                    "type": "status_tick",
                    "target_id": stable_id(*character_id, &id_map),
                    "status": status_name,
                    "amount": damage,
                    "kind": "damage",
                    "target_hp_after": hp_remaining,
                })),
                BattleEvent::StatusHeal {
                    tick_count,
                    character_id,
                    status_name,
                    amount,
                    hp_remaining,
                    ..
                } => replay_events.push(json!({
                    "tick": tick_count,
                    "type": "status_tick",
                    "target_id": stable_id(*character_id, &id_map),
                    "status": status_name,
                    "amount": amount,
                    "kind": "heal",
                    "target_hp_after": hp_remaining,
                })),
                BattleEvent::TurnSkipped {
                    tick_count,
                    character_id,
                    reason,
                    ..
                } => replay_events.push(json!({
                    "tick": tick_count,
                    "type": "turn_skipped",
                    "actor_id": stable_id(*character_id, &id_map),
                    "reason": reason,
                })),
                BattleEvent::Retargeted {
                    tick_count,
                    character_id,
                    new_target_id,
                    mode,
                    ..
                } => replay_events.push(json!({
                    "tick": tick_count,
                    "type": "retargeted",
                    "actor_id": stable_id(*character_id, &id_map),
                    "new_target_id": new_target_id.map(|id| stable_id(id, &id_map)),
                    "mode": mode,
                })),
                BattleEvent::Moved {
                    tick_count,
                    character_id,
                    to_row,
                    to_col,
                    ..
                } => replay_events.push(json!({
                    "tick": tick_count,
                    "type": "moved",
                    "actor_id": stable_id(*character_id, &id_map),
                    "to_row": to_row,
                    "to_col": to_col,
                })),
                BattleEvent::ResourceChanged {
                    tick_count,
                    actor_id,
                    resource,
                    delta,
                    value_after,
                    reason,
                    ..
                } => replay_events.push(json!({
                    "tick": tick_count,
                    "type": "resource_changed",
                    "actor_id": stable_id(*actor_id, &id_map),
                    "resource": resource,
                    "delta": delta,
                    "value_after": value_after,
                    "reason": reason,
                })),
                BattleEvent::PassiveTriggered {
                    tick_count,
                    character_id,
                    passive_name,
                    ..
                } => replay_events.push(json!({
                    "tick": tick_count,
                    "type": "passive_triggered",
                    "actor_id": stable_id(*character_id, &id_map),
                    "passive": passive_name,
                })),
                BattleEvent::DamageReflect {
                    tick_count,
                    reflector_id,
                    target_id,
                    damage,
                    target_hp_remaining,
                    ..
                } => replay_events.push(json!({
                    "tick": tick_count,
                    "type": "damage",
                    "source_id": stable_id(*reflector_id, &id_map),
                    "target_id": stable_id(*target_id, &id_map),
                    "amount": damage,
                    "damage_kind": "reflect",
                    "source_kind": "reflect",
                    "source_name": "Reflect",
                    "target_hp_after": target_hp_remaining,
                })),
                BattleEvent::BattleEnd { tick_count, winner } => replay_events.push(json!({
                    "tick": tick_count,
                    "type": "battle_end",
                    "winner": winner,
                })),
            }
        }

        let winner = self
            .events
            .iter()
            .rev()
            .find_map(|event| match event {
                BattleEvent::BattleEnd { winner, .. } => Some(winner.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "draw".to_string());
        let tick_count = self.events.last().map(BattleEvent::tick_count).unwrap_or(0);

        serde_json::to_string_pretty(&json!({
            "version": 1,
            "seed": seed,
            "winner": winner,
            "tick_count": tick_count,
            "teams": {
                "team_a": team_a_snapshot,
                "team_b": team_b_snapshot,
            },
            "events": replay_events,
        }))
        .unwrap()
    }

    pub fn to_text(&self) -> String {
        let mut out = String::new();
        let mut current_tick: Option<u32> = None;

        for event in &self.events {
            let tick_count = event.tick_count();
            if current_tick != Some(tick_count) {
                if !out.is_empty() {
                    out.push('\n');
                }
                let _ = writeln!(out, "Tick {tick_count}");
                current_tick = Some(tick_count);
            }

            match event {
                BattleEvent::BattleStart { team_a, team_b, .. } => {
                    let _ = writeln!(out, "  Team A: {}", team_a.join(", "));
                    let _ = writeln!(out, "  Team B: {}", team_b.join(", "));
                }
                BattleEvent::TurnStart {
                    actor_name,
                    current_hp,
                    current_mp,
                    ..
                } => {
                    let _ = writeln!(
                        out,
                        "  {actor_name} starts the turn ({current_hp} HP, {current_mp} MP)"
                    );
                }
                BattleEvent::BasicAttack {
                    actor_name,
                    target_name,
                    damage,
                    target_hp_remaining,
                    ..
                } => {
                    let _ = writeln!(
                        out,
                        "  {actor_name} basic attacks {target_name} for {damage} ({target_hp_remaining} HP left)"
                    );
                }
                BattleEvent::Rest {
                    actor_name,
                    mp_restored,
                    mp_after,
                    ..
                } => {
                    let _ = writeln!(
                        out,
                        "  {actor_name} rests and restores {mp_restored} MP ({mp_after} MP total)"
                    );
                }
                BattleEvent::AbilityUsed {
                    actor_name,
                    ability_name,
                    mp_cost,
                    ..
                } => {
                    let _ = writeln!(
                        out,
                        "  {actor_name} uses {ability_name} (costs {mp_cost} MP)"
                    );
                }
                BattleEvent::AbilityDamage {
                    target_name,
                    damage,
                    target_hp_remaining,
                    ..
                } => {
                    let _ = writeln!(
                        out,
                        "    hits {target_name} for {damage} ({target_hp_remaining} HP left)"
                    );
                }
                BattleEvent::StatusApplied {
                    actor_name,
                    target_name,
                    status_name,
                    stacks_added,
                    stacks_after,
                    ..
                } => {
                    let _ = writeln!(
                        out,
                        "  {actor_name} applies {status_name} to {target_name} (+{stacks_added}, {stacks_after} total)"
                    );
                }
                BattleEvent::Defeat { character_name, .. } => {
                    let _ = writeln!(out, "  {character_name} is defeated");
                }
                BattleEvent::StatusDamage {
                    character_name,
                    status_name,
                    damage,
                    hp_remaining,
                    ..
                } => {
                    let _ = writeln!(
                        out,
                        "  {character_name} takes {damage} from {status_name} ({hp_remaining} HP left)"
                    );
                }
                BattleEvent::StatusHeal {
                    character_name,
                    status_name,
                    amount,
                    hp_remaining,
                    ..
                } => {
                    let _ = writeln!(
                        out,
                        "  {character_name} heals {amount} from {status_name} ({hp_remaining} HP left)"
                    );
                }
                BattleEvent::TurnSkipped {
                    character_name,
                    reason,
                    ..
                } => {
                    let _ = writeln!(out, "  {character_name} skips the turn ({reason})");
                }
                BattleEvent::Retargeted {
                    character_name,
                    new_target_name,
                    mode,
                    ..
                } => {
                    let target_label = new_target_name.as_deref().unwrap_or("no target");
                    let _ = writeln!(
                        out,
                        "  {character_name} retargets to {target_label} ({mode})"
                    );
                }
                BattleEvent::Moved {
                    character_name,
                    from_row,
                    from_col,
                    to_row,
                    to_col,
                    ..
                } => {
                    let _ = writeln!(
                        out,
                        "  {character_name} moves from (r{from_row}, c{from_col}) to (r{to_row}, c{to_col})"
                    );
                }
                BattleEvent::ResourceChanged {
                    actor_name,
                    resource,
                    delta,
                    value_after,
                    reason,
                    ..
                } => {
                    let _ = writeln!(
                        out,
                        "  {actor_name} gains {delta} {resource} ({value_after} after, {reason})"
                    );
                }
                BattleEvent::PassiveTriggered {
                    character_name,
                    passive_name,
                    ..
                } => {
                    let _ = writeln!(out, "  {character_name} triggers {passive_name}");
                }
                BattleEvent::DamageReflect {
                    reflector_name,
                    target_name,
                    damage,
                    target_hp_remaining,
                    ..
                } => {
                    let _ = writeln!(
                        out,
                        "  {reflector_name} reflects {damage} to {target_name} ({target_hp_remaining} HP left)"
                    );
                }
                BattleEvent::BattleEnd { winner, .. } => {
                    let _ = writeln!(out, "  Battle ends: {winner}");
                }
            }
        }

        out
    }

    #[cfg(test)]
    pub fn events(&self) -> &[BattleEvent] {
        &self.events
    }
}

impl BattleEvent {
    pub fn tick_count(&self) -> u32 {
        match self {
            BattleEvent::BattleStart { tick_count, .. }
            | BattleEvent::TurnStart { tick_count, .. }
            | BattleEvent::BasicAttack { tick_count, .. }
            | BattleEvent::Rest { tick_count, .. }
            | BattleEvent::AbilityUsed { tick_count, .. }
            | BattleEvent::AbilityDamage { tick_count, .. }
            | BattleEvent::StatusApplied { tick_count, .. }
            | BattleEvent::Defeat { tick_count, .. }
            | BattleEvent::StatusDamage { tick_count, .. }
            | BattleEvent::StatusHeal { tick_count, .. }
            | BattleEvent::TurnSkipped { tick_count, .. }
            | BattleEvent::Retargeted { tick_count, .. }
            | BattleEvent::Moved { tick_count, .. }
            | BattleEvent::ResourceChanged { tick_count, .. }
            | BattleEvent::PassiveTriggered { tick_count, .. }
            | BattleEvent::DamageReflect { tick_count, .. }
            | BattleEvent::BattleEnd { tick_count, .. } => *tick_count,
        }
    }
}

fn build_character_id_map(
    team_a_configs: &[CharacterConfig],
    team_b_configs: &[CharacterConfig],
) -> HashMap<u32, String> {
    let mut id_map = HashMap::new();
    for (idx, config) in team_a_configs.iter().enumerate() {
        id_map.insert(idx as u32, replay_character_id(config, "team_a", idx));
    }
    let offset = team_a_configs.len() as u32;
    for (idx, config) in team_b_configs.iter().enumerate() {
        id_map.insert(offset + idx as u32, replay_character_id(config, "team_b", idx));
    }
    id_map
}

fn build_team_snapshot(
    name: &str,
    configs: &[CharacterConfig],
    team_key: &str,
) -> serde_json::Value {
    json!({
        "name": name,
        "characters": configs.iter().enumerate().map(|(idx, config)| {
            let max_hp = config.stats.get(&Stat::VIT).copied().unwrap_or(0) * 2;
            let max_mp = config.stats.get(&Stat::WIL).copied().unwrap_or(0);
            json!({
                "id": replay_character_id(config, team_key, idx),
                "display_name": replay_display_name(config),
                "position": config.position,
                "max_hp": max_hp,
                "max_mp": max_mp,
                "stats": config.stats,
                "passive": if config.passive.is_empty() { None::<String> } else { Some(config.passive.clone()) },
                "actives": config.actives,
            })
        }).collect::<Vec<_>>()
    })
}

fn replay_character_id(config: &CharacterConfig, team_key: &str, index: usize) -> String {
    config
        .id
        .clone()
        .filter(|id| !id.trim().is_empty())
        .unwrap_or_else(|| format!("{team_key}_{index}"))
}

fn replay_display_name(config: &CharacterConfig) -> String {
    config
        .display_name
        .clone()
        .unwrap_or_else(|| config.base_name.clone())
}

fn stable_id(raw_id: u32, id_map: &HashMap<u32, String>) -> String {
    id_map
        .get(&raw_id)
        .cloned()
        .unwrap_or_else(|| format!("character_{raw_id}"))
}
