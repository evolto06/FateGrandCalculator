use fate_grand_calculator::{
    damage::{TurnBuffs, calculate_turn},
    loader::GameData,
    model::{AttributeType, ClassType, SelectedCard, TurnSelection},
};

fn servant() -> fate_grand_calculator::loader::ServantRecord {
    let mut servant = GameData::bundled().unwrap().servants.remove(0);
    servant.attack = 1000;
    servant
}
fn run(
    slots: [SelectedCard; 3],
    buffs: TurnBuffs,
) -> fate_grand_calculator::damage::TurnDamageResult {
    calculate_turn(
        &servant(),
        &TurnSelection {
            slots,
            np_level: 1,
            overcharge_level: 1,
            affection_level: 1,
            attacker_hp: None,
            enemy_status: None,
            trait_bonus: None,
        },
        buffs,
        ClassType::Saber,
        AttributeType::Earth,
    )
    .unwrap()
}

#[test]
fn np_first_has_no_first_card_bonus_and_following_cards_use_their_positions() {
    use SelectedCard::*;
    let result = run(
        [NoblePhantasm(0), Normal(3), Normal(4)],
        TurnBuffs::default(),
    );
    // NP: 1000 * .23 * 1.5 * 3 * .9 = 931.5. Normal cards get +200 flat Buster chain damage.
    assert_eq!(result.cards[0].minimum_damage, 931);
    assert_eq!(result.cards[0].first_card_bonus, 0.0);
    assert_eq!(result.cards[1].minimum_damage, 676);
    assert_eq!(result.cards[2].minimum_damage, 738);
    assert_eq!(result.extra.unwrap().minimum_damage, 1086);
}

#[test]
fn np_damage_is_independent_of_position_and_receives_its_own_buff() {
    use SelectedCard::*;
    let first = run(
        [NoblePhantasm(0), Normal(3), Normal(4)],
        TurnBuffs::default(),
    );
    let last = run(
        [Normal(3), Normal(4), NoblePhantasm(0)],
        TurnBuffs::default(),
    );
    assert_eq!(first.cards[0].minimum_damage, last.cards[2].minimum_damage);
    let buffed = run(
        [NoblePhantasm(0), Normal(3), Normal(4)],
        TurnBuffs {
            np_damage_buff: 0.5,
            ..Default::default()
        },
    );
    assert_eq!(buffed.cards[0].minimum_damage, 1397);
    assert_eq!(
        buffed.cards[1].minimum_damage,
        first.cards[1].minimum_damage
    );
}

#[test]
fn np_damage_buff_cap_and_negative_floor_leave_normal_and_extra_damage_unchanged() {
    let mut altria = servant();
    altria.attack = 11_221;
    let selection = TurnSelection::default_for(&altria).unwrap();
    let calculate = |np_damage_buff| {
        calculate_turn(
            &altria,
            &selection,
            TurnBuffs {
                np_damage_buff,
                ..Default::default()
            },
            ClassType::Saber,
            AttributeType::Earth,
        )
        .unwrap()
    };
    let baseline = calculate(0.0);
    // Independent fixed endpoints: ATK11221, base Excalibur NP1/OC1,
    // 1.5 card * 3 NP, neutral class/attribute, random .900 / 1.099.
    for (buff, expected) in [
        (4.99, (62_609, 76_453)),
        (5.0, (62_714, 76_580)),
        (9.99, (62_714, 76_580)),
        (-1.0, (10, 12)),
        (-2.0, (10, 12)),
    ] {
        let result = calculate(buff);
        assert_eq!(
            (
                result.cards[0].minimum_damage,
                result.cards[0].maximum_damage
            ),
            expected,
            "NP damage buff {buff}"
        );
        for (actual, unchanged) in result.cards[1..]
            .iter()
            .chain(result.extra.iter())
            .zip(baseline.cards[1..].iter().chain(baseline.extra.iter()))
        {
            assert_eq!(actual.minimum_damage, unchanged.minimum_damage);
            assert_eq!(actual.maximum_damage, unchanged.maximum_damage);
        }
    }
}

