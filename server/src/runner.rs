//! Daily battle runner + scoring for the single pod.
//!
//! A "match day" runs the pod round-robin over players' currently-submitted
//! teams through the native engine (`battle_engine::run_battle_json`), records a
//! [`MatchResult`] + replay for each matchup, and exchanges league points within
//! the pod: **+5 win / −5 loss / 0 draw**, floored at 0.
//!
//! The orchestration is generic over the battle function so the scoring/pairing/
//! persistence flow can be tested with a deterministic stand-in; the public
//! [`run_day`] wires in the real engine. Running a day is **idempotent**: if the
//! day already has results, it is a no-op (points are never double-counted).
//!
//! Wired into the API/admin endpoint in a later issue; allow dead code until
//! then.
#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use crate::db::Db;
use crate::models::MatchResult;

/// Points exchanged for a win (the loser drops the same, floored at 0).
pub const WIN_POINTS: i64 = 5;

/// Who won a single battle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    A,
    B,
    Draw,
}

impl Outcome {
    /// Interpret the replay's top-level `winner` field (`"team_a"` / `"team_b"`
    /// / anything else → draw).
    pub fn from_winner(winner: &str) -> Outcome {
        match winner {
            "team_a" => Outcome::A,
            "team_b" => Outcome::B,
            _ => Outcome::Draw,
        }
    }

    /// The stored `winner` tag for a [`MatchResult`] (`"a"` / `"b"` / `"draw"`).
    pub fn tag(self) -> &'static str {
        match self {
            Outcome::A => "a",
            Outcome::B => "b",
            Outcome::Draw => "draw",
        }
    }

    /// The raw point deltas `(player_a, player_b)` before flooring.
    pub fn point_deltas(self) -> (i64, i64) {
        match self {
            Outcome::A => (WIN_POINTS, -WIN_POINTS),
            Outcome::B => (-WIN_POINTS, WIN_POINTS),
            Outcome::Draw => (0, 0),
        }
    }
}

/// Apply a point delta with the league floor: totals never drop below 0.
pub fn apply_delta(points: i64, delta: i64) -> i64 {
    (points + delta).max(0)
}

/// Every unordered pair of player ids, each once (the pod round-robin). Input
/// order is preserved so the schedule is deterministic for a fixed roster.
pub fn round_robin_pairs(ids: &[String]) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            pairs.push((ids[i].clone(), ids[j].clone()));
        }
    }
    pairs
}

