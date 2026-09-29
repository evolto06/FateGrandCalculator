use fate_grand_calculator::{
    loader::{GameData, NpStatus},
    model::{ClassType, TurnSelection},
    servant_data::enrich_servant,
};
use serde_json::json;

#[test]
fn beast_iv_stays_in_catalog_with_an_explicit_special_deck_limit() {
    let mut servant = GameData::bundled().unwrap().servants.remove(0);
    servant.id = 9943610;
    servant.name = "Beast IV".into();
    servant.class = ClassType::BeastIV;
    enrich_servant(
        &mut servant,
        &json!({"id":9943610,"cards":["10","10","10","10","10"],"noblePhantasms":[]}),
    )
    .unwrap();
    assert!(servant.deck.is_empty());
    assert!(
        servant
            .deck_note
            .as_deref()
            .unwrap()
            .contains("special card type 10")
    );
    assert_eq!(servant.np_status, NpStatus::Unavailable);
    assert!(TurnSelection::default_for(&servant).is_none());
}

#[test]
fn unrecognized_special_cards_still_reject_other_servants() {
    let mut servant = GameData::bundled().unwrap().servants.remove(0);
    let result = enrich_servant(
        &mut servant,
        &json!({"id":100100,"cards":["10","10","10","10","10"],"noblePhantasms":[]}),
    );
    assert!(result.is_err());
}
