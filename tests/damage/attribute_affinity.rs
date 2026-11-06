use fate_grand_calculator::model::AttributeType;

#[test]
fn attribute_affinity_matches_the_game_relation_table() {
    assert_eq!(AttributeType::Man.affinity_against(AttributeType::Sky), 1.1);
    assert_eq!(
        AttributeType::Earth.affinity_against(AttributeType::Sky),
        0.9
    );
    assert_eq!(
        AttributeType::Star.affinity_against(AttributeType::Beast),
        1.1
    );
    assert_eq!(
        AttributeType::Sky.affinity_against(AttributeType::Star),
        1.0
    );
}
