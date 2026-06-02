//! HTTP API for the single-pod season.
//!
//! Lightweight identity (display name → slug id; trust within a friend group)
//! and JSON endpoints under `/api` that tie together the data layer ([`Db`]),
//! the draft schedule ([`crate::draft`]), the battle runner ([`crate::runner`]),
//! and engine content ([`Content`]).
//!
//! The request handling is split in two: the real work lives in methods on
//! [`AppState`] that return `Result<Value, ApiError>` (directly unit-testable),
//! and thin axum handlers adapt extractors to those methods.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::content::Content;
use crate::db::Db;
use crate::draft::{self, BeatKind};
use crate::models::{DraftPick, DraftState, Player, Season};
use crate::runner;

/// Shared, cheaply-cloneable application state.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
    pub content: Arc<Content>,
}

/// An API error carrying an HTTP status and a human-readable message.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: msg.into(),
        }
    }
    fn not_found(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: msg.into(),
        }
    }
    fn internal(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: msg.into(),
        }
    }
}

/// Database/internal errors surface as 500s.
impl From<String> for ApiError {
    fn from(message: String) -> Self {
        ApiError::internal(message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

/// The string tag for a beat kind, for JSON responses.
fn kind_str(kind: BeatKind) -> &'static str {
    match kind {
        BeatKind::Banner => "banner",
        BeatKind::Item => "item",
        BeatKind::Character => "character",
        BeatKind::TeamPassive => "team_passive",
        BeatKind::Swap => "swap",
    }
}

/// Lightweight player id from a display name (trust within a friend group).
fn slug(name: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in name.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.extend(ch.to_lowercase());
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

impl AppState {
    /// Fetch the current season, creating a default one (beat 0 revealed) if the
    /// pod hasn't started yet.
    fn ensure_season(&self) -> Result<Season, ApiError> {
        if let Some(season) = self.db.get_season()? {
            return Ok(season);
        }
        let created = now_unix();
        let season = Season {
            id: "season-1".to_string(),
            name: "Season 1".to_string(),
            day: 0,
            beats_revealed: 1, // the opening beat is immediately claimable
            created_unix: created,
            seed: (created as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03,
        };
        self.db.set_season(&season)?;
        Ok(season)
    }

    /// A player's unlocked pool + budget, given the season clock and their draft.
    fn unlocked_for(&self, season: &Season, draft_state: &DraftState) -> draft::Unlocked {
        draft::unlocked(
            &self.content.starting_roster,
            season.beats_revealed as usize,
            &draft_state.claimed,
        )
    }

    fn require_player(&self, id: &str) -> Result<Player, ApiError> {
        self.db
            .get_player(id)?
            .ok_or_else(|| ApiError::not_found(format!("unknown player '{id}'; join first")))
    }

    // ---- operations ----

    /// Register (or return) a player by display name and ensure the season.
    pub fn join(&self, name: &str) -> Result<Value, ApiError> {
        let id = slug(name);
        if id.is_empty() {
            return Err(ApiError::bad("a display name is required"));
        }
        let season = self.ensure_season()?;
        let player = match self.db.get_player(&id)? {
            Some(existing) => existing,
            None => {
                let p = Player {
                    id: id.clone(),
                    name: name.trim().to_string(),
                    points: 0,
                };
                self.db.upsert_player(&p)?;
                p
            }
        };
        Ok(json!({
            "player": player,
            "season": { "id": season.id, "name": season.name },
        }))
    }

    /// The schedule + season clock (player-agnostic).
    pub fn season_view(&self) -> Result<Value, ApiError> {
        let season = self.ensure_season()?;
        let schedule: Vec<Value> = draft::schedule()
            .iter()
            .enumerate()
            .map(|(i, b)| {
                json!({ "index": i, "kind": kind_str(b.kind), "budget_delta": b.budget_delta })
            })
            .collect();
        // Base budget at the current reveal (no picks).
        let budget = draft::unlocked(&[], season.beats_revealed as usize, &[]).budget;
        Ok(json!({
            "season": season,
            "schedule": schedule,
            "current_beat": season.beats_revealed.saturating_sub(1),
            "budget": budget,
        }))
    }

    /// A player's draft view: revealed beats with offers and their claims.
    pub fn draft_view(&self, player: &str) -> Result<Value, ApiError> {
        self.require_player(player)?;
        let season = self.ensure_season()?;
        let draft_state = self.db.get_draft(player)?;
        let unlocked = self.unlocked_for(&season, &draft_state);

        let revealed = season.beats_revealed as usize;
        let open = revealed.saturating_sub(1);
        let sched = draft::schedule();
        let beats: Vec<Value> = (0..revealed.min(sched.len()))
            .map(|i| {
                let claimed = draft_state
                    .claimed
                    .iter()
                    .find(|p| p.beat as usize == i)
                    .map(|p| p.choice.clone());
                json!({
                    "index": i,
                    "kind": kind_str(sched[i].kind),
                    "budget_delta": sched[i].budget_delta,
                    "offers": draft::offers(i, season.seed, &self.content.pools),
                    "claimed": claimed,
                    "open": i == open,
                })
            })
            .collect();

        Ok(json!({
            "player": player,
            "current_beat": open,
            "budget": unlocked.budget,
            "unlocked": {
                "archetypes": sorted_vec(&unlocked.archetypes),
                "aspects": sorted_vec(&unlocked.aspects),
                "team_passives": sorted_vec(&unlocked.team_passives),
                "banner": unlocked.banner,
            },
            "beats": beats,
        }))
    }

    /// Claim a pick for the currently open beat.
    pub fn claim(&self, player: &str, beat: u32, choice: &str) -> Result<Value, ApiError> {
        self.require_player(player)?;
        let season = self.ensure_season()?;
        let open = season.beats_revealed.saturating_sub(1);
        if beat != open {
            return Err(ApiError::bad(format!(
                "beat {beat} is not open for claims (the open beat is {open})"
            )));
        }
        let offers = draft::offers(beat as usize, season.seed, &self.content.pools);
        if !offers.iter().any(|o| o == choice) {
            return Err(ApiError::bad(format!(
                "'{choice}' is not an offered option for beat {beat}"
            )));
        }
        let mut draft_state = self.db.get_draft(player)?;
        draft_state.claimed.retain(|p| p.beat != beat); // re-claim replaces
        draft_state.claimed.push(DraftPick {
            beat,
            choice: choice.to_string(),
        });
        self.db.set_draft(player, &draft_state)?;
        self.draft_view(player)
    }

    /// Submit (validate + store) a player's team against their unlocked pool.
    pub fn submit_team(&self, player: &str, team: Value) -> Result<Value, ApiError> {
        self.require_player(player)?;
        let season = self.ensure_season()?;
        let draft_state = self.db.get_draft(player)?;
        let unlocked = self.unlocked_for(&season, &draft_state);

        let config: battle_engine::loader::TeamConfig = serde_json::from_value(team.clone())
            .map_err(|e| ApiError::bad(format!("team: {e}")))?;
        self.content
            .validate_team(&config, &unlocked)
            .map_err(ApiError::bad)?;

        let team_json =
            serde_json::to_string(&team).map_err(|e| ApiError::internal(e.to_string()))?;
        self.db.set_team(player, &team_json)?;
        Ok(json!({ "ok": true, "player": player }))
    }

    /// Fetch a player's stored team config.
    pub fn get_team(&self, player: &str) -> Result<Value, ApiError> {
        match self.db.get_team(player)? {
            Some(s) => serde_json::from_str(&s).map_err(|e| ApiError::internal(e.to_string())),
            None => Err(ApiError::not_found(format!(
                "no team submitted for '{player}'"
            ))),
        }
    }

    /// The league table.
    pub fn standings(&self) -> Result<Value, ApiError> {
        Ok(json!({ "standings": self.db.standings()? }))
    }

    /// Daily results, optionally filtered to one player.
    pub fn results(&self, player: Option<&str>) -> Result<Value, ApiError> {
        let results = match player {
            Some(p) => self.db.results_for_player(p)?,
            None => self.db.all_results()?,
        };
        Ok(json!({ "results": results }))
    }

    /// A stored replay's JSON, by id.
    pub fn replay(&self, id: &str) -> Result<Value, ApiError> {
        match self.db.get_replay(id)? {
            Some(s) => serde_json::from_str(&s).map_err(|e| ApiError::internal(e.to_string())),
            None => Err(ApiError::not_found(format!("no replay '{id}'"))),
        }
    }

    /// Minimal per-player post-game stats (wins/losses/draws + points).
    pub fn stats(&self) -> Result<Value, ApiError> {
        // (wins, losses, draws) per player.
        let mut wld: HashMap<String, (u32, u32, u32)> = HashMap::new();
        for r in self.db.all_results()? {
            let (a_won, b_won) = (r.winner == "a", r.winner == "b");
            let a = wld.entry(r.player_a.clone()).or_default();
            if a_won {
                a.0 += 1;
            } else if b_won {
                a.1 += 1;
            } else {
                a.2 += 1;
            }
            let b = wld.entry(r.player_b.clone()).or_default();
            if b_won {
                b.0 += 1;
            } else if a_won {
                b.1 += 1;
            } else {
                b.2 += 1;
            }
        }
        let players = self.db.standings()?;
        let rows: Vec<Value> = players
            .into_iter()
            .map(|p| {
                let (w, l, d) = wld.get(&p.id).copied().unwrap_or((0, 0, 0));
                json!({
                    "player": p.id,
                    "name": p.name,
                    "points": p.points,
                    "wins": w,
                    "losses": l,
                    "draws": d,
                })
            })
            .collect();
        Ok(json!({ "stats": rows }))
    }

    /// Play the current match day. Idempotent: replaying the same day is a no-op
    /// (advancing the clock is the separate `advance_day` admin action).
    pub fn run_day(&self) -> Result<Value, ApiError> {
        let season = self.ensure_season()?;
        let report = runner::run_day(&self.db, season.day, season.seed)?;
        Ok(json!({
            "day": report.day,
            "matches": report.matches,
            "already_run": report.already_run,
            "season_day": season.day,
        }))
    }

    /// Advance the season clock by one day (the next match day).
    pub fn advance_day(&self) -> Result<Value, ApiError> {
        let mut season = self.ensure_season()?;
        season.day += 1;
        self.db.set_season(&season)?;
        Ok(json!({ "season_day": season.day }))
    }

    /// Reveal the next draft beat, auto-resolving any now-closed unclaimed beats
    /// (swaps are skipped, never auto-applied).
    pub fn reveal_beat(&self) -> Result<Value, ApiError> {
        let mut season = self.ensure_season()?;
        let total = draft::beat_count() as u32;
        if season.beats_revealed >= total {
            return Ok(json!({ "beats_revealed": season.beats_revealed, "complete": true }));
        }
        season.beats_revealed += 1;
        self.db.set_season(&season)?;

        // Beats before the newly-open one are closed: auto-resolve unclaimed.
        let closed = season.beats_revealed.saturating_sub(1);
        for player in self.db.all_players()? {
            let mut draft_state = self.db.get_draft(&player.id)?;
            let mut changed = false;
            for beat in 0..closed {
                if draft_state.claimed.iter().any(|p| p.beat == beat) {
                    continue;
                }
                if let Some(pick) =
                    draft::resolve_missed_beat(beat as usize, season.seed, &self.content.pools)
                {
                    draft_state.claimed.push(pick);
                    changed = true;
                }
            }
            if changed {
                self.db.set_draft(&player.id, &draft_state)?;
            }
        }
        Ok(json!({ "beats_revealed": season.beats_revealed, "complete": false }))
    }
}

fn sorted_vec(set: &std::collections::HashSet<String>) -> Vec<String> {
    let mut v: Vec<String> = set.iter().cloned().collect();
    v.sort();
    v
}

// ---- axum router + thin handlers ----

/// Build the `/api` router from shared state.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/join", post(h_join))
        .route("/api/season", get(h_season))
        .route("/api/draft", get(h_draft))
        .route("/api/draft/claim", post(h_claim))
        .route("/api/team", post(h_submit_team).get(h_get_team))
        .route("/api/standings", get(h_standings))
        .route("/api/results", get(h_results))
        .route("/api/replays/{id}", get(h_replay))
        .route("/api/stats", get(h_stats))
        .route("/api/admin/run-day", post(h_run_day))
        .route("/api/admin/advance-day", post(h_advance_day))
        .route("/api/admin/reveal-beat", post(h_reveal_beat))
        .with_state(state)
}

fn q<'a>(params: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    params.get(key).map(|s| s.as_str())
}

async fn h_join(
    State(s): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let name = body
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::bad("'name' is required"))?;
    Ok(Json(s.join(name)?))
}

async fn h_season(State(s): State<AppState>) -> Result<Json<Value>, ApiError> {
    Ok(Json(s.season_view()?))
}

async fn h_draft(
    State(s): State<AppState>,
    Query(p): Query<HashMap<String, String>>,
) -> Result<Json<Value>, ApiError> {
    let player = q(&p, "player").ok_or_else(|| ApiError::bad("'player' query is required"))?;
    Ok(Json(s.draft_view(player)?))
}

async fn h_claim(
    State(s): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let player = body
        .get("player")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::bad("'player' is required"))?;
    let beat = body
        .get("beat")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| ApiError::bad("'beat' (number) is required"))? as u32;
    let choice = body
        .get("choice")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::bad("'choice' is required"))?;
    Ok(Json(s.claim(player, beat, choice)?))
}

