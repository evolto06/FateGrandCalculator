use fate_grand_calculator::{
    loader::{GameData, NpStatus, ServantRecord},
    servant_data::{enrich_servant, normalize_atlas_export},
};
use serde_json::{Value, json};

// Reduced Atlas NA payloads captured during the 2026-09-30 audit. Each row
// retains its source URL; all five NP-level values are copied, not inferred.
fn audited_rows() -> Vec<Value> {
    let fixture: Value =
        serde_json::from_str(include_str!("../fixtures/np_mechanics_defense_pierce.json")).unwrap();
    fixture["servants"].as_array().unwrap().clone()
}

fn record(row: &Value) -> ServantRecord {
    normalize_atlas_export(&json!([row]))
        .unwrap()
        .game_data
        .servants
        .remove(0)
}

fn single_pierce_row() -> Value {
    let mut row = audited_rows().remove(0);
    row["noblePhantasms"] = json!([row["noblePhantasms"][0].clone()]);
    row
}

#[test]
fn imports_every_audited_defense_piercing_variant_without_losing_older_variants() {
    let rows = audited_rows();
    assert_eq!(rows.len(), 43);
    let mut pierce_count = 0;
    for row in rows {
        let mut servant = record(&row);
        enrich_servant(&mut servant, &row).unwrap();
        assert_eq!(servant.np_status, NpStatus::Damaging, "{}", servant.name);
        for source in row["noblePhantasms"].as_array().unwrap() {
            let damage: Vec<_> = source["functions"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|function| function["funcType"].as_str().unwrap().starts_with("damage"))
                .collect();
            if damage.len() != 1 {
                continue;
            }
            let function = damage[0];
            let kind = function["funcType"].as_str().unwrap();
            if !matches!(kind, "damageNp" | "damageNpIndividual" | "damageNpPierce") {
                continue;
            }
            let id = source["id"].as_u64().unwrap() as u32;
            let imported = servant
                .noble_phantasms
                .iter()
                .find(|np| np.id == id)
                .unwrap_or_else(|| panic!("Missing {} NP {id}", servant.name));
            let expected: Vec<_> = function["svals"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value["Value"].as_f64().unwrap() / 1000.0)
                .collect();
            assert_eq!(
                imported.components[0].overcharge[0]
                    .as_ref()
                    .unwrap()
                    .multipliers
                    .as_slice(),
                expected.as_slice()
            );
            assert_eq!(imported.defense_pierce, kind == "damageNpPierce");
            assert!(imported.affection.is_none());
            if imported.defense_pierce {
                pierce_count += 1;
                assert!(
                    imported
                        .notes
                        .iter()
                        .any(|note| note.contains("Defense Down"))
                );
            }
        }
    }
    assert_eq!(pierce_count, 64);
}

#[test]
fn nero_emiya_and_previously_partial_upgrades_are_selectable() {
    for row in audited_rows().into_iter().filter(|row| {
        matches!(
            row["id"].as_u64().unwrap(),
            100500 | 200100 | 102000 | 200800 | 300700 | 502200 | 502300 | 601500
        )
    }) {
        let mut servant = record(&row);
        enrich_servant(&mut servant, &row).unwrap();
        assert_eq!(
            servant.noble_phantasms.len(),
            row["noblePhantasms"].as_array().unwrap().len()
        );
        assert!(
            !servant
                .noble_phantasms
                .iter()
                .flat_map(|np| &np.notes)
                .any(|note| note.contains("Some NP variants"))
        );
    }
}

#[test]
fn defense_piercing_is_dispatched_by_function_not_servant_allowlist() {
    let mut row = single_pierce_row();
    row["id"] = json!(999000);
    let mut servant = record(&row);
    enrich_servant(&mut servant, &row).unwrap();
    assert!(servant.noble_phantasms[0].defense_pierce);
}

#[test]
fn invalid_piercing_multiplier_data_is_unavailable_instead_of_guessed() {
    for invalid in [json!(0), json!(-1), json!("3000"), json!(null), json!(true)] {
        let mut row = single_pierce_row();
        row["noblePhantasms"][0]["functions"][0]["svals"][0]["Value"] = invalid;
        let mut servant = record(&row);
        enrich_servant(&mut servant, &row).unwrap();
        assert_eq!(servant.np_status, NpStatus::Unavailable);
        assert!(servant.noble_phantasms.is_empty());
    }
    let mut row = single_pierce_row();
    row["noblePhantasms"][0]["functions"][0]["svals"]
        .as_array_mut()
        .unwrap()
        .pop();
    let mut servant = record(&row);
    enrich_servant(&mut servant, &row).unwrap();
    assert_eq!(servant.np_status, NpStatus::Unavailable);
}

#[test]
fn support_and_other_audited_mechanics_are_not_imported_as_piercing_base_damage() {
    for kind in [
        "damageNpHpratioLow",
        "damageNpIndividualSum",
        "damageNpStateIndividualFix",
        "damageNpRare",
        "damageNpAndOrCheckIndividuality",
        "damageNpBattlePointPhase",
    ] {
        let mut row = single_pierce_row();
        row["noblePhantasms"][0]["functions"][0]["funcType"] = json!(kind);
        let mut servant = record(&row);
        enrich_servant(&mut servant, &row).unwrap();
        assert_eq!(servant.np_status, NpStatus::Unsupported, "{kind}");
        assert!(servant.noble_phantasms.is_empty());
    }
    let mut row = single_pierce_row();
    row["noblePhantasms"][0]["functions"] = json!([{"funcType":"addState"}]);
    row["noblePhantasms"][0]["effectFlags"] = json!(["support"]);
    let mut servant = record(&row);
    enrich_servant(&mut servant, &row).unwrap();
    assert_eq!(servant.np_status, NpStatus::Support);
    assert!(servant.noble_phantasms.is_empty());
}

#[test]
fn multiple_damage_components_remain_unsupported_and_duplicates_are_deduplicated() {
    let mut row = single_pierce_row();
    let mut extra = row["noblePhantasms"][0]["functions"][0].clone();
    extra["svals"] = json!([{"Value":0},{"Value":0},{"Value":0},{"Value":0},{"Value":0}]);
    row["noblePhantasms"][0]["functions"]
        .as_array_mut()
        .unwrap()
        .push(extra);
    let mut servant = record(&row);
    enrich_servant(&mut servant, &row).unwrap();
    assert_eq!(servant.np_status, NpStatus::Unsupported);
    assert!(servant.noble_phantasms.is_empty());

    let mut row = single_pierce_row();
    let duplicate = row["noblePhantasms"][0].clone();
    row["noblePhantasms"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    enrich_servant(&mut servant, &row).unwrap();
    assert_eq!(servant.noble_phantasms.len(), 1);
}

#[test]
fn bundled_emiya_exposes_all_audited_piercing_variants() {
    let data = GameData::bundled().unwrap();
    let servant = data.servant(200100).unwrap();
    assert_eq!(servant.np_status, NpStatus::Damaging);
    assert_eq!(
        servant
            .noble_phantasms
            .iter()
            .map(|np| np.id)
            .collect::<Vec<_>>(),
        [200101, 200102, 200197, 200198]
    );
    assert!(servant.noble_phantasms.iter().all(|np| np.defense_pierce));
    let source = audited_rows()
        .into_iter()
        .find(|row| row["id"] == 200100)
        .unwrap();
    let mut imported = record(&source);
    enrich_servant(&mut imported, &source).unwrap();
    assert_eq!(
        serde_json::to_value(&servant.noble_phantasms).unwrap(),
        serde_json::to_value(&imported.noble_phantasms).unwrap()
    );
}
