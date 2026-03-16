//! Rule evaluation: checks ordered conditions to select an ability.

use crate::abilities::AbilityMap;
use crate::models::{CharacterState, Comparator, Condition, ConditionSubject};

/// Evaluate the actor's rules in order. Returns the name of the first ability
/// whose conditions are all met AND whose SPI cost the actor can afford.
/// Returns None if no rule matches (caller should fall back to basic attack).
pub fn evaluate_rules(
    actor: &CharacterState,
    target: Option<&CharacterState>,
    allies: &[CharacterState],
    abilities: &AbilityMap,
) -> Option<String> {
    for rule in actor.rules() {
        let ability_def = match abilities.get(&rule.ability) {
            Some(def) => def,
            None => continue,
        };

        // Check SPI cost
        if actor.current_spi() < ability_def.spi_cost {
            continue;
        }

        // Check all conditions (AND)
        let all_met = rule
            .conditions
            .iter()
            .all(|cond| check_condition(cond, actor, target, allies));

        if all_met {
            return Some(rule.ability.clone());
        }
    }
    None
}

/// Check a single condition against the relevant subject.
fn check_condition(
    cond: &Condition,
    actor: &CharacterState,
    target: Option<&CharacterState>,
    allies: &[CharacterState],
) -> bool {
    match &cond.subject {
        ConditionSubject::SelfChar => {
            let val = actor.query_value(&cond.value);
            compare(val, &cond.comparator, cond.threshold)
        }
        ConditionSubject::Target => match target {
            Some(t) => {
                let val = t.query_value(&cond.value);
                compare(val, &cond.comparator, cond.threshold)
            }
            None => false,
        },
        ConditionSubject::Companion => {
            // True if ANY companion matches the condition
            let comp_ids = actor.companions();
            allies
                .iter()
                .filter(|c| c.is_alive() && comp_ids.contains(&c.id()))
                .any(|c| {
                    let val = c.query_value(&cond.value);
                    compare(val, &cond.comparator, cond.threshold)
                })
        }
    }
}

