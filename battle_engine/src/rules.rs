//! Rule evaluation: checks ordered conditions to select an ability.

use crate::abilities::AbilityMap;
use crate::models::{CharacterState, Comparator, Condition, ConditionSubject, QueryValue};

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

        // Check SPI cost (reduced by trait, minimum 1)
        let effective_cost = ability_def.spi_cost
            .saturating_sub(actor.spi_cost_reduction())
            .max(1);
        if actor.current_spi() < effective_cost {
            continue;
        }

        // Check all conditions (AND)
        let all_met = rule
            .conditions
            .iter()
            .all(|cond| check_condition(cond, actor, target, allies, &rule.ability));

        if all_met {
            return Some(rule.ability.clone());
        }
    }
    None
}

/// Check a single condition against the relevant subject.
/// `ability_name` provides context for UseCount and TurnsSinceUse queries.
fn check_condition(
    cond: &Condition,
    actor: &CharacterState,
    target: Option<&CharacterState>,
    allies: &[CharacterState],
    ability_name: &str,
) -> bool {
    // UseCount and TurnsSinceUse are always about the actor's own tracking,
    // regardless of subject field.
    match &cond.value {
        QueryValue::UseCount => {
            let val = actor.ability_use_count(ability_name);
            return compare(val, &cond.comparator, cond.threshold);
        }
        QueryValue::TurnsSinceUse => {
            let val = actor.turns_since_ability_use(ability_name);
            return compare(val, &cond.comparator, cond.threshold);
        }
        _ => {}
    }

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
            // True if ANY adjacent companion matches
            let comp_ids = actor.companions();
            allies
                .iter()
                .filter(|c| c.is_alive() && comp_ids.contains(&c.id()))
                .any(|c| {
                    let val = c.query_value(&cond.value);
                    compare(val, &cond.comparator, cond.threshold)
                })
        }
        ConditionSubject::Ally => {
            // True if ANY living teammate matches (excluding self)
            allies
                .iter()
                .filter(|c| c.is_alive() && c.id() != actor.id())
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

    #[test]
    fn ally_condition_checks_all_living_teammates() {
        let rules = vec![Rule {
            ability: "Embolden".to_string(),
            conditions: vec![Condition {
                subject: ConditionSubject::Ally,
                value: QueryValue::Hp,
                comparator: Comparator::Lte,
                threshold: 5,
            }],
        }];
        let actor = make_char_with_rules(0, vec![(Stat::SPI, 5), (Stat::CON, 10)], rules);
        let abilities = make_abilities();

        // All allies healthy
        let ally = make_char(1, vec![(Stat::CON, 10)]);
        assert!(evaluate_rules(&actor, None, &[ally], &abilities).is_none());

        // One ally low HP — not adjacent (not a companion), but still an ally
        let mut ally_hurt = make_char(1, vec![(Stat::CON, 10)]);
        ally_hurt.take_damage(16); // HP=4
        assert_eq!(evaluate_rules(&actor, None, &[ally_hurt], &abilities).as_deref(), Some("Embolden"));
    }

    #[test]
    fn ally_condition_excludes_self() {
        // Actor has low HP, but Ally condition should not match self
        let rules = vec![Rule {
            ability: "Embolden".to_string(),
            conditions: vec![Condition {
                subject: ConditionSubject::Ally,
                value: QueryValue::Hp,
                comparator: Comparator::Lte,
                threshold: 5,
            }],
        }];
        let mut actor = make_char_with_rules(0, vec![(Stat::SPI, 5), (Stat::CON, 10)], rules);
        actor.take_damage(18); // actor HP=2, but should not self-match

        let abilities = make_abilities();
        // Pass actor in allies list (simulating team slice that includes self)
        let actor_clone = actor.clone();
        assert!(evaluate_rules(&actor, None, &[actor_clone], &abilities).is_none());
    }

    #[test]
    fn use_count_condition() {
        let rules = vec![Rule {
            ability: "Crush".to_string(),
            conditions: vec![Condition {
                subject: ConditionSubject::SelfChar,
                value: QueryValue::UseCount,
                comparator: Comparator::Lte,
                threshold: 2, // use at most 2 times
            }],
        }];
        let mut actor = make_char_with_rules(0, vec![(Stat::SPI, 10)], rules);
        let abilities = make_abilities();

        // Never used → count=0, 0 <= 2 → matches
        assert_eq!(evaluate_rules(&actor, None, &[], &abilities).as_deref(), Some("Crush"));

        // Used twice → count=2, 2 <= 2 → still matches
        actor.record_ability_use("Crush");
        actor.record_ability_use("Crush");
        assert_eq!(evaluate_rules(&actor, None, &[], &abilities).as_deref(), Some("Crush"));

        // Used three times → count=3, 3 <= 2 → fails
        actor.record_ability_use("Crush");
        assert!(evaluate_rules(&actor, None, &[], &abilities).is_none());
    }

    #[test]
    fn turns_since_use_condition() {
        let rules = vec![Rule {
            ability: "Embolden".to_string(),
            conditions: vec![Condition {
                subject: ConditionSubject::SelfChar,
                value: QueryValue::TurnsSinceUse,
                comparator: Comparator::Gte,
                threshold: 3, // only use if >= 3 turns since last use
            }],
        }];
        let mut actor = make_char_with_rules(0, vec![(Stat::SPI, 10)], rules);
        let abilities = make_abilities();

        // Never used → turns_since = MAX → matches
        assert_eq!(evaluate_rules(&actor, None, &[], &abilities).as_deref(), Some("Embolden"));

        // Simulate: turn 1, use Embolden
        actor.increment_turn_count(); // turn 1
        actor.record_ability_use("Embolden");

        // Turn 2: 1 turn since use, 1 < 3 → fails
        actor.increment_turn_count();
        assert!(evaluate_rules(&actor, None, &[], &abilities).is_none());

        // Turn 3: 2 turns since use → fails
        actor.increment_turn_count();
        assert!(evaluate_rules(&actor, None, &[], &abilities).is_none());

        // Turn 4: 3 turns since use → matches
        actor.increment_turn_count();
        assert_eq!(evaluate_rules(&actor, None, &[], &abilities).as_deref(), Some("Embolden"));
    }

    #[test]
    fn spi_cost_reduction_allows_otherwise_unaffordable() {
        use crate::models::TraitEffect;

        let rules = vec![Rule {
            ability: "Embolden".to_string(), // costs 3
            conditions: Vec::new(),
        }];
        let mut actor = make_char_with_rules(0, vec![(Stat::SPI, 5)], rules);
        actor.spend_spi(3); // only 2 left, need 3
        let abilities = make_abilities();

        // Without trait: can't afford
        assert!(evaluate_rules(&actor, None, &[], &abilities).is_none());

        // With trait: effective cost = max(3-1, 1) = 2, can afford
        actor.add_trait(TraitEffect::SpiCostReduction { amount: 1 });
        assert_eq!(evaluate_rules(&actor, None, &[], &abilities).as_deref(), Some("Embolden"));
    }

    #[test]
    fn spi_cost_reduction_enforces_minimum_one() {
        use crate::models::TraitEffect;

        let rules = vec![Rule {
            ability: "Crush".to_string(), // costs 2
            conditions: Vec::new(),
        }];
        let mut actor = make_char_with_rules(0, vec![(Stat::SPI, 5)], rules);
        actor.add_trait(TraitEffect::SpiCostReduction { amount: 100 });
        actor.spend_spi(5); // 0 SPI left

        let abilities = make_abilities();
        // Effective cost = max(2-100, 1) = 1, but 0 < 1, still can't afford
        assert!(evaluate_rules(&actor, None, &[], &abilities).is_none());
    }

    #[test]
    fn turns_since_use_resets_on_reuse() {
        let rules = vec![Rule {
            ability: "Crush".to_string(),
            conditions: vec![Condition {
                subject: ConditionSubject::SelfChar,
                value: QueryValue::TurnsSinceUse,
                comparator: Comparator::Gte,
                threshold: 2,
            }],
        }];
        let mut actor = make_char_with_rules(0, vec![(Stat::SPI, 10)], rules);
        let abilities = make_abilities();

        // Turn 1: use it
        actor.increment_turn_count();
        actor.record_ability_use("Crush");

        // Turn 2: 1 since use → fails
        actor.increment_turn_count();
        assert!(evaluate_rules(&actor, None, &[], &abilities).is_none());

        // Turn 3: 2 since use → matches, use again
        actor.increment_turn_count();
        assert_eq!(evaluate_rules(&actor, None, &[], &abilities).as_deref(), Some("Crush"));
        actor.record_ability_use("Crush");

        // Turn 4: 1 since re-use → fails again
        actor.increment_turn_count();
        assert!(evaluate_rules(&actor, None, &[], &abilities).is_none());
    }
}
