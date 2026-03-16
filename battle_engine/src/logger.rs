//! Battle event logging with JSON serialization.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event_type")]
pub enum BattleEvent {
    BattleStart {
        step: u32,
        team_a: Vec<String>,
        team_b: Vec<String>,
    },
    BasicAttack {
        step: u32,
        actor_id: u32,
        actor_name: String,
        target_id: u32,
        target_name: String,
        damage: u32,
        target_hp_remaining: u32,
    },
    AbilityUsed {
        step: u32,
        actor_id: u32,
        actor_name: String,
        ability_name: String,
        spi_cost: u32,
    },
    AbilityDamage {
        step: u32,
        actor_id: u32,
        target_id: u32,
        target_name: String,
        damage: u32,
        target_hp_remaining: u32,
    },
    Defeat {
        step: u32,
        character_id: u32,
        character_name: String,
    },
    BattleEnd {
        step: u32,
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

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.events).unwrap()
    }

    pub fn events(&self) -> &[BattleEvent] {
        &self.events
    }
}
