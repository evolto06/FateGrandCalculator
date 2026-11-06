//! The three attack positions and the card tiles that fill them.

use super::*;

pub(super) fn turn_selector(
    ui: &mut egui::Ui,
    servant: &ServantRecord,
    selection: &mut Option<TurnSelection>,
    active_slot: &mut usize,
) {
    group_label(ui, "ATTACK ORDER");
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
            note(
                ui,
                "This servant's NP does no damage. Choose three command cards.",
            );
        }
        NpStatus::Unavailable => {
            note(
                ui,
                "NP data is unavailable locally. Choose three command cards or update servant data.",
            );
        }
        NpStatus::Unsupported => {
            note(
                ui,
                "This NP's damage cannot be estimated with the current data. Choose three command cards.",
            );
        }
        NpStatus::Damaging => {}
    }
    let Some(turn) = selection.as_mut() else {
        ui.label(RichText::new("Select a servant with available card data.").color(ERROR));
        return;
    };
    *active_slot = (*active_slot).min(2);
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
                egui::FontId::proportional(TEXT_SMALL),
                TEXT,
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
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(10)
        .inner_margin(12)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new(format!("Card for position {}", *active_slot + 1))
                    .size(TEXT_SUBHEADING)
                    .strong()
                    .color(ACCENT),
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

/// What choosing a tile would do for the position being edited. The badge,
/// the hover text and the screen-reader label all use these words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TileState {
    /// Already in the position being edited.
    Selected,
    /// Another NP variant is in this position; choosing swaps the variant.
    ReplacesNp,
    /// Used by another position, so it cannot be chosen here.
    InPosition(usize),
    Available,
}

impl TileState {
    pub(super) fn of(turn: &TurnSelection, active_slot: usize, candidate: SelectedCard) -> Self {
        match card_owner(turn, candidate) {
            _ if turn.slots[active_slot] == candidate => Self::Selected,
            Some(slot) if slot == active_slot => Self::ReplacesNp,
            Some(slot) => Self::InPosition(slot + 1),
            None => Self::Available,
        }
    }

    pub(super) fn label(self) -> String {
        match self {
            Self::Selected => "Selected".into(),
            Self::ReplacesNp => "Swap NP".into(),
            Self::InPosition(position) => format!("In position {position}"),
            Self::Available => "Available".into(),
        }
    }

    pub(super) fn can_choose(self) -> bool {
        !matches!(self, Self::InPosition(_))
    }
}

pub(super) fn card_tile(
    ui: &mut egui::Ui,
    servant: &ServantRecord,
    turn: &mut TurnSelection,
    active_slot: usize,
    candidate: SelectedCard,
    width: f32,
) -> egui::Response {
    let state = TileState::of(turn, active_slot, candidate);
    let available = state.can_choose();
    let current = state == TileState::Selected;
    let full_label = selected_card_label(servant, candidate);
    let response = card_button(
        ui,
        egui::vec2(width, 96.0),
        selected_card_symbol(servant, candidate),
        selected_card_color(servant, candidate),
        current,
        available,
    );
    let caption_color = if available { TEXT } else { TEXT_MUTED };
    let caption_width = (response.rect.width() - 8.0).max(1.0);
    for (text, anchor, offset) in [
        (
            state.label(),
            egui::Align2::CENTER_TOP,
            response.rect.center_top() + egui::vec2(0.0, 8.0),
        ),
        (
            tile_caption(servant, candidate),
            egui::Align2::CENTER_BOTTOM,
            response.rect.center_bottom() - egui::vec2(0.0, 8.0),
        ),
    ] {
        let galley = ui.painter().layout(
            text,
            egui::FontId::proportional(TEXT_SMALL),
            caption_color,
            caption_width,
        );
        let rect = anchor.anchor_size(offset, galley.size());
        ui.painter().galley(rect.min, galley, caption_color);
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::Button,
            available && ui.is_enabled(),
            current,
            format!(
                "{full_label}, {}, for attack position {}",
                state.label(),
                active_slot + 1
            ),
        )
    });
    let response = response.on_hover_text(format!("{full_label} · {}", state.label()));
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

/// The line under a tile's symbol: the deck position, or which NP variant it is.
pub(super) fn tile_caption(servant: &ServantRecord, card: SelectedCard) -> String {
    match card {
        SelectedCard::Normal(index) => format!("Deck {}", index + 1),
        SelectedCard::NoblePhantasm(index) => servant
            .noble_phantasms
            .get(index)
            .and_then(|np| {
                let variant = np.name.rsplit_once(" (")?.1.strip_suffix(')')?;
                let mut chars = variant.chars();
                let first = chars.next()?;
                Some(first.to_uppercase().chain(chars).collect())
            })
            .unwrap_or_else(|| format!("Variant {}", index + 1)),
    }
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

/// A card's outline color and its fill when selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CardColors {
    pub(super) outline: Color32,
    pub(super) fill: Color32,
}

pub(super) fn selected_card_color(servant: &ServantRecord, card: SelectedCard) -> CardColors {
    match card {
        SelectedCard::NoblePhantasm(_) => CardColors {
            outline: NP,
            fill: NP_FILL,
        },
        SelectedCard::Normal(index) => card_color(servant.deck[index]),
    }
}

pub(super) fn card_button(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    symbol: &str,
    colors: CardColors,
    selected: bool,
    enabled: bool,
) -> egui::Response {
    let (fill, stroke) = if !enabled {
        (BACKGROUND, egui::Stroke::new(1.0, BORDER))
    } else if selected {
        (colors.fill, egui::Stroke::new(3.0, ACCENT))
    } else {
        (
            colors.fill.gamma_multiply(0.45),
            egui::Stroke::new(1.0, colors.outline),
        )
    };
    let response = ui
        .add_enabled_ui(enabled, |ui| {
            ui.add_sized(
                size,
                egui::Button::new("")
                    .fill(fill)
                    .corner_radius(10)
                    .stroke(stroke)
                    .selected(selected),
            )
        })
        .inner;
    ui.painter().text(
        response.rect.center(),
        egui::Align2::CENTER_CENTER,
        symbol,
        egui::FontId::proportional(TEXT_CARD_SYMBOL),
        if enabled { TEXT } else { TEXT_MUTED },
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

pub(super) fn card_color(card: CardType) -> CardColors {
    let (outline, fill) = match card {
        CardType::Buster => (BUSTER, BUSTER_FILL),
        CardType::Arts => (ARTS, ARTS_FILL),
        CardType::Quick => (QUICK, QUICK_FILL),
        CardType::Extra => (ACCENT, RESULT_BORDER),
    };
    CardColors { outline, fill }
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