#[test]
fn mighty_chain_grants_buster_first_bonus_and_ordinary_extra_multiplier() {
    use SelectedCard::*;
    let result = run([Normal(0), Normal(1), Normal(3)], TurnBuffs::default());
    assert_eq!(result.cards[0].minimum_damage, 269);
    assert_eq!(result.cards[1].minimum_damage, 351);
    assert_eq!(result.cards[2].minimum_damage, 538);
    assert_eq!(result.extra.unwrap().minimum_damage, 621);
    assert_eq!(result.extra.unwrap().extra_attack_multiplier, 2.0);
    assert!(
        result.cards.iter().all(|card| {
            card.flat_damage_addition == 0.0 && card.extra_attack_multiplier == 1.0
        })
    );
}

#[test]
fn mixed_color_buffs_apply_only_to_the_matching_card() {
    use SelectedCard::*;
    let base = run([Normal(0), Normal(1), Normal(3)], TurnBuffs::default());
    let buffed = run(
        [Normal(0), Normal(1), Normal(3)],
        TurnBuffs {
            arts_buff: 0.5,
            ..Default::default()
        },
    );
    assert_eq!(base.cards[0].minimum_damage, buffed.cards[0].minimum_damage);
    assert_eq!(buffed.cards[1].minimum_damage, 476);
    assert_eq!(base.cards[2].minimum_damage, buffed.cards[2].minimum_damage);
    assert_eq!(
        base.extra.unwrap().minimum_damage,
        buffed.extra.unwrap().minimum_damage
    );
}

#[test]
fn np_level_and_upgrade_select_independent_multipliers() {
    let base = servant();
    let select = TurnSelection {
        slots: [
            SelectedCard::NoblePhantasm(1),
            SelectedCard::Normal(0),
            SelectedCard::Normal(1),
        ],
        np_level: 5,
        overcharge_level: 1,
        affection_level: 1,
        attacker_hp: None,
        enemy_status: None,
        trait_bonus: None,
    };
    let result = calculate_turn(
        &base,
        &select,
        TurnBuffs::default(),
        ClassType::Saber,
        AttributeType::Earth,
    )
    .unwrap();
    assert_eq!(result.cards[0].minimum_damage, 1863);
}

#[test]
fn arts_first_without_mighty_chain_has_no_buster_bonus() {
    use SelectedCard::*;
    let result = run([Normal(1), Normal(2), Normal(0)], TurnBuffs::default());
    assert_eq!(
        result.cards.map(|card| card.minimum_damage),
        [207, 248, 231]
    );
    assert_eq!(
        result.cards.map(|card| card.maximum_damage),
        [252, 303, 283]
    );
    assert_eq!(result.extra.unwrap().minimum_damage, 414);
}

#[test]
fn buster_chain_flat_damage_is_added_after_random_and_defense() {
    use SelectedCard::*;
    let result = run(
        [NoblePhantasm(0), Normal(3), Normal(4)],
        TurnBuffs {
            enemy_defense: 2.0,
            ..Default::default()
        },
    );
    assert_eq!(result.cards[0].minimum_damage, 0);
    assert_eq!(result.cards[0].flat_damage_addition, 0.0);
    assert_eq!(result.cards[0].extra_attack_multiplier, 1.0);
    assert_eq!(result.cards[1].damage_before_random, 0.0);
    assert_eq!(result.cards[1].flat_damage_addition, 200.0);
    assert_eq!(result.cards[1].extra_attack_multiplier, 1.0);
    assert_eq!(result.cards[1].minimum_damage, 200);
    assert_eq!(result.cards[1].maximum_damage, 200);
    assert_eq!(result.extra.unwrap().minimum_damage, 0);
    assert_eq!(result.extra.unwrap().flat_damage_addition, 0.0);
    assert_eq!(result.extra.unwrap().extra_attack_multiplier, 3.5);
}
