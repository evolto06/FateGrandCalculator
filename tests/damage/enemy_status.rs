use fate_grand_calculator::{
    damage::{TurnBuffs, calculate_turn},
    loader::GameData,
    model::{AttributeType, CardType, ClassType, SelectedCard, TurnSelection},
    np_mechanics::{
        components::{NpDamageComponent, NpTarget, components_are_valid},
        enemy_status::{EnemyStatus, EnemyStatusScaling},
    },
};

fn setup(condition: EnemyStatus) -> (fate_grand_calculator::loader::ServantRecord, TurnSelection) {
    let mut servant = GameData::bundled().unwrap().servants.remove(0);
    servant.attack = 10000;
    servant.class = ClassType::Saber;
    servant.attribute = AttributeType::Man;
    servant.noble_phantasms.truncate(1);
    servant.noble_phantasms[0].card_type = CardType::Arts;
    servant.noble_phantasms[0].affection = None;
    servant.noble_phantasms[0].defense_pierce = false;
    let mut component = NpDamageComponent::legacy([6.0; 5]);
    component.overcharge[0].as_mut().unwrap().enemy_status = Some(EnemyStatusScaling {
        condition,
        source_corrections: [2000; 5],
        include_ignore_individuality: condition.includes_ignored_individuality(),
    });
    servant.noble_phantasms[0].components = vec![component];
    let selection = TurnSelection::default_for(&servant).unwrap();
    (servant, selection)
}

fn calculate(
    servant: &fate_grand_calculator::loader::ServantRecord,
    selection: &TurnSelection,
    buffs: TurnBuffs,
) -> Result<fate_grand_calculator::damage::TurnDamageResult, String> {
    calculate_turn(
        servant,
        selection,
        buffs,
        ClassType::Saber,
        AttributeType::Man,
    )
}

#[test]
fn absent_present_and_wrong_typed_condition_have_explicit_outcomes() {
    let (servant, mut selection) = setup(EnemyStatus::Poison);
    assert_eq!(selection.enemy_status, None);
    let absent = calculate(&servant, &selection, TurnBuffs::default()).unwrap();
    assert_eq!(
        (
            absent.cards[0].minimum_damage,
            absent.cards[0].maximum_damage
        ),
        (12420, 15166)
    );
    selection.enemy_status = Some(EnemyStatus::Poison);
    let present = calculate(&servant, &selection, TurnBuffs::default()).unwrap();
    assert_eq!(
        (
            present.cards[0].minimum_damage,
            present.cards[0].maximum_damage
        ),
        (24840, 30332)
    );
    for position in [1, 2] {
        assert_eq!(
            present.cards[position].minimum_damage,
            absent.cards[position].minimum_damage
        );
        assert_eq!(
            present.cards[position].maximum_damage,
            absent.cards[position].maximum_damage
        );
    }
    assert_eq!(
        present.extra.unwrap().minimum_damage,
        absent.extra.unwrap().minimum_damage
    );
    assert_eq!(
        present.extra.unwrap().maximum_damage,
        absent.extra.unwrap().maximum_damage
    );
    selection.enemy_status = Some(EnemyStatus::Curse);
    assert!(
        calculate(&servant, &selection, TurnBuffs::default())
            .unwrap_err()
            .contains("does not match")
    );
    selection.slots = [
        SelectedCard::Normal(0),
        SelectedCard::Normal(1),
        SelectedCard::Normal(2),
    ];
    let unused = calculate(&servant, &selection, TurnBuffs::default()).unwrap();
    selection.enemy_status = None;
    let clean = calculate(&servant, &selection, TurnBuffs::default()).unwrap();
    assert_eq!(
        unused.cards[0].minimum_damage,
        clean.cards[0].minimum_damage
    );
    assert_eq!(
        unused.extra.unwrap().maximum_damage,
        clean.extra.unwrap().maximum_damage
    );
}

