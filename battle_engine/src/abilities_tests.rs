use super::*;
use crate::logger::BattleEvent;
use crate::models::ConditionKind;
use crate::statuses::{StackType, StatusBehavior, StatusDef};
use crate::test_support::{empty_statuses, make_adjacent_char, make_char, test_statuses};
use rand::SeedableRng;

#[test]
fn deal_physical_damage_with_multiplier() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![make_char(
        0,
        vec![(Stat::MGT, 10), (Stat::VIT, 10), (Stat::WIL, 5)],
    )];
    let mut enemy_team = vec![make_char(1, vec![(Stat::ARM, 4), (Stat::VIT, 20)])];
    actor_team[0].set_target(1);

    let ability = AbilityDef {
        mp_cost: 2,
        primitives: vec![Primitive::DealPhysicalDamage {
            target: SimpleAbilityTarget::CurrentTarget.into(),
            multiplier: 1.5,
            double_empower_stat: None,
        }],
    };

    let dealt = execute_ability(
        0,
        "Crush",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );
    assert_eq!(dealt.len(), 1);
    assert_eq!(dealt[0].target_id, 1);
    assert_eq!(dealt[0].damage, 9);
    assert_eq!(enemy_team[0].current_hp(), 60 - 9);
}

#[test]
fn restore_mp_to_companions() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();

    let stats = vec![(Stat::VIT, 10), (Stat::WIL, 5), (Stat::MGT, 5)];
    let mut actor_team = vec![
        make_adjacent_char(0, 0, 0, stats.clone()),
        make_adjacent_char(1, 0, 1, stats.clone()),
        make_adjacent_char(2, 0, 2, stats.clone()),
    ];
    actor_team[0].set_companions(vec![1]);
    actor_team[1].spend_mp(4);
    assert_eq!(actor_team[1].current_mp(), 1);

    let ability = AbilityDef {
        mp_cost: 3,
        primitives: vec![Primitive::RestoreMp {
            target: SimpleAbilityTarget::Companions.into(),
            amount: 2,
        }],
    };

    let mut enemy_team = vec![make_char(10, vec![(Stat::VIT, 5)])];
    execute_ability(
        0,
        "Embolden",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );
    assert_eq!(actor_team[1].current_mp(), 3);
    assert_eq!(actor_team[2].current_mp(), 5);
}

#[test]
fn apply_status_bleed_on_enemy() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team = vec![make_char(0, vec![(Stat::VIT, 10), (Stat::WIL, 5)])];
    let mut enemy_team = vec![make_char(1, vec![(Stat::VIT, 10)])];
    actor_team[0].set_target(1);

    let ability = AbilityDef {
        mp_cost: 2,
        primitives: vec![Primitive::ApplyStatus {
            target: SimpleAbilityTarget::CurrentTarget.into(),
            status: "Bleed".to_string(),
            stat: None,
            stacks: 3,
        }],
    };

    execute_ability(
        0,
        "Slash",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );
    assert_eq!(enemy_team[0].status_stacks("Bleed"), 3);
}

#[test]
fn apply_status_empower_on_ally() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team = vec![make_char(
        0,
        vec![(Stat::VIT, 10), (Stat::WIL, 5), (Stat::MGT, 8)],
    )];
    let mut enemy_team = vec![make_char(10, vec![(Stat::VIT, 5)])];

    let ability = AbilityDef {
        mp_cost: 2,
        primitives: vec![Primitive::ApplyStatus {
            target: SimpleAbilityTarget::SelfChar.into(),
            status: "Empower".to_string(),
            stat: Some(Stat::MGT),
            stacks: 3,
        }],
    };

    execute_ability(
        0,
        "Rage",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );
    assert_eq!(actor_team[0].get_eff_stat(&Stat::MGT), 11);
}

#[test]
fn apply_status_weaken_on_enemy() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team = vec![make_char(0, vec![(Stat::VIT, 10), (Stat::WIL, 5)])];
    let mut enemy_team = vec![make_char(1, vec![(Stat::VIT, 10), (Stat::MGT, 10)])];
    actor_team[0].set_target(1);

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::ApplyStatus {
            target: SimpleAbilityTarget::CurrentTarget.into(),
            status: "Weaken".to_string(),
            stat: Some(Stat::MGT),
            stacks: 2,
        }],
    };

    execute_ability(
        0,
        "Curse",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );
    assert_eq!(enemy_team[0].get_eff_stat(&Stat::MGT), 8);
}

#[test]
fn remove_status_clears_self_status() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team = vec![make_char(0, vec![(Stat::VIT, 10), (Stat::WIL, 5)])];
    let mut enemy_team = vec![make_char(1, vec![(Stat::VIT, 10)])];

    let bleed = statuses.get("Bleed").unwrap();
    actor_team[0].add_status("Bleed", 2, 99, bleed, None);

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::RemoveStatus {
            target: SimpleAbilityTarget::SelfChar.into(),
            status: "Bleed".to_string(),
            stat: None,
            stacks: 1,
        }],
    };

    execute_ability(
        0,
        "Cleanse",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );
    assert_eq!(actor_team[0].status_stacks("Bleed"), 1);
}

#[test]
fn remove_status_clears_current_target_status() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team = vec![make_char(0, vec![(Stat::VIT, 10), (Stat::WIL, 5)])];
    let mut enemy_team = vec![make_char(1, vec![(Stat::VIT, 10)])];
    actor_team[0].set_target(1);

    let bleed = statuses.get("Bleed").unwrap();
    enemy_team[0].add_status("Bleed", 3, 99, bleed, None);

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::RemoveStatus {
            target: SimpleAbilityTarget::CurrentTarget.into(),
            status: "Bleed".to_string(),
            stat: None,
            stacks: 2,
        }],
    };

    execute_ability(
        0,
        "Dispel",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );
    assert_eq!(enemy_team[0].status_stacks("Bleed"), 1);
}

#[test]
fn remove_status_clears_all_enemy_statuses() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team = vec![make_char(0, vec![(Stat::VIT, 10), (Stat::WIL, 5)])];
    let mut enemy_team = vec![
        make_char(1, vec![(Stat::VIT, 10)]),
        make_char(2, vec![(Stat::VIT, 10)]),
    ];

    let bleed = statuses.get("Bleed").unwrap();
    enemy_team[0].add_status("Bleed", 1, 99, bleed, None);
    enemy_team[1].add_status("Bleed", 1, 99, bleed, None);

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::RemoveStatus {
            target: SimpleAbilityTarget::AllEnemies.into(),
            status: "Bleed".to_string(),
            stat: None,
            stacks: 1,
        }],
    };

    execute_ability(
        0,
        "MassDispel",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );
    assert_eq!(enemy_team[0].status_stacks("Bleed"), 0);
    assert_eq!(enemy_team[1].status_stacks("Bleed"), 0);
}

