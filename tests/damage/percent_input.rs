use fate_grand_calculator::damage::{calculate, percent_to_modifier};
use fate_grand_calculator::loader::GameData;
use fate_grand_calculator::model::{AttributeType, CardType, ClassType};

#[test]
fn whole_percent_entries_use_decimal_modifiers_in_damage_calculation() {
    let data = GameData::bundled().expect("bundled data should load");
    let altria = data.servant(100100).expect("Altria should be included");
    let input = altria.normal_card_input(
        CardType::Buster,
        percent_to_modifier(20.0),
        percent_to_modifier(0.0),
        percent_to_modifier(10.0),
        ClassType::Lancer,
        AttributeType::Sky,
    );

    let result = calculate(input);

    assert_eq!(input.attack_buff, 0.2);
    assert_eq!(input.enemy_defense, 0.1);
    assert!((result.attack_defense_multiplier - 1.1).abs() < 0.000_001);
    assert_eq!(result.minimum_damage, 9_198);
    assert_eq!(result.maximum_damage, 11_231);
}
