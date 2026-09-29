#![cfg(not(target_arch = "wasm32"))]

use fate_grand_calculator::{loader::NpStatus, servant_data::ServantDataService};

#[test]
#[ignore = "downloads Atlas Academy data and updates the local servant snapshot"]
fn current_na_update_saves_the_complete_additional_servant_catalog() {
    let report = ServantDataService::update().unwrap();
    for id in [
        1700100, 9935400, 9935500, 9939130, 9941730, 9943610, 3300100, 9945590, 9945600, 3300200,
    ] {
        assert!(
            report.game_data.servant(id).is_some(),
            "missing servant {id}"
        );
    }
    let startup = ServantDataService::load_local_or_bundled();
    let loaded = startup.game_data.unwrap();
    assert_eq!(loaded.servants.len(), report.game_data.servants.len());
    assert!(loaded.servant(3300200).is_some());
    assert_eq!(
        loaded.servant(3300200).unwrap().np_status,
        NpStatus::Damaging
    );
}