#[test]
fn cleanse_reduces_all_timed_debuffs_on_allies() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team = vec![
        make_adjacent_char(0, 0, 0, vec![(Stat::VIT, 10), (Stat::WIL, 5)]),
        make_adjacent_char(1, 0, 1, vec![(Stat::VIT, 10), (Stat::WIL, 5), (Stat::MGT, 10)]),
    ];
    actor_team[0].set_companions(vec![1]);
    actor_team[1].add_status("Bleed", 3, 99, statuses.get("Bleed").unwrap(), None);
    actor_team[1].add_status(
        &crate::statuses::status_key("Weaken", Some(&Stat::MGT)),
        2,
        99,
        statuses.get("Weaken").unwrap(),
        Some(Stat::MGT),
    );
    let stun = StatusDef {
        behavior: StatusBehavior::SkipTurn,
        stack_type: StackType::NoStack,
        opposes: None,
    };
    actor_team[1].add_status("Stun", 1, 99, &stun, None);

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::Cleanse {
            target: SimpleAbilityTarget::Companions.into(),
            amount: 1,
        }],
    };

    let mut enemy_team = vec![make_char(10, vec![(Stat::VIT, 5)])];
    execute_ability(
        0,
        "Sanctify",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );

    assert_eq!(actor_team[1].status_stacks("Bleed"), 2);
    assert_eq!(
        actor_team[1].status_stacks(&crate::statuses::status_key("Weaken", Some(&Stat::MGT))),
        1
    );
    assert_eq!(actor_team[1].status_stacks("Stun"), 1);
}

#[test]
fn dispel_reduces_all_timed_buffs_on_enemies() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team = vec![make_char(0, vec![(Stat::VIT, 10), (Stat::WIL, 5)])];
    let mut enemy_team = vec![make_char(1, vec![(Stat::VIT, 10), (Stat::MGT, 10)])];
    actor_team[0].set_target(1);

    enemy_team[0].add_status(
        &crate::statuses::status_key("Empower", Some(&Stat::MGT)),
        3,
        99,
        statuses.get("Empower").unwrap(),
        Some(Stat::MGT),
    );
    enemy_team[0].add_status("Regen", 2, 99, statuses.get("Regen").unwrap(), None);
    enemy_team[0].add_status("Ward", 1, 99, statuses.get("Ward").unwrap(), None);

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::Dispel {
            target: SimpleAbilityTarget::CurrentTarget.into(),
            amount: 1,
        }],
    };

    execute_ability(
        0,
        "Dispel",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );

    assert_eq!(
        enemy_team[0].status_stacks(&crate::statuses::status_key("Empower", Some(&Stat::MGT))),
        2
    );
    assert_eq!(enemy_team[0].status_stacks("Regen"), 1);
    assert_eq!(enemy_team[0].status_stacks("Ward"), 1);
}

#[test]
fn magical_consume_status_damage_scales_with_consumed_stacks() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team =
        vec![make_char(0, vec![(Stat::VIT, 10), (Stat::WIL, 5), (Stat::MAG, 8)])];
    let mut enemy_team = vec![make_char(1, vec![(Stat::VIT, 10), (Stat::RES, 3)])];
    actor_team[0].set_target(1);

    let omen = statuses.get("Omen").unwrap();
    enemy_team[0].add_status("Omen", 4, 99, omen, None);

    let ability = AbilityDef {
        mp_cost: 3,
        primitives: vec![Primitive::DealMagicalDamageConsumeStatus {
            target: SimpleAbilityTarget::CurrentTarget.into(),
            multiplier: 1.0,
            status: "Omen".to_string(),
            stat: None,
            bonus_per_stack: 1,
        }],
    };

    execute_ability(
        0,
        "Harvest Night",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );

    assert_eq!(enemy_team[0].current_hp(), 21);
    assert_eq!(enemy_team[0].status_stacks("Omen"), 0);
}

#[test]
fn magical_consume_status_damage_leaves_target_unchanged_without_status() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team =
        vec![make_char(0, vec![(Stat::VIT, 10), (Stat::WIL, 5), (Stat::MAG, 8)])];
    let mut enemy_team = vec![make_char(1, vec![(Stat::VIT, 10), (Stat::RES, 3)])];
    actor_team[0].set_target(1);

    let ability = AbilityDef {
        mp_cost: 3,
        primitives: vec![Primitive::DealMagicalDamageConsumeStatus {
            target: SimpleAbilityTarget::CurrentTarget.into(),
            multiplier: 1.0,
            status: "Omen".to_string(),
            stat: None,
            bonus_per_stack: 1,
        }],
    };

    execute_ability(
        0,
        "Harvest Night",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );

    assert_eq!(enemy_team[0].current_hp(), 25);
    assert_eq!(enemy_team[0].status_stacks("Omen"), 0);
}

#[test]
fn physical_damage_bonus_vs_status_applies_only_when_present() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team =
        vec![make_char(0, vec![(Stat::VIT, 10), (Stat::WIL, 5), (Stat::MGT, 8)])];
    let mut enemy_team = vec![make_char(1, vec![(Stat::VIT, 10), (Stat::ARM, 3)])];
    actor_team[0].set_target(1);

    let omen = statuses.get("Omen").unwrap();
    enemy_team[0].add_status("Omen", 1, 99, omen, None);

    let ability = AbilityDef {
        mp_cost: 2,
        primitives: vec![Primitive::DealPhysicalDamageBonusVsStatus {
            target: SimpleAbilityTarget::CurrentTarget.into(),
            multiplier: 1.0,
            status: "Omen".to_string(),
            stat: None,
            bonus_damage: 3,
        }],
    };

    execute_ability(
        0,
        "Condemn",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );

    assert_eq!(enemy_team[0].current_hp(), 22);
    assert_eq!(enemy_team[0].status_stacks("Omen"), 1);
}

#[test]
fn physical_damage_bonus_vs_status_does_not_apply_without_status() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team =
        vec![make_char(0, vec![(Stat::VIT, 10), (Stat::WIL, 5), (Stat::MGT, 8)])];
    let mut enemy_team = vec![make_char(1, vec![(Stat::VIT, 10), (Stat::ARM, 3)])];
    actor_team[0].set_target(1);

    let ability = AbilityDef {
        mp_cost: 2,
        primitives: vec![Primitive::DealPhysicalDamageBonusVsStatus {
            target: SimpleAbilityTarget::CurrentTarget.into(),
            multiplier: 1.0,
            status: "Omen".to_string(),
            stat: None,
            bonus_damage: 3,
        }],
    };

    execute_ability(
        0,
        "Condemn",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );

    assert_eq!(enemy_team[0].current_hp(), 25);
}

#[test]
fn conditional_primitives_execute_only_when_target_has_status() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team =
        vec![make_char(0, vec![(Stat::VIT, 10), (Stat::WIL, 5), (Stat::MAG, 8)])];
    let mut enemy_team = vec![make_char(1, vec![(Stat::VIT, 10), (Stat::RES, 3)])];
    actor_team[0].set_target(1);

    let omen = statuses.get("Omen").unwrap();
    enemy_team[0].add_status("Omen", 1, 99, omen, None);

    let ability = AbilityDef {
        mp_cost: 2,
        primitives: vec![
            Primitive::DealMagicalDamage {
                target: SimpleAbilityTarget::CurrentTarget.into(),
                multiplier: 1.0,
            },
            Primitive::IfTargetHasStatus {
                target: SimpleAbilityTarget::CurrentTarget.into(),
                status: "Omen".to_string(),
                stat: None,
                primitives: vec![Primitive::RestoreMp {
                    target: SimpleAbilityTarget::SelfChar.into(),
                    amount: 2,
                }],
            },
        ],
    };

    actor_team[0].spend_mp(4);
    execute_ability(
        0,
        "Transmute",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );

    assert_eq!(actor_team[0].current_mp(), 3);
}

