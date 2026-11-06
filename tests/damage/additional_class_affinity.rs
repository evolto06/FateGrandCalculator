use fate_grand_calculator::model::ClassType;

#[test]
fn additional_servant_classes_use_distinct_atlas_na_affinities() {
    use ClassType::*;
    let cases = [
        (Beast, Saber, 1.5),
        (BeastEresh, Ruler, 1.5),
        (BeastI, Archer, 2.0),
        (BeastII, Saber, 1.0),
        (BeastIIIL, Saber, 1.0),
        (BeastIIIR, Saber, 1.0),
        (BeastIV, Caster, 0.5),
        (LoreGrandCaster, Assassin, 2.0),
        (OlgaMarieFlareCollection, Shielder, 0.5),
        (OlgaMarieAquaCollection, Archer, 1.5),
    ];
    for (attacker, target, expected) in cases {
        assert_eq!(attacker.affinity_against(target), expected);
    }
    assert_eq!(LoreGrandCaster.class_default_multiplier(), 0.9);
}
