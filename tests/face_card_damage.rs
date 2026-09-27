use fate_grand_calculator::damage::{DamageInput, calculate};
use fate_grand_calculator::model::{CardType, ClassType};

fn neutral_input(card_type: CardType) -> DamageInput {
    DamageInput {
        attack: 1_000,
        card_type,
        attacker_class: ClassType::Saber,
        attack_buff: 0.0,
        card_buff: 0.0,
        enemy_defense: 0.0,
        class_multiplier: 1.0,
        attribute_multiplier: 1.0,
    }
}

#[test]
fn applies_all_supplied_modifiers_to_buster_damage() {
    let result = calculate(DamageInput {
        attack: 1_000,
        card_type: CardType::Buster,
        attacker_class: ClassType::Saber,
        attack_buff: 0.20,
        card_buff: 0.30,
        enemy_defense: 0.10,
        class_multiplier: 2.0,
        attribute_multiplier: 1.1,
    });

    assert_eq!(result.base_card_multiplier, 1.5);
    assert_eq!(result.first_card_bonus, 0.5);
    assert_eq!(result.card_buff_multiplier, 1.3);
    assert!((result.card_damage_multiplier - 2.45).abs() < 0.000_001);
    assert_eq!(result.class_attack_multiplier, 1.0);
    assert!((result.attack_defense_multiplier - 1.1).abs() < 0.000_001);
    assert_eq!(result.class_multiplier, 2.0);
    assert_eq!(result.attribute_multiplier, 1.1);
    assert!((result.damage_before_random - 1_363.67).abs() < 0.000_001);
    assert_eq!(result.minimum_damage, 1_227);
    assert_eq!(result.maximum_damage, 1_498);
}

#[test]
fn card_types_use_distinct_base_damage_multipliers() {
    assert_eq!(
        calculate(neutral_input(CardType::Buster)).minimum_damage,
        414
    );
    assert_eq!(calculate(neutral_input(CardType::Arts)).minimum_damage, 207);
    assert_eq!(
        calculate(neutral_input(CardType::Quick)).minimum_damage,
        165
    );
}

#[test]
fn random_range_is_applied_after_other_modifiers() {
    let result = calculate(neutral_input(CardType::Arts));

    assert_eq!(result.damage_before_random, 230.0);
    assert_eq!(result.minimum_damage, 207);
    assert_eq!(result.maximum_damage, 252);
}

#[test]
fn excessive_enemy_defense_clamps_damage_to_zero() {
    let result = calculate(DamageInput {
        enemy_defense: 2.0,
        ..neutral_input(CardType::Arts)
    });

    assert_eq!(result.attack_defense_multiplier, 0.0);
    assert_eq!(result.minimum_damage, 0);
    assert_eq!(result.maximum_damage, 0);
}

#[test]
fn class_attack_rate_changes_the_first_card_result() {
    let result = calculate(DamageInput {
        attacker_class: ClassType::Archer,
        ..neutral_input(CardType::Arts)
    });

    assert_eq!(result.class_attack_multiplier, 0.95);
    assert_eq!(result.minimum_damage, 196);
    assert_eq!(result.maximum_damage, 240);
}

#[test]
fn card_buff_does_not_amplify_the_first_buster_bonus() {
    let result = calculate(DamageInput {
        card_buff: 0.5,
        ..neutral_input(CardType::Buster)
    });

    assert_eq!(result.first_card_bonus, 0.5);
    assert_eq!(result.card_damage_multiplier, 2.75);
    assert_eq!(result.damage_before_random, 632.5);
}
