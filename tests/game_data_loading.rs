use fate_grand_calculator::loader::GameData;

#[test]
fn bundled_game_data_loads_with_source_metadata() {
    let data = GameData::bundled().expect("bundled game data should be valid");

    assert_eq!(data.region, "NA");
    assert!(!data.version.is_empty());
    assert!(!data.retrieved_at.is_empty());
    assert!(data.servant(100100).is_some());
    assert!(data.servant(200100).is_some());
}