async fn h_submit_team(
    State(s): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let player = body
        .get("player")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::bad("'player' is required"))?
        .to_string();
    let team = body
        .get("team")
        .cloned()
        .ok_or_else(|| ApiError::bad("'team' is required"))?;
    Ok(Json(s.submit_team(&player, team)?))
}

async fn h_get_team(
    State(s): State<AppState>,
    Query(p): Query<HashMap<String, String>>,
) -> Result<Json<Value>, ApiError> {
    let player = q(&p, "player").ok_or_else(|| ApiError::bad("'player' query is required"))?;
    Ok(Json(s.get_team(player)?))
}

async fn h_standings(State(s): State<AppState>) -> Result<Json<Value>, ApiError> {
    Ok(Json(s.standings()?))
}

async fn h_results(
    State(s): State<AppState>,
    Query(p): Query<HashMap<String, String>>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(s.results(q(&p, "player"))?))
}

async fn h_replay(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(s.replay(&id)?))
}

async fn h_stats(State(s): State<AppState>) -> Result<Json<Value>, ApiError> {
    Ok(Json(s.stats()?))
}

async fn h_run_day(State(s): State<AppState>) -> Result<Json<Value>, ApiError> {
    Ok(Json(s.run_day()?))
}

async fn h_advance_day(State(s): State<AppState>) -> Result<Json<Value>, ApiError> {
    Ok(Json(s.advance_day()?))
}

