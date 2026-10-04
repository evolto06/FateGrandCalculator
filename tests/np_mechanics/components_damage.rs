use fate_grand_calculator::{
    damage::{TurnBuffs, TurnDamageResult, calculate_turn},
    loader::{AffectionScaling, GameData, ServantRecord},
    model::{AttributeType, CardType, ClassType, SelectedCard, TurnSelection},
    np_mechanics::components::{NpDamageComponent, NpDamageValues, NpTarget, components_are_valid},
};

fn component(values: [f64; 5], check_dead: bool) -> NpDamageComponent {
    NpDamageComponent {
        target: Some(NpTarget::EnemyAll),
        overcharge: std::array::from_fn(|_| {
            let mut row = NpDamageValues::guaranteed(values);
            row.check_dead = [check_dead; 5];
            Some(row)
        }),
    }
}

fn servant(id: u32) -> ServantRecord {
    let mut servant = GameData::bundled().unwrap().servants.remove(0);
    servant.attack = 1003;
    servant.class = ClassType::Saber;
    servant.attribute = AttributeType::Earth;
    let (base, extra, card) = match id {
        201301 => (
            [6.0, 8.0, 9.0, 9.5, 10.0],
            [0.0, 1.0, 2.0, 3.0, 4.0],
            CardType::Buster,
        ),
        201302 => (
            [8.0, 10.0, 11.0, 11.5, 12.0],
            [0.0, 2.0, 4.0, 6.0, 8.0],
            CardType::Buster,
        ),
        504401 => (
            [9.0, 12.0, 13.5, 14.25, 15.0],
            [0.0, 2.25, 4.5, 6.75, 9.0],
            CardType::Arts,
        ),
        305401 => (
            [6.0, 8.0, 9.0, 9.5, 10.0],
            [0.0, 2.0, 3.0, 4.0, 5.0],
            CardType::Buster,
        ),
        _ => unreachable!(),
    };
    let mut bonus = component([0.0; 5], false);
    for (oc, value) in extra.into_iter().enumerate() {
        let row = bonus.overcharge[oc].as_mut().unwrap();
        row.multipliers = [value; 5];
        if id == 305401 {
            row.rates = [if oc == 0 { 0 } else { 1000 }; 5];
            row.check_dead = [oc > 0; 5];
        }
    }
    let np = &mut servant.noble_phantasms[0];
    np.id = id;
    np.card_type = card;
    np.defense_pierce = false;
    np.affection = None;
    np.components = vec![component(base, false), bonus];
    servant
}

fn run(
    servant: &ServantRecord,
    level: u8,
    oc: u8,
    affection: u8,
    buffs: TurnBuffs,
) -> TurnDamageResult {
    calculate_turn(
        servant,
        &TurnSelection {
            slots: [
                SelectedCard::NoblePhantasm(0),
                SelectedCard::Normal(0),
                SelectedCard::Normal(1),
            ],
            np_level: level,
            overcharge_level: oc,
            affection_level: affection,
            attacker_hp: None,
            enemy_status: None,
        },
        buffs,
        ClassType::Saber,
        AttributeType::Earth,
    )
    .unwrap()
}

