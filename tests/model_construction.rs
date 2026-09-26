use fate_grand_calculator::model::{
    AttributeType, CardType, ClassType, NoblePhantasmType, Servant, SkillType,
};

#[test]
fn a_loader_can_construct_a_servant_through_the_public_api() {
    let skill = || SkillType::new("Skill".into(), 1, "Attack up".into());
    let servant = Servant::new(
        "Example".into(),
        90,
        10_000,
        12_000,
        CardType::Buster,
        ClassType::Saber,
        AttributeType::Earth,
        NoblePhantasmType::new("NP".into(), 1, 3.0, CardType::Buster),
        skill(),
        skill(),
        skill(),
    );

    assert_eq!(servant.attack(), 12_000);
    assert_eq!(servant.card(), CardType::Buster);
    assert_eq!(servant.class(), ClassType::Saber);
    assert!(matches!(servant.attribute(), AttributeType::Earth));
}