async fn h_reveal_beat(State(s): State<AppState>) -> Result<Json<Value>, ApiError> {
    Ok(Json(s.reveal_beat()?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn state() -> AppState {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("tarot-api-{}-{n}.redb", std::process::id()));
        let _ = std::fs::remove_file(&path);
        AppState {
            db: Arc::new(Db::open(path).unwrap()),
            content: Arc::new(Content::load().unwrap()),
        }
    }

    #[test]
    fn slug_is_friendly() {
        assert_eq!(slug("  Ada  Lovelace! "), "ada-lovelace");
        assert_eq!(slug("J.P."), "j-p");
        assert_eq!(slug(""), "");
    }

    #[test]
    fn join_is_idempotent_and_starts_the_season() {
        let s = state();
        let first = s.join("Ada").unwrap();
        assert_eq!(first["player"]["id"], "ada");
        assert_eq!(first["player"]["points"], 0);
        // Season exists with the opening beat revealed.
        let season = s.season_view().unwrap();
        assert_eq!(season["current_beat"], 0);
        assert_eq!(season["schedule"].as_array().unwrap().len(), 8);
        // Re-join returns the same player.
        let again = s.join("Ada").unwrap();
        assert_eq!(again["player"]["id"], "ada");
    }

    #[test]
    fn claim_rejects_unoffered_or_closed_beats() {
        let s = state();
        s.join("Ada").unwrap();
        // Beat 1 isn't open yet.
        assert!(s.claim("ada", 1, "Rally").is_err());
        // A bogus choice for the open beat is rejected.
        assert!(s.claim("ada", 0, "Not An Option").is_err());
        // A valid offered banner is accepted.
        let offers = draft::offers(0, s.ensure_season().unwrap().seed, &s.content.pools);
        let ok = s.claim("ada", 0, &offers[0]).unwrap();
        assert_eq!(ok["beats"][0]["claimed"], json!(offers[0]));
    }

    #[test]
    fn team_validation_rejects_locked_content() {
        let s = state();
        s.join("Ada").unwrap();
        // A team using a team passive the player hasn't drafted is rejected.
        let team = json!({
            "version": 2,
            "name": "Locked",
            "characters": [],
            "team_passives": ["Aegis"],
        });
        let err = s.submit_team("ada", team).unwrap_err();
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn reveal_auto_resolves_unclaimed_non_swap_beats() {
        let s = state();
        s.join("Ada").unwrap();
        // Don't claim beat 0; reveal beat 1 → beat 0 auto-resolves (it's an item-
        // free Banner beat, which still auto-picks).
        s.reveal_beat().unwrap();
        let draft = s.draft_view("ada").unwrap();
        let claimed = &draft["beats"][0]["claimed"];
        assert!(claimed.is_string(), "closed beat 0 was auto-resolved");
    }

    #[test]
    fn scripted_flow_join_claim_submit_run_read() {
        let s = state();
        // Two players join (seed a points buffer so the 0-floor doesn't mask
        // conservation in this decisive game).
        s.join("Ada").unwrap();
        s.join("Bo").unwrap();
        for id in ["ada", "bo"] {
            let mut p = s.db.get_player(id).unwrap().unwrap();
            p.points = 100;
            s.db.upsert_player(&p).unwrap();
        }

        // Each claims the opening (banner) beat.
        let seed = s.ensure_season().unwrap().seed;
        let banner = draft::offers(0, seed, &s.content.pools)[0].clone();
        s.claim("ada", 0, &banner).unwrap();
        s.claim("bo", 0, &banner).unwrap();

        // Submit real sample teams (starter archetypes are unlocked).
        let team_a: Value = serde_json::from_str(include_str!(
            "../../tools/ui/sample-data/teams/imperial_phalanx.json"
        ))
        .unwrap();
        let team_b: Value = serde_json::from_str(include_str!(
            "../../tools/ui/sample-data/teams/guardian_column.json"
        ))
        .unwrap();
        s.submit_team("ada", team_a).unwrap();
        s.submit_team("bo", team_b).unwrap();

        // Run the day.
        let report = s.run_day().unwrap();
        assert_eq!(report["matches"], 1);
        assert_eq!(report["season_day"], 0);

        // Standings reflect the result; points are conserved across the pod.
        let standings = s.standings().unwrap();
        let rows = standings["standings"].as_array().unwrap();
        let total: i64 = rows.iter().map(|r| r["points"].as_i64().unwrap()).sum();
        assert_eq!(total, 200, "points conserved across the pod");

        // The replay is fetchable for the recorded match.
        let results = s.results(Some("ada")).unwrap();
        let replay_id = results["results"][0]["replay_id"].as_str().unwrap();
        let replay = s.replay(replay_id).unwrap();
        assert!(replay.get("winner").is_some());

        // Re-running the same day is a no-op.
        let again = s.run_day().unwrap();
        assert_eq!(again["already_run"], true);
    }
}
