#![cfg(not(target_arch = "wasm32"))]

use fate_grand_calculator::{
    loader::{GameData, NpStatus, ServantRecord},
    servant_data::{enrich_servant, normalize_atlas_export},
};
use serde_json::{Value, json};

fn rows() -> Vec<Value> {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/np_mechanics_components.json")).unwrap();
    fixture["servants"].as_array().unwrap().clone()
}
fn import(row: &Value) -> ServantRecord {
    let mut servant = normalize_atlas_export(&json!([row]))
        .unwrap()
        .game_data
        .servants
        .remove(0);
    enrich_servant(&mut servant, row).unwrap();
    servant
}
fn single_row() -> Value {
    let mut row = rows().remove(0);
    row["noblePhantasms"] = json!([row["noblePhantasms"][0].clone()]);
    row
}

#[test]
fn imports_all_four_audited_variants_and_their_25_level_overcharge_combinations() {
    let mut variants = 0;
    for row in rows() {
        let servant = import(&row);
        assert_eq!(servant.np_status, NpStatus::Damaging);
        for source in row["noblePhantasms"].as_array().unwrap() {
            variants += 1;
            let np = servant
                .noble_phantasms
                .iter()
                .find(|np| np.id as u64 == source["id"].as_u64().unwrap())
                .unwrap();
            assert_eq!(np.available_overcharges(), [1, 2, 3, 4, 5]);
            assert_eq!(np.components.len(), 2);
            let functions: Vec<_> = source["functions"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|f| f["funcType"] == "damageNp")
                .collect();
            for oc in 1..=5 {
                let key = if oc == 1 {
                    "svals".to_owned()
                } else {
                    format!("svals{oc}")
                };
                for level in 1..=5 {
                    let mut expected = 0.0;
                    for (component, function) in np.components.iter().zip(&functions) {
                        let value = &function[&key][level as usize - 1];
                        let imported = component.overcharge[oc as usize - 1].as_ref().unwrap();
                        assert_eq!(
                            imported.multipliers[level as usize - 1],
                            value["Value"].as_f64().unwrap() / 1000.0
                        );
                        assert_eq!(
                            imported.rates[level as usize - 1],
                            value["Rate"].as_u64().unwrap() as u16
                        );
                        assert_eq!(
                            imported.check_dead[level as usize - 1],
                            value["CheckDead"].as_u64().unwrap_or(0) == 1
                        );
                        if value["Rate"] == 1000 {
                            expected += value["Value"].as_f64().unwrap() / 1000.0;
                        }
                    }
                    assert_eq!(np.base_multiplier(level, oc), Some(expected));
                }
            }
            assert!(np.notes.iter().any(|note| note.contains("not simulated")));
        }
    }
    assert_eq!(variants, 4);
}

#[test]
fn missing_or_malformed_component_oc_rows_are_unavailable_without_oc1_fallback() {
    for malformed in [None, Some(json!([{"Value":1000,"Rate":1000}]))] {
        let mut row = single_row();
        let function = row["noblePhantasms"][0]["functions"][1]
            .as_object_mut()
            .unwrap();
        match malformed {
            None => {
                function.remove("svals3");
            }
            Some(value) => {
                function.insert("svals3".into(), value);
            }
        }
        let servant = import(&row);
        let np = &servant.noble_phantasms[0];
        assert_eq!(np.available_overcharges(), [1, 2, 4, 5]);
        assert_eq!(np.base_multiplier(1, 3), None);
        assert!(
            np.components
                .iter()
                .all(|component| component.overcharge[2].is_none())
        );
    }
    let mut row = single_row();
    row["noblePhantasms"][0]["functions"][0]["svals"][0]["Value"] = json!(-1);
    assert_eq!(import(&row).np_status, NpStatus::Unavailable);
}

#[test]
fn rejects_unverified_ids_mixed_targets_intervening_effects_and_unknown_conditions() {
    for mutation in 0..7 {
        let mut row = single_row();
        match mutation {
            0 => row["noblePhantasms"][0]["id"] = json!(999999),
            1 => row["noblePhantasms"][0]["functions"][1]["funcTargetType"] = json!("enemy"),
            2 => row["noblePhantasms"][0]["functions"]
                .as_array_mut()
                .unwrap()
                .insert(1, json!({"funcType":"addState"})),
            3 => row["noblePhantasms"][0]["functions"][1]["svals"][0]["Rate"] = json!(500),
            4 => row["noblePhantasms"][0]["functions"][1]["svals"][0]["TargetIndiv"] = json!(1000),
            5 => row["noblePhantasms"][0]["functions"][1]["script"] = json!({"condition":1}),
            _ => row["noblePhantasms"][0]["functions"][1]["functvals"] = json!([1000]),
        }
        let servant = import(&row);
        assert_eq!(
            servant.np_status,
            NpStatus::Unsupported,
            "mutation {mutation}"
        );
        assert!(servant.noble_phantasms.is_empty());
    }
}

