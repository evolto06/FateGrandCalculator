use fate_grand_calculator::{
    damage::{TurnBuffs, calculate_turn},
    loader::{AffectionScaling, GameData},
    model::{AttributeType, CardType, ClassType, SelectedCard, TurnSelection},
};

fn servant() -> fate_grand_calculator::loader::ServantRecord {
    let mut servant = GameData::bundled().unwrap().servants.remove(0);
    servant.class = ClassType::BeastEresh;
    servant.noble_phantasms[0].card_type = CardType::Arts;
    servant.noble_phantasms[0].multipliers = [4.5, 6.0, 6.75, 7.125, 7.5];
    servant.noble_phantasms[0].affection = Some(AffectionScaling {
        base: 1.0,
        per_level: 0.1,
        max_level: 10,
        ignore_defense_at: 7,
    });
    servant
}

fn damage(
    affection_level: u8,
    enemy_defense: f64,
) -> fate_grand_calculator::damage::TurnDamageResult {
    let servant = servant();
    let selection = TurnSelection {
        slots: [
            SelectedCard::NoblePhantasm(0),
            SelectedCard::Normal(0),
            SelectedCard::Normal(1),
        ],
        np_level: 1,
        affection_level,
    };
    calculate_turn(
        &servant,
        &selection,
        TurnBuffs {
            enemy_defense,
            ..Default::default()
        },
        ClassType::Saber,
        AttributeType::Earth,
    )
    .unwrap()
}

#[test]
fn affection_scales_only_np_damage() {
    let level_zero = damage(0, 0.0);
    let level_one = damage(1, 0.0);
    let level_ten = damage(10, 0.0);
    assert!(
        (level_one.cards[0].damage_before_random / level_zero.cards[0].damage_before_random - 1.1)
            .abs()
            < 1e-12
    );
    assert!(
        (level_ten.cards[0].damage_before_random / level_one.cards[0].damage_before_random
            - 2.0 / 1.1)
            .abs()
            < 1e-12
    );
    assert_eq!(
        level_one.cards[1].damage_before_random,
        level_ten.cards[1].damage_before_random
    );
}

#[test]
fn affection_seven_ignores_enemy_defense_for_np_only() {
    let level_six = damage(6, 0.2);
    let level_seven = damage(7, 0.2);
    let level_seven_no_defense = damage(7, 0.0);
    assert_eq!(
        level_seven.cards[0].damage_before_random,
        level_seven_no_defense.cards[0].damage_before_random
    );
    assert!(level_six.cards[0].damage_before_random < level_seven.cards[0].damage_before_random);
    assert!(
        level_seven.cards[1].damage_before_random
            < level_seven_no_defense.cards[1].damage_before_random
    );
    let level_seven_defense_down = damage(7, -0.2);
    assert!(
        level_seven_defense_down.cards[0].damage_before_random
            > level_seven.cards[0].damage_before_random
    );
}