#[test]
fn defense_status_is_independent_of_numeric_defense_and_piercing() {
    let (mut servant, mut selection) = setup(EnemyStatus::DefenseUp);
    let buffs = TurnBuffs {
        enemy_defense: 0.2,
        ..TurnBuffs::default()
    };
    let absent = calculate(&servant, &selection, buffs).unwrap();
    assert_eq!(
        (
            absent.cards[0].minimum_damage,
            absent.cards[0].maximum_damage
        ),
        (9936, 12132)
    );
    selection.enemy_status = Some(EnemyStatus::DefenseUp);
    let present = calculate(&servant, &selection, buffs).unwrap();
    assert_eq!(
        (
            present.cards[0].minimum_damage,
            present.cards[0].maximum_damage
        ),
        (19872, 24265)
    );
    assert_eq!(present.cards[0].attack_defense_multiplier, 0.8);
    assert_eq!(
        present.cards[1].minimum_damage,
        absent.cards[1].minimum_damage
    );
    servant.noble_phantasms[0].defense_pierce = true;
    let piercing = calculate(&servant, &selection, buffs).unwrap();
    assert_eq!(
        (
            piercing.cards[0].minimum_damage,
            piercing.cards[0].maximum_damage
        ),
        (24840, 30332)
    );
    assert_eq!(piercing.cards[0].attack_defense_multiplier, 1.0);
    assert_eq!(
        piercing.cards[1].minimum_damage,
        absent.cards[1].minimum_damage
    );
}

#[test]
fn only_affected_component_scales_before_separate_rounding() {
    let (mut servant, mut selection) = setup(EnemyStatus::Burn);
    servant.attack = 123;
    let mut ordinary = NpDamageComponent::legacy([0.7; 5]);
    ordinary.target = Some(NpTarget::Enemy);
    let affected = &mut servant.noble_phantasms[0].components[0];
    affected.target = Some(NpTarget::Enemy);
    affected.overcharge[0].as_mut().unwrap().multipliers = [0.7; 5];
    servant.noble_phantasms[0].components.push(ordinary);
    selection.enemy_status = Some(EnemyStatus::Burn);
    let result = calculate(&servant, &selection, TurnBuffs::default()).unwrap();
    // Independent endpoints: floor(123*.23*.7*2*.9)=35 and ordinary=17;
    // flooring their combined rate instead would incorrectly return 53.
    assert_eq!(result.np_components[0][0].minimum_damage, 35);
    assert_eq!(result.np_components[0][1].minimum_damage, 17);
    assert_eq!(result.cards[0].minimum_damage, 52);
    assert_eq!(result.cards[0].maximum_damage, 64);
}

#[test]
fn malformed_runtime_rows_and_unavailable_oc_return_errors_without_panicking() {
    let (servant, mut selection) = setup(EnemyStatus::SkillSeal);
    selection.enemy_status = Some(EnemyStatus::SkillSeal);
    for invalid in [
        EnemyStatusScaling {
            condition: EnemyStatus::SkillSeal,
            source_corrections: [0; 5],
            include_ignore_individuality: true,
        },
        EnemyStatusScaling {
            condition: EnemyStatus::SkillSeal,
            source_corrections: [2000; 5],
            include_ignore_individuality: false,
        },
    ] {
        let mut malformed = servant.clone();
        malformed.noble_phantasms[0].components[0].overcharge[0]
            .as_mut()
            .unwrap()
            .enemy_status = Some(invalid);
        assert!(calculate(&malformed, &selection, TurnBuffs::default()).is_err());
    }
    selection.overcharge_level = 2;
    assert!(
        calculate(&servant, &selection, TurnBuffs::default())
            .unwrap_err()
            .contains("no data")
    );
    let component = &servant.noble_phantasms[0].components[0];
    assert!(component.enemy_status_breakdown(0, 1, None).is_none());
    assert!(component.enemy_status_breakdown(6, 1, None).is_none());
    assert!(component.enemy_status_breakdown(1, 6, None).is_none());
    let mut changed = component.clone();
    changed.overcharge[1] = changed.overcharge[0].clone();
    let row = changed.overcharge[1].as_mut().unwrap();
    row.enemy_status.as_mut().unwrap().condition = EnemyStatus::Charm;
    assert!(!components_are_valid(&[changed]));
    let mut other = component.clone();
    other.target = Some(NpTarget::Enemy);
    other.overcharge[0]
        .as_mut()
        .unwrap()
        .enemy_status
        .as_mut()
        .unwrap()
        .condition = EnemyStatus::Charm;
    let mut first = component.clone();
    first.target = Some(NpTarget::Enemy);
    assert!(!components_are_valid(&[first, other]));
    let mut contradictory = component.clone();
    contradictory.overcharge[0].as_mut().unwrap().low_hp =
        Some(fate_grand_calculator::np_mechanics::low_hp::LowHpScaling {
            source_base_rates: [6000; 5],
            coefficients: [6000; 5],
        });
    assert!(!components_are_valid(&[contradictory]));
}
