//! Load character configurations and ability definitions from JSON files.

use std::collections::HashSet;
use std::path::Path;

use crate::abilities::{
    AbilityMap, AbilityTarget, PassiveDef, PassiveMap, Primitive, SimpleAbilityTarget,
    TargetCategory,
};
use crate::models::{CharacterConfig, QueryValue, Stat};
use crate::statuses::{StatusBehavior, StatusDef, StatusMap};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TeamConfig {
    pub version: u32,
    pub name: String,
    pub characters: Vec<TeamCharacterLoadout>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TeamCharacterLoadout {
    pub id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    pub position: crate::models::Position,
    pub stats: std::collections::HashMap<crate::models::Stat, u32>,
    pub passive: String,
    pub actives: Vec<String>,
    pub item: Option<String>,
    #[serde(default)]
    pub rules: Vec<crate::models::Rule>,
}

pub fn load_characters(path: &Path) -> Result<Vec<CharacterConfig>, String> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
}

pub fn load_team_config(path: &Path) -> Result<TeamConfig, String> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
}

pub fn load_abilities(path: &Path) -> Result<AbilityMap, String> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
}

pub fn load_passives(path: &Path) -> Result<PassiveMap, String> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
}

pub fn load_statuses(path: &Path) -> Result<StatusMap, String> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
}

pub fn validate_content(
    characters: &[CharacterConfig],
    abilities: &AbilityMap,
    passives: &PassiveMap,
    statuses: &StatusMap,
) -> Result<(), String> {
    let (team_a, team_b) = characters.split_at(characters.len() / 2);
    validate_teams(team_a, team_b, abilities, passives, statuses)
}

