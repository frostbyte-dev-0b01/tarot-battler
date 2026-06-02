//! Commander banners: a team designates one character as its **Commander**, who
//! flies a **banner** — a battle-start buff scoped to the Commander, the
//! Commander's column, or the whole team. Banners reuse the declarative effect
//! vocabulary from `team_passives` (status grants, opening haste, traits) and
//! the same per-member application path.
//!
//! Triggered banners (e.g. "Last Stand", on the Commander dropping low) are a
//! follow-up once the engine has the matching trigger hooks; the battle-start
//! banners here cover the shared Rally / Bulwark / Resolve set.

use std::collections::HashMap;
use std::path::Path;

use crate::team_passives::StatusGrant;

pub type BannerMap = HashMap<String, BannerDef>;

/// Who a banner's effect applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BannerScope {
    /// Only the Commander.
    Commander,
    /// Living allies sharing the Commander's column (same depth row).
    Column,
    /// All living allies.
    Team,
}

/// A battle-start banner effect. All effect fields are optional; a banner sets
/// the few it uses, applied to everyone in `scope`.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct BannerDef {
    #[serde(default)]
    pub description: String,
    pub scope: BannerScope,
    #[serde(default)]
    pub status: Option<StatusGrant>,
    #[serde(default)]
    pub opening_haste: Option<u32>,
    #[serde(default)]
    pub debuff_resistance: Option<u32>,
    #[serde(default)]
    pub mp_cost_reduction: Option<u32>,
}

pub fn load_banners(path: &Path) -> Result<BannerMap, String> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
}

/// Resolve a banner name against a catalog, erroring on an unknown reference.
pub fn resolve(name: &str, catalog: &BannerMap) -> Result<BannerDef, String> {
    catalog
        .get(name)
        .cloned()
        .ok_or_else(|| format!("unknown banner '{name}'"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_banners_load_and_resolve() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/data/banners.json");
        let catalog = load_banners(&path).expect("bundled banners parse");
        assert!(catalog.contains_key("Rally"));
        assert!(catalog.contains_key("Bulwark"));
        assert_eq!(resolve("Rally", &catalog).unwrap().scope, BannerScope::Team);
        assert_eq!(
            resolve("Bulwark", &catalog).unwrap().scope,
            BannerScope::Column
        );
        assert!(resolve("Nope", &catalog).is_err());
    }
}
