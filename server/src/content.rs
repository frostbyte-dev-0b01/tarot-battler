//! Engine content loaded once at startup: the catalogs needed to validate
//! season teams, plus the draft [`Pools`] and starting roster the schedule draws
//! from. Everything is parsed from the engine's embedded JSON so the server has
//! no separate data files to keep in sync.

use battle_engine::abilities::{AbilityMap, PassiveMap};
use battle_engine::banners::{self, BannerMap};
use battle_engine::loader::{self, ArchetypeMap, AspectMap, TeamConfig, UnlockedPool};
use battle_engine::statuses::StatusMap;
use battle_engine::team_passives::{self, TeamPassiveMap};

use crate::draft::{self, BeatKind, Pools, Unlocked};

/// The season's starter roster — always unlocked, enough to field the opening
/// 3-character budget with room to script.
pub const STARTING_ROSTER: [&str; 5] = [
    "the_emperor",
    "the_hierophant",
    "the_chariot",
    "justice",
    "the_moon",
];

/// Hardcoded, curated draft offers for the pod — **edit this to control exactly
/// what each beat presents** while testing what friends play with. Then restart
/// the server (`cargo run`) and `New season` to apply.
///
/// One entry per beat (0-based), matching `draft::schedule()`:
///   0 Banner · 1 Item · 2 Character · 3 TeamPassive · 4 Item · 5 Character · 6 Swap · 7 Item
///
/// Each entry lists ids from that beat's category — archetype ids (characters /
/// swap), aspect ids (items), team-passive names, or banner names. A **non-empty**
/// entry is shown verbatim (any length you like); an **empty** entry (`&[]`) or a
/// missing index falls back to seeded random sampling from the full category.
/// Unknown ids fail fast at startup with a clear message.
///
/// Related knobs: `STARTING_ROSTER` (above) sets the always-unlocked starters,
/// and `draft::schedule()` sets the beat order + budget growth.
const CURATED_OFFERS: &[&[&str]] = &[
    &["Rally", "Bulwark", "Resolve"],            // 0 — Banner
    &["aspect_of_ruin", "aspect_of_grace"],      // 1 — Item
    &["the_magician", "the_hermit", "the_fool"], // 2 — Character
    &["Aegis", "War Drums", "Iron Standard"],    // 3 — TeamPassive
    &["aspect_of_ruin", "aspect_of_grace"],      // 4 — Item
    &["the_magician", "the_hermit", "the_fool"], // 5 — Character
    &["the_magician", "the_hermit", "the_fool"], // 6 — Swap
    &["aspect_of_ruin", "aspect_of_grace"],      // 7 — Item
];

/// Parsed engine catalogs + derived draft pools.
pub struct Content {
    pub archetypes: ArchetypeMap,
    pub aspects: AspectMap,
    pub abilities: AbilityMap,
    pub passives: PassiveMap,
    pub statuses: StatusMap,
    pub team_passives: TeamPassiveMap,
    pub banners: BannerMap,
    pub pools: Pools,
    pub starting_roster: Vec<String>,
}

