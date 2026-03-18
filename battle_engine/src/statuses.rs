//! Named status effect definitions loaded from JSON.

use std::collections::HashMap;

use crate::models::Stat;

/// How stacks behave over time.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StackType {
    /// Each turn all stacks fire, then one stack falls off.
    TickDown,
    /// Cannot stack additively. Reapplying replaces stacks if higher.
    NoStack,
    /// Never decays. Only removed by RemoveStatus.
    Permanent,
}

/// What each stack does.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "behavior", rename_all = "snake_case")]
pub enum StatusBehavior {
    DamagePerStack { value: u32 },
    HealPerStack { value: u32 },
    StatModPerStack { magnitude: i32 },
    SkipTurn,
    Ward,
}

/// A named status effect definition loaded from JSON.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct StatusDef {
    #[serde(flatten)]
    pub behavior: StatusBehavior,
    pub stack_type: StackType,
    /// Name of the opposing status (e.g., Empower opposes Weaken).
    /// When applied, cancels stacks of the opposing status on the same stat.
    #[serde(default)]
    pub opposes: Option<String>,
}

pub type StatusMap = HashMap<String, StatusDef>;

/// A live status instance on a character.
#[derive(Debug, Clone)]
pub struct StatusInstance {
    pub stacks: u32,
    pub source_id: u32,
    pub behavior: StatusBehavior,
    pub stack_type: StackType,
    /// For StatModPerStack: which stat is modified.
    pub stat: Option<Stat>,
}

/// Build the HashMap key for a status. Stat-mod statuses include the stat
/// (e.g., "Empower:MGT"), others are just the name (e.g., "Bleed").
pub fn status_key(name: &str, stat: Option<&Stat>) -> String {
    match stat {
        Some(s) => format!("{}:{:?}", name, s),
        None => name.to_string(),
    }
}

/// Given a status key like "Empower:MGT" and an opposing name "Weaken",
/// produce the opposite key "Weaken:MGT". For keyless statuses, returns
/// just the opposing name.
pub fn opposite_key(key: &str, opposes: &str) -> String {
    match key.split_once(':') {
        Some((_, suffix)) => format!("{}:{}", opposes, suffix),
        None => opposes.to_string(),
    }
}
