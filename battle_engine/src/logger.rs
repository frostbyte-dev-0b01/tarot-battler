//! Battle event logging with JSON serialization.

use std::fmt::Write;

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
    StatusDamage {
        step: u32,
        character_id: u32,
        character_name: String,
        status_name: String,
        damage: u32,
        hp_remaining: u32,
    },
    StatusHeal {
        step: u32,
        character_id: u32,
        character_name: String,
        status_name: String,
        amount: u32,
        hp_remaining: u32,
    },
    TurnSkipped {
        step: u32,
        character_id: u32,
        character_name: String,
        reason: String,
    },
    PassiveTriggered {
        step: u32,
        character_id: u32,
        character_name: String,
        passive_name: String,
    },
    DamageReflect {
        step: u32,
        reflector_id: u32,
        reflector_name: String,
        target_id: u32,
        target_name: String,
        damage: u32,
        target_hp_remaining: u32,
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

    pub fn to_text(&self) -> String {
        let mut out = String::new();
        let mut current_step: Option<u32> = None;

        for event in &self.events {
            let step = event.step();
            if current_step != Some(step) {
                if !out.is_empty() {
                    out.push('\n');
                }
                let _ = writeln!(out, "Step {step}");
                current_step = Some(step);
            }

            match event {
                BattleEvent::BattleStart { team_a, team_b, .. } => {
                    let _ = writeln!(out, "  Team A: {}", team_a.join(", "));
                    let _ = writeln!(out, "  Team B: {}", team_b.join(", "));
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
                BattleEvent::AbilityUsed {
                    actor_name,
                    ability_name,
                    spi_cost,
                    ..
                } => {
                    let _ = writeln!(
                        out,
                        "  {actor_name} uses {ability_name} (costs {spi_cost} SPI)"
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
    fn step(&self) -> u32 {
        match self {
            BattleEvent::BattleStart { step, .. }
            | BattleEvent::BasicAttack { step, .. }
            | BattleEvent::AbilityUsed { step, .. }
            | BattleEvent::AbilityDamage { step, .. }
            | BattleEvent::Defeat { step, .. }
            | BattleEvent::StatusDamage { step, .. }
            | BattleEvent::StatusHeal { step, .. }
            | BattleEvent::TurnSkipped { step, .. }
            | BattleEvent::PassiveTriggered { step, .. }
            | BattleEvent::DamageReflect { step, .. }
            | BattleEvent::BattleEnd { step, .. } => *step,
        }
    }
}
