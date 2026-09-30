use super::*;

fn servant() -> ServantRecord {
    let mut servant = GameData::bundled().unwrap().servants.remove(0);
    servant.deck = vec![
        CardType::Buster,
        CardType::Arts,
        CardType::Arts,
        CardType::Quick,
        CardType::Buster,
    ];
    let mut variant = servant.noble_phantasms[0].clone();
    variant.id += 1;
    variant.name = "Alternative NP".into();
    servant.noble_phantasms.push(variant);
    servant
}

fn input(width: f32, events: Vec<egui::Event>) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(width, 2000.0),
        )),
        events,
        ..Default::default()
    }
}

fn pointer(pos: egui::Pos2, pressed: bool) -> Vec<egui::Event> {
    vec![
        egui::Event::PointerMoved(pos),
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        },
    ]
}

fn tile_frame(
    ctx: &egui::Context,
    events: Vec<egui::Event>,
    servant: &ServantRecord,
    turn: &mut TurnSelection,
    slot: usize,
    candidate: SelectedCard,
) -> egui::Response {
    let mut response = None;
    let output = ctx.run_ui(input(360.0, events), |ui| {
        apply_canvas(ui);
        response = Some(card_tile(ui, servant, turn, slot, candidate, 100.0));
    });
    output.drop_without_applying_deltas();
    response.unwrap()
}

fn click_tile(
    servant: &ServantRecord,
    turn: &mut TurnSelection,
    slot: usize,
    candidate: SelectedCard,
) -> bool {
    let ctx = egui::Context::default();
    let response = tile_frame(&ctx, vec![], servant, turn, slot, candidate);
    let pos = response.rect.center();
    tile_frame(&ctx, pointer(pos, true), servant, turn, slot, candidate);
    tile_frame(&ctx, pointer(pos, false), servant, turn, slot, candidate).enabled()
}

#[test]
fn tiles_replace_only_the_active_position_and_keep_same_color_cards_distinct() {
    let servant = servant();
    let mut turn = TurnSelection::default_for(&servant).unwrap();
    let before = turn.slots;
    assert!(click_tile(&servant, &mut turn, 2, SelectedCard::Normal(2)));
    assert_eq!(turn.slots, [before[0], before[1], SelectedCard::Normal(2)]);
    assert!(click_tile(&servant, &mut turn, 1, SelectedCard::Normal(1)));
    assert_eq!(
        turn.slots[1..],
        [SelectedCard::Normal(1), SelectedCard::Normal(2)]
    );
    assert!(turn.validate(&servant).is_ok());
    let before = turn.clone();
    assert!(!click_tile(&servant, &mut turn, 2, SelectedCard::Normal(1)));
    assert_eq!(turn, before);
}

#[test]
fn np_variants_replace_the_existing_np_but_cannot_occupy_another_position() {
    let servant = servant();
    let mut turn = TurnSelection::default_for(&servant).unwrap();
    assert!(click_tile(
        &servant,
        &mut turn,
        0,
        SelectedCard::NoblePhantasm(1)
    ));
    assert_eq!(turn.slots[0], SelectedCard::NoblePhantasm(1));
    let before = turn.clone();
    assert!(!click_tile(
        &servant,
        &mut turn,
        1,
        SelectedCard::NoblePhantasm(0)
    ));
    assert_eq!(turn, before);
    assert!(click_tile(&servant, &mut turn, 0, SelectedCard::Normal(4)));
    assert!(click_tile(
        &servant,
        &mut turn,
        1,
        SelectedCard::NoblePhantasm(0)
    ));
    assert!(turn.validate(&servant).is_ok());
}

