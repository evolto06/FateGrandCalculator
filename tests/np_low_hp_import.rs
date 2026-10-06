#![cfg(not(target_arch = "wasm32"))]

use fate_grand_calculator::{
    damage::{TurnBuffs, calculate_turn},
    loader::{NpStatus, ServantRecord},
    model::{AttackerHp, AttributeType, ClassType, SelectedCard, TurnSelection},
    servant_data::{enrich_servant, normalize_atlas_export},
};
use serde_json::{Value, json};

fn rows() -> Vec<Value> {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/np_mechanics_low_hp.json")).unwrap();
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
    rows().remove(0)
}

fn damage(row: &mut Value) -> &mut Value {
    row["noblePhantasms"][0]["functions"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|f| f["funcType"] == "damageNpHpratioLow")
        .unwrap()
}

#[test]
fn imports_all_six_variants_with_explicit_source_units_and_manual_effect_notes() {
    let mut ids = Vec::new();
    for source in rows() {
        let servant = import(&source);
        assert_eq!(servant.np_status, NpStatus::Damaging);
        assert_eq!(
            servant.max_hp,
            Some(source["hpMax"].as_u64().unwrap() as u32)
        );
        for np in &servant.noble_phantasms {
            ids.push(np.id);
            assert_eq!(np.components.len(), 1);
            assert!(np.requires_attacker_hp());
            assert_eq!(np.available_overcharges(), [1, 2, 3, 4, 5]);
            assert!(
                np.notes
                    .iter()
                    .any(|note| note.contains("HP at NP damage time"))
            );
            let source_np = source["noblePhantasms"]
                .as_array()
                .unwrap()
                .iter()
                .find(|source| source["id"].as_u64() == Some(np.id as u64))
                .unwrap();
            if source_np["functions"].as_array().unwrap().len() > 1 {
                assert!(
                    np.notes
                        .iter()
                        .any(|note| note.contains("Other NP effects"))
                );
            }
            let function = source_np["functions"]
                .as_array()
                .unwrap()
                .iter()
                .find(|f| f["funcType"] == "damageNpHpratioLow")
                .unwrap();
            for oc in 1..=5 {
                let key = if oc == 1 {
                    "svals".to_owned()
                } else {
                    format!("svals{oc}")
                };
                let values = np.components[0].overcharge[oc - 1].as_ref().unwrap();
                let scaling = values.low_hp.as_ref().unwrap();
                for level in 0..5 {
                    assert_eq!(
                        scaling.source_base_rates[level] as u64,
                        function[&key][level]["Value"].as_u64().unwrap()
                    );
                    assert_eq!(
                        scaling.coefficients[level] as u64,
                        function[&key][level]["Target"].as_u64().unwrap()
                    );
                    assert_eq!(
                        values.multipliers[level],
                        scaling.source_base_rates[level] as f64 / 1000.0
                    );
                    assert_eq!(values.rates[level], 1000);
                }
            }
        }
    }
    assert_eq!(ids, [202501, 203301, 400901, 400902, 702501, 702502]);
}

#[test]
fn imported_variants_match_450_fixed_hp_rates_and_damage_ranges() {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/np_mechanics_low_hp_golden.json")).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 450);
    let mut servants: Vec<_> = rows().iter().map(import).collect();
    // The fixed fixture controls stats to isolate the mechanic and rounding.
    for servant in &mut servants {
        servant.attack = 1003;
        servant.class = ClassType::Saber;
        servant.attribute = AttributeType::Earth;
    }
    for case in cases {
        let values: Vec<_> = case
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap())
            .collect();
        let (servant, index) = servants
            .iter()
            .find_map(|s| {
                s.noble_phantasms
                    .iter()
                    .position(|np| np.id as u64 == values[0])
                    .map(|i| (s, i))
            })
            .unwrap();
        let hp = AttackerHp {
            current: values[3] as u32,
            max: values[4] as u32,
        };
        let np = &servant.noble_phantasms[index];
        assert_eq!(
            np.components[0].effective_multiplier(values[1] as u8, values[2] as u8, Some(hp)),
            Some(values[5] as f64 / 1000.0),
            "case {case}"
        );
        let selection = TurnSelection {
            slots: [
                SelectedCard::NoblePhantasm(index),
                SelectedCard::Normal(0),
                SelectedCard::Normal(1),
            ],
            np_level: values[1] as u8,
            overcharge_level: values[2] as u8,
            affection_level: 0,
            attacker_hp: Some(hp),
            enemy_status: None,
        };
        let result = calculate_turn(
            servant,
            &selection,
            TurnBuffs::default(),
            ClassType::Saber,
            AttributeType::Earth,
        )
        .unwrap();
        assert_eq!(
            (
                result.cards[0].minimum_damage,
                result.cards[0].maximum_damage
            ),
            (values[6] as u32, values[7] as u32),
            "case {case}"
        );
        assert_eq!(result.np_components[0].len(), 1);
        if hp.current == hp.max {
            assert_eq!(
                np.base_multiplier(selection.np_level, selection.overcharge_level),
                Some(values[5] as f64 / 1000.0)
            );
        }
    }
}