#[test]
fn conditional_primitives_do_not_execute_without_matching_status() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team =
        vec![make_char(0, vec![(Stat::VIT, 10), (Stat::WIL, 5), (Stat::MAG, 8)])];
    let mut enemy_team = vec![make_char(1, vec![(Stat::VIT, 10), (Stat::RES, 3)])];
    actor_team[0].set_target(1);

    let ability = AbilityDef {
        mp_cost: 2,
        primitives: vec![
            Primitive::DealMagicalDamage {
                target: SimpleAbilityTarget::CurrentTarget.into(),
                multiplier: 1.0,
            },
            Primitive::IfTargetHasStatus {
                target: SimpleAbilityTarget::CurrentTarget.into(),
                status: "Omen".to_string(),
                stat: None,
                primitives: vec![Primitive::RestoreMp {
                    target: SimpleAbilityTarget::SelfChar.into(),
                    amount: 2,
                }],
            },
        ],
    };

    actor_team[0].spend_mp(4);
    execute_ability(
        0,
        "Transmute",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );

    assert_eq!(actor_team[0].current_mp(), 1);
}

#[test]
fn remove_one_buff_removes_highest_stack_buff() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team = vec![make_char(0, vec![(Stat::VIT, 10), (Stat::WIL, 5)])];
    let mut enemy_team = vec![make_char(1, vec![(Stat::VIT, 10), (Stat::ARM, 3)])];
    actor_team[0].set_target(1);

    let empower = statuses.get("Empower").unwrap();
    let ward = statuses.get("Ward").unwrap();
    enemy_team[0].add_status("Empower:MGT", 3, 99, empower, Some(Stat::MGT));
    enemy_team[0].add_status("Ward", 1, 99, ward, None);

    let ability = AbilityDef {
        mp_cost: 2,
        primitives: vec![Primitive::RemoveOneBuff {
            target: SimpleAbilityTarget::CurrentTarget.into(),
        }],
    };

    execute_ability(
        0,
        "Distill",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );

    assert_eq!(enemy_team[0].status_stacks("Empower:MGT"), 0);
    assert_eq!(enemy_team[0].status_stacks("Ward"), 1);
}

#[test]
fn if_target_lacks_status_executes_nested_primitives() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team =
        vec![make_char(0, vec![(Stat::VIT, 10), (Stat::WIL, 5), (Stat::MAG, 8)])];
    let mut enemy_team = vec![make_char(1, vec![(Stat::VIT, 10), (Stat::RES, 3)])];
    actor_team[0].set_target(1);

    let ability = AbilityDef {
        mp_cost: 2,
        primitives: vec![Primitive::IfTargetLacksStatus {
            target: SimpleAbilityTarget::CurrentTarget.into(),
            status: "Omen".to_string(),
            stat: None,
            primitives: vec![Primitive::ApplyStatus {
                target: SimpleAbilityTarget::CurrentTarget.into(),
                status: "Omen".to_string(),
                stat: None,
                stacks: 1,
            }],
        }],
    };

    execute_ability(
        0,
        "Invocation",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );

    assert_eq!(enemy_team[0].status_stacks("Omen"), 1);
}

#[test]
fn physical_consume_self_statuses_adds_bonus_damage_and_removes_statuses() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team =
        vec![make_char(0, vec![(Stat::VIT, 10), (Stat::WIL, 5), (Stat::MGT, 8), (Stat::ARM, 5)])];
    let mut enemy_team = vec![make_char(1, vec![(Stat::VIT, 10), (Stat::ARM, 3)])];
    actor_team[0].set_target(1);
    actor_team[0].add_status(
        &crate::statuses::status_key("Empower", Some(&Stat::MGT)),
        2,
        99,
        statuses.get("Empower").unwrap(),
        Some(Stat::MGT),
    );
    actor_team[0].add_status(
        &crate::statuses::status_key("Empower", Some(&Stat::ARM)),
        1,
        99,
        statuses.get("Empower").unwrap(),
        Some(Stat::ARM),
    );

    let ability = AbilityDef {
        mp_cost: 3,
        primitives: vec![Primitive::DealPhysicalDamageConsumeSelfStatuses {
            target: SimpleAbilityTarget::CurrentTarget.into(),
            multiplier: 1.0,
            statuses: vec![
                StatusRef {
                    status: "Empower".to_string(),
                    stat: Some(Stat::MGT),
                },
                StatusRef {
                    status: "Empower".to_string(),
                    stat: Some(Stat::ARM),
                },
            ],
            bonus_per_stack: 1,
        }],
    };

    execute_ability(
        0,
        "Sever",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );

    assert_eq!(enemy_team[0].current_hp(), 20);
    assert_eq!(
        actor_team[0].status_stacks(&crate::statuses::status_key("Empower", Some(&Stat::MGT))),
        0
    );
    assert_eq!(
        actor_team[0].status_stacks(&crate::statuses::status_key("Empower", Some(&Stat::ARM))),
        0
    );
}

#[test]
fn all_enemies_target_resolves_to_all_living() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![make_char(
        0,
        vec![(Stat::VIT, 10), (Stat::WIL, 5), (Stat::MGT, 10)],
    )];
    let mut enemy_team = vec![
        make_char(1, vec![(Stat::VIT, 10), (Stat::ARM, 3)]),
        make_char(2, vec![(Stat::VIT, 10), (Stat::ARM, 3)]),
        make_char(3, vec![(Stat::VIT, 10), (Stat::ARM, 3)]),
    ];
    enemy_team[1].take_damage(100);

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::DealPhysicalDamage {
            target: SimpleAbilityTarget::AllEnemies.into(),
            multiplier: 1.0,
            double_empower_stat: None,
        }],
    };

    let dealt = execute_ability(
        0,
        "Sweep",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );
    assert_eq!(dealt.len(), 2);
    assert!(dealt.iter().any(|record| record.target_id == 1));
    assert!(dealt.iter().any(|record| record.target_id == 3));
}

#[test]
fn all_allies_target_excludes_self() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![
        make_char(0, vec![(Stat::VIT, 10), (Stat::WIL, 5)]),
        make_char(1, vec![(Stat::VIT, 10), (Stat::WIL, 3)]),
        make_char(2, vec![(Stat::VIT, 10), (Stat::WIL, 3)]),
    ];
    actor_team[1].spend_mp(2);
    actor_team[2].spend_mp(2);
    let mut enemy_team = vec![make_char(10, vec![(Stat::VIT, 5)])];

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::RestoreMp {
            target: SimpleAbilityTarget::AllAllies.into(),
            amount: 5,
        }],
    };

    execute_ability(
        0,
        "Rally",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );
    assert_eq!(actor_team[0].current_mp(), 5);
    assert_eq!(actor_team[1].current_mp(), 3);
    assert_eq!(actor_team[2].current_mp(), 3);
}

