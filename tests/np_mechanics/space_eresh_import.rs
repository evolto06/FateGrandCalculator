use fate_grand_calculator::{
    loader::{GameData, NpStatus},
    servant_data::enrich_servant,
};
use serde_json::json;

#[test]
fn imports_affection_np_values_from_atlas() {
    let mut servant = GameData::bundled().unwrap().servants.remove(0);
    servant.id = 3_300_200;
    let values: Vec<_> = [4500, 6000, 6750, 7125, 7500]
        .into_iter()
        .map(|value| json!({"Value":value,"Value2":1000,"Correction":100,"Target":3300200}))
        .collect();
    let row = json!({"id":3300200,"cards":["1","1","1","2","3"],"noblePhantasms":[
        {"id":3300201,"name":"Edin Shugurra Collapsar","card":"1","functions":[
            {"funcType":"addStateShort"},
            {"funcType":"damageNpBattlePointPhase","svals":values}
        ]}
    ]});
    enrich_servant(&mut servant, &row).unwrap();
    assert_eq!(servant.np_status, NpStatus::Damaging);
    assert_eq!(
        servant.noble_phantasms[0].components[0].overcharge[0]
            .as_ref()
            .unwrap()
            .multipliers,
        [4.5, 6.0, 6.75, 7.125, 7.5]
    );
    let scale = servant.noble_phantasms[0].affection.as_ref().unwrap();
    assert!(!servant.noble_phantasms[0].defense_pierce);
    assert!((scale.multiplier(1) - 1.1).abs() < 1e-12);
    assert!((scale.multiplier(10) - 2.0).abs() < 1e-12);
}

#[test]
fn rejects_mismatched_affection_target_without_guessing() {
    let mut servant = GameData::bundled().unwrap().servants.remove(0);
    servant.id = 3_300_200;
    let row = json!({"id":3300200,"cards":["1","1","1","2","3"],"noblePhantasms":[
        {"id":3300201,"name":"Edin Shugurra Collapsar","card":"1","functions":[
            {"funcType":"damageNpBattlePointPhase","svals":[
                {"Value":4500,"Value2":1000,"Correction":100,"Target":999},
                {"Value":6000,"Value2":1000,"Correction":100,"Target":999},
                {"Value":6750,"Value2":1000,"Correction":100,"Target":999},
                {"Value":7125,"Value2":1000,"Correction":100,"Target":999},
                {"Value":7500,"Value2":1000,"Correction":100,"Target":999}
            ]}
        ]}
    ]});
    enrich_servant(&mut servant, &row).unwrap();
    assert_eq!(servant.np_status, NpStatus::Unavailable);
}

#[test]
fn affection_overcharge_rows_keep_manual_scaling_and_reject_inconsistent_targets() {
    let mut servant = GameData::bundled().unwrap().servants.remove(0);
    servant.id = 3_300_200;
    let values: Vec<_> = [4500, 6000, 6750, 7125, 7500].into_iter()
        .map(|value| json!({"Value":value,"Rate":1000,"Value2":1000,"Correction":100,"Target":3300200})).collect();
    let mut damage =
        json!({"funcType":"damageNpBattlePointPhase","funcTargetType":"enemyAll","svals":values});
    for oc in 2..=5 {
        damage[format!("svals{oc}")] = damage["svals"].clone();
    }
    let mut row = json!({"id":3300200,"cards":["1","1","1","2","3"],"noblePhantasms":[
        {"id":3300201,"name":"Edin Shugurra Collapsar","card":"1","functions":[damage]}
    ]});
    enrich_servant(&mut servant, &row).unwrap();
    assert_eq!(
        servant.noble_phantasms[0].available_overcharges(),
        [1, 2, 3, 4, 5]
    );
    assert_eq!(servant.noble_phantasms[0].base_multiplier(3, 5), Some(6.75));
    assert!(
        servant.noble_phantasms[0]
            .notes
            .iter()
            .any(|note| note.contains("manually"))
    );
    row["noblePhantasms"][0]["functions"][0]["svals3"][0]["Target"] = json!(999);
    row["noblePhantasms"][0]["functions"][0]["svals4"][0]["Correction"] = json!(200);
    enrich_servant(&mut servant, &row).unwrap();
    assert_eq!(servant.np_status, NpStatus::Damaging);
    assert_eq!(
        servant.noble_phantasms[0].available_overcharges(),
        [1, 2, 5]
    );
}
