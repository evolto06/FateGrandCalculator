use fate_grand_calculator::damage::calculate;
use fate_grand_calculator::loader::GameData;
use fate_grand_calculator::model::{AttributeType, CardType, ClassType};

#[test]
fn bundled_servant_data_drives_the_expected_damage_result() {
    let data = GameData::bundled().expect("bundled data should load");
    let altria = data.servant(100100).expect("Altria should be included");
    let input = altria.normal_card_input(
        CardType::Buster,
        0.0,
        0.0,
        0.0,
        ClassType::Lancer,
        AttributeType::Sky,
    );

    let result = calculate(input);

    assert_eq!(input.attack, 11_221);
    assert_eq!(result.class_multiplier, 2.0);
    assert_eq!(result.attribute_multiplier, 0.9);
    assert_eq!(result.minimum_damage, 27_267);
    assert_eq!(result.maximum_damage, 33_326);
}
