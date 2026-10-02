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
fn mighty_chain_grants_buster_first_bonus_and_ordinary_extra_multiplier() {
    use SelectedCard::*;
    let result = run([Normal(0), Normal(1), Normal(3)], TurnBuffs::default());
    assert_eq!(result.cards[0].minimum_damage, 269);
    assert_eq!(result.cards[1].minimum_damage, 351);
    assert_eq!(result.cards[2].minimum_damage, 538);
    assert_eq!(result.extra.unwrap().minimum_damage, 621);
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
    assert_eq!(result.cards[1].minimum_damage, 200);
    assert_eq!(result.cards[1].maximum_damage, 200);
    assert_eq!(result.extra.unwrap().minimum_damage, 0);
}
