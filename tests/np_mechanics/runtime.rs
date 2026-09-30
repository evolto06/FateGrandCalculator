use fate_grand_calculator::{
    damage::{TurnBuffs, TurnDamageResult, calculate_turn},
    loader::{GameData, NoblePhantasmRecord, ServantRecord},
    model::{AttributeType, ClassType, SelectedCard, TurnSelection},
};

fn run(servant: &ServantRecord, np_level: u8, defense: f64) -> TurnDamageResult {
    calculate_turn(
        servant,
        &TurnSelection {
            slots: [
                SelectedCard::NoblePhantasm(0),
                SelectedCard::Normal(0),
                SelectedCard::Normal(1),
            ],
            np_level,
            affection_level: 0,
        },
        TurnBuffs {
            attack_buff: 0.3,
            enemy_defense: defense,
            ..Default::default()
        },
        ClassType::Saber,
        AttributeType::Earth,
    )
    .unwrap()
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn imported_defense_pierce_applies_only_to_np_across_levels() {
    use fate_grand_calculator::servant_data::enrich_servant;
    use serde_json::json;

    let mut piercing = GameData::bundled().unwrap().servants.remove(0);
    piercing.id = 200100;
    let values: Vec<_> = [3000, 4000, 4500, 4750, 5000]
        .into_iter()
        .map(|value| json!({"Value":value}))
        .collect();
    enrich_servant(
        &mut piercing,
        &json!({"id":200100,"cards":["3","1","1","1","2"],"noblePhantasms":[
            {"id":200101,"name":"Unlimited Blade Works","card":"2","functions":[
                {"funcType":"damageNpPierce","svals":values}
            ]}
        ]}),
    )
    .unwrap();
    assert!(piercing.noble_phantasms[0].defense_pierce);
    let mut standard = piercing.clone();
    standard.noble_phantasms[0].defense_pierce = false;

    for level in 1..=5 {
        let baseline = run(&piercing, level, 0.0);
        for defense in [-0.2, 0.0, 0.2, 1.5] {
            let pierced = run(&piercing, level, defense);
            let ordinary = run(&standard, level, defense);
            let expected_np = run(&standard, level, defense.min(0.0));
            assert_eq!(
                pierced.cards[0].minimum_damage,
                expected_np.cards[0].minimum_damage
            );
            assert_eq!(
                pierced.cards[0].maximum_damage,
                expected_np.cards[0].maximum_damage
            );
            assert_eq!(
                pierced.cards[0].attack_defense_multiplier,
                expected_np.cards[0].attack_defense_multiplier
            );
            for position in 1..=2 {
                assert_eq!(
                    pierced.cards[position].minimum_damage,
                    ordinary.cards[position].minimum_damage
                );
                assert_eq!(
                    pierced.cards[position].maximum_damage,
                    ordinary.cards[position].maximum_damage
                );
            }
            assert_eq!(
                pierced.extra.unwrap().minimum_damage,
                ordinary.extra.unwrap().minimum_damage
            );
            assert_eq!(
                pierced.extra.unwrap().maximum_damage,
                ordinary.extra.unwrap().maximum_damage
            );
            if defense > 0.0 {
                assert_eq!(
                    pierced.cards[0].minimum_damage,
                    baseline.cards[0].minimum_damage
                );
                assert!(pierced.cards[0].minimum_damage > ordinary.cards[0].minimum_damage);
            } else if defense < 0.0 {
                assert!(pierced.cards[0].minimum_damage > baseline.cards[0].minimum_damage);
            }
        }
    }
}

#[test]
fn legacy_snapshot_defaults_to_non_piercing_and_new_metadata_round_trips() {
    let bundled = GameData::bundled().unwrap();
    let mut legacy = serde_json::to_value(&bundled).unwrap();
    for servant in legacy["servants"].as_array_mut().unwrap() {
        for np in servant["noble_phantasms"].as_array_mut().unwrap() {
            np.as_object_mut().unwrap().remove("defense_pierce");
        }
    }
    let mut data = GameData::from_json(&legacy.to_string()).unwrap();
    assert!(
        data.servants
            .iter()
            .flat_map(|servant| &servant.noble_phantasms)
            .all(|np| !np.defense_pierce)
    );
    let np = &mut data.servants[0].noble_phantasms[0];
    np.defense_pierce = true;
    np.affection = Some(fate_grand_calculator::loader::AffectionScaling {
        base: 1.0,
        per_level: 0.1,
        max_level: 10,
        ignore_defense_at: 7,
    });
    let json = serde_json::to_string(&data).unwrap();
    let restored = GameData::from_json(&json).unwrap();
    let np = &restored.servants[0].noble_phantasms[0];
    assert!(np.defense_pierce);
    assert_eq!(np.affection.as_ref().unwrap().multiplier(10), 2.0);
    let raw_np: NoblePhantasmRecord = serde_json::from_value(serde_json::json!({
        "id":100101,"name":"legacy","card_type":"buster","multipliers":[3.0,4.0,4.5,4.75,5.0]
    }))
    .unwrap();
    assert!(!raw_np.defense_pierce);
    assert!(raw_np.affection.is_none());
}
