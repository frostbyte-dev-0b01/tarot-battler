//! Team passives: small, team-wide buffs drafted in season play and applied to a
//! whole team at battle start. Kept declarative (a flat set of optional effects)
//! so each is data-defined, balanced as a lateral sidegrade, and reuses existing
//! `CharacterState` mechanisms (statuses, traits, haste).
//!
//! Event/damage-hook passives (on-kill MP, execute bonuses, formation mitigation)
//! are a follow-up once the corresponding engine hooks exist.

use std::collections::HashMap;
use std::path::Path;

use crate::models::Stat;

pub type TeamPassiveMap = HashMap<String, TeamPassiveDef>;

/// A status to grant (e.g. `Ward` x1, or `Empower:MGT` x1).
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct StatusGrant {
    /// Base status name as it appears in `statuses.json` (e.g. "Ward", "Empower").
    pub status: String,
    /// Stat for stat-keyed statuses (Empower/Weaken); None otherwise.
    #[serde(default)]
    pub stat: Option<Stat>,
    pub stacks: u32,
}

/// A declarative team-wide buff. All fields are optional; a passive sets the few
/// it uses.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct TeamPassiveDef {
    #[serde(default)]
    pub description: String,
    /// Applied to every living ally at battle start.
    #[serde(default)]
    pub team_status: Option<StatusGrant>,
    /// Applied to living allies in the front column (row 0) at battle start.
    #[serde(default)]
    pub front_status: Option<StatusGrant>,
    /// Brings every ally's first turn sooner by this many ticks.
    #[serde(default)]
    pub opening_haste: Option<u32>,
    /// Grants every ally this many DebuffResistance charges.
    #[serde(default)]
    pub debuff_resistance: Option<u32>,
    /// Reduces every ally's ability MP cost by this (minimum 1).
    #[serde(default)]
    pub mp_cost_reduction: Option<u32>,
}

pub fn load_team_passives(path: &Path) -> Result<TeamPassiveMap, String> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
}

/// Resolve a team's chosen team-passive names against a catalog, erroring on any
/// unknown reference.
pub fn resolve(names: &[String], catalog: &TeamPassiveMap) -> Result<Vec<TeamPassiveDef>, String> {
    names
        .iter()
        .map(|name| {
            catalog
                .get(name)
                .cloned()
                .ok_or_else(|| format!("unknown team passive '{name}'"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_catalog_loads_and_resolves() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/data/team_passives.json");
        let catalog = load_team_passives(&path).expect("bundled team passives parse");
        assert!(catalog.contains_key("Vanguard's Oath"));
        assert!(catalog.contains_key("Aegis"));

        let resolved = resolve(&["Aegis".to_string(), "War Drums".to_string()], &catalog)
            .expect("known passives resolve");
        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].debuff_resistance, Some(1));
        assert_eq!(resolved[1].opening_haste, Some(2));

        assert!(resolve(&["Nonexistent".to_string()], &catalog).is_err());
    }
}
