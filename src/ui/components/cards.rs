//! The three attack positions and the card tiles that fill them.

use super::*;

pub(super) fn turn_selector(
    ui: &mut egui::Ui,
    servant: &ServantRecord,
    selection: &mut Option<TurnSelection>,
    active_slot: &mut usize,
) {
    ui.label(RichText::new("ATTACK ORDER").size(13.0).color(TEXT_MUTED));
    ui.add_space(6.0);
    if servant.deck.len() != 5 {
        ui.label(
            RichText::new(servant.deck_note.as_deref().unwrap_or(
                "Card data is unavailable. Update servant data to load this servant's deck.",
            ))
            .color(ERROR),
        );
        return;
    }
    match servant.np_status {
        NpStatus::Support => {
            ui.label(
                RichText::new("This servant's NP does no damage. Choose three command cards.")
                    .size(12.0)
                    .color(TEXT_MUTED),
            );
        }
        NpStatus::Unavailable => {
            ui.label(RichText::new("NP data is unavailable locally. Choose three command cards or update servant data.").size(12.0).color(TEXT_MUTED));
        }
        NpStatus::Unsupported => {
            ui.label(RichText::new("This NP's damage cannot be estimated with the current data. Choose three command cards.").size(12.0).color(TEXT_MUTED));
        }
        NpStatus::Damaging => {}
    }
    let Some(turn) = selection.as_mut() else {
        ui.label(RichText::new("Select a servant with available card data.").color(ERROR));
        return;
    };
    *active_slot = (*active_slot).min(2);
    ui.label(
        RichText::new("Select a card to edit")
            .size(13.0)
            .color(TEXT_MUTED),
    );
    let gap = ui.spacing().item_spacing.x;
    let slot_width = ((ui.available_width() - gap * 2.0) / 3.0).max(1.0);
    ui.horizontal(|ui| {
        for slot in 0..3 {
            let editing = *active_slot == slot;
            let card = turn.slots[slot];
            let response = card_button(
                ui,
                egui::vec2(slot_width, 80.0),
                selected_card_symbol(servant, card),
                selected_card_color(servant, card),
                editing,
                true,
            );
            ui.painter().text(
                response.rect.left_top() + egui::vec2(10.0, 8.0),
                egui::Align2::LEFT_TOP,
                (slot + 1).to_string(),
                egui::FontId::proportional(12.0),
                Color32::WHITE,
            );
            response.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::WidgetType::Button,
                    ui.is_enabled(),
                    editing,
                    format!(
                        "Attack position {}: {}{}",
                        slot + 1,
                        selected_card_label(servant, turn.slots[slot]),
                        if editing { ", editing" } else { "" }
                    ),
                )
            });
            if response
                .on_hover_text(selected_card_label(servant, turn.slots[slot]))
                .clicked()
            {
                *active_slot = slot;
            }
        }
    });
    ui.add_space(12.0);
    egui::Frame::new()
        .fill(PANEL_MUTED)
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(59, 69, 91)))
        .corner_radius(10)
        .inner_margin(12)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new(format!("Replace card {}", *active_slot + 1))
                    .size(15.0)
                    .strong()
                    .color(ACCENT),
            );
            ui.label(
                RichText::new("Choose a card below.")
                    .size(12.0)
                    .color(TEXT_MUTED),
            );
            let columns = if ui.available_width() >= 250.0 { 3 } else { 2 };
            let tile_width = ((ui.available_width() - gap * (columns - 1) as f32) / columns as f32)
                .clamp(1.0, 130.0);
            egui::Grid::new("attack_card_tiles")
                .num_columns(columns)
                .spacing([gap, gap])
                .show(ui, |ui| {
                    let mut tile_index = 0;
                    if servant.np_status == NpStatus::Damaging {
                        for (index, _) in servant.noble_phantasms.iter().enumerate() {
                            card_tile(
                                ui,
                                servant,
                                turn,
                                *active_slot,
                                SelectedCard::NoblePhantasm(index),
                                tile_width,
                            );
                            tile_index += 1;
                            if tile_index % columns == 0 {
                                ui.end_row();
                            }
                        }
                    }
                    for index in 0..servant.deck.len() {
                        card_tile(
                            ui,
                            servant,
                            turn,
                            *active_slot,
                            SelectedCard::Normal(index),
                            tile_width,
                        );
                        tile_index += 1;
                        if tile_index % columns == 0 {
                            ui.end_row();
                        }
                    }
                });
            ui.label(
                RichText::new("Cards marked In 1, In 2 or In 3 are used in another position.")
                    .size(12.0)
                    .color(TEXT_MUTED),
            );
        });
    ui.add_space(8.0);
    np_settings(ui, servant, turn);
}

pub(super) fn card_owner(turn: &TurnSelection, candidate: SelectedCard) -> Option<usize> {
    turn.slots
        .iter()
        .position(|selected| match (candidate, *selected) {
            (SelectedCard::NoblePhantasm(_), SelectedCard::NoblePhantasm(_)) => true,
            _ => candidate == *selected,
        })
}

