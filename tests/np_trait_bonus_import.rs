#![cfg(not(target_arch = "wasm32"))]

use fate_grand_calculator::{
    damage::{TurnBuffs, calculate_turn},
    loader::{NpStatus, ServantRecord},
    model::{AttributeType, ClassType, SelectedCard, TurnSelection},
    servant_data::{enrich_servant, normalize_atlas_export},
};
use serde_json::{Value, json};

fn rows() -> Vec<Value> {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/np_mechanics_trait_bonus.json")).unwrap();
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
fn damage(np: &Value) -> &Value {
    np["functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|function| function["funcType"] == "damageNpIndividual")
        .unwrap()
}
fn assert_source_rows(np: &fate_grand_calculator::loader::NoblePhantasmRecord, source: &Value) {
    assert_eq!(np.available_overcharges(), [1, 2, 3, 4, 5]);
    for oc in 0..5 {
        let key = if oc == 0 {
            "svals".to_owned()
        } else {
            format!("svals{}", oc + 1)
        };
        let row = np.components[0].overcharge[oc].as_ref().unwrap();
        let scaling = row.trait_bonus.as_ref().unwrap();
        for level in 0..5 {
            assert_eq!(
                scaling.condition.source_target as i64,
                source[&key][level]["Target"].as_i64().unwrap()
            );
            assert_eq!(
                scaling.source_corrections[level] as u64,
                source[&key][level]["Correction"].as_u64().unwrap()
            );
            assert_eq!(
                row.multipliers[level],
                source[&key][level]["Value"].as_u64().unwrap() as f64 / 1000.
            );
        }
    }
}

#[test]
fn representative_base_upgraded_and_np_level_scaled_rows_preserve_source_units() {
    for row in rows() {
        let servant = import(&row);
        assert_eq!(servant.np_status, NpStatus::Damaging);
        assert_eq!(
            servant.noble_phantasms.len(),
            row["noblePhantasms"].as_array().unwrap().len()
        );
        for np in &servant.noble_phantasms {
            let source = row["noblePhantasms"]
                .as_array()
                .unwrap()
                .iter()
                .find(|source| source["id"].as_u64() == Some(np.id as u64))
                .unwrap();
            assert_source_rows(np, damage(source));
            assert!(
                np.notes
                    .iter()
                    .any(|note| note.contains("trait at NP damage time"))
            );
            assert!(
                !np.notes
                    .iter()
                    .any(|note| note.contains("this is base damage"))
            );
        }
    }
}

#[test]
fn fixed_primary_source_cases_match_independent_decimal_endpoints() {
    let servants: Vec<_> = rows().iter().map(import).collect();
    // floor(1003*.23*sourceCardRate*Value/1000*Correction/1000*random).
    for (id, level, oc, active, minimum, maximum) in [
        (100801, 1, 1, false, 934, 1140),
        (100801, 1, 1, true, 1401, 1711),
        (100802, 5, 5, true, 3737, 4563),
        (100901, 1, 5, true, 2055, 2509),
        (100902, 5, 1, true, 3363, 4107),
        (201201, 1, 5, true, 2802, 3422),
        (201201, 5, 1, true, 4671, 5704),
        (201202, 1, 1, true, 3737, 4563),
        (201202, 5, 5, true, 6228, 7605),
        (203501, 1, 1, true, 2989, 3650),
        (203501, 5, 5, true, 6643, 8112),
    ] {
        let mut servant = servants
            .iter()
            .find(|servant| servant.noble_phantasms.iter().any(|np| np.id == id))
            .unwrap()
            .clone();
        servant.attack = 1003;
        servant.class = ClassType::Saber;
        servant.attribute = AttributeType::Earth;
        let index = servant
            .noble_phantasms
            .iter()
            .position(|np| np.id == id)
            .unwrap();
        let mut selection = TurnSelection::default_for(&servant).unwrap();
        selection.slots[0] = SelectedCard::NoblePhantasm(index);
        selection.np_level = level;
        selection.overcharge_level = oc;
        selection.trait_bonus = active.then(|| {
            servant.noble_phantasms[index]
                .trait_bonus_condition()
                .unwrap()
        });
        let result = calculate_turn(
            &servant,
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
            (minimum, maximum),
            "NP{id}/NP{level}/OC{oc}"
        );
    }
}

#[test]
fn unsupported_conditional_metadata_retains_only_preexisting_valid_base_data() {
    for mutation in 0..16 {
        let mut row = rows().remove(0);
        row["noblePhantasms"] = json!([row["noblePhantasms"][0].clone()]);
        let function = &mut row["noblePhantasms"][0]["functions"][0];
        match mutation {
            0 => function["svals"][0]["Target"] = json!(-2002),
            1 => function["svals"][0]["Target"] = json!(0),
            2 => function["svals"][0]["Target"] = json!(999999),
            3 => function["svals"][0]["Correction"] = json!(0),
            4 => function["svals"][0]["Correction"] = json!(-1),
            5 => function["svals"][0]["Correction"] = json!(1500.5),
            6 => function["svals"][0]
                .as_object_mut()
                .unwrap()
                .remove("Correction")
                .map(|_| ())
                .unwrap(),
            7 => function["svals2"][4]["Target"] = json!(2010),
            8 => function["script"] = json!({"unverified":1}),
            9 => function["functvals"] = json!([2002]),
            10 => function["svals"][0]["IgnoreIndividuality"] = json!(1),
            11 => row["noblePhantasms"][0]["script"] = json!({"tdTypeChangeIDs":[1]}),
            12 => function["svals"][0]["unknownDamageField"] = json!(1),
            13 => function["funcTargetType"] = json!("ally"),
            14 => function["svals"][0]["Rate"] = json!(500),
            _ => function["svals"][0]["Value"] = json!(0),
        }
        let servant = import(&row);
        if mutation >= 12 {
            assert!(servant.noble_phantasms.is_empty(), "mutation {mutation}");
        } else {
            let np = &servant.noble_phantasms[0];
            assert_eq!(np.base_multiplier(1, 1), Some(3.));
            assert!(np.trait_bonus_condition().is_none(), "mutation {mutation}");
            assert!(
                np.notes
                    .iter()
                    .any(|note| note.contains("this is base damage"))
            );
        }
    }
    let mut row = rows().remove(0);
    row["noblePhantasms"][0]["functions"][0]
        .as_object_mut()
        .unwrap()
        .remove("svals5");
    let servant = import(&row);
    let np = &servant.noble_phantasms[0];
    assert_eq!(np.available_overcharges(), [1, 2, 3, 4]);
    assert!(np.trait_bonus_condition().is_some());
    assert!(
        np.components[0]
            .trait_bonus_multiplier(1, 5, np.trait_bonus_condition())
            .is_none()
    );
}

#[test]
#[ignore = "requires complete saved primary export at FGO_TRAIT_AUDIT_EXPORT; no network or snapshot writes"]
fn complete_saved_primary_catalog_matches_audit_eligibility_and_all_source_rows() {
    let payload: Value = serde_json::from_slice(
        &std::fs::read(std::env::var("FGO_TRAIT_AUDIT_EXPORT").expect("FGO_TRAIT_AUDIT_EXPORT"))
            .unwrap(),
    )
    .unwrap();
    let mut variants = std::collections::HashSet::new();
    let (mut supported, mut base_only) = (0, 0);
    for source in payload.as_array().unwrap() {
        let candidates: Vec<_> = source["noblePhantasms"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|np| {
                np["functions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|function| function["funcType"] == "damageNpIndividual")
            })
            .collect();
        if candidates.is_empty() {
            continue;
        }
        let servant = import(source);
        for candidate in candidates {
            let id = candidate["id"].as_u64().unwrap() as u32;
            if !variants.insert(id) {
                continue;
            }
            let np = servant
                .noble_phantasms
                .iter()
                .find(|np| np.id == id)
                .unwrap();
            if candidate["script"].as_object().unwrap().is_empty() {
                assert_source_rows(np, damage(candidate));
                supported += 1;
            } else {
                assert_eq!(id, 2300601);
                assert!(np.trait_bonus_condition().is_none());
                assert!(np.base_multiplier(1, 1).is_some());
                base_only += 1;
            }
        }
    }
    assert_eq!((variants.len(), supported, base_only), (112, 111, 1));
}
