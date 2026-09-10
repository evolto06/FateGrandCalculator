use fate_grand_calculator::damage::{CardType, DamageInput, calculate};

fn neutral_input(card_type: CardType) -> DamageInput {
    DamageInput {
        attack: 1_000,
        card_type,
        attack_buff: 0.0,
        card_buff: 0.0,
        enemy_defense: 0.0,
        class_multiplier: 1.0,
        attribute_multiplier: 1.0,
    }
}

#[test]
fn applies_every_basic_multiplier_to_buster_damage() {
    let result = calculate(DamageInput {
        attack: 1_000,
        card_type: CardType::Buster,
        attack_buff: 0.20,
        card_buff: 0.30,
        enemy_defense: 0.10,
        class_multiplier: 2.0,
        attribute_multiplier: 1.1,
    });

    assert_eq!(result.base_card_multiplier, 1.5);
    assert_eq!(result.attack_multiplier, 1.2);
    assert_eq!(result.card_buff_multiplier, 1.3);
    assert_eq!(result.defense_multiplier, 0.9);
    assert_eq!(result.class_multiplier, 2.0);
    assert_eq!(result.attribute_multiplier, 1.1);
    assert!((result.damage_before_random - 4_633.2).abs() < 0.000_001);
    assert_eq!(result.minimum_damage, 4_169);
    assert_eq!(result.maximum_damage, 5_096);
}

#[test]
fn card_types_use_distinct_base_damage_multipliers() {
    assert_eq!(
        calculate(neutral_input(CardType::Buster)).minimum_damage,
        1_350
    );
    assert_eq!(calculate(neutral_input(CardType::Arts)).minimum_damage, 900);
    assert_eq!(
        calculate(neutral_input(CardType::Quick)).minimum_damage,
        720
    );
}

#[test]
fn random_range_is_applied_after_other_modifiers() {
    let result = calculate(neutral_input(CardType::Arts));

    assert_eq!(result.damage_before_random, 1_000.0);
    assert_eq!(result.minimum_damage, 900);
    assert_eq!(result.maximum_damage, 1_100);
}

#[test]
fn excessive_defense_is_clamped_to_zero_damage() {
    let result = calculate(DamageInput {
        enemy_defense: 2.0,
        ..neutral_input(CardType::Arts)
    });

    assert_eq!(result.defense_multiplier, 0.0);
    assert_eq!(result.minimum_damage, 0);
    assert_eq!(result.maximum_damage, 0);
}
