#![cfg(not(target_arch = "wasm32"))]

use fate_grand_calculator::np_mechanics::enemy_status::EnemyStatus;
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
    for (servant_id, np_ids) in [
        (202500, vec![202501]),
        (203300, vec![203301]),
        (400900, vec![400901, 400902]),
        (702500, vec![702501, 702502]),
    ] {
        let servant = loaded.servant(servant_id).unwrap();
        assert!(servant.max_hp.is_some_and(|hp| hp > 0));
        for id in np_ids {
            let np = servant
                .noble_phantasms
                .iter()
                .find(|np| np.id == id)
                .unwrap();
            assert!(np.requires_attacker_hp());
            assert_eq!(np.available_overcharges(), [1, 2, 3, 4, 5]);
        }
    }
    for (servant_id, np_id, condition) in [
        (200300, 200301, EnemyStatus::Poison),
        (504800, 504801, EnemyStatus::SkillSeal),
        (604900, 604901, EnemyStatus::Bind),
        (704900, 704901, EnemyStatus::DefenseUp),
        (1101100, 1101101, EnemyStatus::Charm),
        (1101400, 1101401, EnemyStatus::Curse),
        (2500400, 2500401, EnemyStatus::Burn),
    ] {
        let servant = loaded.servant(servant_id).unwrap();
        let np = servant
            .noble_phantasms
            .iter()
            .find(|np| np.id == np_id)
            .unwrap();
        assert_eq!(np.enemy_status_condition(), Some(condition));
        assert_eq!(np.available_overcharges(), [1, 2, 3, 4, 5]);
    }
    for (servant_id, np_id, target) in [
        (100800, 100802, 2002),
        (201200, 201202, 1),
        (203500, 203501, 2467),
    ] {
        let np = loaded
            .servant(servant_id)
            .unwrap()
            .noble_phantasms
            .iter()
            .find(|np| np.id == np_id)
            .unwrap();
        assert_eq!(np.trait_bonus_condition().unwrap().source_target, target);
        assert_eq!(np.available_overcharges(), [1, 2, 3, 4, 5]);
    }
    let dubai = loaded
        .servant(2300600)
        .unwrap()
        .noble_phantasms
        .iter()
        .find(|np| np.id == 2300601)
        .unwrap();
    assert!(dubai.trait_bonus_condition().is_none());
    assert!(dubai.base_multiplier(1, 1).is_some());
}
