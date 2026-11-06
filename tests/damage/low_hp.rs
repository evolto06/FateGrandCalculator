use fate_grand_calculator::{
    damage::{TurnBuffs, calculate_turn},
    loader::{AffectionScaling, GameData, ServantRecord},
    model::{AttackerHp, AttributeType, CardType, ClassType, SelectedCard, TurnSelection},
    np_mechanics::{
        components::{NpDamageComponent, NpDamageValues, NpTarget},
        low_hp::LowHpScaling,
    },
};

fn servant(base: [u32; 5], coefficients: [u32; 5]) -> ServantRecord {
    let mut servant = GameData::bundled().unwrap().servants.remove(0);
    servant.attack = 1003;
    servant.max_hp = Some(10000);
    servant.class = ClassType::Saber;
    servant.attribute = AttributeType::Earth;
    let np = &mut servant.noble_phantasms[0];
    np.card_type = CardType::Buster;
    np.affection = None;
    np.defense_pierce = false;
    np.components = vec![NpDamageComponent {
        target: Some(NpTarget::Enemy),
        overcharge: std::array::from_fn(|oc| {
            let mut values = NpDamageValues::guaranteed(base.map(|v| f64::from(v) / 1000.0));
            values.low_hp = Some(LowHpScaling {
                source_base_rates: base,
                coefficients: [coefficients[oc]; 5],
            });
            Some(values)
        }),
    }];
    servant
}

#[test]
fn all_six_variants_match_450_independent_damage_endpoints() {
    let variants = [
        (202501, [6000, 8000, 9000, 9500, 10000], [6000; 5]),
        (
            203301,
            [6000, 8000, 9000, 9500, 10000],
            [6000, 7000, 8000, 9000, 10000],
        ),
        (
            400901,
            [12000, 16000, 18000, 19000, 20000],
            [12000, 14000, 16000, 18000, 20000],
        ),
        (
            400902,
            [16000, 20000, 22000, 23000, 24000],
            [12000, 14000, 16000, 18000, 20000],
        ),
        (
            702501,
            [6000, 8000, 9000, 9500, 10000],
            [6000, 7000, 8000, 9000, 10000],
        ),
        (
            702502,
            [8000, 10000, 11000, 11500, 12000],
            [8000, 9000, 10000, 11000, 12000],
        ),
    ];
    for (id, base, coefficients) in variants {
        let servant = servant(base, coefficients);
        for (oc, coefficient) in coefficients.into_iter().enumerate() {
            for (level, base) in base.into_iter().enumerate() {
                for (current, contribution) in [
                    (10000, 0),
                    (5000, coefficient / 2),
                    (1, coefficient - coefficient.div_ceil(10000)),
                ] {
                    let mut selection = TurnSelection::default_for(&servant).unwrap();
                    selection.np_level = level as u8 + 1;
                    selection.overcharge_level = oc as u8 + 1;
                    selection.attacker_hp = Some(AttackerHp::new(current, 10000).unwrap());
                    let result = calculate_turn(
                        &servant,
                        &selection,
                        TurnBuffs::default(),
                        ClassType::Saber,
                        AttributeType::Earth,
                    )
                    .unwrap();
                    // Exact rational damage endpoint arithmetic independent of
                    // production floats: ATK1003 * .23 * 1.5 * rate/1000 * RNG.
                    let rate = u64::from(base + contribution);
                    let minimum = (1003 * 3105 * rate / 10_000_000) as u32;
                    let maximum = (1003 * 379155 * rate / 1_000_000_000) as u32;
                    assert_eq!(
                        (
                            result.cards[0].minimum_damage,
                            result.cards[0].maximum_damage
                        ),
                        (minimum, maximum),
                        "NP{id} NP{} OC{} HP{current}",
                        level + 1,
                        oc + 1
                    );
                }
            }
        }
    }
}