// Atlas NA damage functions (201300, 504400, 305400), retrieved 2026-09-30.
// Gold ranges calculated independently with Python Decimal, applying floor to
// each component at random .900 / 1.099. Controlled ATK1003, Saber vs Saber,
// Earth vs Earth; rows OC1-5 and columns NP1-5. Actual servant stats are replaced
// deliberately so rounding and component mechanics have independent constants.
const GOLD: [(u32, [[(u32, u32); 5]; 5]); 4] = [
    (
        201301,
        [
            [
                (1868, 2281),
                (2491, 3042),
                (2802, 3422),
                (2958, 3612),
                (3114, 3802),
            ],
            [
                (2179, 2661),
                (2802, 3422),
                (3113, 3802),
                (3269, 3992),
                (3425, 4182),
            ],
            [
                (2490, 3041),
                (3113, 3802),
                (3424, 4182),
                (3580, 4372),
                (3736, 4562),
            ],
            [
                (2802, 3421),
                (3425, 4182),
                (3736, 4562),
                (3892, 4752),
                (4048, 4942),
            ],
            [
                (3113, 3802),
                (3736, 4563),
                (4047, 4943),
                (4203, 5133),
                (4359, 5323),
            ],
        ],
    ),
    (
        201302,
        [
            [
                (2491, 3042),
                (3114, 3802),
                (3425, 4183),
                (3581, 4373),
                (3737, 4563),
            ],
            [
                (3113, 3802),
                (3736, 4562),
                (4047, 4943),
                (4203, 5133),
                (4359, 5323),
            ],
            [
                (3736, 4563),
                (4359, 5323),
                (4670, 5704),
                (4826, 5894),
                (4982, 6084),
            ],
            [
                (4359, 5323),
                (4982, 6083),
                (5293, 6464),
                (5449, 6654),
                (5605, 6844),
            ],
            [
                (4982, 6084),
                (5605, 6844),
                (5916, 7225),
                (6072, 7415),
                (6228, 7605),
            ],
        ],
    ),
    (
        504401,
        [
            [
                (1868, 2281),
                (2491, 3042),
                (2802, 3422),
                (2958, 3612),
                (3114, 3802),
            ],
            [
                (2335, 2851),
                (2958, 3612),
                (3269, 3992),
                (3425, 4182),
                (3581, 4372),
            ],
            [
                (2802, 3421),
                (3425, 4182),
                (3736, 4562),
                (3892, 4752),
                (4048, 4942),
            ],
            [
                (3269, 3992),
                (3892, 4753),
                (4203, 5133),
                (4359, 5323),
                (4515, 5513),
            ],
            [
                (3736, 4562),
                (4359, 5323),
                (4670, 5703),
                (4826, 5893),
                (4982, 6083),
            ],
        ],
    ),
    (
        305401,
        [
            [
                (1868, 2281),
                (2491, 3042),
                (2802, 3422),
                (2958, 3612),
                (3114, 3802),
            ],
            [
                (2490, 3041),
                (3113, 3802),
                (3424, 4182),
                (3580, 4372),
                (3736, 4562),
            ],
            [
                (2802, 3421),
                (3425, 4182),
                (3736, 4562),
                (3892, 4752),
                (4048, 4942),
            ],
            [
                (3113, 3802),
                (3736, 4563),
                (4047, 4943),
                (4203, 5133),
                (4359, 5323),
            ],
            [
                (3425, 4182),
                (4048, 4943),
                (4359, 5323),
                (4515, 5513),
                (4671, 5703),
            ],
        ],
    ),
];

#[test]
fn all_four_variants_match_independent_np_level_overcharge_ranges() {
    for (id, expected) in GOLD {
        let servant = servant(id);
        let baseline = run(&servant, 1, 1, 0, TurnBuffs::default());
        for (oc, row) in expected.iter().enumerate() {
            for (level, &(min, max)) in row.iter().enumerate() {
                let result = run(
                    &servant,
                    level as u8 + 1,
                    oc as u8 + 1,
                    0,
                    TurnBuffs::default(),
                );
                assert_eq!(
                    (
                        result.cards[0].minimum_damage,
                        result.cards[0].maximum_damage
                    ),
                    (min, max),
                    "NP {id} level {} OC {}",
                    level + 1,
                    oc + 1
                );
                assert_eq!(result.np_components[0].len(), 2);
                assert!(result.np_components[1].is_empty() && result.np_components[2].is_empty());
                for slot in 1..3 {
                    assert_eq!(
                        result.cards[slot].minimum_damage,
                        baseline.cards[slot].minimum_damage
                    );
                    assert_eq!(
                        result.cards[slot].maximum_damage,
                        baseline.cards[slot].maximum_damage
                    );
                }
                assert_eq!(
                    result.extra.unwrap().minimum_damage,
                    baseline.extra.unwrap().minimum_damage
                );
                assert_eq!(
                    result.extra.unwrap().maximum_damage,
                    baseline.extra.unwrap().maximum_damage
                );
                if oc == 0 {
                    assert_eq!(result.np_components[0][1].maximum_damage, 0);
                    assert_eq!(
                        result.cards[0].minimum_damage,
                        result.np_components[0][0].minimum_damage
                    );
                }
            }
        }
    }
}

#[test]
fn components_are_floored_separately_before_totaling() {
    let result = run(&servant(201301), 1, 2, 0, TurnBuffs::default());
    assert_eq!(
        result.np_components[0]
            .iter()
            .map(|p| p.minimum_damage)
            .collect::<Vec<_>>(),
        [1868, 311]
    );
    assert_eq!(result.cards[0].minimum_damage, 2179);
    assert_eq!(
        (result.cards[0].damage_before_random * 0.9).floor() as u32,
        2180
    );
    assert_eq!(result.cards[0].maximum_damage, 2661);
    assert_eq!(
        (result.cards[0].damage_before_random * 1.099).floor() as u32,
        2662
    );
}

