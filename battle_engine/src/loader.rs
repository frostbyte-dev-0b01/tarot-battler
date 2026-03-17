//! Load character configurations and ability definitions from JSON files.

use std::collections::HashSet;
use std::path::Path;

use crate::abilities::{AbilityMap, AbilityTarget, PassiveDef, PassiveMap, Primitive};
use crate::models::CharacterConfig;
use crate::statuses::{StatusBehavior, StatusDef, StatusMap};

pub fn load_characters(path: &Path) -> Result<Vec<CharacterConfig>, String> {
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
    let mut errors = Vec::new();

    for character in characters {
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
        }
    }

    for (team_name, team) in [
        ("team_a", &characters[..characters.len() / 2]),
        ("team_b", &characters[characters.len() / 2..]),
    ] {
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
        AbilityTarget::CurrentTarget | AbilityTarget::AllEnemies
    )
}

fn is_ally_target(target: &AbilityTarget) -> bool {
    matches!(
        target,
        AbilityTarget::SelfChar | AbilityTarget::Companions | AbilityTarget::AllAllies
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
        StatusBehavior::SkipTurn => StatusPolarity::Debuff,
    }
}

fn status_requires_stat(def: &StatusDef) -> bool {
    matches!(def.behavior, StatusBehavior::StatModPerStack { .. })
}

fn target_label(target: &AbilityTarget) -> &'static str {
    match target {
        AbilityTarget::CurrentTarget => "current_target",
        AbilityTarget::SelfChar => "self",
        AbilityTarget::Companions => "companions",
        AbilityTarget::AllEnemies => "all_enemies",
        AbilityTarget::AllAllies => "all_allies",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Stat;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn load_characters_from_bundled_file() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/data/characters.json");
        let chars = load_characters(&path).unwrap();
        assert!(chars.len() >= 2);
        assert_eq!(chars[0].base_name, "The Emperor");
        assert_eq!(*chars[0].stats.get(&Stat::CON).unwrap(), 10);
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
            base_name: "Tester".to_string(),
            passive: "UnknownPassive".to_string(),
            actives: vec!["KnownAbility".to_string(), "MissingAbility".to_string()],
            item: None,
            position: crate::models::Position { row: 0, col: 0 },
            stats: [(Stat::CON, 10)].into_iter().collect(),
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
            base_name: "Tester".to_string(),
            passive: String::new(),
            actives: vec!["KnownAbility".to_string()],
            item: None,
            position: crate::models::Position { row: 0, col: 0 },
            stats: [(Stat::CON, 10)].into_iter().collect(),
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
                base_name: "A".to_string(),
                passive: String::new(),
                actives: Vec::new(),
                item: None,
                position: crate::models::Position { row: 3, col: 0 },
                stats: [(Stat::CON, 10)].into_iter().collect(),
                rules: Vec::new(),
            },
            CharacterConfig {
                base_name: "B".to_string(),
                passive: String::new(),
                actives: Vec::new(),
                item: None,
                position: crate::models::Position { row: 0, col: 0 },
                stats: [(Stat::CON, 10)].into_iter().collect(),
                rules: Vec::new(),
            },
            CharacterConfig {
                base_name: "C".to_string(),
                passive: String::new(),
                actives: Vec::new(),
                item: None,
                position: crate::models::Position { row: 1, col: 1 },
                stats: [(Stat::CON, 10)].into_iter().collect(),
                rules: Vec::new(),
            },
            CharacterConfig {
                base_name: "D".to_string(),
                passive: String::new(),
                actives: Vec::new(),
                item: None,
                position: crate::models::Position { row: 1, col: 1 },
                stats: [(Stat::CON, 10)].into_iter().collect(),
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
                    target: AbilityTarget::CurrentTarget,
                    status: "Empower".to_string(),
                    stat: Some(Stat::STR),
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
                    target: AbilityTarget::CurrentTarget,
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
    fn validate_content_rejects_missing_stat_for_stat_mod_status() {
        let chars = Vec::new();
        let abilities = [(
            "BadEmpower".to_string(),
            crate::abilities::AbilityDef {
                mp_cost: 1,
                primitives: vec![Primitive::ApplyStatus {
                    target: AbilityTarget::SelfChar,
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
                    target: AbilityTarget::CurrentTarget,
                    status: "Bleed".to_string(),
                    stat: Some(Stat::STR),
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
}