#[test]
fn hp_scaling_is_local_and_preserves_component_rounding_and_np_modifiers() {
    let mut servant = servant([6000; 5], [6000; 5]);
    let second = NpDamageComponent {
        target: Some(NpTarget::Enemy),
        overcharge: std::array::from_fn(|_| Some(NpDamageValues::guaranteed([1.0; 5]))),
    };
    servant.noble_phantasms[0].components.push(second);
    servant.noble_phantasms[0].defense_pierce = true;
    servant.noble_phantasms[0].affection = Some(AffectionScaling {
        base: 1.0,
        per_level: 0.1,
        max_level: 10,
        ignore_defense_at: 7,
    });
    let mut selection = TurnSelection::default_for(&servant).unwrap();
    selection.affection_level = 6;
    let buffs = TurnBuffs {
        enemy_defense: 0.3,
        np_damage_buff: 0.5,
        ..Default::default()
    };
    let full = calculate_turn(
        &servant,
        &selection,
        buffs,
        ClassType::Saber,
        AttributeType::Earth,
    )
    .unwrap();
    selection.attacker_hp = Some(AttackerHp::new(5000, 10000).unwrap());
    let half = calculate_turn(
        &servant,
        &selection,
        buffs,
        ClassType::Saber,
        AttributeType::Earth,
    )
    .unwrap();
    assert!(half.np_components[0][0].minimum_damage > full.np_components[0][0].minimum_damage);
    assert_eq!(
        half.np_components[0][1].minimum_damage,
        full.np_components[0][1].minimum_damage
    );
    for position in 1..3 {
        assert_eq!(
            half.cards[position].minimum_damage,
            full.cards[position].minimum_damage
        );
        assert_eq!(
            half.cards[position].maximum_damage,
            full.cards[position].maximum_damage
        );
    }
    assert_eq!(
        half.extra.unwrap().minimum_damage,
        full.extra.unwrap().minimum_damage
    );
    assert_eq!(
        half.extra.unwrap().maximum_damage,
        full.extra.unwrap().maximum_damage
    );
    assert_eq!(
        half.cards[0].minimum_damage,
        half.np_components[0]
            .iter()
            .map(|p| p.minimum_damage)
            .sum::<u32>()
    );
    assert_eq!(
        half.cards[0].maximum_damage,
        half.np_components[0]
            .iter()
            .map(|p| p.maximum_damage)
            .sum::<u32>()
    );
    assert_eq!(half.cards[0].attack_defense_multiplier, 1.0);
    assert_eq!(
        half.np_components[0][0].minimum_damage,
        (1003.0_f64 * 0.23 * 1.5 * 9.0 * 1.6 * 1.5 * 0.9).floor() as u32
    );
}

#[test]
fn invalid_hp_is_required_only_for_the_selected_low_hp_np() {
    let servant = servant([6000; 5], [6000; 5]);
    let mut selection = TurnSelection::default_for(&servant).unwrap();
    for hp in [
        None,
        Some(AttackerHp { current: 1, max: 0 }),
        Some(AttackerHp {
            current: 0,
            max: 10000,
        }),
        Some(AttackerHp {
            current: 10001,
            max: 10000,
        }),
    ] {
        selection.attacker_hp = hp;
        assert!(
            calculate_turn(
                &servant,
                &selection,
                TurnBuffs::default(),
                ClassType::Saber,
                AttributeType::Earth
            )
            .is_err()
        );
        let mut ordinary = selection.clone();
        ordinary.slots = [
            SelectedCard::Normal(0),
            SelectedCard::Normal(1),
            SelectedCard::Normal(2),
        ];
        assert!(
            calculate_turn(
                &servant,
                &ordinary,
                TurnBuffs::default(),
                ClassType::Saber,
                AttributeType::Earth
            )
            .is_ok()
        );
    }
    let mut unknown = servant.clone();
    unknown.max_hp = None;
    assert_eq!(
        TurnSelection::default_for(&unknown).unwrap().attacker_hp,
        None
    );
    assert_eq!(
        TurnSelection::default_for(&servant).unwrap().attacker_hp,
        Some(AttackerHp::new(10000, 10000).unwrap())
    );
}

#[test]
fn invalid_metadata_is_rejected_without_runtime_panics() {
    let valid = servant([6000; 5], [6000; 5]);
    for mutation in 0..4 {
        let mut invalid = valid.clone();
        let row = invalid.noble_phantasms[0].components[0].overcharge[1]
            .as_mut()
            .unwrap();
        match mutation {
            0 => row.low_hp = None,
            1 => row.low_hp.as_mut().unwrap().source_base_rates[0] = 6001,
            2 => {
                row.multipliers[0] = 0.0;
                row.rates[0] = 0;
                row.low_hp.as_mut().unwrap().source_base_rates[0] = 0;
            }
            _ => row.multipliers[0] = f64::NAN,
        }
        let selection = TurnSelection::default_for(&invalid).unwrap();
        assert!(
            calculate_turn(
                &invalid,
                &selection,
                TurnBuffs::default(),
                ClassType::Saber,
                AttributeType::Earth
            )
            .is_err()
        );
    }
}