#[test]
fn ally_selector_targets_lowest_hp_ally() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![
        make_char(0, vec![(Stat::VIT, 10), (Stat::WIL, 5)]),
        make_char(1, vec![(Stat::VIT, 10)]),
        make_char(2, vec![(Stat::VIT, 10)]),
    ];
    actor_team[1].take_damage(3);
    actor_team[2].take_damage(8);
    let mut enemy_team = vec![make_char(10, vec![(Stat::VIT, 5)])];

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::RestoreHp {
            target: AbilityTarget::Detailed(TargetSpec {
                category: TargetCategory::Ally,
                selector: Some(TargetSelector::LowestHp),
                position: None,
                bypass_row_protection: false,
            }),
            amount: 4,
        }],
    };

    execute_ability(
        0,
        "Rescue",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );
    assert_eq!(actor_team[1].current_hp(), 27);
    assert_eq!(actor_team[2].current_hp(), 26);
}

#[test]
fn enemy_selector_can_target_backmost_with_row_bypass() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![make_adjacent_char(
        0,
        0,
        0,
        vec![(Stat::MGT, 10), (Stat::VIT, 10), (Stat::WIL, 5)],
    )];
    let mut enemy_team = vec![
        make_adjacent_char(1, 0, 0, vec![(Stat::ARM, 3), (Stat::VIT, 10)]),
        make_adjacent_char(2, 2, 0, vec![(Stat::ARM, 3), (Stat::VIT, 10)]),
    ];

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::DealPhysicalDamage {
            target: AbilityTarget::Detailed(TargetSpec {
                category: TargetCategory::Enemy,
                selector: Some(TargetSelector::Random),
                position: Some(PositionalCondition::Backmost),
                bypass_row_protection: true,
            }),
            multiplier: 1.0,
            double_empower_stat: None,
        }],
    };

    let dealt = execute_ability(
        0,
        "Snipe",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(dealt.len(), 1);
    assert_eq!(dealt[0].target_id, 2);
}

#[test]
fn enemy_selector_can_target_same_column_enemy() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![make_adjacent_char(
        0,
        0,
        1,
        vec![(Stat::MGT, 10), (Stat::VIT, 10), (Stat::WIL, 5)],
    )];
    let mut enemy_team = vec![
        make_adjacent_char(1, 0, 0, vec![(Stat::ARM, 3), (Stat::VIT, 10)]),
        make_adjacent_char(2, 0, 1, vec![(Stat::ARM, 3), (Stat::VIT, 10)]),
    ];

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::DealPhysicalDamage {
            target: AbilityTarget::Detailed(TargetSpec {
                category: TargetCategory::Enemy,
                selector: Some(TargetSelector::Random),
                position: Some(PositionalCondition::SameColumn),
                bypass_row_protection: false,
            }),
            multiplier: 1.0,
            double_empower_stat: None,
        }],
    };

    let dealt = execute_ability(
        0,
        "LineStrike",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(dealt.len(), 1);
    assert_eq!(dealt[0].target_id, 2);
}

#[test]
fn current_target_and_companions_hits_target_and_living_companions() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![make_adjacent_char(
        0,
        0,
        0,
        vec![(Stat::MAG, 10), (Stat::VIT, 10), (Stat::WIL, 5)],
    )];
    let mut enemy_team = vec![
        make_adjacent_char(1, 0, 1, vec![(Stat::RES, 3), (Stat::VIT, 10)]),
        make_adjacent_char(2, 0, 2, vec![(Stat::RES, 3), (Stat::VIT, 10)]),
        make_adjacent_char(3, 1, 1, vec![(Stat::RES, 3), (Stat::VIT, 10)]),
        make_adjacent_char(4, 2, 2, vec![(Stat::RES, 3), (Stat::VIT, 10)]),
    ];
    enemy_team[0].set_companions(vec![2, 3]);
    enemy_team[2].set_companions(vec![1]);
    enemy_team[3].set_companions(vec![1]);
    enemy_team[1].take_damage(100);
    actor_team[0].set_target(1);

    let ability = AbilityDef {
        mp_cost: 2,
        primitives: vec![Primitive::DealMagicalDamage {
            target: SimpleAbilityTarget::CurrentTargetAndCompanions.into(),
            multiplier: 1.0,
        }],
    };

    let dealt = execute_ability(
        0,
        "Consecrate",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(dealt.len(), 2);
    assert!(dealt.iter().any(|record| record.target_id == 1));
    assert!(dealt.iter().any(|record| record.target_id == 3));
    assert!(!dealt.iter().any(|record| record.target_id == 2));
    assert!(!dealt.iter().any(|record| record.target_id == 4));
}

#[test]
fn current_target_and_companions_handles_missing_target() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![make_char(
        0,
        vec![(Stat::MAG, 10), (Stat::VIT, 10), (Stat::WIL, 5)],
    )];
    let mut enemy_team = vec![make_char(1, vec![(Stat::RES, 3), (Stat::VIT, 10)])];

    let ability = AbilityDef {
        mp_cost: 2,
        primitives: vec![Primitive::DealMagicalDamage {
            target: SimpleAbilityTarget::CurrentTargetAndCompanions.into(),
            multiplier: 1.0,
        }],
    };

    let dealt = execute_ability(
        0,
        "Consecrate",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert!(dealt.is_empty());
}

#[test]
fn ally_selector_can_target_same_row_ally() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![
        make_adjacent_char(0, 1, 1, vec![(Stat::VIT, 10), (Stat::WIL, 6)]),
        make_adjacent_char(1, 1, 2, vec![(Stat::VIT, 10), (Stat::WIL, 4)]),
        make_adjacent_char(2, 2, 1, vec![(Stat::VIT, 10), (Stat::WIL, 2)]),
    ];
    actor_team[1].spend_mp(3);
    actor_team[2].spend_mp(2);
    let mut enemy_team = vec![make_char(10, vec![(Stat::VIT, 5)])];

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::RestoreMp {
            target: AbilityTarget::Detailed(TargetSpec {
                category: TargetCategory::Ally,
                selector: Some(TargetSelector::LowestMp),
                position: Some(PositionalCondition::SameRow),
                bypass_row_protection: false,
            }),
            amount: 2,
        }],
    };

    execute_ability(
        0,
        "RowBlessing",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(actor_team[1].current_mp(), 3);
    assert_eq!(actor_team[2].current_mp(), 0);
}

#[test]
fn ally_detailed_target_without_selector_hits_all_matching_allies() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![
        make_adjacent_char(0, 1, 1, vec![(Stat::VIT, 10), (Stat::WIL, 6)]),
        make_adjacent_char(1, 1, 2, vec![(Stat::VIT, 10), (Stat::WIL, 4)]),
        make_adjacent_char(2, 1, 0, vec![(Stat::VIT, 10), (Stat::WIL, 2)]),
        make_adjacent_char(3, 2, 1, vec![(Stat::VIT, 10), (Stat::WIL, 1)]),
    ];
    actor_team[1].spend_mp(3);
    actor_team[2].spend_mp(1);
    actor_team[3].spend_mp(1);
    let mut enemy_team = vec![make_char(10, vec![(Stat::VIT, 5)])];

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::RestoreMp {
            target: AbilityTarget::Detailed(TargetSpec {
                category: TargetCategory::Ally,
                selector: None,
                position: Some(PositionalCondition::SameRow),
                bypass_row_protection: false,
            }),
            amount: 2,
        }],
    };

    execute_ability(
        0,
        "RowBlessing",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(actor_team[1].current_mp(), 3);
    assert_eq!(actor_team[2].current_mp(), 2);
    assert_eq!(actor_team[3].current_mp(), 0);
}

