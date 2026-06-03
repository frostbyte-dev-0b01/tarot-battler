//! Embedded database (redb) and the typed access layer for the single-pod season.
//!
//! redb is a pure-Rust embedded key/value store (chosen over SQLite so the
//! server builds with no C toolchain). Each entity is stored in its own table as
//! JSON keyed by id; teams are stored as their raw team-config JSON. Tables are
//! created lazily on first write.
//!
//! The access layer is exercised by tests now and wired into the HTTP API and
//! battle runner in later issues; allow dead code until then.
#![allow(dead_code)]

use std::path::Path;

use redb::{Database, ReadableTable, TableDefinition};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::models::{DraftState, MatchResult, Player, Season};

const PLAYERS: TableDefinition<&str, &str> = TableDefinition::new("players");
const TEAMS: TableDefinition<&str, &str> = TableDefinition::new("teams");
const DRAFTS: TableDefinition<&str, &str> = TableDefinition::new("drafts");
const SEASON: TableDefinition<&str, &str> = TableDefinition::new("season");
const RESULTS: TableDefinition<&str, &str> = TableDefinition::new("results");
const REPLAYS: TableDefinition<&str, &str> = TableDefinition::new("replays");

/// Single-row key for the current season.
const SEASON_KEY: &str = "current";

pub struct Db {
    inner: Database,
}