#[test]
fn focused_tiles_support_keyboard_activation() {
    for key in [egui::Key::Enter, egui::Key::Space] {
        let servant = servant();
        let mut turn = TurnSelection::default_for(&servant).unwrap();
        let ctx = egui::Context::default();
        let response = tile_frame(
            &ctx,
            vec![],
            &servant,
            &mut turn,
            2,
            SelectedCard::Normal(3),
        );
        ctx.memory_mut(|memory| memory.request_focus(response.id));
        let events = vec![egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }];
        tile_frame(
            &ctx,
            events,
            &servant,
            &mut turn,
            2,
            SelectedCard::Normal(3),
        );
        tile_frame(
            &ctx,
            vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            &servant,
            &mut turn,
            2,
            SelectedCard::Normal(3),
        );
        assert_eq!(turn.slots[2], SelectedCard::Normal(3), "{key:?}");
    }
}

#[test]
fn attack_setup_fits_narrow_and_wide_columns_with_long_names() {
    for width in [328.0, 412.0, 450.0, 500.0, 760.0] {
        let mut data = GameData::bundled().unwrap();
        data.servants[0].name =
            "A servant with a deliberately long display name to exercise wrapping".into();
        data.servants[0].noble_phantasms[0].name =
            "A very long Noble Phantasm name for the card tile".into();
        let mut id = data.servants[0].id;
        let mut selection = TurnSelection::default_for(&data.servants[0]);
        let mut search = String::new();
        let mut active_slot = 0;
        let mut buffs = TurnBuffInputs::default();
        let ctx = egui::Context::default();
        for _ in 0..2 {
            let output = ctx.run_ui(input(width, vec![]), |ui| {
                apply_canvas(ui);
                assert_eq!(ui.available_width(), width);
                let left = ui.cursor().left();
                attack_panel(
                    ui,
                    &data,
                    &mut id,
                    &mut search,
                    None,
                    "No portrait available",
                    None,
                    &mut selection,
                    &mut active_slot,
                    &mut buffs,
                );
                assert!(
                    ui.min_rect().right() <= left + width + 1.0,
                    "column width {width}, content {:?}",
                    ui.min_rect()
                );
            });
            output.drop_without_applying_deltas();
        }
    }
}

fn painted_text_rect(shape: &egui::epaint::Shape, text: &str) -> Option<egui::Rect> {
    match shape {
        egui::epaint::Shape::Text(shape) if shape.galley.text() == text => {
            Some(shape.galley.rect.translate(shape.pos.to_vec2()))
        }
        egui::epaint::Shape::Vec(shapes) => shapes
            .iter()
            .find_map(|shape| painted_text_rect(shape, text)),
        _ => None,
    }
}

#[test]
fn searchable_dropdown_is_below_portrait_and_changing_servant_resets_attack_order() {
    let data = GameData::bundled().unwrap();
    let target = &data.servants[1];
    let mut id = data.servants[0].id;
    let mut search = String::new();
    let mut selection = TurnSelection::default_for(&data.servants[0]);
    let mut active_slot = 2;
    let ctx = egui::Context::default();
    let mut response = None;
    let mut frame = |events| {
        let output = ctx.run_ui(input(360.0, events), |ui| {
            apply_canvas(ui);
            response = Some(servant_setup(
                ui,
                &data,
                &mut id,
                &mut search,
                None,
                "No portrait available",
                None,
                &mut selection,
                &mut active_slot,
            ));
        });
        (response.clone().unwrap(), output)
    };
    let (response, output) = frame(vec![]);
    assert!(response.rect.top() > 300.0);
    output.drop_without_applying_deltas();
    let pos = response.rect.center();
    frame(pointer(pos, true)).1.drop_without_applying_deltas();
    frame(pointer(pos, false)).1.drop_without_applying_deltas();
    let (_, output) = frame(vec![]);
    let search_field = output
        .shapes
        .iter()
        .find_map(|shape| painted_text_rect(&shape.shape, "Type a servant name"))
        .expect("dropdown contains its search field");
    output.drop_without_applying_deltas();
    frame(pointer(search_field.center(), true))
        .1
        .drop_without_applying_deltas();
    frame(pointer(search_field.center(), false))
        .1
        .drop_without_applying_deltas();
    frame(vec![egui::Event::Text(target.name.clone())])
        .1
        .drop_without_applying_deltas();
    let (_, output) = frame(vec![]);
    let result = output
        .shapes
        .iter()
        .rev()
        .find_map(|shape| painted_text_rect(&shape.shape, &target.name))
        .expect("filtered servant appears in dropdown");
    assert!(
        output.shapes.iter().all(|shape| painted_text_rect(
            &shape.shape,
            "No servants match that search."
        )
        .is_none())
    );
    output.drop_without_applying_deltas();
    frame(pointer(result.center(), true))
        .1
        .drop_without_applying_deltas();
    frame(pointer(result.center(), false))
        .1
        .drop_without_applying_deltas();
    assert_eq!(id, target.id);
    assert!(search.is_empty());
    assert_eq!(active_slot, 0);
    assert_eq!(selection, TurnSelection::default_for(target));
}

