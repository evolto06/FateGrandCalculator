use fate_grand_calculator::{
    loader::{GameData, NpStatus},
    servant_data::enrich_servant,
};
use serde_json::json;

fn fixture() -> serde_json::Value {
    // Representative Atlas nice-servant fields for Altria.
    json!({"id":100100,"cards":["3","1","1","2","2"],"noblePhantasms":[
        {"id":100101,"name":"Excalibur","card":"2","strengthStatus":1,"functions":[
            {"funcType":"damageNp","svals":[{"Value":3000},{"Value":4000},{"Value":4500},{"Value":4750},{"Value":5000}]}
        ]},
        {"id":100102,"name":"Excalibur","card":"2","strengthStatus":99,"functions":[
            {"funcType":"damageNp","svals":[{"Value":4000},{"Value":5000},{"Value":5500},{"Value":5750},{"Value":6000}]}
        ]}
    ]})
}
#[test]
fn imports_physical_deck_and_base_and_strengthened_np_variants() {
    let mut servant = GameData::bundled().unwrap().servants.remove(0);
    enrich_servant(&mut servant, &fixture()).unwrap();
    assert_eq!(servant.deck.len(), 5);
    assert_eq!(
        servant.noble_phantasms[0].multipliers,
        [3., 4., 4.5, 4.75, 5.]
    );
    assert_eq!(
        servant.noble_phantasms[1].multipliers,
        [4., 5., 5.5, 5.75, 6.]
    );
}
#[test]
fn support_missing_and_unsupported_nps_are_distinct() {
    let mut servant = GameData::bundled().unwrap().servants.remove(0);
    let mut source = fixture();
    source["noblePhantasms"] = json!([{"functions":[{"funcType":"addState"}]}]);
    enrich_servant(&mut servant, &source).unwrap();
    assert_eq!(servant.np_status, NpStatus::Support);
    source["noblePhantasms"] = json!([]);
    enrich_servant(&mut servant, &source).unwrap();
    assert_eq!(servant.np_status, NpStatus::Unavailable);
    source["noblePhantasms"] = json!([{"functions":[{"funcType":"damageNpHpratioHigh"}]}]);
    enrich_servant(&mut servant, &source).unwrap();
    assert_eq!(servant.np_status, NpStatus::Unsupported);
}
#[test]
fn conditional_and_pre_damage_effects_are_explicit() {
    let mut servant = GameData::bundled().unwrap().servants.remove(0);
    let mut source = fixture();
    source["noblePhantasms"][0]["functions"][0]["funcType"] = json!("damageNpIndividual");
    source["noblePhantasms"][0]["functions"]
        .as_array_mut()
        .unwrap()
        .insert(0, json!({"funcType":"addState"}));
    enrich_servant(&mut servant, &source).unwrap();
    assert!(
        servant.noble_phantasms[0]
            .notes
            .iter()
            .any(|n| n.contains("Conditional"))
    );
    assert!(
        servant.noble_phantasms[0]
            .notes
            .iter()
            .any(|n| n.contains("before damage"))
    );
}
#[test]
fn v1_snapshot_stays_readable_with_explicit_missing_deck() {
    let mut old = serde_json::to_value(GameData::bundled().unwrap()).unwrap();
    for servant in old["servants"].as_array_mut().unwrap() {
        for key in ["deck", "noble_phantasms", "np_status"] {
            servant.as_object_mut().unwrap().remove(key);
        }
    }
    let migrated = GameData::from_json(&old.to_string()).unwrap();
    assert!(migrated.servants[0].deck.is_empty());
    assert_eq!(migrated.servants[0].np_status, NpStatus::Unavailable);
}
#[test]
fn wrong_servant_and_incomplete_decks_reject_updates() {
    let mut servant = GameData::bundled().unwrap().servants.remove(0);
    let mut source = fixture();
    source["id"] = json!(999);
    assert!(enrich_servant(&mut servant, &source).is_err());
    source["id"] = json!(100100);
    source["cards"] = json!(["1", "2"]);
    assert!(enrich_servant(&mut servant, &source).is_err());
}