#[test]
fn companion_selector_can_target_same_row_companion() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![
        make_adjacent_char(0, 1, 1, vec![(Stat::VIT, 10)]),
        make_adjacent_char(1, 1, 2, vec![(Stat::VIT, 10)]),
        make_adjacent_char(2, 2, 1, vec![(Stat::VIT, 10)]),
    ];
    actor_team[0].set_companions(vec![1, 2]);
    let mut enemy_team = vec![make_char(10, vec![(Stat::VIT, 5)])];

    let mut statuses = empty_statuses();
    statuses.insert(
        "Empower".to_string(),
        StatusDef {
            behavior: StatusBehavior::StatModPerStack { magnitude: 1 },
            stack_type: StackType::TickDown,
            opposes: Some("Weaken".to_string()),
        },
    );

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::ApplyStatus {
            target: AbilityTarget::Detailed(TargetSpec {
                category: TargetCategory::Companion,
                selector: Some(TargetSelector::Random),
                position: Some(PositionalCondition::SameRow),
                bypass_row_protection: false,
            }),
            status: "Empower".to_string(),
            stat: Some(Stat::MGT),
            stacks: 1,
        }],
    };

    execute_ability(
        0,
        "RowCommand",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );

    let key = status_key("Empower", Some(&Stat::MGT));
    assert_eq!(actor_team[1].status_stacks(&key), 1);
    assert_eq!(actor_team[2].status_stacks(&key), 0);
}

#[test]
fn retarget_to_self_only_affects_physical_attackers() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![
        make_adjacent_char(0, 0, 0, vec![(Stat::VIT, 10)]),
        make_adjacent_char(1, 0, 1, vec![(Stat::VIT, 10)]),
    ];
    actor_team[0].set_companions(vec![1]);
    let mut enemy_team = vec![
        make_adjacent_char(10, 0, 0, vec![(Stat::MGT, 7), (Stat::MAG, 2), (Stat::VIT, 10)]),
        make_adjacent_char(11, 0, 1, vec![(Stat::MGT, 2), (Stat::MAG, 7), (Stat::VIT, 10)]),
    ];
    enemy_team[0].set_target(1);
    enemy_team[1].set_target(1);

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::Retarget {
            target: SimpleAbilityTarget::AllEnemies.into(),
            mode: RetargetMode::ToSelf,
            filter: Some(RetargetFilter::PhysicalAttackers),
        }],
    };

    execute_ability(
        0,
        "Taunt",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(enemy_team[0].target(), Some(0));
    assert_eq!(enemy_team[1].target(), Some(1));
}

#[test]
fn retarget_to_companion_chooses_living_companion() {
    let mut rng = StdRng::seed_from_u64(2);
    let mut log = BattleLog::new();
    let mut actor_team = vec![
        make_adjacent_char(0, 0, 0, vec![(Stat::VIT, 10)]),
        make_adjacent_char(1, 0, 1, vec![(Stat::VIT, 10)]),
        make_adjacent_char(2, 1, 0, vec![(Stat::VIT, 10)]),
    ];
    actor_team[0].set_companions(vec![1, 2]);
    actor_team[2].take_damage(100);
    let mut enemy_team = vec![make_adjacent_char(
        10,
        0,
        2,
        vec![(Stat::MGT, 7), (Stat::VIT, 10)],
    )];
    enemy_team[0].set_target(0);

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::Retarget {
            target: SimpleAbilityTarget::AllEnemies.into(),
            mode: RetargetMode::ToCompanion,
            filter: None,
        }],
    };

    execute_ability(
        0,
        "DecoyOrders",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(enemy_team[0].target(), Some(1));
}

#[test]
fn default_retarget_uses_normal_target_selection() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![
        make_adjacent_char(0, 0, 0, vec![(Stat::ARM, 9), (Stat::RES, 2), (Stat::VIT, 10)]),
        make_adjacent_char(1, 0, 1, vec![(Stat::ARM, 2), (Stat::RES, 9), (Stat::VIT, 10)]),
    ];
    let mut enemy_team = vec![make_adjacent_char(
        10,
        0,
        2,
        vec![(Stat::MGT, 8), (Stat::MAG, 3), (Stat::VIT, 10)],
    )];
    enemy_team[0].set_target(0);

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::Retarget {
            target: SimpleAbilityTarget::AllEnemies.into(),
            mode: RetargetMode::DefaultRetarget,
            filter: None,
        }],
    };

    execute_ability(
        0,
        "Confuse",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(enemy_team[0].target(), Some(1));
}

#[test]
fn retarget_to_self_only_affects_enemies_targeting_companions() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![
        make_adjacent_char(0, 0, 0, vec![(Stat::VIT, 10)]),
        make_adjacent_char(1, 0, 1, vec![(Stat::VIT, 10)]),
    ];
    actor_team[0].set_companions(vec![1]);
    actor_team[1].set_companions(vec![0]);

    let mut enemy_team = vec![
        make_adjacent_char(10, 0, 2, vec![(Stat::MGT, 7), (Stat::VIT, 10)]),
        make_adjacent_char(11, 1, 2, vec![(Stat::MGT, 7), (Stat::VIT, 10)]),
    ];
    enemy_team[0].set_target(1);
    enemy_team[1].set_target(0);

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::Retarget {
            target: SimpleAbilityTarget::AllEnemies.into(),
            mode: RetargetMode::ToSelf,
            filter: Some(RetargetFilter::TargetingCompanion),
        }],
    };

    execute_ability(
        0,
        "Interpose",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(enemy_team[0].target(), Some(0));
    assert_eq!(enemy_team[1].target(), Some(0));
}

#[test]
fn retarget_to_companion_only_affects_enemies_targeting_self() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![
        make_adjacent_char(0, 0, 0, vec![(Stat::VIT, 10)]),
        make_adjacent_char(1, 0, 1, vec![(Stat::VIT, 10)]),
    ];
    actor_team[0].set_companions(vec![1]);
    actor_team[1].set_companions(vec![0]);

    let mut enemy_team = vec![
        make_adjacent_char(10, 0, 2, vec![(Stat::MGT, 7), (Stat::VIT, 10)]),
        make_adjacent_char(11, 1, 2, vec![(Stat::MGT, 7), (Stat::VIT, 10)]),
    ];
    enemy_team[0].set_target(0);
    enemy_team[1].set_target(1);

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::Retarget {
            target: SimpleAbilityTarget::AllEnemies.into(),
            mode: RetargetMode::ToCompanion,
            filter: Some(RetargetFilter::TargetingSelf),
        }],
    };

    execute_ability(
        0,
        "Decoy",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(enemy_team[0].target(), Some(1));
    assert_eq!(enemy_team[1].target(), Some(1));
}

