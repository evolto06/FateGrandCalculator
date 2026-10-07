use fate_grand_calculator::{
    damage::{TurnBuffs, calculate_turn},
    loader::GameData,
    model::{AttributeType, CardType, ClassType, SelectedCard, TurnSelection},
    np_mechanics::{
        components::{NpDamageComponent, NpTarget, components_are_valid},
        trait_bonus::{TraitBonusScaling, TraitCondition},
    },
};

fn setup() -> (fate_grand_calculator::loader::ServantRecord, TurnSelection) {
    let mut servant = GameData::bundled().unwrap().servants.remove(0);
    servant.attack = 1003;
    servant.class = ClassType::Saber;
    servant.attribute = AttributeType::Earth;
    servant.noble_phantasms.truncate(1);
    let np = &mut servant.noble_phantasms[0];
    np.card_type = CardType::Arts;
    np.affection = None;
    np.defense_pierce = false;
    let mut component = NpDamageComponent::legacy([6., 7., 8., 9., 10.]);
    for (oc, corrections) in [
        (0, [1500, 1600, 1700, 1800, 1900]),
        (4, [2500, 2600, 2700, 2800, 2900]),
    ] {
        let mut row = component.overcharge[0].clone().unwrap();
        row.trait_bonus = Some(TraitBonusScaling {
            condition: TraitCondition::from_source_target(2002).unwrap(),
            source_corrections: corrections,
        });
        component.overcharge[oc] = Some(row);
    }
    np.components = vec![component];
    let selection = TurnSelection::default_for(&servant).unwrap();
    (servant, selection)
}

fn run(
    servant: &fate_grand_calculator::loader::ServantRecord,
    selection: &TurnSelection,
    buffs: TurnBuffs,
) -> Result<fate_grand_calculator::damage::TurnDamageResult, String> {
    calculate_turn(
        servant,
        selection,
        buffs,
        ClassType::Saber,
        AttributeType::Earth,
    )
}

#[test]
fn fixed_endpoints_keep_np_levels_oc_and_additive_buffs_independent() {
    let (servant, mut selection) = setup();
    assert_eq!(selection.trait_bonus, None);
    let baseline = run(&servant, &selection, TurnBuffs::default()).unwrap();
    assert_eq!(
        (
            baseline.cards[0].minimum_damage,
            baseline.cards[0].maximum_damage
        ),
        (1245, 1521)
    );
    selection.trait_bonus = servant.noble_phantasms[0].trait_bonus_condition();
    // Independent decimal endpoints: floor(1003*.23*base*Correction/1000*random).
    for (level, oc, minimum, maximum) in [
        (1, 1, 1868, 2281),
        (5, 1, 3944, 4817),
        (1, 5, 3114, 3802),
        (5, 5, 6021, 7352),
    ] {
        selection.np_level = level;
        selection.overcharge_level = oc;
        let result = run(&servant, &selection, TurnBuffs::default()).unwrap();
        assert_eq!(
            (
                result.cards[0].minimum_damage,
                result.cards[0].maximum_damage
            ),
            (minimum, maximum)
        );
        for slot in [1, 2] {
            assert_eq!(
                result.cards[slot].minimum_damage,
                baseline.cards[slot].minimum_damage
            );
        }
        assert_eq!(
            result.extra.unwrap().maximum_damage,
            baseline.extra.unwrap().maximum_damage
        );
    }
    selection.np_level = 1;
    selection.overcharge_level = 1;
    let result = run(
        &servant,
        &selection,
        TurnBuffs {
            attack_buff: 0.35,
            enemy_defense: 0.2,
            arts_buff: 0.4,
            np_damage_buff: 0.3,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        (
            result.cards[0].minimum_damage,
            result.cards[0].maximum_damage
        ),
        (3910, 4775)
    );
    let breakdown = servant.noble_phantasms[0].components[0]
        .trait_bonus_breakdown(1, 1, selection.trait_bonus)
        .unwrap();
    assert_eq!(
        (
            breakdown.base_multiplier,
            breakdown.applied_multiplier,
            breakdown.effective_multiplier
        ),
        (6., 1.5, 9.)
    );
}

#[test]
fn owning_component_correction_is_applied_once_before_its_endpoint_floor() {
    let (mut servant, mut selection) = setup();
    servant.attack = 123;
    let affected = &mut servant.noble_phantasms[0].components[0];
    affected.target = Some(NpTarget::Enemy);
    affected.overcharge[4] = None;
    affected.overcharge[0].as_mut().unwrap().multipliers = [0.7; 5];
    let mut ordinary = NpDamageComponent::legacy([0.7; 5]);
    ordinary.target = Some(NpTarget::Enemy);
    servant.noble_phantasms[0].components.push(ordinary);
    selection.trait_bonus = servant.noble_phantasms[0].trait_bonus_condition();
    let result = run(&servant, &selection, TurnBuffs::default()).unwrap();
    assert_eq!(
        (
            result.np_components[0][0].minimum_damage,
            result.np_components[0][1].minimum_damage
        ),
        (26, 17)
    );
    assert_eq!(
        (
            result.cards[0].minimum_damage,
            result.cards[0].maximum_damage
        ),
        (43, 53)
    );
}

#[test]
fn invalid_metadata_and_stale_assumptions_cannot_activate() {
    let (servant, mut selection) = setup();
    assert!(TraitCondition::from_source_target(-2002).is_none());
    assert!(TraitCondition::from_source_target(0).is_none());
    for mutation in 0..4 {
        let mut invalid = servant.clone();
        let row = invalid.noble_phantasms[0].components[0].overcharge[0]
            .as_mut()
            .unwrap();
        match mutation {
            0 => row.trait_bonus.as_mut().unwrap().source_corrections[2] = 0,
            1 => row.trait_bonus.as_mut().unwrap().condition.source_target = -2002,
            2 => {
                row.enemy_status = Some(
                    fate_grand_calculator::np_mechanics::enemy_status::EnemyStatusScaling {
                        condition:
                            fate_grand_calculator::np_mechanics::enemy_status::EnemyStatus::Poison,
                        source_corrections: [2000; 5],
                        include_ignore_individuality: false,
                    },
                )
            }
            _ => row.trait_bonus.as_mut().unwrap().condition.source_target = 2010,
        }
        assert!(!components_are_valid(
            &invalid.noble_phantasms[0].components
        ));
        assert!(run(&invalid, &selection, TurnBuffs::default()).is_err());
    }
    selection.trait_bonus = TraitCondition::from_source_target(2010);
    assert!(
        run(&servant, &selection, TurnBuffs::default())
            .unwrap_err()
            .contains("does not match")
    );
    selection.slots = [
        SelectedCard::Normal(0),
        SelectedCard::Normal(1),
        SelectedCard::Normal(2),
    ];
    let unused = run(&servant, &selection, TurnBuffs::default()).unwrap();
    selection.trait_bonus = None;
    assert_eq!(
        unused.cards[0].minimum_damage,
        run(&servant, &selection, TurnBuffs::default())
            .unwrap()
            .cards[0]
            .minimum_damage
    );
    let component = &servant.noble_phantasms[0].components[0];
    for (level, oc) in [(0, 1), (6, 1), (1, 0), (1, 2), (1, 6)] {
        assert!(component.trait_bonus_breakdown(level, oc, None).is_none());
    }
}