pub fn validate_teams(
    team_a: &[CharacterConfig],
    team_b: &[CharacterConfig],
    abilities: &AbilityMap,
    passives: &PassiveMap,
    statuses: &StatusMap,
) -> Result<(), String> {
    let mut errors = Vec::new();

    for (team_name, team) in [("team_a", team_a), ("team_b", team_b)] {
        validate_team(team_name, team, abilities, passives, statuses, &mut errors);
    }

    for (team_name, team) in [("team_a", team_a), ("team_b", team_b)] {
        let mut seen_positions = HashSet::new();
        for character in team {
            if !seen_positions.insert((character.position.row, character.position.col)) {
                errors.push(format!(
                    "{} has duplicate position ({}, {}) in {}",
                    character.base_name, character.position.row, character.position.col, team_name
                ));
            }
        }
    }

    for (ability_name, ability) in abilities {
        validate_primitives(
            &format!("ability '{}'", ability_name),
            &ability.primitives,
            statuses,
            &mut errors,
        );
    }

    for (passive_name, passive) in passives {
        if let PassiveDef::Triggered { primitives, .. } = passive {
            validate_primitives(
                &format!("passive '{}'", passive_name),
                primitives,
                statuses,
                &mut errors,
            );
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

pub fn validate_team_config(
    team: &TeamConfig,
    abilities: &AbilityMap,
    passives: &PassiveMap,
    statuses: &StatusMap,
) -> Result<Vec<CharacterConfig>, String> {
    let mut errors = Vec::new();
    let mut seen_ids = HashSet::new();

    if team.version != 1 {
        errors.push(format!(
            "team '{}' uses unsupported schema version {}",
            team.name, team.version
        ));
    }

    if team.characters.is_empty() {
        errors.push(format!("team '{}' must contain at least 1 character", team.name));
    }

    for character in &team.characters {
        if !seen_ids.insert(character.id.as_str()) {
            errors.push(format!(
                "team '{}' has duplicate character id '{}'",
                team.name, character.id
            ));
        }
    }

    let characters: Vec<CharacterConfig> = team
        .characters
        .iter()
        .map(|character| CharacterConfig {
            id: Some(character.id.clone()),
            base_name: character
                .display_name
                .clone()
                .unwrap_or_else(|| character.id.clone()),
            display_name: character.display_name.clone(),
            passive: character.passive.clone(),
            actives: character.actives.clone(),
            item: character.item.clone(),
            position: character.position.clone(),
            stats: character.stats.clone(),
            rules: character.rules.clone(),
        })
        .collect();

    validate_team(&team.name, &characters, abilities, passives, statuses, &mut errors);

    let mut seen_positions = HashSet::new();
    for character in &characters {
        if !seen_positions.insert((character.position.row, character.position.col)) {
            errors.push(format!(
                "{} has duplicate position ({}, {}) in team '{}'",
                character.base_name, character.position.row, character.position.col, team.name
            ));
        }
    }

    for (ability_name, ability) in abilities {
        validate_primitives(
            &format!("ability '{}'", ability_name),
            &ability.primitives,
            statuses,
            &mut errors,
        );
    }

    for (passive_name, passive) in passives {
        if let PassiveDef::Triggered { primitives, .. } = passive {
            validate_primitives(
                &format!("passive '{}'", passive_name),
                primitives,
                statuses,
                &mut errors,
            );
        }
    }

    if errors.is_empty() {
        Ok(characters)
    } else {
        Err(errors.join("\n"))
    }
}

fn validate_team(
    team_name: &str,
    team: &[CharacterConfig],
    abilities: &AbilityMap,
    passives: &PassiveMap,
    statuses: &StatusMap,
    errors: &mut Vec<String>,
) {
    for character in team {
        if !character.position.is_valid() {
            errors.push(format!(
                "{} has invalid position ({}, {})",
                character.base_name, character.position.row, character.position.col
            ));
        }

        if !character.passive.is_empty() && !passives.contains_key(&character.passive) {
            errors.push(format!(
                "{} references unknown passive '{}'",
                character.base_name, character.passive
            ));
        }

        let equipped: HashSet<&str> = character.actives.iter().map(String::as_str).collect();

        for ability in &character.actives {
            if !abilities.contains_key(ability) {
                errors.push(format!(
                    "{} equips unknown ability '{}'",
                    character.base_name, ability
                ));
            }
        }

        for rule in &character.rules {
            if !abilities.contains_key(&rule.ability) {
                errors.push(format!(
                    "{} has rule for unknown ability '{}'",
                    character.base_name, rule.ability
                ));
            } else if !equipped.contains(rule.ability.as_str()) {
                errors.push(format!(
                    "{} has rule for unequipped ability '{}'",
                    character.base_name, rule.ability
                ));
            }

            for condition in &rule.conditions {
                validate_rule_query_value(
                    &character.base_name,
                    &condition.value,
                    statuses,
                    errors,
                );
            }
        }
        if let Some(id) = &character.id {
            if id.trim().is_empty() {
                errors.push(format!("{} in team '{}' has an empty id", character.base_name, team_name));
            }
        }
    }
}

fn validate_rule_query_value(
    character_name: &str,
    value: &QueryValue,
    statuses: &StatusMap,
    errors: &mut Vec<String>,
) {
    match value {
        QueryValue::HasStatus(key) | QueryValue::StatusStacks(key) => {
            validate_status_query_key(character_name, key, statuses, errors);
        }
        _ => {}
    }
}

fn validate_status_query_key(
    character_name: &str,
    key: &str,
    statuses: &StatusMap,
    errors: &mut Vec<String>,
) {
    let (status_name, stat) = match key.split_once(':') {
        Some((name, suffix)) => (name, parse_status_query_stat(suffix)),
        None => (key, None),
    };

    let Some(def) = statuses.get(status_name) else {
        errors.push(format!(
            "{} references unknown status '{}' in rule query",
            character_name, status_name
        ));
        return;
    };

    if key.contains(':') && stat.is_none() {
        errors.push(format!(
            "{} uses invalid stat suffix in rule query '{}'",
            character_name, key
        ));
        return;
    }

    let requires_stat = status_requires_stat(def);
    if requires_stat && stat.is_none() {
        errors.push(format!(
            "{} uses status query '{}' without required stat suffix",
            character_name, key
        ));
    } else if !requires_stat && stat.is_some() {
        errors.push(format!(
            "{} uses status query '{}' with unexpected stat suffix",
            character_name, key
        ));
    }
}

fn parse_status_query_stat(value: &str) -> Option<Stat> {
    match value {
        "VIT" => Some(Stat::VIT),
        "MGT" => Some(Stat::MGT),
        "MAG" => Some(Stat::MAG),
        "ARM" => Some(Stat::ARM),
        "RES" => Some(Stat::RES),
        "SPD" => Some(Stat::SPD),
        "WIL" => Some(Stat::WIL),
        _ => None,
    }
}

fn validate_primitives(
    source_name: &str,
    primitives: &[Primitive],
    statuses: &StatusMap,
    errors: &mut Vec<String>,
) {
    for primitive in primitives {
        match primitive {
            Primitive::ApplyStatus {
                target,
                status,
                stat,
                ..
            } => {
                let Some(def) = statuses.get(status) else {
                    errors.push(format!(
                        "{} references unknown status '{}'",
                        source_name, status
                    ));
                    continue;
                };

                validate_status_stat_usage(source_name, status, stat.as_ref(), def, errors);

                if !apply_status_target_is_valid(target, def) {
                    errors.push(format!(
                        "{} applies status '{}' to invalid target side '{}'",
                        source_name,
                        status,
                        target_label(target),
                    ));
                }
            }
            Primitive::RemoveStatus {
                target,
                status,
                stat,
                ..
            } => {
                let Some(def) = statuses.get(status) else {
                    errors.push(format!(
                        "{} references unknown status '{}'",
                        source_name, status
                    ));
                    continue;
                };

                validate_status_stat_usage(source_name, status, stat.as_ref(), def, errors);

                if !remove_status_target_is_valid(target, def) {
                    errors.push(format!(
                        "{} removes status '{}' from invalid target side '{}'",
                        source_name,
                        status,
                        target_label(target),
                    ));
                }
            }
            Primitive::DealPhysicalDamageBonusVsStatus {
                status, stat, ..
            } => {
                let Some(def) = statuses.get(status) else {
                    errors.push(format!(
                        "{} references unknown status '{}'",
                        source_name, status
                    ));
                    continue;
                };

                validate_status_stat_usage(source_name, status, stat.as_ref(), def, errors);
            }
            Primitive::DealMagicalDamageConsumeStatus {
                status, stat, ..
            } => {
                let Some(def) = statuses.get(status) else {
                    errors.push(format!(
                        "{} references unknown status '{}'",
                        source_name, status
                    ));
                    continue;
                };

                validate_status_stat_usage(source_name, status, stat.as_ref(), def, errors);
            }
            Primitive::Retarget { target, .. } => {
                if !is_enemy_target(target) {
                    errors.push(format!(
                        "{} retargets invalid target side '{}'",
                        source_name,
                        target_label(target),
                    ));
                }
            }
            Primitive::IfTargetHasStatus {
                target,
                status,
                stat,
                primitives,
            } => {
                let Some(def) = statuses.get(status) else {
                    errors.push(format!(
                        "{} references unknown status '{}'",
                        source_name, status
                    ));
                    continue;
                };

                validate_status_stat_usage(source_name, status, stat.as_ref(), def, errors);

                if !is_enemy_target(target) {
                    errors.push(format!(
                        "{} checks status '{}' on invalid target side '{}'",
                        source_name,
                        status,
                        target_label(target),
                    ));
                }

                validate_primitives(source_name, primitives, statuses, errors);
            }
            _ => {}
        }
    }
}

fn validate_status_stat_usage(
    source_name: &str,
    status_name: &str,
    stat: Option<&crate::models::Stat>,
    def: &StatusDef,
    errors: &mut Vec<String>,
) {
    let requires_stat = status_requires_stat(def);
    if requires_stat && stat.is_none() {
        errors.push(format!(
            "{} uses status '{}' without required stat field",
            source_name, status_name
        ));
    } else if !requires_stat && stat.is_some() {
        errors.push(format!(
            "{} uses status '{}' with unexpected stat field",
            source_name, status_name
        ));
    }
}

fn apply_status_target_is_valid(target: &AbilityTarget, def: &StatusDef) -> bool {
    match status_polarity(def) {
        StatusPolarity::Buff => is_ally_target(target),
        StatusPolarity::Debuff => is_enemy_target(target),
    }
}

fn remove_status_target_is_valid(target: &AbilityTarget, def: &StatusDef) -> bool {
    match status_polarity(def) {
        StatusPolarity::Buff => is_enemy_target(target),
        StatusPolarity::Debuff => is_ally_target(target),
    }
}

fn is_enemy_target(target: &AbilityTarget) -> bool {
    matches!(
        target,
        AbilityTarget::Simple(SimpleAbilityTarget::CurrentTarget)
            | AbilityTarget::Simple(SimpleAbilityTarget::TriggerTarget)
            | AbilityTarget::Simple(SimpleAbilityTarget::FrontRow)
            | AbilityTarget::Simple(SimpleAbilityTarget::AllEnemies)
            | AbilityTarget::Detailed(crate::abilities::TargetSpec {
                category: TargetCategory::Enemy,
                ..
            })
    )
}

fn is_ally_target(target: &AbilityTarget) -> bool {
    matches!(
        target,
        AbilityTarget::Simple(SimpleAbilityTarget::SelfChar)
            | AbilityTarget::Simple(SimpleAbilityTarget::Companions)
            | AbilityTarget::Simple(SimpleAbilityTarget::AllAllies)
            | AbilityTarget::Detailed(crate::abilities::TargetSpec {
                category: TargetCategory::Ally | TargetCategory::Companion,
                ..
            })
    )
}

#[derive(Clone, Copy)]
enum StatusPolarity {
    Buff,
    Debuff,
}

fn status_polarity(def: &StatusDef) -> StatusPolarity {
    match &def.behavior {
        StatusBehavior::DamagePerStack { .. } => StatusPolarity::Debuff,
        StatusBehavior::HealPerStack { .. } => StatusPolarity::Buff,
        StatusBehavior::StatModPerStack { magnitude } if *magnitude < 0 => StatusPolarity::Debuff,
        StatusBehavior::StatModPerStack { .. } => StatusPolarity::Buff,
        StatusBehavior::Ward => StatusPolarity::Buff,
        StatusBehavior::SkipTurn => StatusPolarity::Debuff,
    }
}

fn status_requires_stat(def: &StatusDef) -> bool {
    matches!(def.behavior, StatusBehavior::StatModPerStack { .. })
}

fn target_label(target: &AbilityTarget) -> &'static str {
    match target {
        AbilityTarget::Simple(SimpleAbilityTarget::CurrentTarget) => "current_target",
        AbilityTarget::Simple(SimpleAbilityTarget::CurrentTargetAndCompanions) => {
            "current_target_and_companions"
        }
        AbilityTarget::Simple(SimpleAbilityTarget::TriggerTarget) => "trigger_target",
        AbilityTarget::Simple(SimpleAbilityTarget::SelfChar) => "self",
        AbilityTarget::Simple(SimpleAbilityTarget::Companions) => "companions",
        AbilityTarget::Simple(SimpleAbilityTarget::FrontRow) => "front_row",
        AbilityTarget::Simple(SimpleAbilityTarget::AllEnemies) => "all_enemies",
        AbilityTarget::Simple(SimpleAbilityTarget::AllAllies) => "all_allies",
        AbilityTarget::Detailed(spec) => match spec.category {
            TargetCategory::Companion => "companion",
            TargetCategory::Ally => "ally",
            TargetCategory::Enemy => "enemy",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Stat;
    use std::collections::HashMap;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn load_characters_from_bundled_file() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/data/characters.json");
        let chars = load_characters(&path).unwrap();
        assert!(chars.len() >= 2);
        assert_eq!(chars[0].base_name, "The Emperor");
        assert_eq!(*chars[0].stats.get(&Stat::VIT).unwrap(), 10);
    }

    #[test]
    fn load_characters_error_on_missing_file() {
        let result = load_characters(Path::new("nonexistent.json"));
        assert!(result.is_err());
    }

    #[test]
    fn load_characters_error_on_invalid_json() {
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "not valid json").unwrap();
        let result = load_characters(tmp.path());
        assert!(result.is_err());
    }

    #[test]
    fn load_team_config_from_ui_schema() {
        let mut tmp = NamedTempFile::new().unwrap();
        write!(
            tmp,
            r#"{{
                "version": 1,
                "name": "Test Team",
                "characters": [
                    {{
                        "id": "the_emperor",
                        "display_name": "The Emperor",
                        "position": {{ "row": 0, "col": 0 }},
                        "stats": {{
                            "vit": 10,
                            "mgt": 6,
                            "mag": 4,
                            "arm": 3,
                            "res": 2,
                            "spd": 4,
                            "wil": 5
                        }},
                        "passive": "Authority",
                        "actives": ["Crush", "Embolden"],
                        "item": null,
                        "rules": [
                            {{
                                "ability": "Crush",
                                "when": [
                                    {{
                                        "subject": "self",
                                        "value": "mp",
                                        "op": "gte",
                                        "threshold": 2
                                    }}
                                ]
                            }}
                        ]
                    }}
                ]
            }}"#
        )
        .unwrap();

        let team = load_team_config(tmp.path()).unwrap();
        assert_eq!(team.name, "Test Team");
        assert_eq!(team.characters.len(), 1);
        assert_eq!(team.characters[0].id, "the_emperor");
        assert_eq!(team.characters[0].display_name.as_deref(), Some("The Emperor"));
        assert_eq!(team.characters[0].rules[0].conditions.len(), 1);
    }

    #[test]
    fn validate_team_config_rejects_duplicate_ids() {
        let abilities = load_abilities(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("src/data/abilities.json"),
        )
        .unwrap();
        let passives = load_passives(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("src/data/passives.json"),
        )
        .unwrap();
        let statuses = load_statuses(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("src/data/statuses.json"),
        )
        .unwrap();

        let team = TeamConfig {
            version: 1,
            name: "Bad Team".to_string(),
            characters: vec![
                TeamCharacterLoadout {
                    id: "dup".to_string(),
                    display_name: Some("One".to_string()),
                    position: crate::models::Position { row: 0, col: 0 },
                    stats: HashMap::from([
                        (Stat::VIT, 8),
                        (Stat::MGT, 4),
                        (Stat::MAG, 4),
                        (Stat::ARM, 3),
                        (Stat::RES, 3),
                        (Stat::SPD, 4),
                        (Stat::WIL, 4),
                    ]),
                    passive: "Authority".to_string(),
                    actives: vec!["Crush".to_string()],
                    item: None,
                    rules: vec![],
                },
                TeamCharacterLoadout {
                    id: "dup".to_string(),
                    display_name: Some("Two".to_string()),
                    position: crate::models::Position { row: 0, col: 1 },
                    stats: HashMap::from([
                        (Stat::VIT, 8),
                        (Stat::MGT, 4),
                        (Stat::MAG, 4),
                        (Stat::ARM, 3),
                        (Stat::RES, 3),
                        (Stat::SPD, 4),
                        (Stat::WIL, 4),
                    ]),
                    passive: "Collapse".to_string(),
                    actives: vec!["Shatter".to_string()],
                    item: None,
                    rules: vec![],
                },
            ],
        };

        let err = validate_team_config(&team, &abilities, &passives, &statuses).unwrap_err();
        assert!(err.contains("duplicate character id 'dup'"));
    }

    #[test]
    fn roundtrip_character_config() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/data/characters.json");
        let chars = load_characters(&path).unwrap();
        let json = serde_json::to_string_pretty(&chars).unwrap();
        let reloaded: Vec<CharacterConfig> = serde_json::from_str(&json).unwrap();
        assert_eq!(chars.len(), reloaded.len());
        for (a, b) in chars.iter().zip(reloaded.iter()) {
            assert_eq!(a.base_name, b.base_name);
            assert_eq!(a.stats, b.stats);
        }
    }

    #[test]
    fn bundled_content_references_are_valid() {
        let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/data");
        let chars = load_characters(&data_dir.join("characters.json")).unwrap();
        let abilities = load_abilities(&data_dir.join("abilities.json")).unwrap();
        let passives = load_passives(&data_dir.join("passives.json")).unwrap();
        let statuses = load_statuses(&data_dir.join("statuses.json")).unwrap();

        validate_content(&chars, &abilities, &passives, &statuses).unwrap();
    }

    #[test]
    fn validate_content_rejects_unknown_passives_and_abilities() {
        let chars = vec![CharacterConfig {
            id: None,
            base_name: "Tester".to_string(),
            display_name: None,
            passive: "UnknownPassive".to_string(),
            actives: vec!["KnownAbility".to_string(), "MissingAbility".to_string()],
            item: None,
            position: crate::models::Position { row: 0, col: 0 },
            stats: [(Stat::VIT, 10)].into_iter().collect(),
            rules: vec![crate::models::Rule {
                ability: "MissingAbility".to_string(),
                conditions: Vec::new(),
            }],
        }];
        let abilities = [(
            "KnownAbility".to_string(),
            crate::abilities::AbilityDef {
                mp_cost: 1,
                primitives: Vec::new(),
            },
        )]
        .into_iter()
        .collect();
        let passives = PassiveMap::new();

        let err = validate_content(&chars, &abilities, &passives, &StatusMap::new()).unwrap_err();
        assert!(err.contains("unknown passive"));
        assert!(err.contains("equips unknown ability"));
        assert!(err.contains("rule for unknown ability"));
    }

    #[test]
    fn validate_content_rejects_rules_for_unequipped_abilities() {
        let chars = vec![CharacterConfig {
            id: None,
            base_name: "Tester".to_string(),
            display_name: None,
            passive: String::new(),
            actives: vec!["KnownAbility".to_string()],
            item: None,
            position: crate::models::Position { row: 0, col: 0 },
            stats: [(Stat::VIT, 10)].into_iter().collect(),
            rules: vec![crate::models::Rule {
                ability: "OtherAbility".to_string(),
                conditions: Vec::new(),
            }],
        }];
        let abilities = [
            (
                "KnownAbility".to_string(),
                crate::abilities::AbilityDef {
                    mp_cost: 1,
                    primitives: Vec::new(),
                },
            ),
            (
                "OtherAbility".to_string(),
                crate::abilities::AbilityDef {
                    mp_cost: 1,
                    primitives: Vec::new(),
                },
            ),
        ]
        .into_iter()
        .collect();

        let err = validate_content(&chars, &abilities, &PassiveMap::new(), &StatusMap::new())
            .unwrap_err();
        assert!(err.contains("unequipped ability"));
    }

    #[test]
    fn validate_content_rejects_invalid_and_duplicate_positions() {
        let chars = vec![
            CharacterConfig {
                id: None,
                base_name: "A".to_string(),
                display_name: None,
                passive: String::new(),
                actives: Vec::new(),
                item: None,
                position: crate::models::Position { row: 3, col: 0 },
                stats: [(Stat::VIT, 10)].into_iter().collect(),
                rules: Vec::new(),
            },
            CharacterConfig {
                id: None,
                base_name: "B".to_string(),
                display_name: None,
                passive: String::new(),
                actives: Vec::new(),
                item: None,
                position: crate::models::Position { row: 0, col: 0 },
                stats: [(Stat::VIT, 10)].into_iter().collect(),
                rules: Vec::new(),
            },
            CharacterConfig {
                id: None,
                base_name: "C".to_string(),
                display_name: None,
                passive: String::new(),
                actives: Vec::new(),
                item: None,
                position: crate::models::Position { row: 1, col: 1 },
                stats: [(Stat::VIT, 10)].into_iter().collect(),
                rules: Vec::new(),
            },
            CharacterConfig {
                id: None,
                base_name: "D".to_string(),
                display_name: None,
                passive: String::new(),
                actives: Vec::new(),
                item: None,
                position: crate::models::Position { row: 1, col: 1 },
                stats: [(Stat::VIT, 10)].into_iter().collect(),
                rules: Vec::new(),
            },
        ];

        let err = validate_content(
            &chars,
            &AbilityMap::new(),
            &PassiveMap::new(),
            &StatusMap::new(),
        )
        .unwrap_err();
        assert!(err.contains("invalid position"));
        assert!(err.contains("duplicate position"));
    }

    #[test]
    fn validate_content_rejects_invalid_apply_status_target_side() {
        let chars = Vec::new();
        let abilities = [(
            "BadBuff".to_string(),
            crate::abilities::AbilityDef {
                mp_cost: 1,
                primitives: vec![Primitive::ApplyStatus {
                    target: SimpleAbilityTarget::CurrentTarget.into(),
                    status: "Empower".to_string(),
                    stat: Some(Stat::MGT),
                    stacks: 1,
                }],
            },
        )]
        .into_iter()
        .collect();
        let statuses = [(
            "Empower".to_string(),
            StatusDef {
                behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
                stack_type: crate::statuses::StackType::TickDown,
                opposes: None,
            },
        )]
        .into_iter()
        .collect();

        let err = validate_content(&chars, &abilities, &PassiveMap::new(), &statuses).unwrap_err();
        assert!(err.contains("applies status 'Empower'"));
    }

    #[test]
    fn validate_content_rejects_invalid_remove_status_target_side() {
        let chars = Vec::new();
        let abilities = [(
            "BadCleanse".to_string(),
            crate::abilities::AbilityDef {
                mp_cost: 1,
                primitives: vec![Primitive::RemoveStatus {
                    target: SimpleAbilityTarget::CurrentTarget.into(),
                    status: "Bleed".to_string(),
                    stat: None,
                    stacks: 1,
                }],
            },
        )]
        .into_iter()
        .collect();
        let statuses = [(
            "Bleed".to_string(),
            StatusDef {
                behavior: StatusBehavior::DamagePerStack { value: 1 },
                stack_type: crate::statuses::StackType::TickDown,
                opposes: None,
            },
        )]
        .into_iter()
        .collect();

        let err = validate_content(&chars, &abilities, &PassiveMap::new(), &statuses).unwrap_err();
        assert!(err.contains("removes status 'Bleed'"));
    }

    #[test]
    fn validate_content_rejects_invalid_retarget_target_side() {
        let chars = vec![CharacterConfig {
            id: None,
            base_name: "Tester".to_string(),
            display_name: None,
            passive: String::new(),
            actives: vec!["BadTaunt".to_string()],
            item: None,
            position: crate::models::Position { row: 0, col: 0 },
            stats: [(Stat::VIT, 10)].into_iter().collect(),
            rules: Vec::new(),
        }];
        let abilities = [(
            "BadTaunt".to_string(),
            crate::abilities::AbilityDef {
                mp_cost: 1,
                primitives: vec![crate::abilities::Primitive::Retarget {
                    target: crate::abilities::SimpleAbilityTarget::SelfChar.into(),
                    mode: crate::abilities::RetargetMode::ToSelf,
                    filter: None,
                }],
            },
        )]
        .into_iter()
        .collect();

        let err = validate_content(&chars, &abilities, &PassiveMap::new(), &StatusMap::new())
            .unwrap_err();
        assert!(err.contains("retargets invalid target side"));
    }

    #[test]
    fn validate_content_rejects_missing_stat_for_stat_mod_status() {
        let chars = Vec::new();
        let abilities = [(
            "BadEmpower".to_string(),
            crate::abilities::AbilityDef {
                mp_cost: 1,
                primitives: vec![Primitive::ApplyStatus {
                    target: SimpleAbilityTarget::SelfChar.into(),
                    status: "Empower".to_string(),
                    stat: None,
                    stacks: 1,
                }],
            },
        )]
        .into_iter()
        .collect();
        let statuses = [(
            "Empower".to_string(),
            StatusDef {
                behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
                stack_type: crate::statuses::StackType::TickDown,
                opposes: None,
            },
        )]
        .into_iter()
        .collect();

        let err = validate_content(&chars, &abilities, &PassiveMap::new(), &statuses).unwrap_err();
        assert!(err.contains("without required stat field"));
    }

    #[test]
    fn validate_content_rejects_unexpected_stat_for_non_stat_status() {
        let chars = Vec::new();
        let abilities = [(
            "BadBleed".to_string(),
            crate::abilities::AbilityDef {
                mp_cost: 1,
                primitives: vec![Primitive::ApplyStatus {
                    target: SimpleAbilityTarget::CurrentTarget.into(),
                    status: "Bleed".to_string(),
                    stat: Some(Stat::MGT),
                    stacks: 1,
                }],
            },
        )]
        .into_iter()
        .collect();
        let statuses = [(
            "Bleed".to_string(),
            StatusDef {
                behavior: StatusBehavior::DamagePerStack { value: 1 },
                stack_type: crate::statuses::StackType::TickDown,
                opposes: None,
            },
        )]
        .into_iter()
        .collect();

        let err = validate_content(&chars, &abilities, &PassiveMap::new(), &statuses).unwrap_err();
        assert!(err.contains("with unexpected stat field"));
    }

    #[test]
    fn validate_content_rejects_unknown_status_rule_queries() {
        let chars = vec![CharacterConfig {
            id: None,
            base_name: "Tester".to_string(),
            display_name: None,
            passive: String::new(),
            actives: vec!["Crush".to_string()],
            item: None,
            position: crate::models::Position { row: 0, col: 0 },
            stats: [(Stat::VIT, 10), (Stat::WIL, 5)].into_iter().collect(),
            rules: vec![crate::models::Rule {
                ability: "Crush".to_string(),
                conditions: vec![crate::models::Condition {
                    subject: crate::models::ConditionSubject::SelfChar,
                    value: crate::models::QueryValue::HasStatus("Ward".to_string()),
                    comparator: crate::models::Comparator::Gte,
                    threshold: 1,
                }],
            }],
        }];
        let abilities = [(
            "Crush".to_string(),
            crate::abilities::AbilityDef {
                mp_cost: 1,
                primitives: Vec::new(),
            },
        )]
        .into_iter()
        .collect();

        let err = validate_content(&chars, &abilities, &PassiveMap::new(), &StatusMap::new())
            .unwrap_err();
        assert!(err.contains("unknown status 'Ward' in rule query"));
    }

    #[test]
    fn validate_content_rejects_invalid_status_query_suffixes() {
        let chars = vec![CharacterConfig {
            id: None,
            base_name: "Tester".to_string(),
            display_name: None,
            passive: String::new(),
            actives: vec!["Crush".to_string()],
            item: None,
            position: crate::models::Position { row: 0, col: 0 },
            stats: [(Stat::VIT, 10), (Stat::WIL, 5)].into_iter().collect(),
            rules: vec![
                crate::models::Rule {
                    ability: "Crush".to_string(),
                    conditions: vec![crate::models::Condition {
                        subject: crate::models::ConditionSubject::SelfChar,
                        value: crate::models::QueryValue::StatusStacks("Empower".to_string()),
                        comparator: crate::models::Comparator::Gte,
                        threshold: 2,
                    }],
                },
                crate::models::Rule {
                    ability: "Crush".to_string(),
                    conditions: vec![crate::models::Condition {
                        subject: crate::models::ConditionSubject::SelfChar,
                        value: crate::models::QueryValue::HasStatus("Ward:MGT".to_string()),
                        comparator: crate::models::Comparator::Gte,
                        threshold: 1,
                    }],
                },
                crate::models::Rule {
                    ability: "Crush".to_string(),
                    conditions: vec![crate::models::Condition {
                        subject: crate::models::ConditionSubject::SelfChar,
                        value: crate::models::QueryValue::HasStatus("Ward:BAD".to_string()),
                        comparator: crate::models::Comparator::Gte,
                        threshold: 1,
                    }],
                },
            ],
        }];
        let abilities = [(
            "Crush".to_string(),
            crate::abilities::AbilityDef {
                mp_cost: 1,
                primitives: Vec::new(),
            },
        )]
        .into_iter()
        .collect();
        let statuses = [
            (
                "Empower".to_string(),
                StatusDef {
                    behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
                    stack_type: crate::statuses::StackType::TickDown,
                    opposes: None,
                },
            ),
            (
                "Ward".to_string(),
                StatusDef {
                    behavior: StatusBehavior::Ward,
                    stack_type: crate::statuses::StackType::Permanent,
                    opposes: None,
                },
            ),
        ]
        .into_iter()
        .collect();

        let err = validate_content(&chars, &abilities, &PassiveMap::new(), &statuses).unwrap_err();
        assert!(err.contains("without required stat suffix"));
        assert!(err.contains("with unexpected stat suffix"));
        assert!(err.contains("invalid stat suffix"));
    }
}