impl Content {
    /// Parse every embedded catalog and build the draft pools. Returns `Err` if
    /// any bundled catalog fails to parse (a build-time data bug).
    pub fn load() -> Result<Self, String> {
        let archetypes: ArchetypeMap = serde_json::from_str(battle_engine::ARCHETYPES_JSON)
            .map_err(|e| format!("archetypes: {e}"))?;
        let aspects: AspectMap = serde_json::from_str(battle_engine::ASPECTS_JSON)
            .map_err(|e| format!("aspects: {e}"))?;
        let abilities: AbilityMap = serde_json::from_str(battle_engine::ABILITIES_JSON)
            .map_err(|e| format!("abilities: {e}"))?;
        let passives: PassiveMap = serde_json::from_str(battle_engine::PASSIVES_JSON)
            .map_err(|e| format!("passives: {e}"))?;
        let statuses: StatusMap = serde_json::from_str(battle_engine::STATUSES_JSON)
            .map_err(|e| format!("statuses: {e}"))?;
        let team_passives: TeamPassiveMap = serde_json::from_str(battle_engine::TEAM_PASSIVES_JSON)
            .map_err(|e| format!("team passives: {e}"))?;
        let banners: BannerMap = serde_json::from_str(battle_engine::BANNERS_JSON)
            .map_err(|e| format!("banners: {e}"))?;

        // Sorted keys → deterministic, pod-wide draft offers.
        let sorted = |keys: Vec<String>| {
            let mut v = keys;
            v.sort();
            v
        };
        let curated_offers = build_curated_offers(&archetypes, &aspects, &team_passives, &banners)?;
        let pools = Pools {
            characters: sorted(archetypes.keys().cloned().collect()),
            items: sorted(aspects.keys().cloned().collect()),
            team_passives: sorted(team_passives.keys().cloned().collect()),
            banners: sorted(banners.keys().cloned().collect()),
            curated_offers,
        };

        Ok(Content {
            archetypes,
            aspects,
            abilities,
            passives,
            statuses,
            team_passives,
            banners,
            pools,
            starting_roster: STARTING_ROSTER.iter().map(|s| s.to_string()).collect(),
        })
    }

    /// Validate a season team config against a player's unlocked pool + budget.
    /// Checks archetype/aspect references and budget (via the loader), then the
    /// season-only content (team passives + banner/commander) against `unlocked`.
    pub fn validate_team(&self, team: &TeamConfig, unlocked: &Unlocked) -> Result<(), String> {
        let pool = UnlockedPool {
            archetypes: unlocked.archetypes.clone(),
            aspects: unlocked.aspects.clone(),
        };
        loader::validate_team_config_with(
            team,
            &self.archetypes,
            &self.aspects,
            &self.abilities,
            &self.passives,
            &self.statuses,
            unlocked.budget,
            Some(&pool),
        )?;

        // Team passives must be drafted (unlocked) and exist in the catalog.
        for tp in &team.team_passives {
            if !unlocked.team_passives.contains(tp) {
                return Err(format!("team passive '{tp}' is not in this season's pool"));
            }
        }
        team_passives::resolve(&team.team_passives, &self.team_passives)?;

        // A banner requires a commander and must match the drafted banner.
        match (&team.commander, &team.banner) {
            (_, Some(banner)) if team.commander.is_none() => {
                return Err(format!("banner '{banner}' requires a commander"));
            }
            (_, Some(banner)) => {
                if unlocked.banner.as_deref() != Some(banner.as_str()) {
                    return Err(format!(
                        "banner '{banner}' is not this season's drafted banner"
                    ));
                }
                banners::resolve(banner, &self.banners)?;
            }
            _ => {}
        }

        Ok(())
    }
}

/// Turn the hardcoded [`CURATED_OFFERS`] table into per-beat offer lists,
/// validating every id against its beat's category so a typo fails at startup.
fn build_curated_offers(
    archetypes: &ArchetypeMap,
    aspects: &AspectMap,
    team_passives: &TeamPassiveMap,
    banners: &BannerMap,
) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::with_capacity(CURATED_OFFERS.len());
    for (beat, entry) in CURATED_OFFERS.iter().enumerate() {
        let kind = draft::beat_kind(beat).ok_or_else(|| {
            format!(
                "curated offers: beat {beat} is past the {}-beat schedule",
                draft::beat_count()
            )
        })?;
        for id in *entry {
            let (known, category) = match kind {
                BeatKind::Banner => (banners.contains_key(*id), "banner"),
                BeatKind::Item => (aspects.contains_key(*id), "item"),
                BeatKind::Character | BeatKind::Swap => (archetypes.contains_key(*id), "character"),
                BeatKind::TeamPassive => (team_passives.contains_key(*id), "team passive"),
            };
            if !known {
                return Err(format!(
                    "curated offers: beat {beat} ({category}) lists unknown id '{id}'"
                ));
            }
        }
        out.push(entry.iter().map(|s| s.to_string()).collect());
    }
    Ok(out)
}