#[test]
fn disorient_retarget_picks_less_favorable_target() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();

    let mut actor_team = vec![
        make_adjacent_char(0, 0, 0, vec![(Stat::MGT, 8), (Stat::VIT, 10)]),
        make_adjacent_char(1, 0, 1, vec![(Stat::ARM, 9), (Stat::RES, 2), (Stat::VIT, 10)]),
        make_adjacent_char(2, 0, 2, vec![(Stat::ARM, 2), (Stat::RES, 9), (Stat::VIT, 10)]),
    ];
    actor_team[0].set_target(10);

    let mut enemy_team = vec![make_adjacent_char(
        10,
        0,
        3,
        vec![(Stat::MGT, 10), (Stat::MAG, 3), (Stat::VIT, 10)],
    )];
    enemy_team[0].set_target(2);

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::Retarget {
            target: SimpleAbilityTarget::CurrentTarget.into(),
            mode: RetargetMode::Disorient,
            filter: None,
        }],
    };

    execute_ability(
        0,
        "Rebuke",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(enemy_team[0].target(), Some(1));
}

#[test]
fn deal_true_damage_bypasses_ward() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team = vec![make_char(0, vec![(Stat::VIT, 10), (Stat::WIL, 5)])];
    let mut enemy_team = vec![make_char(1, vec![(Stat::VIT, 10)])];
    enemy_team[0].add_status("Ward", 1, 99, statuses.get("Ward").unwrap(), None);
    actor_team[0].set_target(1);

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::DealTrueDamage {
            target: SimpleAbilityTarget::CurrentTarget.into(),
            amount: 4,
        }],
    };

    execute_ability(
        0,
        "True Strike",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );

    assert_eq!(enemy_team[0].current_hp(), 26);
    assert_eq!(enemy_team[0].status_stacks("Ward"), 1);
}

#[test]
fn lose_current_hp_percent_costs_current_hp() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![make_char(0, vec![(Stat::VIT, 10), (Stat::WIL, 5)])];
    let mut enemy_team = vec![make_char(1, vec![(Stat::VIT, 10)])];
    actor_team[0].take_damage(10);

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::LoseCurrentHpPercent { percent: 20 }],
    };

    execute_ability(
        0,
        "Offer",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(actor_team[0].current_hp(), 16);
}

#[test]
fn split_magical_damage_uses_separate_primary_and_companion_values() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![make_adjacent_char(
        0,
        0,
        0,
        vec![(Stat::MAG, 10), (Stat::VIT, 10), (Stat::WIL, 5)],
    )];
    actor_team[0].set_target(10);

    let mut enemy_team = vec![
        make_adjacent_char(10, 0, 1, vec![(Stat::RES, 4), (Stat::VIT, 10)]),
        make_adjacent_char(11, 0, 2, vec![(Stat::RES, 4), (Stat::VIT, 10)]),
    ];
    enemy_team[0].set_companions(vec![11]);
    enemy_team[1].set_companions(vec![10]);

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::DealMagicalDamageCurrentTargetAndCompanions {
            primary_multiplier: 1.0,
            companion_multiplier: 0.5,
        }],
    };

    execute_ability(
        0,
        "Consecrate",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(enemy_team[0].current_hp(), 24);
    assert_eq!(enemy_team[1].current_hp(), 27);
}

#[test]
fn transform_statuses_converts_empower_into_weaken() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team = vec![make_char(0, vec![(Stat::MAG, 10), (Stat::VIT, 10)])];
    actor_team[0].set_target(1);
    let mut enemy_team = vec![make_char(1, vec![(Stat::VIT, 10)])];
    enemy_team[0].add_status(
        "Empower:MGT",
        3,
        99,
        statuses.get("Empower").unwrap(),
        Some(Stat::MGT),
    );
    enemy_team[0].add_status(
        "Empower:ARM",
        2,
        99,
        statuses.get("Empower").unwrap(),
        Some(Stat::ARM),
    );

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::TransformStatuses {
            target: SimpleAbilityTarget::CurrentTarget.into(),
            from_status: "Empower".to_string(),
            to_status: "Weaken".to_string(),
            stats: vec![Stat::MGT, Stat::ARM],
        }],
    };

    execute_ability(
        0,
        "Transmute",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );

    assert_eq!(enemy_team[0].status_stacks("Empower:MGT"), 0);
    assert_eq!(enemy_team[0].status_stacks("Empower:ARM"), 0);
    assert_eq!(enemy_team[0].status_stacks("Weaken:MGT"), 3);
    assert_eq!(enemy_team[0].status_stacks("Weaken:ARM"), 2);
}

#[test]
fn true_damage_consume_target_status_removes_stacks_and_deals_damage() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team = vec![make_char(0, vec![(Stat::MAG, 10), (Stat::VIT, 10)])];
    actor_team[0].set_target(1);
    let mut enemy_team = vec![make_char(1, vec![(Stat::VIT, 10)])];
    enemy_team[0].add_status("Omen", 3, 99, statuses.get("Omen").unwrap(), None);

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::DealTrueDamageConsumeTargetStatus {
            target: SimpleAbilityTarget::CurrentTarget.into(),
            status: "Omen".to_string(),
            stat: None,
            damage_per_stack: 2,
        }],
    };

    execute_ability(
        0,
        "Harvest Night",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );

    assert_eq!(enemy_team[0].current_hp(), 24);
    assert_eq!(enemy_team[0].status_stacks("Omen"), 0);
}

#[test]
fn true_damage_consume_self_statuses_removes_stacks_and_deals_damage() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team = vec![make_char(0, vec![(Stat::MGT, 10), (Stat::VIT, 10)])];
    actor_team[0].set_target(1);
    actor_team[0].add_status(
        "Empower:MGT",
        2,
        99,
        statuses.get("Empower").unwrap(),
        Some(Stat::MGT),
    );
    actor_team[0].add_status(
        "Empower:ARM",
        1,
        99,
        statuses.get("Empower").unwrap(),
        Some(Stat::ARM),
    );
    let mut enemy_team = vec![make_char(1, vec![(Stat::VIT, 10)])];

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::DealTrueDamageConsumeSelfStatuses {
            target: SimpleAbilityTarget::CurrentTarget.into(),
            statuses: vec![
                StatusRef {
                    status: "Empower".to_string(),
                    stat: Some(Stat::MGT),
                },
                StatusRef {
                    status: "Empower".to_string(),
                    stat: Some(Stat::ARM),
                },
            ],
            damage_per_stack: 1,
        }],
    };

    execute_ability(
        0,
        "Sever",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );

    assert_eq!(enemy_team[0].current_hp(), 27);
    assert_eq!(actor_team[0].status_stacks("Empower:MGT"), 0);
    assert_eq!(actor_team[0].status_stacks("Empower:ARM"), 0);
}