pub(super) fn card_tile(
    ui: &mut egui::Ui,
    servant: &ServantRecord,
    turn: &mut TurnSelection,
    active_slot: usize,
    candidate: SelectedCard,
    width: f32,
) -> egui::Response {
    let owner = card_owner(turn, candidate);
    let available = owner.is_none_or(|slot| slot == active_slot);
    let current = turn.slots[active_slot] == candidate;
    let state = if current {
        "Current".to_owned()
    } else if owner == Some(active_slot) {
        "Replace NP".to_owned()
    } else if let Some(slot) = owner {
        format!("In slot {}", slot + 1)
    } else {
        "Choose".to_owned()
    };
    let detail = match candidate {
        SelectedCard::Normal(index) => format!("Deck {}", index + 1),
        SelectedCard::NoblePhantasm(index) => format!("Variant {}", index + 1),
    };
    let full_label = selected_card_label(servant, candidate);
    let response = card_button(
        ui,
        egui::vec2(width, 96.0),
        selected_card_symbol(servant, candidate),
        selected_card_color(servant, candidate),
        current,
        available,
    );
    let badge = if current {
        "Selected".to_owned()
    } else if owner == Some(active_slot) {
        "Replace".to_owned()
    } else if let Some(slot) = owner {
        format!("In {}", slot + 1)
    } else {
        "Use".to_owned()
    };
    let caption_color = if available {
        Color32::WHITE
    } else {
        TEXT_MUTED
    };
    ui.painter().text(
        response.rect.center_top() + egui::vec2(0.0, 10.0),
        egui::Align2::CENTER_TOP,
        badge,
        egui::FontId::proportional(10.0),
        caption_color,
    );
    ui.painter().text(
        response.rect.center_bottom() - egui::vec2(0.0, 10.0),
        egui::Align2::CENTER_BOTTOM,
        detail,
        egui::FontId::proportional(11.0),
        caption_color,
    );
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::Button,
            available && ui.is_enabled(),
            current,
            format!(
                "{full_label}, {state}, for attack position {}",
                active_slot + 1
            ),
        )
    });
    let response = response.on_hover_text(format!("{full_label} · {state}"));
    if response.clicked() {
        turn.slots[active_slot] = candidate;
        ui.ctx().data_mut(|data| {
            data.remove::<String>(overcharge_feedback_id(servant.id));
        });
        if let SelectedCard::NoblePhantasm(index) = candidate
            && let Some(np) = servant.noble_phantasms.get(index)
            && !np.available_overcharges().contains(&turn.overcharge_level)
        {
            let previous = turn.overcharge_level;
            turn.overcharge_level = 1;
            ui.ctx().data_mut(|data| data.insert_temp(overcharge_feedback_id(servant.id), format!("Overcharge reset from {previous} to 1: this NP needs updated servant data for higher levels.")));
        }
    }
    response
}

pub(super) fn selected_card_symbol(servant: &ServantRecord, card: SelectedCard) -> &'static str {
    match card {
        SelectedCard::NoblePhantasm(_) => "NP",
        SelectedCard::Normal(index) => match servant.deck[index] {
            CardType::Buster => "B",
            CardType::Arts => "A",
            CardType::Quick => "Q",
            CardType::Extra => "EX",
        },
    }
}

pub(super) fn selected_card_color(servant: &ServantRecord, card: SelectedCard) -> Color32 {
    match card {
        SelectedCard::NoblePhantasm(_) => Color32::from_rgb(85, 83, 148),
        SelectedCard::Normal(index) => card_color(servant.deck[index]),
    }
}

pub(super) fn card_button(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    symbol: &str,
    color: Color32,
    selected: bool,
    enabled: bool,
) -> egui::Response {
    let fill = if !enabled {
        PANEL_MUTED
    } else if selected {
        color
    } else {
        color.gamma_multiply(0.45)
    };
    let response = ui
        .add_enabled_ui(enabled, |ui| {
            ui.add_sized(
                size,
                egui::Button::new("")
                    .fill(fill)
                    .corner_radius(10)
                    .stroke(egui::Stroke::new(
                        if selected { 3.0 } else { 1.0 },
                        if selected { ACCENT } else { color },
                    ))
                    .selected(selected),
            )
        })
        .inner;
    ui.painter().text(
        response.rect.center(),
        egui::Align2::CENTER_CENTER,
        symbol,
        egui::FontId::proportional(28.0),
        if enabled { Color32::WHITE } else { TEXT_MUTED },
    );
    card_focus_outline(ui, &response);
    response
}

pub(super) fn card_focus_outline(ui: &egui::Ui, response: &egui::Response) {
    if response.enabled() && (response.has_focus() || response.hovered()) {
        ui.painter().rect_stroke(
            response.rect,
            8.0,
            egui::Stroke::new(if response.has_focus() { 3.0 } else { 1.5 }, ACCENT),
            egui::StrokeKind::Inside,
        );
    }
}

pub(super) fn card_name(card: CardType) -> &'static str {
    match card {
        CardType::Buster => "Buster",
        CardType::Arts => "Arts",
        CardType::Quick => "Quick",
        CardType::Extra => "Extra",
    }
}

pub(super) fn card_color(card: CardType) -> Color32 {
    match card {
        CardType::Buster => BUSTER,
        CardType::Arts => ARTS,
        CardType::Quick => QUICK,
        CardType::Extra => ACCENT,
    }
}

pub(super) fn selected_card_label(servant: &ServantRecord, card: SelectedCard) -> String {
    match card {
        SelectedCard::Normal(index) => servant.deck.get(index).map_or_else(
            || "Unavailable card".to_owned(),
            |color| format!("{} · deck card {}", card_name(*color), index + 1),
        ),
        SelectedCard::NoblePhantasm(index) => servant.noble_phantasms.get(index).map_or_else(
            || "Unavailable NP".to_owned(),
            |np| format!("NP · {}", np.name),
        ),
    }
}

pub(super) fn selected_card_short_label(servant: &ServantRecord, card: SelectedCard) -> String {
    match card {
        SelectedCard::Normal(index) => servant
            .deck
            .get(index)
            .map_or_else(|| "Card".to_owned(), |color| card_name(*color).to_owned()),
        SelectedCard::NoblePhantasm(_) => "NP".to_owned(),
    }
}
