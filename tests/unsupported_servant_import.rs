use fate_grand_calculator::{model::ClassType, servant_data::normalize_atlas_export};
use serde_json::json;

#[test]
fn atlas_na_additional_servant_classes_are_kept() {
    let cases = [
        (
            1700100,
            "Solomon",
            "loreGrandCaster",
            ClassType::LoreGrandCaster,
        ),
        (9935400, "Tiamat", "beastII", ClassType::BeastII),
        (9935500, "Goetia", "beastI", ClassType::BeastI),
        (9939130, "Beast III/R", "beastIIIR", ClassType::BeastIIIR),
        (9941730, "Beast III/L", "beastIIIL", ClassType::BeastIIIL),
        (9943610, "Beast IV", "beastIV", ClassType::BeastIV),
        (3300100, "Sodom's Beast/Draco", "beast", ClassType::Beast),
        (
            9945590,
            "E-Flare Marie",
            "uOlgaMarieFlareCollection",
            ClassType::OlgaMarieFlareCollection,
        ),
        (
            9945600,
            "E-Aqua Marie",
            "uOlgaMarieAquaCollection",
            ClassType::OlgaMarieAquaCollection,
        ),
        (3300200, "Ereshkigal", "beastEresh", ClassType::BeastEresh),
    ];
    let payload = cases
        .iter()
        .map(|(id, name, class_name, _)| {
            json!({"id":id,"name":name,"className":class_name,"attribute":"beast","atkMax":10000})
        })
        .collect::<Vec<_>>();
    let report = normalize_atlas_export(&json!(payload)).unwrap();
    assert_eq!(report.skipped_rows, 0);
    assert_eq!(report.game_data.servants.len(), cases.len());
    for (id, _, _, class) in cases {
        assert_eq!(report.game_data.servant(id).unwrap().class, class);
    }
}
