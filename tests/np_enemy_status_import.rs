#![cfg(not(target_arch = "wasm32"))]

use fate_grand_calculator::{
    damage::{TurnBuffs, calculate_turn},
    loader::{NpStatus, ServantRecord},
    model::{AttributeType, ClassType, SelectedCard, TurnSelection},
    np_mechanics::enemy_status::EnemyStatus,
    servant_data::{enrich_servant, normalize_atlas_export},
};
use serde_json::{Value, json};

fn rows() -> Vec<Value> {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/np_mechanics_enemy_status.json")).unwrap();
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

fn damage(row: &mut Value) -> &mut Value {
    row["noblePhantasms"][0]["functions"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|f| f["funcType"] == "damageNpStateIndividualFix")
        .unwrap()
}

#[test]
fn imports_seven_verified_conditions_source_units_and_matching_policies() {
    let conditions = [
        EnemyStatus::Poison,
        EnemyStatus::SkillSeal,
        EnemyStatus::Bind,
        EnemyStatus::DefenseUp,
        EnemyStatus::Charm,
        EnemyStatus::Curse,
        EnemyStatus::Burn,
    ];
    let ids = [200301, 504801, 604901, 704901, 1101101, 1101401, 2500401];
    for ((mut source, condition), id) in rows().into_iter().zip(conditions).zip(ids) {
        let servant = import(&source);
        assert_eq!(servant.np_status, NpStatus::Damaging);
        assert_eq!(servant.noble_phantasms.len(), 1);
        let np = &servant.noble_phantasms[0];
        assert_eq!(np.id, id);
        assert_eq!(np.available_overcharges(), [1, 2, 3, 4, 5]);
        assert_eq!(np.components.len(), 1);
        assert!(
            np.notes
                .iter()
                .any(|n| n.contains("enemy status at NP damage time"))
        );
        let function = damage(&mut source);
        for oc in 1..=5 {
            let key = if oc == 1 {
                "svals".to_owned()
            } else {
                format!("svals{oc}")
            };
            let row = np.components[0].overcharge[oc - 1].as_ref().unwrap();
            let scaling = row.enemy_status.as_ref().unwrap();
            assert_eq!(scaling.condition, condition);
            assert_eq!(
                scaling.include_ignore_individuality,
                condition.includes_ignored_individuality()
            );
            for level in 0..5 {
                assert_eq!(
                    scaling.source_corrections[level] as u64,
                    function[&key][level]["Correction"].as_u64().unwrap()
                );
                assert_eq!(
                    row.multipliers[level],
                    function[&key][level]["Value"].as_u64().unwrap() as f64 / 1000.0
                );
                assert_eq!(
                    condition.source_target() as u64,
                    function[&key][level]["Target"].as_u64().unwrap()
                );
            }
        }
    }
}

#[test]
fn imported_variants_match_350_independent_fixed_rates_and_damage_endpoints() {
    let fixture: Value = serde_json::from_str(include_str!(
        "fixtures/np_mechanics_enemy_status_golden.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 350);
    let mut servants: Vec<_> = rows().iter().map(import).collect();
    for servant in &mut servants {
        servant.attack = 1003;
        servant.class = ClassType::Saber;
        servant.attribute = AttributeType::Earth;
    }
    for case in cases {
        let v: Vec<_> = case
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap())
            .collect();
        let servant = servants
            .iter()
            .find(|s| s.noble_phantasms[0].id as u64 == v[0])
            .unwrap();
        let np = &servant.noble_phantasms[0];
        let component = &np.components[0];
        let condition = component.overcharge[0]
            .as_ref()
            .unwrap()
            .enemy_status
            .as_ref()
            .unwrap()
            .condition;
        let selection = TurnSelection {
            slots: [
                SelectedCard::NoblePhantasm(0),
                SelectedCard::Normal(0),
                SelectedCard::Normal(1),
            ],
            np_level: v[1] as u8,
            overcharge_level: v[2] as u8,
            enemy_status: (v[3] == 1).then_some(condition),
            affection_level: 0,
            attacker_hp: None,
        };
        let breakdown = component
            .enemy_status_breakdown(
                selection.np_level,
                selection.overcharge_level,
                selection.enemy_status,
            )
            .unwrap();
        assert_eq!(breakdown.base_multiplier, v[4] as f64 / 1000.0, "{case}");
        assert_eq!(
            breakdown.conditional_multiplier,
            v[5] as f64 / 1000.0,
            "{case}"
        );
        assert!(
            (breakdown.effective_multiplier - v[6] as f64 / 1000000.0).abs() < 1e-12,
            "{case}"
        );
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
            (v[7] as u32, v[8] as u32),
            "{case}"
        );
        let absent = TurnSelection {
            enemy_status: None,
            ..selection
        };
        let base = calculate_turn(
            servant,
            &absent,
            TurnBuffs::default(),
            ClassType::Saber,
            AttributeType::Earth,
        )
        .unwrap();
        assert_eq!(result.cards[1].minimum_damage, base.cards[1].minimum_damage);
        assert_eq!(result.cards[2].maximum_damage, base.cards[2].maximum_damage);
        assert_eq!(
            result.extra.map(|e| e.minimum_damage),
            base.extra.map(|e| e.minimum_damage)
        );
    }
}

#[test]
fn status_import_rejects_unknown_scripts_conditions_flags_and_variants() {
    for mutation in 0..21 {
        let mut row = rows().remove(0);
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
            12 => damage(&mut row)["svals"][0]["TargetIndiv"] = json!(3011),
            13 => damage(&mut row)["svals"][0]["Target"] = json!(9999),
            14 => damage(&mut row)["svals"][0]["Target"] = json!(-3011),
            15 => damage(&mut row)["svals"][0]["IncludeIgnoreIndividuality"] = json!(2),
            16 => damage(&mut row)["svals"][0]["IncludeIgnoreIndividuality"] = json!(1),
            17 => damage(&mut row)["svals"][0]["IgnoreIndividuality"] = json!(1),
            18 => {
                row["id"] = json!(999000);
                row["noblePhantasms"][0]["id"] = json!(999001);
            }
            19 => damage(&mut row)["svals"][1]["Target"] = json!(3015),
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
fn status_rates_require_guaranteed_activation_and_positive_integer_source_units() {
    for key in ["Value", "Correction"] {
        for invalid in [
            json!(0),
            json!(-1),
            json!(1.5),
            json!("1500"),
            json!(null),
            json!(true),
            json!(4294967296_u64),
        ] {
            let mut row = rows().remove(0);
            damage(&mut row)["svals"][0][key] = invalid;
            assert_eq!(import(&row).np_status, NpStatus::Unavailable, "{key}");
        }
        let mut row = rows().remove(0);
        damage(&mut row)["svals"][0]
            .as_object_mut()
            .unwrap()
            .remove(key);
        assert_eq!(import(&row).np_status, NpStatus::Unavailable);
    }
    for rate in [json!(0), json!(500), json!(null), json!(1000.0)] {
        let mut row = rows().remove(0);
        damage(&mut row)["svals"][0]["Rate"] = rate;
        assert_eq!(import(&row).np_status, NpStatus::Unsupported);
    }
}

#[test]
fn unsupported_or_missing_higher_oc_never_inherits_another_row() {
    for mutation in 0..6 {
        let mut row = rows().remove(0);
        match mutation {
            0 => {
                damage(&mut row).as_object_mut().unwrap().remove("svals3");
            }
            1 => damage(&mut row)["svals3"] = json!([]),
            2 => damage(&mut row)["svals3"][0]["Target"] = json!(3015),
            3 => damage(&mut row)["svals3"][0]["IncludeIgnoreIndividuality"] = json!(1),
            4 => damage(&mut row)["svals3"][0]["Correction"] = json!(0),
            _ => damage(&mut row)["svals3"][0]["Unknown"] = json!(1),
        }
        let servant = import(&row);
        assert_eq!(servant.np_status, NpStatus::Damaging);
        let np = &servant.noble_phantasms[0];
        assert_eq!(np.available_overcharges(), [1, 2, 4, 5]);
        assert_eq!(
            np.components[0].enemy_status_multiplier(1, 3, Some(EnemyStatus::Poison)),
            None
        );
        assert!(np.notes.iter().any(|n| n.contains("never inferred")));
    }
}

#[test]
fn source_effect_timing_remains_a_manual_success_assumption() {
    let sources = rows();
    let koji = import(&sources[2]);
    let yang = import(&sources[6]);
    assert_eq!(
        sources[2]["noblePhantasms"][0]["functions"][0]["funcPopupText"],
        "Bind"
    );
    assert_eq!(
        sources[6]["noblePhantasms"][0]["functions"][1]["funcPopupText"],
        "Burn"
    );
    assert!(
        koji.noble_phantasms[0]
            .notes
            .iter()
            .any(|n| n.contains("NP effects before damage"))
    );
    for servant in [koji, yang] {
        let component = &servant.noble_phantasms[0].components[0];
        assert_eq!(component.enemy_status_multiplier(1, 1, None), Some(1.0));
        assert!(
            servant.noble_phantasms[0]
                .notes
                .iter()
                .any(|n| n.contains("Status application and success are not simulated"))
        );
        assert!(
            servant.noble_phantasms[0]
                .notes
                .iter()
                .any(|n| n.contains("Other NP effects"))
        );
    }
}

#[test]
fn exact_ignored_individuality_matching_policy_is_required() {
    for index in [1, 2, 4] {
        let mut row = rows().remove(index);
        damage(&mut row)["svals"][0]
            .as_object_mut()
            .unwrap()
            .remove("IncludeIgnoreIndividuality");
        assert_eq!(import(&row).np_status, NpStatus::Unsupported);
    }
    let mut row = rows().remove(0);
    for key in ["svals", "svals2", "svals3", "svals4", "svals5"] {
        for value in damage(&mut row)[key].as_array_mut().unwrap() {
            value["IncludeIgnoreIndividuality"] = json!(0);
        }
    }
    assert_eq!(import(&row).np_status, NpStatus::Damaging);
}