#[test]
fn retarget_enemies_focusing_targets_only_updates_matching_enemies() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![
        make_adjacent_char(0, 0, 0, vec![(Stat::VIT, 10), (Stat::ARM, 2), (Stat::RES, 8)]),
        make_adjacent_char(1, 0, 1, vec![(Stat::VIT, 10), (Stat::ARM, 8), (Stat::RES, 2)]),
        make_adjacent_char(2, 0, 2, vec![(Stat::VIT, 10), (Stat::ARM, 8), (Stat::RES, 2)]),
    ];
    actor_team[0].set_companions(vec![1, 2]);
    actor_team[1].take_damage(5);

    let mut enemy_team = vec![
        make_adjacent_char(10, 0, 3, vec![(Stat::MGT, 8), (Stat::VIT, 10)]),
        make_adjacent_char(11, 1, 3, vec![(Stat::MGT, 8), (Stat::VIT, 10)]),
    ];
    enemy_team[0].set_target(1);
    enemy_team[1].set_target(2);

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::RetargetEnemiesFocusingTargets {
            target: AbilityTarget::Detailed(TargetSpec {
                category: TargetCategory::Companion,
                selector: Some(TargetSelector::LowestHp),
                position: None,
                bypass_row_protection: false,
            }),
            mode: RetargetMode::DefaultRetarget,
        }],
    };

    execute_ability(
        0,
        "Rescue",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(enemy_team[0].target(), Some(0));
    assert_eq!(enemy_team[1].target(), Some(2));
}

#[test]
fn refocus_allies_to_actors_target_updates_selected_allies() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![
        make_adjacent_char(0, 0, 0, vec![(Stat::VIT, 10)]),
        make_adjacent_char(1, 0, 1, vec![(Stat::VIT, 10)]),
        make_adjacent_char(2, 1, 0, vec![(Stat::VIT, 10)]),
    ];
    actor_team[0].set_target(10);
    actor_team[1].set_target(11);
    actor_team[2].set_target(11);

    let mut enemy_team = vec![
        make_adjacent_char(10, 0, 3, vec![(Stat::VIT, 10)]),
        make_adjacent_char(11, 1, 3, vec![(Stat::VIT, 10)]),
    ];

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::RefocusAlliesToActorsTarget {
            target: AbilityTarget::Detailed(TargetSpec {
                category: TargetCategory::Ally,
                selector: None,
                position: Some(PositionalCondition::SameRow),
                bypass_row_protection: false,
            }),
        }],
    };

    execute_ability(
        0,
        "Blessing",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(actor_team[1].target(), Some(10));
    assert_eq!(actor_team[2].target(), Some(11));
}

#[test]
fn move_target_repositions_selected_companion() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![
        make_adjacent_char(0, 1, 0, vec![(Stat::VIT, 10)]),
        make_adjacent_char(1, 1, 1, vec![(Stat::VIT, 10)]),
    ];
    actor_team[0].set_companions(vec![1]);
    actor_team[1].set_companions(vec![0]);
    let mut enemy_team = vec![make_adjacent_char(10, 0, 3, vec![(Stat::VIT, 10)])];

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::MoveTarget {
            target: SimpleAbilityTarget::Companions.into(),
            direction: MoveDirection::Backward,
            if_empty: true,
        }],
    };

    execute_ability(
        0,
        "Rescue",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(actor_team[1].position().row, 2);
}

#[test]
fn cleanse_and_apply_status_if_changed_only_applies_when_cleanse_works() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team = vec![
        make_adjacent_char(0, 0, 0, vec![(Stat::VIT, 10)]),
        make_adjacent_char(1, 0, 1, vec![(Stat::VIT, 10)]),
        make_adjacent_char(2, 1, 0, vec![(Stat::VIT, 10)]),
    ];
    actor_team[1].add_status("Bleed", 2, 99, statuses.get("Bleed").unwrap(), None);
    let mut enemy_team = vec![make_adjacent_char(10, 0, 3, vec![(Stat::VIT, 10)])];

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::CleanseAndApplyStatusIfChanged {
            target: AbilityTarget::Detailed(TargetSpec {
                category: TargetCategory::Ally,
                selector: None,
                position: Some(PositionalCondition::SameRow),
                bypass_row_protection: false,
            }),
            amount: 1,
            status: "Ward".to_string(),
            stat: None,
            stacks: 1,
        }],
    };

    execute_ability(
        0,
        "Sanctify",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );

    assert_eq!(actor_team[1].status_stacks("Ward"), 1);
    assert_eq!(actor_team[2].status_stacks("Ward"), 0);
}

#[test]
fn command_attack_uses_highest_str_living_companion() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![
        make_adjacent_char(0, 0, 0, vec![(Stat::VIT, 10)]),
        make_adjacent_char(1, 0, 1, vec![(Stat::MGT, 8), (Stat::VIT, 10)]),
        make_adjacent_char(2, 1, 0, vec![(Stat::MGT, 5), (Stat::VIT, 10)]),
    ];
    actor_team[0].set_companions(vec![1, 2]);
    actor_team[0].set_target(10);
    let mut enemy_team = vec![make_adjacent_char(
        10,
        0,
        2,
        vec![(Stat::ARM, 3), (Stat::VIT, 10)],
    )];

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::CommandAttack],
    };

    let dealt = execute_ability(
        0,
        "Command",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(dealt.len(), 1);
    assert_eq!(dealt[0].source_id, 1);
    assert_eq!(dealt[0].target_id, 10);
    assert_eq!(enemy_team[0].current_hp(), 25);
}

#[test]
fn ward_negates_physical_ability_damage() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team = vec![make_char(
        0,
        vec![(Stat::MGT, 10), (Stat::VIT, 10), (Stat::WIL, 5)],
    )];
    let mut enemy_team = vec![make_char(1, vec![(Stat::ARM, 4), (Stat::VIT, 20)])];
    enemy_team[0].add_status("Ward", 1, 99, statuses.get("Ward").unwrap(), None);
    actor_team[0].set_target(1);

    let ability = AbilityDef {
        mp_cost: 2,
        primitives: vec![Primitive::DealPhysicalDamage {
            target: SimpleAbilityTarget::CurrentTarget.into(),
            multiplier: 1.5,
            double_empower_stat: None,
        }],
    };

    let dealt = execute_ability(
        0,
        "Crush",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );

    assert_eq!(dealt[0].damage, 0);
    assert_eq!(enemy_team[0].current_hp(), 60);
    assert_eq!(enemy_team[0].status_stacks("Ward"), 0);
}

#[test]
fn ward_negates_magical_ability_damage() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team = vec![make_char(
        0,
        vec![(Stat::MAG, 10), (Stat::VIT, 10), (Stat::WIL, 5)],
    )];
    let mut enemy_team = vec![make_char(1, vec![(Stat::RES, 4), (Stat::VIT, 20)])];
    enemy_team[0].add_status("Ward", 1, 99, statuses.get("Ward").unwrap(), None);
    actor_team[0].set_target(1);

    let ability = AbilityDef {
        mp_cost: 2,
        primitives: vec![Primitive::DealMagicalDamage {
            target: SimpleAbilityTarget::CurrentTarget.into(),
            multiplier: 1.5,
        }],
    };

    let dealt = execute_ability(
        0,
        "Smite",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );

    assert_eq!(dealt[0].damage, 0);
    assert_eq!(enemy_team[0].current_hp(), 60);
    assert_eq!(enemy_team[0].status_stacks("Ward"), 0);
}