#[test]
fn legacy_np_data_migrates_to_oc1_only_and_serializes_only_components() {
    let mut data = serde_json::to_value(GameData::bundled().unwrap()).unwrap();
    for servant in data["servants"].as_array_mut().unwrap() {
        for np in servant["noble_phantasms"].as_array_mut().unwrap() {
            let multipliers = np["components"][0]["overcharge"][0]["multipliers"].clone();
            np.as_object_mut().unwrap().remove("components");
            np["multipliers"] = multipliers;
        }
    }
    let migrated = GameData::from_json(&data.to_string()).unwrap();
    for np in migrated.servants.iter().flat_map(|s| &s.noble_phantasms) {
        assert_eq!(np.available_overcharges(), [1]);
        assert_eq!(np.base_multiplier(1, 2), None);
        let serialized = serde_json::to_value(np).unwrap();
        assert!(serialized.get("multipliers").is_none());
        assert!(serialized.get("components").is_some());
    }
}

#[test]
fn malformed_new_components_never_fall_back_to_legacy_multipliers() {
    for invalid in [json!(null), json!([]), json!([{"overcharge":[]}])] {
        let mut data = serde_json::to_value(GameData::bundled().unwrap()).unwrap();
        let np = &mut data["servants"][0]["noble_phantasms"][0];
        np["multipliers"] = json!([3.0, 4.0, 4.5, 4.75, 5.0]);
        np["components"] = invalid;
        assert!(GameData::from_json(&data.to_string()).is_err());
    }
}

#[test]
fn snapshot_validation_rejects_malformed_rows_rates_and_partial_coverage() {
    for mutation in 0..5 {
        let mut data = GameData::bundled().unwrap();
        let np = &mut data.servants[0].noble_phantasms[0];
        match mutation {
            0 => np.components[0].overcharge[0].as_mut().unwrap().multipliers[0] = f64::NAN,
            1 => np.components[0].overcharge[0].as_mut().unwrap().rates[0] = 500,
            2 => np.components[0].overcharge[0].as_mut().unwrap().multipliers = [0.0; 5],
            3 => np.components[0].overcharge[0].as_mut().unwrap().rates[0] = 0,
            _ => {
                let mut second = np.components[0].clone();
                second.overcharge[2] = None;
                np.components.push(second);
            }
        }
        assert!(data.validate().is_err(), "mutation {mutation}");
    }
}

#[test]
fn unsupported_higher_oc_conditions_keep_oc1_selectable() {
    let mut row = single_row();
    row["noblePhantasms"][0]["functions"][1]["svals3"][0]["TargetIndiv"] = json!(1000);
    let servant = import(&row);
    assert_eq!(servant.np_status, NpStatus::Damaging);
    assert_eq!(
        servant.noble_phantasms[0].available_overcharges(),
        [1, 2, 4, 5]
    );
    assert_eq!(servant.noble_phantasms[0].base_multiplier(1, 3), None);
}

#[test]
fn ordinary_trait_base_np_keeps_oc1_when_higher_oc_bonus_shape_is_unknown() {
    let mut row = single_row();
    row["noblePhantasms"][0]["functions"]
        .as_array_mut()
        .unwrap()
        .truncate(1);
    let function = &mut row["noblePhantasms"][0]["functions"][0];
    function["funcType"] = json!("damageNpIndividual");
    for value in function["svals"].as_array_mut().unwrap() {
        value["Target"] = json!(2001);
        value["Correction"] = json!(1500);
    }
    function["svals3"][0]["UnknownCondition"] = json!(1);
    let servant = import(&row);
    assert_eq!(servant.np_status, NpStatus::Damaging);
    let np = &servant.noble_phantasms[0];
    assert_eq!(np.available_overcharges(), [1, 2, 4, 5]);
    assert_eq!(np.base_multiplier(1, 1), Some(6.0));
    assert!(np.notes.iter().any(|note| note.contains("Conditional")));
}