/// A stable per-match seed so a given day's battles are reproducible.
pub fn match_seed(season_seed: u64, day: u32, a: &str, b: &str) -> u64 {
    let mut h = season_seed
        .wrapping_add((day as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .wrapping_add(0x1000_0000);
    for s in [a, b] {
        for byte in s.bytes() {
            h = h.wrapping_mul(0x0100_0000_01b3).wrapping_add(byte as u64);
        }
        h = h.wrapping_add(0xD1B5_4A32_D192_ED03);
    }
    h
}

/// Deterministic id for a matchup's result/replay on a given day.
fn match_id(day: u32, a: &str, b: &str) -> String {
    format!("d{day}-{a}-vs-{b}")
}

/// What a day's run did.
#[derive(Debug, Clone)]
pub struct DayReport {
    pub day: u32,
    /// Number of matchups played (or already on record).
    pub matches: usize,
    /// True if the day already had results and this call was a no-op.
    pub already_run: bool,
}

/// Run match day `day` using the real native engine. `eligible`, when `Some`,
/// restricts play to those player ids (season-legal teams, validated against the
/// season budget + pool by the caller); `None` plays everyone (used in tests).
pub fn run_day(
    db: &Db,
    day: u32,
    season_seed: u64,
    eligible: Option<&HashSet<String>>,
) -> Result<DayReport, String> {
    run_day_with(db, day, season_seed, eligible, |a, b, seed| {
        battle_engine::run_battle_json(a, b, seed)
    })
}

/// Run match day `day`, resolving each battle through `battle`. Idempotent: if
/// the day already has results, returns early without re-scoring.
pub fn run_day_with<F>(
    db: &Db,
    day: u32,
    season_seed: u64,
    eligible: Option<&HashSet<String>>,
    battle: F,
) -> Result<DayReport, String>
where
    F: Fn(&str, &str, u64) -> Result<String, String>,
{
    // Idempotency: a day is scored at most once.
    let existing = db.all_results()?;
    let already = existing.iter().filter(|r| r.day == day).count();
    if already > 0 {
        return Ok(DayReport {
            day,
            matches: already,
            already_run: true,
        });
    }

    // Play over currently-submitted teams, in a deterministic order. Skip any
    // player not in the eligible set (their team isn't season-legal today).
    let mut teams = db.all_teams()?;
    teams.sort_by(|a, b| a.0.cmp(&b.0));
    teams.retain(|(id, _)| eligible.is_none_or(|e| e.contains(id)));
    let ids: Vec<String> = teams.iter().map(|(id, _)| id.clone()).collect();
    let team_json: HashMap<&str, &str> = teams
        .iter()
        .map(|(id, j)| (id.as_str(), j.as_str()))
        .collect();

    let mut deltas: HashMap<String, i64> = HashMap::new();
    let mut played = 0;

    for (a, b) in round_robin_pairs(&ids) {
        let seed = match_seed(season_seed, day, &a, &b);
        let replay = battle(team_json[a.as_str()], team_json[b.as_str()], seed)
            .map_err(|e| format!("battle {a} vs {b}: {e}"))?;

        let winner = serde_json::from_str::<serde_json::Value>(&replay)
            .ok()
            .and_then(|v| v.get("winner").and_then(|w| w.as_str()).map(String::from))
            .unwrap_or_else(|| "draw".to_string());
        let outcome = Outcome::from_winner(&winner);
        let (da, db_) = outcome.point_deltas();
        *deltas.entry(a.clone()).or_default() += da;
        *deltas.entry(b.clone()).or_default() += db_;

        let id = match_id(day, &a, &b);
        let result = MatchResult {
            id: id.clone(),
            day,
            player_a: a.clone(),
            player_b: b.clone(),
            winner: outcome.tag().to_string(),
            seed,
            replay_id: id,
        };
        db.add_result(&result, &replay)?;
        played += 1;
    }

    // Apply the accumulated point deltas (floored) to persistent totals.
    for (id, delta) in deltas {
        if let Some(mut player) = db.get_player(&id)? {
            player.points = apply_delta(player.points, delta);
            db.upsert_player(&player)?;
        }
    }

    Ok(DayReport {
        day,
        matches: played,
        already_run: false,
    })
}

/// Sentinel "day" for finals (Victors round) results, so they never collide
/// with real match days and can be filtered out of daily stats.
pub const FINALS_DAY: u32 = u32::MAX;

/// What the Victors round produced.
#[derive(Debug, Clone)]
pub struct FinalsReport {
    pub matches: usize,
    /// The id of the player awarded the cosmetic title, if any.
    pub victor: Option<String>,
    /// True if finals already ran (idempotent no-op).
    pub already_run: bool,
}

/// Run the end-of-season Victors round among the pod's top `top_n` players (by
/// standings) who have a submitted team: a round-robin reusing the engine. The
/// player with the most finals wins is awarded a cosmetic `title` (no points
/// change). Idempotent: if finals already ran, it's a no-op.
pub fn run_finals(
    db: &Db,
    season_seed: u64,
    season_label: &str,
    top_n: usize,
    eligible: Option<&HashSet<String>>,
) -> Result<FinalsReport, String> {
    run_finals_with(
        db,
        season_seed,
        season_label,
        top_n,
        eligible,
        battle_engine::run_battle_json,
    )
}

/// Finals, resolving battles through `battle` (testable).
pub fn run_finals_with<F>(
    db: &Db,
    season_seed: u64,
    season_label: &str,
    top_n: usize,
    eligible: Option<&HashSet<String>>,
    battle: F,
) -> Result<FinalsReport, String>
where
    F: Fn(&str, &str, u64) -> Result<String, String>,
{
    // Idempotency.
    if db.all_results()?.iter().any(|r| r.day == FINALS_DAY) {
        let victor = db
            .all_players()?
            .into_iter()
            .find(|p| p.title.is_some())
            .map(|p| p.id);
        return Ok(FinalsReport {
            matches: db
                .all_results()?
                .iter()
                .filter(|r| r.day == FINALS_DAY)
                .count(),
            victor,
            already_run: true,
        });
    }

    // Finalists: top standings with a submitted, season-legal team.
    let teams: HashMap<String, String> = db.all_teams()?.into_iter().collect();
    let finalists: Vec<String> = db
        .standings()?
        .into_iter()
        .filter(|p| teams.contains_key(&p.id) && eligible.is_none_or(|e| e.contains(&p.id)))
        .map(|p| p.id)
        .take(top_n.max(2))
        .collect();

    if finalists.len() < 2 {
        return Ok(FinalsReport {
            matches: 0,
            victor: None,
            already_run: false,
        });
    }

    let mut wins: HashMap<String, u32> = HashMap::new();
    let mut played = 0;
    for (a, b) in round_robin_pairs(&finalists) {
        let seed = match_seed(season_seed, FINALS_DAY, &a, &b);
        let replay =
            battle(&teams[&a], &teams[&b], seed).map_err(|e| format!("finals {a} vs {b}: {e}"))?;
        let winner = serde_json::from_str::<serde_json::Value>(&replay)
            .ok()
            .and_then(|v| v.get("winner").and_then(|w| w.as_str()).map(String::from))
            .unwrap_or_else(|| "draw".to_string());
        let outcome = Outcome::from_winner(&winner);
        match outcome {
            Outcome::A => *wins.entry(a.clone()).or_default() += 1,
            Outcome::B => *wins.entry(b.clone()).or_default() += 1,
            Outcome::Draw => {}
        }
        let id = format!("final-{a}-vs-{b}");
        let result = MatchResult {
            id: id.clone(),
            day: FINALS_DAY,
            player_a: a.clone(),
            player_b: b.clone(),
            winner: outcome.tag().to_string(),
            seed,
            replay_id: id,
        };
        db.add_result(&result, &replay)?;
        played += 1;
    }

    // Victor = most finals wins, tie-broken by standings order (finalists are
    // already in standings order, so the earliest finalist wins ties).
    let victor = finalists
        .iter()
        .max_by_key(|id| wins.get(*id).copied().unwrap_or(0))
        .cloned();
    if let Some(ref id) = victor {
        if let Some(mut player) = db.get_player(id)? {
            player.title = Some(format!("Victor — {season_label}"));
            db.upsert_player(&player)?;
        }
    }

    Ok(FinalsReport {
        matches: played,
        victor,
        already_run: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Player;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn temp_db() -> Db {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("tarot-runner-{}-{n}.redb", std::process::id()));
        let _ = std::fs::remove_file(&path);
        Db::open(path).unwrap()
    }

    #[test]
    fn scoring_is_zero_sum_for_decisive_games() {
        assert_eq!(Outcome::A.point_deltas(), (5, -5));
        assert_eq!(Outcome::B.point_deltas(), (-5, 5));
        assert_eq!(Outcome::Draw.point_deltas(), (0, 0));
        assert_eq!(Outcome::from_winner("team_b"), Outcome::B);
        assert_eq!(Outcome::from_winner("nobody"), Outcome::Draw);
    }

    #[test]
    fn delta_floors_at_zero() {
        assert_eq!(apply_delta(3, -5), 0, "loss cannot push a total negative");
        assert_eq!(apply_delta(0, -5), 0);
        assert_eq!(apply_delta(10, 5), 15);
        assert_eq!(apply_delta(10, -5), 5);
    }

    #[test]
    fn round_robin_is_each_unordered_pair_once() {
        let ids: Vec<String> = ["a", "b", "c", "d"].iter().map(|s| s.to_string()).collect();
        let pairs = round_robin_pairs(&ids);
        assert_eq!(pairs.len(), 6); // n*(n-1)/2
        assert!(pairs.contains(&("a".into(), "b".into())));
        assert!(!pairs.contains(&("b".into(), "a".into())));
    }

    /// A deterministic stand-in: the team whose JSON contains `"win":"a"` /
    /// `"win":"b"` wins; otherwise a draw. Lets us test the full day flow
    /// without engine content.
    fn fake_battle(a_json: &str, b_json: &str, _seed: u64) -> Result<String, String> {
        let winner = if a_json.contains("\"win\":\"a\"") {
            "team_a"
        } else if b_json.contains("\"win\":\"b\"") {
            "team_b"
        } else {
            "draw"
        };
        Ok(format!("{{\"winner\":\"{winner}\"}}"))
    }

    fn seed_player(db: &Db, id: &str, points: i64, team: &str) {
        db.upsert_player(&Player {
            id: id.to_string(),
            name: id.to_string(),
            points,
            title: None,
        })
        .unwrap();
        db.set_team(id, team).unwrap();
    }

    #[test]
    fn end_to_end_pod_day_scores_and_stores_and_is_idempotent() {
        let db = temp_db();
        // p1 always wins (team carries the win marker); p2 and p3 draw each other.
        seed_player(&db, "p1", 100, r#"{"win":"a"}"#);
        seed_player(&db, "p2", 100, r#"{"neutral":true}"#);
        seed_player(&db, "p3", 100, r#"{"neutral":true}"#);

        let report = run_day_with(&db, 0, 7, None, fake_battle).unwrap();
        assert!(!report.already_run);
        assert_eq!(report.matches, 3); // 3 players → 3 matchups

        // p1 beat p2 and p3: +10. p2 and p3 each lost once to p1, drew each other: -5.
        let pts = |id: &str| db.get_player(id).unwrap().unwrap().points;
        assert_eq!(pts("p1"), 110);
        assert_eq!(pts("p2"), 95);
        assert_eq!(pts("p3"), 95);

        // Every matchup stored a result + a loadable replay.
        assert_eq!(db.all_results().unwrap().len(), 3);
        for r in db.all_results().unwrap() {
            assert!(db.get_replay(&r.replay_id).unwrap().is_some());
        }

        // Re-running the same day is a no-op (no double counting).
        let again = run_day_with(&db, 0, 7, None, fake_battle).unwrap();
        assert!(again.already_run);
        assert_eq!(again.matches, 3);
        assert_eq!(pts("p1"), 110, "points unchanged on re-run");
        assert_eq!(db.all_results().unwrap().len(), 3);
    }

    #[test]
    fn day_floors_totals_at_zero() {
        let db = temp_db();
        // p2 starts at 3; losing 5 in a day must floor to 0, not go negative.
        seed_player(&db, "p1", 0, r#"{"win":"a"}"#);
        seed_player(&db, "p2", 3, r#"{"neutral":true}"#);
        run_day_with(&db, 0, 1, None, fake_battle).unwrap();
        assert_eq!(db.get_player("p2").unwrap().unwrap().points, 0);
        assert_eq!(db.get_player("p1").unwrap().unwrap().points, 5);
    }

    #[test]
    fn finals_award_a_title_without_changing_points() {
        let db = temp_db();
        // p1 always wins finals; p2/p3 draw each other.
        seed_player(&db, "p1", 120, r#"{"win":"a"}"#);
        seed_player(&db, "p2", 110, r#"{"neutral":true}"#);
        seed_player(&db, "p3", 100, r#"{"neutral":true}"#);

        let report = run_finals_with(&db, 7, "Season 1", 4, None, fake_battle).unwrap();
        assert_eq!(report.matches, 3);
        assert_eq!(report.victor.as_deref(), Some("p1"));

        // The victor gets a cosmetic title; points are untouched.
        let p1 = db.get_player("p1").unwrap().unwrap();
        assert_eq!(p1.title.as_deref(), Some("Victor — Season 1"));
        assert_eq!(p1.points, 120);
        assert!(db.get_player("p2").unwrap().unwrap().title.is_none());

        // Finals results are tagged with the sentinel day.
        assert!(db
            .all_results()
            .unwrap()
            .iter()
            .all(|r| r.day == FINALS_DAY));

        // Idempotent: a second run is a no-op that still reports the victor.
        let again = run_finals_with(&db, 7, "Season 1", 4, None, fake_battle).unwrap();
        assert!(again.already_run);
        assert_eq!(again.victor.as_deref(), Some("p1"));
    }

    #[test]
    fn finals_need_at_least_two_teams() {
        let db = temp_db();
        seed_player(&db, "solo", 100, r#"{"win":"a"}"#);
        let report = run_finals_with(&db, 1, "Season 1", 4, None, fake_battle).unwrap();
        assert_eq!(report.matches, 0);
        assert!(report.victor.is_none());
    }

    #[test]
    fn real_engine_runs_a_pod_day() {
        // Integration: two real sample teams through the native engine.
        let team_a = include_str!("../../tools/ui/sample-data/teams/front_row.json");
        let team_b = include_str!("../../tools/ui/sample-data/teams/good_stats.json");
        let db = temp_db();
        seed_player(&db, "p1", 50, team_a);
        seed_player(&db, "p2", 50, team_b);

        let report = run_day(&db, 0, 42, None).unwrap();
        assert_eq!(report.matches, 1);

        // Exactly one decisive/draw result, with a replay the viewer can load.
        let results = db.all_results().unwrap();
        assert_eq!(results.len(), 1);
        let replay = db.get_replay(&results[0].replay_id).unwrap().unwrap();
        assert!(replay.contains("\"winner\""));

        // Total points are conserved across the pod (no floor hit here).
        let total = db.get_player("p1").unwrap().unwrap().points
            + db.get_player("p2").unwrap().unwrap().points;
        assert_eq!(total, 100);
    }
}