fn compare(val: u32, comparator: &Comparator, threshold: u32) -> bool {
    match comparator {
        Comparator::Gte => val >= threshold,
        Comparator::Lte => val <= threshold,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abilities::AbilityDef;
    use crate::models::*;
    use std::collections::HashMap;

    fn make_char_with_rules(
        id: u32,
        stats: Vec<(Stat, u32)>,
        rules: Vec<Rule>,
    ) -> CharacterState {
        let config = CharacterConfig {
            base_name: format!("Char{}", id),
            passive: String::new(),
            actives: Vec::new(),
            item: None,
            position: Position { row: 0, col: 0 },
            stats: stats.into_iter().collect(),
            rules,
        };
        CharacterState::from_config(id, &config)
    }

    fn make_char(id: u32, stats: Vec<(Stat, u32)>) -> CharacterState {
        make_char_with_rules(id, stats, Vec::new())
    }

    fn make_abilities() -> AbilityMap {
        let mut map = HashMap::new();
        map.insert(
            "Crush".to_string(),
            AbilityDef {
                spi_cost: 2,
                primitives: Vec::new(),
            },
        );
        map.insert(
            "Embolden".to_string(),
            AbilityDef {
                spi_cost: 3,
                primitives: Vec::new(),
            },
        );
        map
    }

    #[test]
    fn empty_conditions_always_matches() {
        let rules = vec![Rule {
            ability: "Crush".to_string(),
            conditions: Vec::new(),
        }];
        let actor = make_char_with_rules(0, vec![(Stat::SPI, 5)], rules);
        let abilities = make_abilities();
        let result = evaluate_rules(&actor, None, &[], &abilities);
        assert_eq!(result.as_deref(), Some("Crush"));
    }

    #[test]
    fn spi_gating_skips_expensive_ability() {
        let rules = vec![Rule {
            ability: "Embolden".to_string(), // costs 3
            conditions: Vec::new(),
        }];
        let mut actor = make_char_with_rules(0, vec![(Stat::SPI, 5)], rules);
        actor.spend_spi(3); // only 2 left, need 3
        let abilities = make_abilities();
        let result = evaluate_rules(&actor, None, &[], &abilities);
        assert_eq!(result, None);
    }

    #[test]
    fn target_hp_lte_condition() {
        let rules = vec![Rule {
            ability: "Crush".to_string(),
            conditions: vec![Condition {
                subject: ConditionSubject::Target,
                value: QueryValue::Hp,
                comparator: Comparator::Lte,
                threshold: 5,
            }],
        }];
        let actor = make_char_with_rules(0, vec![(Stat::SPI, 5)], rules);
        let abilities = make_abilities();

        // Target HP=20 → doesn't match
        let target_high = make_char(1, vec![(Stat::CON, 10)]); // HP=20
        assert!(evaluate_rules(&actor, Some(&target_high), &[], &abilities).is_none());

        // Target HP=4 → matches
        let mut target_low = make_char(1, vec![(Stat::CON, 10)]);
        target_low.take_damage(16); // HP=4
        assert_eq!(evaluate_rules(&actor, Some(&target_low), &[], &abilities).as_deref(), Some("Crush"));
    }

    #[test]
    fn companion_spi_lte_condition() {
        let rules = vec![Rule {
            ability: "Embolden".to_string(),
            conditions: vec![Condition {
                subject: ConditionSubject::Companion,
                value: QueryValue::Spi,
                comparator: Comparator::Lte,
                threshold: 1,
            }],
        }];
        let mut actor = make_char_with_rules(0, vec![(Stat::SPI, 5)], rules);
        actor.set_companions(vec![1]);
        let abilities = make_abilities();

        // Companion has plenty of SPI
        let companion_full = make_char(1, vec![(Stat::CON, 5), (Stat::SPI, 5)]);
        assert!(evaluate_rules(&actor, None, &[companion_full], &abilities).is_none());

        // Companion has low SPI
        let mut companion_low = make_char(1, vec![(Stat::CON, 5), (Stat::SPI, 5)]);
        companion_low.spend_spi(4); // SPI=1
        assert_eq!(evaluate_rules(&actor, None, &[companion_low], &abilities).as_deref(), Some("Embolden"));
    }

    #[test]
    fn self_stat_gte_condition() {
        let rules = vec![Rule {
            ability: "Crush".to_string(),
            conditions: vec![Condition {
                subject: ConditionSubject::SelfChar,
                value: QueryValue::Stat(Stat::STR),
                comparator: Comparator::Gte,
                threshold: 10,
            }],
        }];
        let actor = make_char_with_rules(0, vec![(Stat::SPI, 5), (Stat::STR, 12)], rules.clone());
        let abilities = make_abilities();
        assert_eq!(evaluate_rules(&actor, None, &[], &abilities).as_deref(), Some("Crush"));

        let actor_weak = make_char_with_rules(0, vec![(Stat::SPI, 5), (Stat::STR, 8)], rules);
        assert!(evaluate_rules(&actor_weak, None, &[], &abilities).is_none());
    }

    #[test]
    fn rules_evaluated_in_order() {
        let rules = vec![
            Rule {
                ability: "Crush".to_string(),
                conditions: vec![Condition {
                    subject: ConditionSubject::Target,
                    value: QueryValue::Hp,
                    comparator: Comparator::Lte,
                    threshold: 3,
                }],
            },
            Rule {
                ability: "Embolden".to_string(),
                conditions: Vec::new(), // always matches
            },
        ];
        let actor = make_char_with_rules(0, vec![(Stat::SPI, 5)], rules);
        let abilities = make_abilities();

        // Target HP high → first rule fails, Embolden matches
        let target = make_char(1, vec![(Stat::CON, 10)]);
        assert_eq!(evaluate_rules(&actor, Some(&target), &[], &abilities).as_deref(), Some("Embolden"));

        // Target HP low → first rule matches
        let mut target_low = make_char(1, vec![(Stat::CON, 10)]);
        target_low.take_damage(18); // HP=2
        assert_eq!(evaluate_rules(&actor, Some(&target_low), &[], &abilities).as_deref(), Some("Crush"));
    }

    #[test]
    fn no_target_means_target_condition_fails() {
        let rules = vec![Rule {
            ability: "Crush".to_string(),
            conditions: vec![Condition {
                subject: ConditionSubject::Target,
                value: QueryValue::Hp,
                comparator: Comparator::Lte,
                threshold: 100,
            }],
        }];
        let actor = make_char_with_rules(0, vec![(Stat::SPI, 5)], rules);
        let abilities = make_abilities();
        assert!(evaluate_rules(&actor, None, &[], &abilities).is_none());
    }
}
