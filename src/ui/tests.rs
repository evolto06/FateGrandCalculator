use super::*;

fn servant() -> ServantRecord {
    use fate_grand_calculator::np_mechanics::components::NpDamageComponent;
    let mut servant = GameData::bundled().unwrap().servants.remove(0);
    for np in &mut servant.noble_phantasms {
        let multipliers =
            std::array::from_fn(|level| np.base_multiplier(level as u8 + 1, 1).unwrap());
        np.components = vec![NpDamageComponent::legacy(multipliers)];
    }
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
    selection.as_mut().unwrap().np_level = 4;
    selection.as_mut().unwrap().overcharge_level = 4;
    selection.as_mut().unwrap().affection_level = 7;
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

fn full_overcharge_servant() -> ServantRecord {
    use fate_grand_calculator::np_mechanics::components::{NpDamageComponent, NpDamageValues};
    let mut servant = servant();
    servant.noble_phantasms[0].components = vec![NpDamageComponent {
        target: None,
        overcharge: std::array::from_fn(|oc| {
            Some(NpDamageValues::guaranteed([3.0 + oc as f64; 5]))
        }),
    }];
    servant
}

#[test]
fn overcharge_changes_independently_and_legacy_choices_are_disabled() {
    for available in [false, true] {
        let servant = if available {
            full_overcharge_servant()
        } else {
            servant()
        };
        let mut turn = TurnSelection::default_for(&servant).unwrap();
        turn.np_level = 4;
        turn.affection_level = 7;
        let ctx = egui::Context::default();
        let mut frame = |events| {
            ctx.run_ui(input(328.0, events), |ui| {
                apply_canvas(ui);
                np_settings(ui, &servant, &mut turn);
            })
        };
        let output = frame(vec![]);
        let combo = output
            .shapes
            .iter()
            .find_map(|shape| painted_text_rect(&shape.shape, "OC 1"))
            .unwrap();
        if !available {
            assert!(output.shapes.iter().any(|shape| {
                painted_text_rect(
                    &shape.shape,
                    "Update servant data to enable missing Overcharge levels.",
                )
                .is_some()
            }));
        }
        output.drop_without_applying_deltas();
        frame(pointer(combo.center(), true)).drop_without_applying_deltas();
        frame(pointer(combo.center(), false)).drop_without_applying_deltas();
        let output = frame(vec![]);
        let option = output
            .shapes
            .iter()
            .find_map(|shape| painted_text_rect(&shape.shape, "OC 3"))
            .expect("all OC choices shown");
        output.drop_without_applying_deltas();
        frame(pointer(option.center(), true)).drop_without_applying_deltas();
        frame(pointer(option.center(), false)).drop_without_applying_deltas();
        assert_eq!(turn.overcharge_level, if available { 3 } else { 1 });
        assert_eq!(turn.np_level, 4);
        assert_eq!(turn.affection_level, 7);
    }
}

#[test]
fn replacing_np_resets_unavailable_overcharge_with_visible_feedback() {
    let servant = full_overcharge_servant();
    let mut turn = TurnSelection::default_for(&servant).unwrap();
    turn.overcharge_level = 5;
    turn.np_level = 3;
    turn.affection_level = 7;
    let ctx = egui::Context::default();
    let candidate = SelectedCard::NoblePhantasm(1);
    let response = tile_frame(&ctx, vec![], &servant, &mut turn, 0, candidate);
    tile_frame(
        &ctx,
        pointer(response.rect.center(), true),
        &servant,
        &mut turn,
        0,
        candidate,
    );
    tile_frame(
        &ctx,
        pointer(response.rect.center(), false),
        &servant,
        &mut turn,
        0,
        candidate,
    );
    assert_eq!(turn.overcharge_level, 1);
    assert_eq!(turn.np_level, 3);
    assert_eq!(turn.affection_level, 7);
    let output = ctx.run_ui(input(328.0, vec![]), |ui| {
        np_settings(ui, &servant, &mut turn)
    });
    assert!(output.shapes.iter().any(|shape| {
        painted_text_rect(
            &shape.shape,
            "Overcharge reset from 5 to 1: this NP needs updated servant data for higher levels.",
        )
        .is_some()
    }));
    output.drop_without_applying_deltas();
}

#[test]
fn unavailable_overcharge_is_explicit_and_never_silently_changed_by_rendering() {
    let servant = servant();
    let mut turn = TurnSelection::default_for(&servant).unwrap();
    turn.overcharge_level = 5;
    let ctx = egui::Context::default();
    let output = ctx.run_ui(input(328.0, vec![]), |ui| {
        np_settings(ui, &servant, &mut turn)
    });
    assert!(output.shapes.iter().any(|shape| {
        painted_text_rect(
            &shape.shape,
            "Selected Overcharge is unavailable. Choose an available level or update servant data.",
        )
        .is_some()
    }));
    output.drop_without_applying_deltas();
    assert_eq!(turn.overcharge_level, 5);
    assert!(turn.validate(&servant).is_err());
}

#[test]
fn multiple_np_components_show_aggregate_and_separate_ranges_in_narrow_panels() {
    use fate_grand_calculator::np_mechanics::components::NpTarget;
    let mut servant = full_overcharge_servant();
    servant.noble_phantasms[0].components[0].target = Some(NpTarget::EnemyAll);
    let second_component = servant.noble_phantasms[0].components[0].clone();
    servant.noble_phantasms[0].components.push(second_component);
    let turn = TurnSelection::default_for(&servant).unwrap();
    let result = fate_grand_calculator::damage::calculate_turn(
        &servant,
        &turn,
        TurnBuffs::default(),
        ClassType::Lancer,
        AttributeType::Sky,
    )
    .unwrap();
    for width in [328.0, 760.0] {
        let ctx = egui::Context::default();
        let frame = |events| {
            ctx.run_ui(input(width, events), |ui| {
                ui.style_mut().animation_time = 0.0;
                apply_canvas(ui);
                result_panel(ui, &servant, &turn, &result, false);
                assert!(ui.min_rect().right() <= width + 1.0);
            })
        };
        let output = frame(vec![]);
        assert!(output.shapes.iter().any(|shape| {
            painted_text_rect(
                &shape.shape,
                "NP level 1 · Overcharge 1 · base damage ×6.00",
            )
            .is_some()
        }));
        let toggle = output
            .shapes
            .iter()
            .find_map(|shape| painted_text_rect(&shape.shape, "NP component breakdown"))
            .unwrap();
        output.drop_without_applying_deltas();
        frame(pointer(toggle.center(), true)).drop_without_applying_deltas();
        frame(pointer(toggle.center(), false)).drop_without_applying_deltas();
        for _ in 0..3 {
            frame(vec![]).drop_without_applying_deltas();
        }
        let output = frame(vec![]);
        for (index, component) in result.np_components[0].iter().enumerate() {
            let label = format!(
                "Component {} · {}",
                index + 1,
                format_damage_range(*component)
            );
            assert!(
                output
                    .shapes
                    .iter()
                    .any(|shape| painted_text_rect(&shape.shape, &label).is_some()),
                "missing {label}"
            );
        }
        output.drop_without_applying_deltas();
    }
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

fn low_hp_servant(max_hp: Option<u32>) -> ServantRecord {
    use fate_grand_calculator::np_mechanics::low_hp::LowHpScaling;
    let mut servant = full_overcharge_servant();
    servant.max_hp = max_hp;
    for row in servant.noble_phantasms[0].components[0]
        .overcharge
        .iter_mut()
        .flatten()
    {
        row.multipliers = [6.0; 5];
        row.low_hp = Some(LowHpScaling {
            source_base_rates: [6000; 5],
            coefficients: [6000; 5],
        });
    }
    servant
}

#[test]
fn hp_text_rejects_invalid_input_without_reusing_previous_damage() {
    let servant = low_hp_servant(Some(100));
    let mut turn = TurnSelection::default_for(&servant).unwrap();
    let ctx = egui::Context::default();
    for text in ["", "0", "-1", "1.5", "101", "4294967296", "+1", " 1"] {
        ctx.data_mut(|data| {
            data.insert_temp(
                hp_inputs_id(servant.id),
                HpInputs {
                    current: text.into(),
                    maximum: "100".into(),
                    reset_feedback: None,
                },
            )
        });
        let output = ctx.run_ui(input(328.0, vec![]), |ui| {
            hp_settings(ui, &servant, &mut turn)
        });
        output.drop_without_applying_deltas();
        assert_eq!(turn.attacker_hp, None, "invalid text {text:?}");
        assert!(
            fate_grand_calculator::damage::calculate_turn(
                &servant,
                &turn,
                TurnBuffs::default(),
                ClassType::Lancer,
                AttributeType::Sky,
            )
            .is_err(),
            "invalid text must suspend calculation"
        );
    }
    for maximum in ["", "0", "-1", "1.5", "4294967296"] {
        let inputs = HpInputs {
            current: "1".into(),
            maximum: maximum.into(),
            reset_feedback: None,
        };
        assert!(inputs.value().is_err(), "maximum {maximum:?}");
    }
    turn.slots[0] = SelectedCard::Normal(2);
    assert!(
        fate_grand_calculator::damage::calculate_turn(
            &servant,
            &turn,
            TurnBuffs::default(),
            ClassType::Lancer,
            AttributeType::Sky,
        )
        .is_ok(),
        "unused invalid HP must not block normal attacks"
    );
}

#[test]
fn hp_initialization_missing_metadata_and_level_changes_are_explicit() {
    for max_hp in [None, Some(101)] {
        let servant = low_hp_servant(max_hp);
        let mut turn = TurnSelection::default_for(&servant).unwrap();
        let ctx = egui::Context::default();
        let output = ctx.run_ui(input(328.0, vec![]), |ui| {
            np_settings(ui, &servant, &mut turn)
        });
        output.drop_without_applying_deltas();
        let initial = ctx
            .data(|data| data.get_temp::<HpInputs>(hp_inputs_id(servant.id)))
            .unwrap();
        assert_eq!(
            initial.maximum,
            max_hp.map_or_else(String::new, |hp| hp.to_string())
        );
        assert_eq!(initial.current, initial.maximum);
        assert_eq!(turn.attacker_hp.is_some(), max_hp.is_some());
        ctx.data_mut(|data| {
            data.insert_temp(
                hp_inputs_id(servant.id),
                HpInputs {
                    current: "1".into(),
                    maximum: "101".into(),
                    reset_feedback: None,
                },
            )
        });
        turn.np_level = 5;
        turn.overcharge_level = 5;
        let output = ctx.run_ui(input(328.0, vec![]), |ui| {
            np_settings(ui, &servant, &mut turn)
        });
        output.drop_without_applying_deltas();
        assert_eq!(turn.attacker_hp, Some(AttackerHp::new(1, 101).unwrap()));
        let mut inputs = ctx
            .data(|data| data.get_temp::<HpInputs>(hp_inputs_id(servant.id)))
            .unwrap();
        inputs.maximum = "102".into();
        inputs.maximum_changed();
        assert_eq!(inputs.current, "102");
        assert!(inputs.reset_feedback.unwrap().contains("reset to full HP"));
    }
}

#[test]
fn hp_controls_and_effective_breakdown_fit_narrow_panels() {
    let servant = low_hp_servant(Some(100));
    for width in [214.0, 280.0, 328.0, 760.0] {
        let mut turn = TurnSelection::default_for(&servant).unwrap();
        let ctx = egui::Context::default();
        let output = ctx.run_ui(input(width, vec![]), |ui| {
            apply_canvas(ui);
            np_settings(ui, &servant, &mut turn);
            assert!(ui.min_rect().right() <= width + 1.0);
        });
        assert!(
            output.shapes.iter().any(|shape| painted_text_rect(
                &shape.shape,
                "HP at NP damage time"
            )
            .is_some())
        );
        output.drop_without_applying_deltas();
        turn.attacker_hp = Some(AttackerHp::new(50, 100).unwrap());
        let result = fate_grand_calculator::damage::calculate_turn(
            &servant,
            &turn,
            TurnBuffs::default(),
            ClassType::Lancer,
            AttributeType::Sky,
        )
        .unwrap();
        let output = ctx.run_ui(input(width, vec![]), |ui| {
            apply_canvas(ui);
            result_panel(ui, &servant, &turn, &result, false);
            assert!(ui.min_rect().right() <= width + 1.0);
        });
        assert!(output.shapes.iter().any(|shape| {
            painted_text_rect(
                &shape.shape,
                "Base ×6.000 + missing-HP bonus ×3.000 = effective ×9.000",
            )
            .is_some()
        }));
        output.drop_without_applying_deltas();
    }
}

#[test]
fn typing_invalid_current_hp_clears_the_calculated_state_in_the_same_frame() {
    let servant = low_hp_servant(Some(100));
    let mut turn = TurnSelection::default_for(&servant).unwrap();
    let ctx = egui::Context::default();
    let output = ctx.run_ui(input(328.0, vec![]), |ui| {
        hp_settings(ui, &servant, &mut turn);
    });
    let current_position = output
        .shapes
        .iter()
        .rev()
        .find_map(|shape| painted_text_rect(&shape.shape, "100"))
        .expect("current HP is painted")
        .center();
    output.drop_without_applying_deltas();
    assert_eq!(turn.attacker_hp, Some(AttackerHp::new(100, 100).unwrap()));
    for pressed in [true, false] {
        ctx.run_ui(input(328.0, pointer(current_position, pressed)), |ui| {
            hp_settings(ui, &servant, &mut turn);
        })
        .drop_without_applying_deltas();
    }
    let select_all = egui::Event::Key {
        key: egui::Key::A,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        },
    };
    let output = ctx.run_ui(
        input(328.0, vec![select_all, egui::Event::Text("101".into())]),
        |ui| {
            hp_settings(ui, &servant, &mut turn);
        },
    );
    output.drop_without_applying_deltas();
    let text = ctx
        .data(|data| data.get_temp::<HpInputs>(hp_inputs_id(servant.id)))
        .unwrap();
    assert_eq!(text.current, "101");
    assert_eq!(turn.attacker_hp, None);
}