#[test]
fn np_components_follow_the_selected_slot_and_keep_function_order() {
    let servant = servant(201301);
    let first = run(&servant, 1, 2, 0, TurnBuffs::default());
    let mut selection = TurnSelection::default_for(&servant).unwrap();
    selection.slots = [
        SelectedCard::Normal(0),
        SelectedCard::Normal(1),
        SelectedCard::NoblePhantasm(0),
    ];
    selection.overcharge_level = 2;
    let last = calculate_turn(
        &servant,
        &selection,
        TurnBuffs::default(),
        ClassType::Saber,
        AttributeType::Earth,
    )
    .unwrap();
    assert!(last.np_components[0].is_empty() && last.np_components[1].is_empty());
    assert_eq!(
        last.np_components[2]
            .iter()
            .map(|part| part.minimum_damage)
            .collect::<Vec<_>>(),
        [1868, 311]
    );
    assert_eq!(last.cards[2].minimum_damage, first.cards[0].minimum_damage);
    assert_eq!(last.cards[2].maximum_damage, first.cards[0].maximum_damage);
}

#[test]
fn np_buffs_and_defense_pierce_apply_to_every_component_only() {
    let mut pierced = servant(305401);
    pierced.noble_phantasms[0].defense_pierce = true;
    for defense in [0.3, -0.2] {
        let buffs = TurnBuffs {
            attack_buff: 0.25,
            buster_buff: 0.2,
            np_damage_buff: 0.5,
            enemy_defense: defense,
            ..Default::default()
        };
        let result = run(&pierced, 3, 4, 0, buffs);
        let mut ordinary = pierced.clone();
        ordinary.noble_phantasms[0].defense_pierce = false;
        let expected = run(
            &ordinary,
            3,
            4,
            0,
            TurnBuffs {
                enemy_defense: defense.min(0.0),
                ..buffs
            },
        );
        for (actual, expected) in result.np_components[0]
            .iter()
            .zip(&expected.np_components[0])
        {
            assert_eq!(actual.minimum_damage, expected.minimum_damage);
            assert_eq!(actual.maximum_damage, expected.maximum_damage);
        }
        let normal = run(&ordinary, 3, 4, 0, buffs);
        assert_eq!(
            result.cards[1].minimum_damage,
            normal.cards[1].minimum_damage
        );
        assert_eq!(
            result.extra.unwrap().maximum_damage,
            normal.extra.unwrap().maximum_damage
        );
    }
}

#[test]
fn overcharge_does_not_automatically_increase_space_eresh_affection() {
    let mut eresh = servant(201301);
    eresh.id = 3300200;
    eresh.noble_phantasms[0].card_type = CardType::Arts;
    eresh.noble_phantasms[0].components = vec![component([4.5, 6.0, 6.75, 7.125, 7.5], false)];
    eresh.noble_phantasms[0].affection = Some(AffectionScaling {
        base: 1.0,
        per_level: 0.1,
        max_level: 10,
        ignore_defense_at: 7,
    });
    for affection in [0, 6, 7, 10] {
        let baseline = run(
            &eresh,
            3,
            1,
            affection,
            TurnBuffs {
                enemy_defense: 0.3,
                ..Default::default()
            },
        );
        for oc in 2..=5 {
            let result = run(
                &eresh,
                3,
                oc,
                affection,
                TurnBuffs {
                    enemy_defense: 0.3,
                    ..Default::default()
                },
            );
            assert_eq!(
                result.cards[0].minimum_damage,
                baseline.cards[0].minimum_damage
            );
            assert_eq!(
                result.cards[0].maximum_damage,
                baseline.cards[0].maximum_damage
            );
        }
    }
}

#[test]
fn rejects_invalid_overcharge_and_missing_legacy_coverage() {
    let mut servant = servant(201301);
    let mut selection = TurnSelection::default_for(&servant).unwrap();
    for oc in [0, 6] {
        selection.overcharge_level = oc;
        assert!(selection.validate(&servant).is_err());
    }
    servant.noble_phantasms[0].components =
        vec![NpDamageComponent::legacy([6.0, 8.0, 9.0, 9.5, 10.0])];
    selection.overcharge_level = 1;
    assert!(selection.validate(&servant).is_ok());
    selection.overcharge_level = 2;
    assert!(
        selection
            .validate(&servant)
            .unwrap_err()
            .contains("Update servant data")
    );
}

#[test]
fn component_validation_rejects_probability_mixed_targets_and_incomplete_rows() {
    let valid = servant(305401).noble_phantasms.remove(0).components;
    assert!(components_are_valid(&valid));
    let mut invalid = valid.clone();
    invalid[1].overcharge[1].as_mut().unwrap().rates[0] = 500;
    assert!(!components_are_valid(&invalid));
    let mut invalid = valid.clone();
    invalid[1].target = Some(NpTarget::Enemy);
    assert!(!components_are_valid(&invalid));
    let mut invalid = valid.clone();
    invalid[1].overcharge[2] = None;
    assert!(!components_are_valid(&invalid));
    let mut invalid = valid;
    invalid[1].overcharge[0].as_mut().unwrap().multipliers[0] = 1.0;
    assert!(!components_are_valid(&invalid));
}