#[test]
fn low_hp_validation_rejects_unknown_fields_conditions_scripts_and_compound_damage() {
    for mutation in 0..14 {
        let mut row = single_row();
        match mutation {
            0 => damage(&mut row)["script"] = json!({"condition": 1}),
            1 => row["noblePhantasms"][0]["script"] = json!({"condition": 1}),
            2 => damage(&mut row)["functvals"] = json!([1000]),
            3 => damage(&mut row)["funcquestTvals"] = json!([1000]),
            4 => damage(&mut row)["overWriteTvalsList"] = json!([1000]),
            5 => damage(&mut row)["funcGroup"] = json!([1000]),
            6 => damage(&mut row)["traitVals"] = json!([1000]),
            7 => damage(&mut row)["buffs"] = json!([{}]),
            8 => damage(&mut row)["funcTargetType"] = json!("self"),
            9 => {
                damage(&mut row)
                    .as_object_mut()
                    .unwrap()
                    .remove("funcTargetType");
            }
            10 => damage(&mut row)["funcTargetTeam"] = json!("player"),
            11 => damage(&mut row)["unknownCondition"] = json!(1),
            12 => damage(&mut row)["svals"][0]["TargetIndiv"] = json!(1000),
            _ => {
                let extra = damage(&mut row).clone();
                row["noblePhantasms"][0]["functions"]
                    .as_array_mut()
                    .unwrap()
                    .push(extra);
            }
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
fn invalid_source_integer_units_are_unavailable_and_random_activation_is_unsupported() {
    for key in ["Value", "Target"] {
        for invalid in [
            json!(-1),
            json!(1.5),
            json!("6000"),
            json!(null),
            json!(true),
            json!(4294967296_u64),
        ] {
            let mut row = single_row();
            damage(&mut row)["svals"][0][key] = invalid;
            assert_eq!(import(&row).np_status, NpStatus::Unavailable, "field {key}");
        }
        let mut row = single_row();
        damage(&mut row)["svals"][0]
            .as_object_mut()
            .unwrap()
            .remove(key);
        assert_eq!(import(&row).np_status, NpStatus::Unavailable);
    }
    for rate in [json!(0), json!(500), json!(null), json!(1000.0)] {
        let mut row = single_row();
        damage(&mut row)["svals"][0]["Rate"] = rate;
        assert_eq!(import(&row).np_status, NpStatus::Unsupported);
    }
    let mut row = single_row();
    damage(&mut row)["svals"][0]
        .as_object_mut()
        .unwrap()
        .remove("Rate");
    assert_eq!(import(&row).np_status, NpStatus::Unsupported);
}

#[test]
fn missing_or_bad_higher_oc_rows_stay_unavailable_without_invention() {
    for mutation in 0..5 {
        let mut row = single_row();
        match mutation {
            0 => {
                damage(&mut row).as_object_mut().unwrap().remove("svals3");
            }
            1 => damage(&mut row)["svals3"] = json!([]),
            2 => damage(&mut row)["svals3"][0]["Target"] = json!(-1),
            3 => damage(&mut row)["svals3"][0]["Rate"] = json!(500),
            _ => damage(&mut row)["svals3"][0]["Unknown"] = json!(1),
        }
        let servant = import(&row);
        assert_eq!(servant.np_status, NpStatus::Damaging);
        let np = &servant.noble_phantasms[0];
        assert_eq!(np.available_overcharges(), [1, 2, 4, 5]);
        assert_eq!(
            np.components[0].effective_multiplier(
                1,
                3,
                Some(AttackerHp {
                    current: 1,
                    max: 10000
                })
            ),
            None
        );
        assert!(np.notes.iter().any(|note| note.contains("never inferred")));
    }
}

#[test]
fn max_hp_is_optional_for_legacy_data_and_positive_integer_when_present() {
    let mut missing = single_row();
    missing.as_object_mut().unwrap().remove("hpMax");
    assert_eq!(import(&missing).max_hp, None);
    for invalid in [
        json!(0),
        json!(-1),
        json!(1.5),
        json!("11521"),
        json!(4294967296_u64),
    ] {
        let mut row = single_row();
        row["hpMax"] = invalid;
        assert!(normalize_atlas_export(&json!([row])).is_err());
        let mut servant = import(&missing);
        assert!(enrich_servant(&mut servant, &row).is_err());
    }
}

#[test]
fn low_hp_import_uses_function_contract_instead_of_a_servant_allowlist() {
    let mut row = single_row();
    row["id"] = json!(999000);
    row["noblePhantasms"][0]["id"] = json!(999001);
    let servant = import(&row);
    assert_eq!(servant.np_status, NpStatus::Damaging);
    assert!(servant.noble_phantasms[0].requires_attacker_hp());
}