#[test]
fn doubled_empower_stat_increases_only_the_flagged_attack() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let statuses = test_statuses();
    let mut actor_team = vec![make_char(
        0,
        vec![(Stat::MGT, 10), (Stat::MAG, 8), (Stat::VIT, 10), (Stat::WIL, 5)],
    )];
    let mut enemy_team =
        vec![make_char(1, vec![(Stat::ARM, 4), (Stat::RES, 4), (Stat::VIT, 20)])];
    actor_team[0].set_target(1);
    actor_team[0].add_status(
        &status_key("Empower", Some(&Stat::MGT)),
        2,
        99,
        statuses.get("Empower").unwrap(),
        Some(Stat::MGT),
    );

    let normal = AbilityDef {
        mp_cost: 2,
        primitives: vec![Primitive::DealPhysicalDamage {
            target: SimpleAbilityTarget::CurrentTarget.into(),
            multiplier: 1.0,
            double_empower_stat: None,
        }],
    };
    let payoff = AbilityDef {
        mp_cost: 2,
        primitives: vec![Primitive::DealPhysicalDamage {
            target: SimpleAbilityTarget::CurrentTarget.into(),
            multiplier: 1.0,
            double_empower_stat: Some(Stat::MGT),
        }],
    };

    let normal_dealt = execute_ability(
        0,
        "Charge",
        &normal,
        &mut actor_team.clone(),
        &mut enemy_team.clone(),
        &mut rng,
        &mut log,
        1,
        &statuses,
    );

    let payoff_dealt = execute_ability(
        0,
        "Breakthrough",
        &payoff,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &statuses,
    );

    assert_eq!(normal_dealt[0].damage, 8);
    assert_eq!(payoff_dealt[0].damage, 10);
}

#[test]
fn move_forward_updates_position_and_companions() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![
        make_adjacent_char(0, 1, 1, vec![(Stat::VIT, 10)]),
        make_adjacent_char(1, 1, 2, vec![(Stat::VIT, 10)]),
        make_adjacent_char(2, 0, 0, vec![(Stat::VIT, 10)]),
    ];
    actor_team[0].set_companions(vec![1]);
    actor_team[1].set_companions(vec![0]);
    actor_team[2].set_companions(vec![]);
    let mut enemy_team = vec![make_adjacent_char(10, 0, 3, vec![(Stat::VIT, 10)])];

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::Move {
            direction: MoveDirection::Forward,
            if_empty: true,
        }],
    };

    execute_ability(
        0,
        "Charge",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(actor_team[0].position().row, 0);
    assert_eq!(actor_team[0].position().col, 1);
    assert_eq!(actor_team[0].companions(), &[2]);
    assert!(actor_team[1].companions().is_empty());
    assert_eq!(actor_team[2].companions(), &[0]);

    let moved_event = log.events().iter().find_map(|event| match event {
        BattleEvent::Moved {
            character_id,
            from_row,
            from_col,
            to_row,
            to_col,
            ..
        } => Some((*character_id, *from_row, *from_col, *to_row, *to_col)),
        _ => None,
    });
    assert_eq!(moved_event, Some((0, 1, 1, 0, 1)));
}

#[test]
fn move_backward_requires_empty_destination_when_flagged() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![
        make_adjacent_char(0, 1, 1, vec![(Stat::VIT, 10)]),
        make_adjacent_char(1, 2, 1, vec![(Stat::VIT, 10)]),
    ];
    actor_team[0].set_companions(vec![]);
    actor_team[1].set_companions(vec![]);
    let mut enemy_team = vec![make_adjacent_char(10, 0, 0, vec![(Stat::VIT, 10)])];

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::Move {
            direction: MoveDirection::Backward,
            if_empty: true,
        }],
    };

    execute_ability(
        0,
        "Withdraw",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(actor_team[0].position().row, 1);
    assert!(log
        .events()
        .iter()
        .all(|event| !matches!(event, BattleEvent::Moved { .. })));
}

#[test]
fn apply_condition_applies_marked_and_logs_event() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![make_char(0, vec![(Stat::VIT, 10), (Stat::WIL, 5)])];
    let mut enemy_team = vec![make_char(1, vec![(Stat::VIT, 10)])];
    actor_team[0].set_target(1);

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::ApplyCondition {
            target: SimpleAbilityTarget::CurrentTarget.into(),
            condition: "Marked".to_string(),
            stacks: 2,
        }],
    };

    execute_ability(
        0,
        "Mark",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(enemy_team[0].condition_stacks(ConditionKind::Marked), 2);
    assert!(log.events().iter().any(|event| matches!(
        event,
        BattleEvent::ConditionApplied {
            condition_name,
            stacks_after,
            ..
        } if condition_name == "Marked" && *stacks_after == 2
    )));
}

#[test]
fn current_target_and_companions_ignores_severed_companions() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![make_char(
        0,
        vec![(Stat::MAG, 10), (Stat::VIT, 10), (Stat::WIL, 5)],
    )];
    let mut enemy_team = vec![
        make_adjacent_char(1, 0, 0, vec![(Stat::RES, 1), (Stat::VIT, 10)]),
        make_adjacent_char(2, 0, 1, vec![(Stat::RES, 1), (Stat::VIT, 10)]),
    ];
    enemy_team[0].set_companions(vec![2]);
    enemy_team[1].set_companions(vec![1]);
    enemy_team[0].add_condition(ConditionKind::Severed, 2, 99);
    actor_team[0].set_target(1);

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::DealMagicalDamage {
            target: SimpleAbilityTarget::CurrentTargetAndCompanions.into(),
            multiplier: 1.0,
        }],
    };

    let dealt = execute_ability(
        0,
        "Eclipse",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(dealt.len(), 1);
    assert_eq!(dealt[0].target_id, 1);
    assert_eq!(enemy_team[1].current_hp(), 30);
}

#[test]
fn bind_targets_keeps_same_ally_across_nested_primitives() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut log = BattleLog::new();
    let mut actor_team = vec![
        make_adjacent_char(0, 0, 0, vec![(Stat::VIT, 10), (Stat::WIL, 5)]),
        make_adjacent_char(1, 0, 1, vec![(Stat::VIT, 10), (Stat::WIL, 5)]),
        make_adjacent_char(2, 0, 2, vec![(Stat::VIT, 10), (Stat::WIL, 5)]),
    ];
    actor_team[0].set_companions(vec![1, 2]);
    actor_team[1].take_damage(12); // 18 HP
    actor_team[2].take_damage(10); // 20 HP
    let mut enemy_team = vec![make_char(10, vec![(Stat::VIT, 10)])];

    let ability = AbilityDef {
        mp_cost: 1,
        primitives: vec![Primitive::BindTargets {
            target: AbilityTarget::Detailed(TargetSpec {
                category: TargetCategory::Companion,
                selector: Some(TargetSelector::LowestHp),
                position: None,
                bypass_row_protection: false,
            }),
            primitives: vec![
                Primitive::RestoreHp {
                    target: SimpleAbilityTarget::BoundAlly.into(),
                    amount: 10,
                },
                Primitive::ApplyCondition {
                    target: SimpleAbilityTarget::BoundAlly.into(),
                    condition: "Marked".to_string(),
                    stacks: 1,
                },
            ],
        }],
    };

    execute_ability(
        0,
        "RescueLike",
        &ability,
        &mut actor_team,
        &mut enemy_team,
        &mut rng,
        &mut log,
        1,
        &empty_statuses(),
    );

    assert_eq!(actor_team[1].current_hp(), 28);
    assert_eq!(actor_team[1].condition_stacks(ConditionKind::Marked), 1);
    assert_eq!(actor_team[2].condition_stacks(ConditionKind::Marked), 0);
}