impl Db {
    /// Open the database at `path`, creating the file and any parent directories.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("create data dir {}: {e}", parent.display()))?;
            }
        }
        let inner =
            Database::create(path).map_err(|e| format!("open redb {}: {e}", path.display()))?;
        Ok(Self { inner })
    }

    // ---- generic string/json helpers ----

    fn put_str(
        &self,
        table: TableDefinition<&str, &str>,
        key: &str,
        value: &str,
    ) -> Result<(), String> {
        let tx = self.inner.begin_write().map_err(|e| e.to_string())?;
        {
            let mut t = tx.open_table(table).map_err(|e| e.to_string())?;
            t.insert(key, value).map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_str(
        &self,
        table: TableDefinition<&str, &str>,
        key: &str,
    ) -> Result<Option<String>, String> {
        let tx = self.inner.begin_read().map_err(|e| e.to_string())?;
        let t = match tx.open_table(table) {
            Ok(t) => t,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
            Err(e) => return Err(e.to_string()),
        };
        match t.get(key).map_err(|e| e.to_string())? {
            Some(g) => Ok(Some(g.value().to_string())),
            None => Ok(None),
        }
    }

    fn list_pairs(
        &self,
        table: TableDefinition<&str, &str>,
    ) -> Result<Vec<(String, String)>, String> {
        let tx = self.inner.begin_read().map_err(|e| e.to_string())?;
        let t = match tx.open_table(table) {
            Ok(t) => t,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
            Err(e) => return Err(e.to_string()),
        };
        let mut out = Vec::new();
        for row in t.iter().map_err(|e| e.to_string())? {
            let (k, v) = row.map_err(|e| e.to_string())?;
            out.push((k.value().to_string(), v.value().to_string()));
        }
        Ok(out)
    }

    fn put_json<T: Serialize>(
        &self,
        table: TableDefinition<&str, &str>,
        key: &str,
        value: &T,
    ) -> Result<(), String> {
        let json = serde_json::to_string(value).map_err(|e| e.to_string())?;
        self.put_str(table, key, &json)
    }

    fn get_json<T: DeserializeOwned>(
        &self,
        table: TableDefinition<&str, &str>,
        key: &str,
    ) -> Result<Option<T>, String> {
        match self.get_str(table, key)? {
            Some(s) => Ok(Some(serde_json::from_str(&s).map_err(|e| e.to_string())?)),
            None => Ok(None),
        }
    }

    fn list_json<T: DeserializeOwned>(
        &self,
        table: TableDefinition<&str, &str>,
    ) -> Result<Vec<T>, String> {
        self.list_pairs(table)?
            .into_iter()
            .map(|(_, v)| serde_json::from_str(&v).map_err(|e| e.to_string()))
            .collect()
    }

    // ---- players ----

    pub fn upsert_player(&self, player: &Player) -> Result<(), String> {
        self.put_json(PLAYERS, &player.id, player)
    }

    pub fn get_player(&self, id: &str) -> Result<Option<Player>, String> {
        self.get_json(PLAYERS, id)
    }

    pub fn all_players(&self) -> Result<Vec<Player>, String> {
        self.list_json(PLAYERS)
    }

    /// Players sorted by points (desc), then name — the legible league table.
    pub fn standings(&self) -> Result<Vec<Player>, String> {
        let mut players = self.all_players()?;
        players.sort_by(|a, b| b.points.cmp(&a.points).then_with(|| a.name.cmp(&b.name)));
        Ok(players)
    }

    // ---- teams (raw team-config JSON) ----

    pub fn set_team(&self, player_id: &str, team_json: &str) -> Result<(), String> {
        self.put_str(TEAMS, player_id, team_json)
    }

    pub fn get_team(&self, player_id: &str) -> Result<Option<String>, String> {
        self.get_str(TEAMS, player_id)
    }

    /// All submitted teams as `(player_id, team_json)` — used by the battle runner.
    pub fn all_teams(&self) -> Result<Vec<(String, String)>, String> {
        self.list_pairs(TEAMS)
    }

    // ---- drafts ----

    pub fn get_draft(&self, player_id: &str) -> Result<DraftState, String> {
        Ok(self.get_json(DRAFTS, player_id)?.unwrap_or_default())
    }

    pub fn set_draft(&self, player_id: &str, draft: &DraftState) -> Result<(), String> {
        self.put_json(DRAFTS, player_id, draft)
    }

    // ---- season ----

    pub fn get_season(&self) -> Result<Option<Season>, String> {
        self.get_json(SEASON, SEASON_KEY)
    }

    pub fn set_season(&self, season: &Season) -> Result<(), String> {
        self.put_json(SEASON, SEASON_KEY, season)
    }

    // ---- results + replays ----

    pub fn add_result(&self, result: &MatchResult, replay_json: &str) -> Result<(), String> {
        self.put_str(REPLAYS, &result.replay_id, replay_json)?;
        self.put_json(RESULTS, &result.id, result)
    }

    pub fn all_results(&self) -> Result<Vec<MatchResult>, String> {
        self.list_json(RESULTS)
    }

    pub fn results_for_player(&self, player_id: &str) -> Result<Vec<MatchResult>, String> {
        Ok(self
            .all_results()?
            .into_iter()
            .filter(|r| r.player_a == player_id || r.player_b == player_id)
            .collect())
    }

    pub fn get_replay(&self, replay_id: &str) -> Result<Option<String>, String> {
        self.get_str(REPLAYS, replay_id)
    }

    // ---- season reset ----

    /// Clear the per-season data (teams, drafts, results, replays) while keeping
    /// players (their points + titles carry across season resets). Tables are
    /// dropped and recreated lazily on the next write.
    pub fn clear_season_data(&self) -> Result<(), String> {
        let tx = self.inner.begin_write().map_err(|e| e.to_string())?;
        for table in [TEAMS, DRAFTS, RESULTS, REPLAYS] {
            match tx.delete_table(table) {
                Ok(_) => {}
                Err(redb::TableError::TableDoesNotExist(_)) => {}
                Err(e) => return Err(e.to_string()),
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn temp_db() -> Db {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("tarot-test-{}-{n}.redb", std::process::id()));
        let _ = std::fs::remove_file(&path);
        Db::open(path).unwrap()
    }

    fn player(id: &str, name: &str, points: i64) -> Player {
        Player {
            id: id.to_string(),
            name: name.to_string(),
            points,
            title: None,
        }
    }

    #[test]
    fn players_round_trip_and_standings_sort_by_points() {
        let db = temp_db();
        assert!(db.all_players().unwrap().is_empty());
        db.upsert_player(&player("p1", "Ada", 1100)).unwrap();
        db.upsert_player(&player("p2", "Bo", 1500)).unwrap();
        db.upsert_player(&player("p3", "Cy", 1500)).unwrap();

        assert_eq!(db.get_player("p2").unwrap().unwrap().points, 1500);
        assert_eq!(db.all_players().unwrap().len(), 3);

        // Standings: points desc, then name asc → Bo(1500), Cy(1500), Ada(1100).
        let names: Vec<String> = db
            .standings()
            .unwrap()
            .into_iter()
            .map(|p| p.name)
            .collect();
        assert_eq!(names, vec!["Bo", "Cy", "Ada"]);

        // Upsert overwrites.
        db.upsert_player(&player("p1", "Ada", 1700)).unwrap();
        assert_eq!(db.standings().unwrap()[0].name, "Ada");
    }

    #[test]
    fn team_and_draft_round_trip() {
        let db = temp_db();
        assert!(db.get_team("p1").unwrap().is_none());
        db.set_team("p1", r#"{"version":2,"name":"T","characters":[]}"#)
            .unwrap();
        assert!(db
            .get_team("p1")
            .unwrap()
            .unwrap()
            .contains("\"name\":\"T\""));
        assert_eq!(db.all_teams().unwrap().len(), 1);

        // Draft defaults to empty, then persists.
        assert!(db.get_draft("p1").unwrap().claimed.is_empty());
        let mut draft = db.get_draft("p1").unwrap();
        draft.claimed.push(crate::models::DraftPick {
            beat: 0,
            choice: "Aegis".to_string(),
        });
        db.set_draft("p1", &draft).unwrap();
        assert_eq!(db.get_draft("p1").unwrap().claimed.len(), 1);
    }

    #[test]
    fn results_and_replays() {
        let db = temp_db();
        let result = MatchResult {
            id: "r1".to_string(),
            day: 0,
            player_a: "p1".to_string(),
            player_b: "p2".to_string(),
            winner: "a".to_string(),
            seed: 7,
            replay_id: "rep1".to_string(),
        };
        db.add_result(&result, r#"{"winner":"team_a"}"#).unwrap();
        assert_eq!(db.all_results().unwrap().len(), 1);
        assert_eq!(db.results_for_player("p2").unwrap().len(), 1);
        assert!(db.results_for_player("p3").unwrap().is_empty());
        assert!(db.get_replay("rep1").unwrap().unwrap().contains("team_a"));
    }
}
