//! Tarot Battler battle engine library.
//!
//! Exposes the engine modules plus [`run_battle_json`], a self-contained entry
//! point that runs a battle from two team-config JSON strings and returns
//! replay-schema JSON. The bundled content data is embedded with `include_str!`
//! so the engine can run with no filesystem — e.g. compiled to WebAssembly for
//! the browser dev tool.

pub mod abilities;
pub mod abilities_targeting;
pub mod engine;
pub mod loader;
pub mod logger;
pub mod models;
pub mod passive_system;
pub mod rules;
pub mod statuses;
pub mod targeting;
pub mod turns;

#[cfg(test)]
mod test_support;

use abilities::{AbilityMap, PassiveMap};
use loader::{ArchetypeMap, AspectMap, TeamConfig};
use statuses::StatusMap;

/// Bundled content data, embedded at compile time so the engine is fully
/// self-contained (no filesystem reads at runtime).
pub const ARCHETYPES_JSON: &str = include_str!("data/archetypes.json");
pub const ABILITIES_JSON: &str = include_str!("data/abilities.json");
pub const PASSIVES_JSON: &str = include_str!("data/passives.json");
pub const STATUSES_JSON: &str = include_str!("data/statuses.json");
pub const ASPECTS_JSON: &str = include_str!("data/aspects.json");

/// Run a battle from two team-config JSON strings, using the embedded content
/// data, and return replay-schema JSON. Returns `Err` with a human-readable
/// message if either team fails to parse or validate.
pub fn run_battle_json(team_a_json: &str, team_b_json: &str, seed: u64) -> Result<String, String> {
    let archetypes: ArchetypeMap =
        serde_json::from_str(ARCHETYPES_JSON).map_err(|e| format!("bundled archetypes: {e}"))?;
    let abilities: AbilityMap =
        serde_json::from_str(ABILITIES_JSON).map_err(|e| format!("bundled abilities: {e}"))?;
    let passives: PassiveMap =
        serde_json::from_str(PASSIVES_JSON).map_err(|e| format!("bundled passives: {e}"))?;
    let statuses: StatusMap =
        serde_json::from_str(STATUSES_JSON).map_err(|e| format!("bundled statuses: {e}"))?;
    let aspects: AspectMap =
        serde_json::from_str(ASPECTS_JSON).map_err(|e| format!("bundled aspects: {e}"))?;

    let team_a_config: TeamConfig =
        serde_json::from_str(team_a_json).map_err(|e| format!("team A JSON: {e}"))?;
    let team_b_config: TeamConfig =
        serde_json::from_str(team_b_json).map_err(|e| format!("team B JSON: {e}"))?;

    let team_a = loader::validate_team_config(
        &team_a_config,
        &archetypes,
        &aspects,
        &abilities,
        &passives,
        &statuses,
    )
    .map_err(|e| format!("team A: {e}"))?;
    let team_b = loader::validate_team_config(
        &team_b_config,
        &archetypes,
        &aspects,
        &abilities,
        &passives,
        &statuses,
    )
    .map_err(|e| format!("team B: {e}"))?;
    loader::validate_teams(&team_a, &team_b, &abilities, &passives, &statuses)
        .map_err(|e| format!("battle content: {e}"))?;

    let battle = engine::BattleState::new(&team_a, &team_b, abilities, passives, statuses, seed);
    let log = battle.run();
    Ok(log.to_replay_json(seed, &team_a_config.name, &team_b_config.name, &team_a, &team_b))
}

/// WebAssembly entry point. Returns replay-schema JSON on success, or a
/// `{"error": "..."}` JSON object the UI can detect, so failures never throw.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn run_battle(team_a_json: &str, team_b_json: &str, seed: u32) -> String {
    match run_battle_json(team_a_json, team_b_json, seed as u64) {
        Ok(replay) => replay,
        Err(message) => {
            serde_json::json!({ "error": message }).to_string()
        }
    }
}
