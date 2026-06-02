//! Tarot Battler battle engine library.
//!
//! Exposes the engine modules plus [`run_battle_json`], a self-contained entry
//! point that runs a battle from two team-config JSON strings and returns
//! replay-schema JSON. The bundled content data is embedded with `include_str!`
//! so the engine can run with no filesystem — e.g. compiled to WebAssembly for
//! the browser dev tool.

pub mod abilities;
pub mod abilities_targeting;
pub mod banners;
pub mod engine;
pub mod loader;
pub mod logger;
pub mod models;
pub mod passive_system;
pub mod rules;
pub mod statuses;
pub mod targeting;
pub mod team_passives;
pub mod turns;

#[cfg(test)]
mod test_support;

use abilities::{AbilityMap, PassiveMap};
use banners::BannerMap;
use loader::{ArchetypeMap, AspectMap, TeamConfig};
use statuses::StatusMap;
use team_passives::TeamPassiveMap;

/// Bundled content data, embedded at compile time so the engine is fully
/// self-contained (no filesystem reads at runtime).
pub const ARCHETYPES_JSON: &str = include_str!("data/archetypes.json");
pub const ABILITIES_JSON: &str = include_str!("data/abilities.json");
pub const PASSIVES_JSON: &str = include_str!("data/passives.json");
pub const STATUSES_JSON: &str = include_str!("data/statuses.json");
pub const ASPECTS_JSON: &str = include_str!("data/aspects.json");
pub const TEAM_PASSIVES_JSON: &str = include_str!("data/team_passives.json");
pub const BANNERS_JSON: &str = include_str!("data/banners.json");

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
    let team_passive_catalog: TeamPassiveMap = serde_json::from_str(TEAM_PASSIVES_JSON)
        .map_err(|e| format!("bundled team passives: {e}"))?;
    let banner_catalog: BannerMap =
        serde_json::from_str(BANNERS_JSON).map_err(|e| format!("bundled banners: {e}"))?;

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

    let team_a_passives =
        team_passives::resolve(&team_a_config.team_passives, &team_passive_catalog)
            .map_err(|e| format!("team A: {e}"))?;
    let team_b_passives =
        team_passives::resolve(&team_b_config.team_passives, &team_passive_catalog)
            .map_err(|e| format!("team B: {e}"))?;

    let team_a_banner =
        resolve_banner(&team_a_config, &banner_catalog).map_err(|e| format!("team A: {e}"))?;
    let team_b_banner =
        resolve_banner(&team_b_config, &banner_catalog).map_err(|e| format!("team B: {e}"))?;

    let mut battle =
        engine::BattleState::new(&team_a, &team_b, abilities, passives, statuses, seed);
    battle.set_team_passives(team_a_passives, team_b_passives);
    battle.set_team_banners(team_a_banner, team_b_banner);
    let log = battle.run();
    Ok(log.to_replay_json(
        seed,
        &team_a_config.name,
        &team_b_config.name,
        &team_a,
        &team_b,
    ))
}

/// Resolve a team's Commander + banner into `(commander_id, BannerDef)`, or
/// `None` if no banner is flown. A banner requires a Commander.
fn resolve_banner(
    config: &TeamConfig,
    catalog: &BannerMap,
) -> Result<Option<(String, banners::BannerDef)>, String> {
    match (&config.commander, &config.banner) {
        (Some(commander), Some(banner)) => Ok(Some((
            commander.clone(),
            banners::resolve(banner, catalog)?,
        ))),
        (None, Some(_)) => Err("a banner requires a Commander".to_string()),
        _ => Ok(None),
    }
}

/// WebAssembly entry point. Returns replay-schema JSON on success, or a
/// `{"error": "..."}` JSON object the UI can detect, so failures never throw.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn run_battle(team_a_json: &str, team_b_json: &str, seed: u32) -> String {
    match run_battle_json(team_a_json, team_b_json, seed as u64) {
        Ok(replay) => replay,
        Err(message) => serde_json::json!({ "error": message }).to_string(),
    }
}

/// Return an embedded content-data catalog by name so the UI can populate the
/// team builder without fetching files (keeps the static site self-contained).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn catalog_json(name: &str) -> String {
    match name {
        "archetypes" => ARCHETYPES_JSON,
        "abilities" => ABILITIES_JSON,
        "passives" => PASSIVES_JSON,
        "statuses" => STATUSES_JSON,
        "aspects" => ASPECTS_JSON,
        "team_passives" => TEAM_PASSIVES_JSON,
        "banners" => BANNERS_JSON,
        _ => "null",
    }
    .to_string()
}
