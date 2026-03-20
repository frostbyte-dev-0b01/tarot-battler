//! Rule evaluation: checks ordered conditions to select an ability.

use crate::abilities::AbilityMap;
use crate::models::{CharacterState, Comparator, Condition, ConditionSubject, QueryValue};

#[derive(Clone, Copy, Debug, Default)]
pub struct WorldState {
    pub tick_count: u32,
    pub ally_count: u32,
    pub enemy_count: u32,
}

/// Evaluate the actor's rules in order. Returns the name of the first ability
/// whose conditions are all met AND whose MP cost the actor can afford.
/// Returns None if no rule matches (caller should fall back to Rest).
pub fn evaluate_rules(
    actor: &CharacterState,
    target: Option<&CharacterState>,
    allies: &[CharacterState],
    world: WorldState,
    abilities: &AbilityMap,
) -> Option<String> {
    for rule in actor.rules() {
        if !actor.has_active(&rule.ability) {
            continue;
        }

        let ability_def = match abilities.get(&rule.ability) {
            Some(def) => def,
            None => continue,
        };

        // Check MP cost (reduced by trait, minimum 1)
        let effective_cost = ability_def
            .mp_cost
            .saturating_sub(actor.mp_cost_reduction())
            .max(1);
        if actor.current_mp() < effective_cost {
            continue;
        }

        // Check all conditions (AND)
        let all_met = rule
            .conditions
            .iter()
            .all(|cond| check_condition(cond, actor, target, allies, world, &rule.ability));

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
    world: WorldState,
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

    let living_companion_count = |character: &CharacterState, same_team: &[CharacterState]| -> u32 {
        let companion_ids = character.effective_companion_ids();
        same_team
            .iter()
            .filter(|other| other.is_alive() && companion_ids.contains(&other.id()))
            .count() as u32
    };

    match &cond.subject {
        ConditionSubject::SelfChar => {
            let val = match &cond.value {
                QueryValue::SelfCompanionCount => living_companion_count(actor, allies),
                QueryValue::TargetCompanionCount => 0,
                _ => actor.query_value(&cond.value),
            };
            compare(val, &cond.comparator, cond.threshold)
        }
        ConditionSubject::Target => match target {
            Some(t) => {
                let val = match &cond.value {
                    QueryValue::SelfCompanionCount | QueryValue::TargetCompanionCount => {
                        living_companion_count(t, allies)
                    }
                    _ => t.query_value(&cond.value),
                };
                compare(val, &cond.comparator, cond.threshold)
            }
            None => false,
        },
        ConditionSubject::Companion => {
            // True if ANY adjacent companion matches
            let comp_ids = actor.effective_companion_ids();
            allies
                .iter()
                .filter(|c| c.is_alive() && comp_ids.contains(&c.id()))
                .any(|c| {
                    let val = match &cond.value {
                        QueryValue::SelfCompanionCount | QueryValue::TargetCompanionCount => {
                            living_companion_count(c, allies)
                        }
                        _ => c.query_value(&cond.value),
                    };
                    compare(val, &cond.comparator, cond.threshold)
                })
        }
        ConditionSubject::World => match cond.value {
            QueryValue::TickCount => compare(world.tick_count, &cond.comparator, cond.threshold),
            QueryValue::AllyCount => compare(world.ally_count, &cond.comparator, cond.threshold),
            QueryValue::EnemyCount => compare(world.enemy_count, &cond.comparator, cond.threshold),
            _ => false,
        },
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

    fn make_char_with_rules(id: u32, stats: Vec<(Stat, u32)>, rules: Vec<Rule>) -> CharacterState {
        let config = CharacterConfig {
            id: None,
            base_name: format!("Char{}", id),
            display_name: None,
            passive: String::new(),
            actives: rules.iter().map(|r| r.ability.clone()).collect(),
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
                mp_cost: 2,
                primitives: Vec::new(),
            },
        );
        map.insert(
            "Embolden".to_string(),
            AbilityDef {
                mp_cost: 3,
                primitives: Vec::new(),
            },
        );
        map
    }

    fn world() -> WorldState {
        WorldState::default()
    }

    #[test]
    fn empty_conditions_always_matches() {
        let rules = vec![Rule {
            ability: "Crush".to_string(),
            conditions: Vec::new(),
        }];
        let actor = make_char_with_rules(0, vec![(Stat::WIL, 5)], rules);
        let abilities = make_abilities();
        let result = evaluate_rules(&actor, None, &[], world(), &abilities);
        assert_eq!(result.as_deref(), Some("Crush"));
    }

    #[test]
    fn spi_gating_skips_expensive_ability() {
        let rules = vec![Rule {
            ability: "Embolden".to_string(), // costs 3
            conditions: Vec::new(),
        }];
        let mut actor = make_char_with_rules(0, vec![(Stat::WIL, 5)], rules);
        actor.spend_mp(3); // only 2 left, need 3
        let abilities = make_abilities();
        let result = evaluate_rules(&actor, None, &[], world(), &abilities);
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
        let actor = make_char_with_rules(0, vec![(Stat::WIL, 5)], rules);
        let abilities = make_abilities();

        // Target HP=30 → doesn't match
        let target_high = make_char(1, vec![(Stat::VIT, 10)]); // HP=30
        assert!(evaluate_rules(&actor, Some(&target_high), &[], world(), &abilities).is_none());

        // Target HP=4 → matches
        let mut target_low = make_char(1, vec![(Stat::VIT, 10)]);
        target_low.take_damage(26); // HP=4
        assert_eq!(
            evaluate_rules(&actor, Some(&target_low), &[], world(), &abilities).as_deref(),
            Some("Crush")
        );
    }

    #[test]
    fn companion_spi_lte_condition() {
        let rules = vec![Rule {
            ability: "Embolden".to_string(),
            conditions: vec![Condition {
                subject: ConditionSubject::Companion,
                value: QueryValue::Mp,
                comparator: Comparator::Lte,
                threshold: 1,
            }],
        }];
        let mut actor = make_char_with_rules(0, vec![(Stat::WIL, 5)], rules);
        actor.set_companions(vec![1]);
        let abilities = make_abilities();

        // Companion has plenty of WIL
        let companion_full = make_char(1, vec![(Stat::VIT, 5), (Stat::WIL, 5)]);
        assert!(evaluate_rules(&actor, None, &[companion_full], world(), &abilities).is_none());

        // Companion has low WIL
        let mut companion_low = make_char(1, vec![(Stat::VIT, 5), (Stat::WIL, 5)]);
        companion_low.spend_mp(4); // MP=1
        assert_eq!(
            evaluate_rules(&actor, None, &[companion_low], world(), &abilities).as_deref(),
            Some("Embolden")
        );
    }

    #[test]
    fn self_stat_gte_condition() {
        let rules = vec![Rule {
            ability: "Crush".to_string(),
            conditions: vec![Condition {
                subject: ConditionSubject::SelfChar,
                value: QueryValue::Stat(Stat::MGT),
                comparator: Comparator::Gte,
                threshold: 10,
            }],
        }];
        let actor = make_char_with_rules(0, vec![(Stat::WIL, 5), (Stat::MGT, 12)], rules.clone());
        let abilities = make_abilities();
        assert_eq!(
            evaluate_rules(&actor, None, &[], world(), &abilities).as_deref(),
            Some("Crush")
        );

        let actor_weak = make_char_with_rules(0, vec![(Stat::WIL, 5), (Stat::MGT, 8)], rules);
        assert!(evaluate_rules(&actor_weak, None, &[], world(), &abilities).is_none());
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
        let actor = make_char_with_rules(0, vec![(Stat::WIL, 5)], rules);
        let abilities = make_abilities();

        // Target HP high → first rule fails, Embolden matches
        let target = make_char(1, vec![(Stat::VIT, 10)]);
        assert_eq!(
            evaluate_rules(&actor, Some(&target), &[], world(), &abilities).as_deref(),
            Some("Embolden")
        );

        // Target HP low → first rule matches
        let mut target_low = make_char(1, vec![(Stat::VIT, 10)]);
        target_low.take_damage(28); // HP=2
        assert_eq!(
            evaluate_rules(&actor, Some(&target_low), &[], world(), &abilities).as_deref(),
            Some("Crush")
        );
    }

    #[test]
    fn unequipped_rule_ability_is_skipped() {
        let actor = CharacterState::from_config(
            0,
            &CharacterConfig {
                id: None,
                base_name: "Char0".to_string(),
                display_name: None,
                passive: String::new(),
                actives: vec!["Embolden".to_string()],
                item: None,
                position: Position { row: 0, col: 0 },
                stats: vec![(Stat::WIL, 5)].into_iter().collect(),
                rules: vec![Rule {
                    ability: "Crush".to_string(),
                    conditions: Vec::new(),
                }],
            },
        );
        let abilities = make_abilities();
        let result = evaluate_rules(&actor, None, &[], world(), &abilities);
        assert_eq!(result, None);
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
        let actor = make_char_with_rules(0, vec![(Stat::WIL, 5)], rules);
        let abilities = make_abilities();
        assert!(evaluate_rules(&actor, None, &[], world(), &abilities).is_none());
    }

    #[test]
    fn world_ally_count_condition_uses_live_counts() {
        let rules = vec![Rule {
            ability: "Embolden".to_string(),
            conditions: vec![Condition {
                subject: ConditionSubject::World,
                value: QueryValue::AllyCount,
                comparator: Comparator::Gte,
                threshold: 1,
            }],
        }];
        let actor = make_char_with_rules(0, vec![(Stat::WIL, 5), (Stat::VIT, 10)], rules);
        let abilities = make_abilities();

        assert_eq!(
            evaluate_rules(
                &actor,
                None,
                &[],
                WorldState {
                    tick_count: 3,
                    ally_count: 1,
                    enemy_count: 2,
                },
                &abilities
            )
            .as_deref(),
            Some("Embolden")
        );
        assert!(evaluate_rules(&actor, None, &[], world(), &abilities).is_none());
    }

    #[test]
    fn world_tick_and_enemy_counts_can_gate_rules() {
        let rules = vec![Rule {
            ability: "Crush".to_string(),
            conditions: vec![
                Condition {
                    subject: ConditionSubject::World,
                    value: QueryValue::TickCount,
                    comparator: Comparator::Gte,
                    threshold: 5,
                },
                Condition {
                    subject: ConditionSubject::World,
                    value: QueryValue::EnemyCount,
                    comparator: Comparator::Lte,
                    threshold: 2,
                },
            ],
        }];
        let actor = make_char_with_rules(0, vec![(Stat::WIL, 5)], rules);
        let abilities = make_abilities();

        assert!(evaluate_rules(&actor, None, &[], world(), &abilities).is_none());
        assert_eq!(
            evaluate_rules(
                &actor,
                None,
                &[],
                WorldState {
                    tick_count: 5,
                    ally_count: 3,
                    enemy_count: 2,
                },
                &abilities
            )
            .as_deref(),
            Some("Crush")
        );
    }

    #[test]
    fn self_row_condition_reads_actor_row() {
        let rules = vec![Rule {
            ability: "Crush".to_string(),
            conditions: vec![Condition {
                subject: ConditionSubject::SelfChar,
                value: QueryValue::SelfRow,
                comparator: Comparator::Gte,
                threshold: 2,
            }],
        }];
        let mut actor = make_char_with_rules(0, vec![(Stat::WIL, 5)], rules);
        actor.set_position(Position { row: 2, col: 1 });

        let abilities = make_abilities();
        let result = evaluate_rules(&actor, None, &[], world(), &abilities);
        assert_eq!(result.as_deref(), Some("Crush"));
    }

    #[test]
    fn self_companion_count_uses_living_companions() {
        let rules = vec![Rule {
            ability: "Crush".to_string(),
            conditions: vec![Condition {
                subject: ConditionSubject::SelfChar,
                value: QueryValue::SelfCompanionCount,
                comparator: Comparator::Gte,
                threshold: 2,
            }],
        }];
        let mut actor = make_char_with_rules(0, vec![(Stat::WIL, 5)], rules);
        actor.set_companions(vec![1, 2, 3]);

        let ally_one = make_char(1, vec![(Stat::VIT, 5)]);
        let ally_two = make_char(2, vec![(Stat::VIT, 5)]);
        let mut defeated_ally = make_char(3, vec![(Stat::VIT, 5)]);
        defeated_ally.take_damage(15);

        let abilities = make_abilities();
        let result = evaluate_rules(
            &actor,
            None,
            &[ally_one, ally_two, defeated_ally],
            world(),
            &abilities,
        );
        assert_eq!(result.as_deref(), Some("Crush"));
    }

    #[test]
    fn target_companion_count_reads_living_target_companions() {
        let rules = vec![Rule {
            ability: "Crush".to_string(),
            conditions: vec![Condition {
                subject: ConditionSubject::Target,
                value: QueryValue::TargetCompanionCount,
                comparator: Comparator::Gte,
                threshold: 1,
            }],
        }];
        let actor = make_char_with_rules(0, vec![(Stat::WIL, 5)], rules);
        let mut target = make_char(10, vec![(Stat::VIT, 5)]);
        target.set_companions(vec![11, 12]);

        let living_companion = make_char(11, vec![(Stat::VIT, 5)]);
        let mut defeated_companion = make_char(12, vec![(Stat::VIT, 5)]);
        defeated_companion.take_damage(15);

        let abilities = make_abilities();
        let result = evaluate_rules(
            &actor,
            Some(&target),
            &[living_companion, defeated_companion],
            world(),
            &abilities,
        );
        assert_eq!(result.as_deref(), Some("Crush"));
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
        let mut actor = make_char_with_rules(0, vec![(Stat::WIL, 10)], rules);
        let abilities = make_abilities();

        // Never used → count=0, 0 <= 2 → matches
        assert_eq!(
            evaluate_rules(&actor, None, &[], world(), &abilities).as_deref(),
            Some("Crush")
        );

        // Used twice → count=2, 2 <= 2 → still matches
        actor.record_ability_use("Crush");
        actor.record_ability_use("Crush");
        assert_eq!(
            evaluate_rules(&actor, None, &[], world(), &abilities).as_deref(),
            Some("Crush")
        );

        // Used three times → count=3, 3 <= 2 → fails
        actor.record_ability_use("Crush");
        assert!(evaluate_rules(&actor, None, &[], world(), &abilities).is_none());
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
        let mut actor = make_char_with_rules(0, vec![(Stat::WIL, 10)], rules);
        let abilities = make_abilities();

        // Never used → turns_since = MAX → matches
        assert_eq!(
            evaluate_rules(&actor, None, &[], world(), &abilities).as_deref(),
            Some("Embolden")
        );

        // Simulate: turn 1, use Embolden
        actor.increment_turn_count(); // turn 1
        actor.record_ability_use("Embolden");

        // Turn 2: 1 turn since use, 1 < 3 → fails
        actor.increment_turn_count();
        assert!(evaluate_rules(&actor, None, &[], world(), &abilities).is_none());

        // Turn 3: 2 turns since use → fails
        actor.increment_turn_count();
        assert!(evaluate_rules(&actor, None, &[], world(), &abilities).is_none());

        // Turn 4: 3 turns since use → matches
        actor.increment_turn_count();
        assert_eq!(
            evaluate_rules(&actor, None, &[], world(), &abilities).as_deref(),
            Some("Embolden")
        );
    }

    #[test]
    fn spi_cost_reduction_allows_otherwise_unaffordable() {
        use crate::models::TraitEffect;

        let rules = vec![Rule {
            ability: "Embolden".to_string(), // costs 3
            conditions: Vec::new(),
        }];
        let mut actor = make_char_with_rules(0, vec![(Stat::WIL, 5)], rules);
        actor.spend_mp(3); // only 2 left, need 3
        let abilities = make_abilities();

        // Without trait: can't afford
        assert!(evaluate_rules(&actor, None, &[], world(), &abilities).is_none());

        // With trait: effective cost = max(3-1, 1) = 2, can afford
        actor.add_trait(TraitEffect::MpCostReduction { amount: 1 });
        assert_eq!(
            evaluate_rules(&actor, None, &[], world(), &abilities).as_deref(),
            Some("Embolden")
        );
    }

    #[test]
    fn spi_cost_reduction_enforces_minimum_one() {
        use crate::models::TraitEffect;

        let rules = vec![Rule {
            ability: "Crush".to_string(), // costs 2
            conditions: Vec::new(),
        }];
        let mut actor = make_char_with_rules(0, vec![(Stat::WIL, 5)], rules);
        actor.add_trait(TraitEffect::MpCostReduction { amount: 100 });
        actor.spend_mp(5); // 0 MP left

        let abilities = make_abilities();
        // Effective cost = max(2-100, 1) = 1, but 0 < 1, still can't afford
        assert!(evaluate_rules(&actor, None, &[], world(), &abilities).is_none());
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
        let mut actor = make_char_with_rules(0, vec![(Stat::WIL, 10)], rules);
        let abilities = make_abilities();

        // Turn 1: use it
        actor.increment_turn_count();
        actor.record_ability_use("Crush");

        // Turn 2: 1 since use → fails
        actor.increment_turn_count();
        assert!(evaluate_rules(&actor, None, &[], world(), &abilities).is_none());

        // Turn 3: 2 since use → matches, use again
        actor.increment_turn_count();
        assert_eq!(
            evaluate_rules(&actor, None, &[], world(), &abilities).as_deref(),
            Some("Crush")
        );
        actor.record_ability_use("Crush");

        // Turn 4: 1 since re-use → fails again
        actor.increment_turn_count();
        assert!(evaluate_rules(&actor, None, &[], world(), &abilities).is_none());
    }

    #[test]
    fn self_has_status_condition_matches() {
        let rules = vec![Rule {
            ability: "Crush".to_string(),
            conditions: vec![Condition {
                subject: ConditionSubject::SelfChar,
                value: QueryValue::HasStatus("Ward".to_string()),
                comparator: Comparator::Gte,
                threshold: 1,
            }],
        }];
        let mut actor = make_char_with_rules(0, vec![(Stat::WIL, 5), (Stat::VIT, 10)], rules);
        actor.add_status(
            "Ward",
            1,
            99,
            &crate::statuses::StatusDef {
                behavior: crate::statuses::StatusBehavior::Ward,
                stack_type: crate::statuses::StackType::Permanent,
                opposes: None,
            },
            None,
        );
        let abilities = make_abilities();

        assert_eq!(
            evaluate_rules(&actor, None, &[], world(), &abilities).as_deref(),
            Some("Crush")
        );
    }

    #[test]
    fn target_status_stacks_condition_matches() {
        let rules = vec![Rule {
            ability: "Crush".to_string(),
            conditions: vec![Condition {
                subject: ConditionSubject::Target,
                value: QueryValue::StatusStacks("Bleed".to_string()),
                comparator: Comparator::Gte,
                threshold: 2,
            }],
        }];
        let actor = make_char_with_rules(0, vec![(Stat::WIL, 5)], rules);
        let mut target = make_char(1, vec![(Stat::VIT, 10)]);
        target.add_status(
            "Bleed",
            2,
            99,
            &crate::statuses::StatusDef {
                behavior: crate::statuses::StatusBehavior::DamagePerStack { value: 1 },
                stack_type: crate::statuses::StackType::TickDown,
                opposes: None,
            },
            None,
        );
        let abilities = make_abilities();

        assert_eq!(
            evaluate_rules(&actor, Some(&target), &[], world(), &abilities).as_deref(),
            Some("Crush")
        );
    }

    #[test]
    fn companion_status_query_supports_stat_keyed_statuses() {
        let rules = vec![Rule {
            ability: "Embolden".to_string(),
            conditions: vec![Condition {
                subject: ConditionSubject::Companion,
                value: QueryValue::StatusStacks("Empower:MGT".to_string()),
                comparator: Comparator::Gte,
                threshold: 2,
            }],
        }];
        let mut actor = make_char_with_rules(0, vec![(Stat::WIL, 5)], rules);
        actor.set_companions(vec![1]);
        let mut companion = make_char(1, vec![(Stat::VIT, 5), (Stat::MGT, 5)]);
        companion.add_status(
            &crate::statuses::status_key("Empower", Some(&Stat::MGT)),
            2,
            99,
            &crate::statuses::StatusDef {
                behavior: crate::statuses::StatusBehavior::StatModPerStack { magnitude: 1 },
                stack_type: crate::statuses::StackType::TickDown,
                opposes: Some("Weaken".to_string()),
            },
            Some(Stat::MGT),
        );
        let abilities = make_abilities();

        assert_eq!(
            evaluate_rules(&actor, None, &[companion], world(), &abilities).as_deref(),
            Some("Embolden")
        );
    }

    #[test]
    fn target_condition_query_matches() {
        let rules = vec![Rule {
            ability: "Crush".to_string(),
            conditions: vec![Condition {
                subject: ConditionSubject::Target,
                value: QueryValue::HasCondition("Marked".to_string()),
                comparator: Comparator::Gte,
                threshold: 1,
            }],
        }];
        let actor = make_char_with_rules(0, vec![(Stat::WIL, 5)], rules);
        let mut target = make_char(1, vec![(Stat::VIT, 10)]);
        target.add_condition(crate::models::ConditionKind::Marked, 2, 99);
        let abilities = make_abilities();

        assert_eq!(
            evaluate_rules(&actor, Some(&target), &[], world(), &abilities).as_deref(),
            Some("Crush")
        );
    }
}
