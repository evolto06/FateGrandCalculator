use fate_grand_calculator::{
    loader::GameData,
    model::{SelectedCard, TurnSelection},
};
#[test]
fn rejects_duplicate_physical_cards_multiple_nps_and_invalid_levels() {
    let data = GameData::bundled().unwrap();
    let servant = &data.servants[0];
    for slots in [
        [
            SelectedCard::Normal(0),
            SelectedCard::Normal(0),
            SelectedCard::Normal(1),
        ],
        [
            SelectedCard::NoblePhantasm(0),
            SelectedCard::NoblePhantasm(1),
            SelectedCard::Normal(0),
        ],
        [
            SelectedCard::Normal(5),
            SelectedCard::Normal(1),
            SelectedCard::Normal(2),
        ],
    ] {
        assert!(
            TurnSelection { slots, np_level: 1 }
                .validate(servant)
                .is_err()
        );
    }
    let mut selection = TurnSelection::default_for(servant).unwrap();
    selection.np_level = 0;
    assert!(selection.validate(servant).is_err());
    selection.np_level = 6;
    assert!(selection.validate(servant).is_err());
}
#[test]
fn same_color_distinct_cards_are_valid_and_legacy_decks_are_not_guessed() {
    let mut servant = GameData::bundled().unwrap().servants.remove(0);
    let selection = TurnSelection {
        slots: [
            SelectedCard::Normal(3),
            SelectedCard::Normal(4),
            SelectedCard::Normal(1),
        ],
        np_level: 1,
    };
    assert!(selection.validate(&servant).is_ok());
    servant.deck.clear();
    assert!(TurnSelection::default_for(&servant).is_none());
    assert!(selection.validate(&servant).is_err());
}
