#![cfg(not(target_arch = "wasm32"))]

use std::time::Duration;

use fate_grand_calculator::servant_data::{enrich_servant, normalize_atlas_export};
use serde_json::Value;

#[test]
#[ignore = "requires a live Atlas Academy connection"]
fn current_na_export_enriches_all_ten_additional_servants() {
    const IDS: [u32; 10] = [
        1700100, 9935400, 9935500, 9939130, 9941730, 9943610, 3300100, 9945590, 9945600, 3300200,
    ];
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();
    let basic_text = client
        .get("https://api.atlasacademy.io/export/NA/basic_servant.json")
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .text()
        .unwrap();
    let basic: Value = serde_json::from_str(&basic_text).unwrap();
    let rows = basic.as_array().unwrap();
    let selected = rows
        .iter()
        .filter(|row| {
            row["id"]
                .as_u64()
                .is_some_and(|id| IDS.contains(&(id as u32)))
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut report = normalize_atlas_export(&Value::Array(selected)).unwrap();
    assert_eq!(report.game_data.servants.len(), IDS.len());
    for servant in &mut report.game_data.servants {
        let nice_text = client
            .get(format!(
                "https://api.atlasacademy.io/nice/NA/servant/{}?lang=en",
                servant.id
            ))
            .send()
            .unwrap()
            .error_for_status()
            .unwrap()
            .text()
            .unwrap();
        let nice: Value = serde_json::from_str(&nice_text).unwrap();
        enrich_servant(servant, &nice).unwrap();
    }
    report.game_data.validate().unwrap();
    assert_eq!(report.game_data.servant(9943610).unwrap().deck.len(), 0);
    assert_eq!(report.game_data.servant(3300200).unwrap().deck.len(), 5);
}
