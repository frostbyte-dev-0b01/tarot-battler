//! Persistence models for the single-pod season.
//!
//! Kept deliberately small for the friends MVP: one pod, everyone stays in it.
//! Teams are stored as their raw team-config JSON (the same shape the engine and
//! UI already use), so there is one source of truth for a team's shape.
//!
//! Used by the access layer/tests now and wired into the API in later issues;
//! allow dead code until then.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// A player in the pod. `points` is the persistent total that drives tiering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub points: i64,
    /// Cosmetic title awarded by the Victors round (no power). Carries across
    /// season resets.
    #[serde(default)]
    pub title: Option<String>,
}

/// The live season clock for the pod. The fixed beat schedule + content pool
/// live in code (the draft logic); this records where the season currently is.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Season {
    pub id: String,
    pub name: String,
    /// Day index since the season started (0-based).
    #[serde(default)]
    pub day: u32,
    /// Number of draft beats revealed so far.
    #[serde(default)]
    pub beats_revealed: u32,
    pub created_unix: i64,
    /// Seed driving deterministic, pod-wide draft offers and match seeds.
    #[serde(default)]
    pub seed: u64,
}

/// One draft pick a player has claimed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftPick {
    /// 0-based beat index.
    pub beat: u32,
    /// The chosen option id (character / item / passive / banner / ...).
    pub choice: String,
}

/// A player's accumulated draft choices for the season.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DraftState {
    #[serde(default)]
    pub claimed: Vec<DraftPick>,
}

/// The outcome of one battle between two pod members.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    pub id: String,
    pub day: u32,
    pub player_a: String,
    pub player_b: String,
    /// "a", "b", or "draw".
    pub winner: String,
    pub seed: u64,
    /// Key into the replay store.
    pub replay_id: String,
}