#[test]
fn clicking_an_order_slot_changes_the_editing_position() {
    let servant = servant();
    let mut selection = TurnSelection::default_for(&servant);
    let mut active_slot = 0;
    let ctx = egui::Context::default();
    let mut frame = |events| {
        ctx.run_ui(input(360.0, events), |ui| {
            apply_canvas(ui);
            turn_selector(ui, &servant, &mut selection, &mut active_slot);
        })
    };
    let output = frame(vec![]);
    let slot = output
        .shapes
        .iter()
        .find_map(|shape| painted_text_rect(&shape.shape, "2"))
        .expect("second attack position is an editable button");
    output.drop_without_applying_deltas();
    frame(pointer(slot.center(), true)).drop_without_applying_deltas();
    frame(pointer(slot.center(), false)).drop_without_applying_deltas();
    assert_eq!(active_slot, 1);
}

#[test]
fn compact_card_labels_render_without_truncation_at_small_widths() {
    let servant = servant();
    for width in [68.0, 88.0, 130.0] {
        for candidate in [
            SelectedCard::NoblePhantasm(0),
            SelectedCard::Normal(0),
            SelectedCard::Normal(1),
            SelectedCard::Normal(3),
        ] {
            let ctx = egui::Context::default();
            let mut response = None;
            let output = ctx.run_ui(input(360.0, vec![]), |ui| {
                apply_canvas(ui);
                response = Some(card_button(
                    ui,
                    egui::vec2(width, 80.0),
                    selected_card_symbol(&servant, candidate),
                    selected_card_color(&servant, candidate),
                    true,
                    true,
                ));
            });
            let response = response.unwrap();
            let label = output
                .shapes
                .iter()
                .find_map(|shape| {
                    painted_text_rect(&shape.shape, selected_card_symbol(&servant, candidate))
                })
                .expect("complete NP/Q/A/B symbol is painted");
            assert!(response.rect.shrink(8.0).contains_rect(label));
            assert!(output.shapes.iter().any(|shape| match &shape.shape {
                egui::epaint::Shape::Rect(rect) =>
                    rect.stroke.width == 3.0 && rect.stroke.color == ACCENT,
                _ => false,
            }));
            output.drop_without_applying_deltas();
        }
    }
}

#[test]
fn damage_ranges_fit_inside_padded_result_panels() {
    let servant = servant();
    let turn = TurnSelection::default_for(&servant).unwrap();
    let result = fate_grand_calculator::damage::calculate_turn(
        &servant,
        &turn,
        TurnBuffs::default(),
        ClassType::Lancer,
        AttributeType::Sky,
    )
    .unwrap();
    let mut damage = result.cards[0];
    damage.minimum_damage = u32::MAX;
    damage.maximum_damage = u32::MAX;
    for width in [280.0, 328.0, 412.0] {
        let ctx = egui::Context::default();
        let output = ctx.run_ui(input(width, vec![]), |ui| {
            apply_canvas(ui);
            damage_result(ui, damage, false);
            assert!(ui.min_rect().right() <= width + 1.0);
        });
        let range = output
            .shapes
            .iter()
            .find_map(|shape| painted_text_rect(&shape.shape, &format_damage_range(damage)))
            .unwrap();
        assert!(range.left() >= 18.0 && range.right() <= width - 18.0 + 1.0);
        output.drop_without_applying_deltas();
    }
}
