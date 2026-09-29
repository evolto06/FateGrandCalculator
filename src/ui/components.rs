use eframe::egui::{self, Color32, RichText};
use fate_grand_calculator::damage::{
    DamageResult, TurnBuffs, TurnDamageResult, percent_to_modifier,
};
use fate_grand_calculator::loader::{GameData, NpStatus, ServantRecord};
use fate_grand_calculator::model::{
    AttributeType, CardType, ClassType, SelectedCard, TurnSelection,
};

use super::theme::{
    ACCENT, ARTS, BACKGROUND, BUSTER, ERROR, PANEL, PANEL_MUTED, QUICK, RESULT_PANEL, TEXT_MUTED,
};

pub fn apply_canvas(ui: &mut egui::Ui) {
    ui.painter().rect_filled(ui.max_rect(), 0.0, BACKGROUND);
    ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);
    ui.spacing_mut().interact_size = egui::vec2(44.0, 44.0);
    ui.visuals_mut().widgets.active.bg_stroke = egui::Stroke::new(2.0, ACCENT);
    ui.visuals_mut().widgets.hovered.bg_stroke = egui::Stroke::new(1.5, ACCENT);
}

pub fn header(ui: &mut egui::Ui) {
    ui.add_space(14.0);
    ui.horizontal_wrapped(|ui| {
        ui.vertical(|ui| {
            let title_size = if ui.available_width() >= 420.0 {
                27.0
            } else {
                22.0
            };
            ui.heading(
                RichText::new("FateGrandCalculator")
                    .size(title_size)
                    .strong(),
            );
            ui.label(
                RichText::new("Unofficial FGO damage calculator")
                    .size(14.0)
                    .color(TEXT_MUTED),
            );
        });
        ui.add_space(12.0);
        ui.label(
            RichText::new("THREE CARDS · EARLY BUILD")
                .size(12.0)
                .color(ACCENT),
        );
    });
}

pub fn attack_panel(
    ui: &mut egui::Ui,
    data: &GameData,
    selected_servant_id: &mut u32,
    servant_search: &mut String,
    portrait: Option<&egui::TextureHandle>,
    portrait_message: &str,
    portrait_error: Option<&str>,
    selection: &mut Option<TurnSelection>,
    buffs: &mut TurnBuffInputs,
) {
    section_frame(ui, |ui| {
        ui.heading(RichText::new("Attack setup").size(18.0));
        ui.label(RichText::new("Choose three cards in attack order.").color(TEXT_MUTED));
        ui.add_space(14.0);

        let previous_servant_id = *selected_servant_id;
        servant_selector(
            ui,
            data,
            selected_servant_id,
            servant_search,
            portrait,
            portrait_message,
            portrait_error,
        );
        if *selected_servant_id != previous_servant_id {
            *selection = data
                .servant(*selected_servant_id)
                .and_then(TurnSelection::default_for);
        }
        if portrait_error.is_some() {
            let response =
                ui.colored_label(ERROR, "Portrait unavailable. Change servant to try again.");
            mark_live_status(ui.ctx(), response.id);
        }
        ui.add_space(12.0);

        if let Some(servant) = data.servant(*selected_servant_id) {
            turn_selector(ui, servant, selection);
        }
        ui.add_space(14.0);
        ui.separator();
        ui.add_space(8.0);
        ui.label(RichText::new("BUFFS & TARGET").size(13.0).color(TEXT_MUTED));
        ui.label(
            RichText::new("Enter 20 for a 20% value.")
                .size(13.0)
                .color(TEXT_MUTED),
        );
        ui.add_space(4.0);
        percent_input(ui, "Attack buff (%)", &mut buffs.attack);
        percent_input(ui, "Buster buff (%)", &mut buffs.buster);
        percent_input(ui, "Arts buff (%)", &mut buffs.arts);
        percent_input(ui, "Quick buff (%)", &mut buffs.quick);
        percent_input(ui, "NP damage buff (%)", &mut buffs.np_damage);
        percent_input(ui, "Enemy defense (%)", &mut buffs.enemy_defense);
    });
}

pub fn matchup_panel(
    ui: &mut egui::Ui,
    enemy_class: &mut ClassType,
    enemy_attribute: &mut AttributeType,
) {
    section_frame(ui, |ui| {
        ui.heading(RichText::new("Matchup").size(18.0));
        ui.label(RichText::new("Set the enemy's class and attribute.").color(TEXT_MUTED));
        ui.add_space(14.0);

        class_selector(ui, "Enemy class", enemy_class);
        if *enemy_class == ClassType::Beast {
            ui.label(
                RichText::new("Beast affinity depends on the specific encounter.")
                    .size(13.0)
                    .color(TEXT_MUTED),
            );
        }
        attribute_selector(ui, "Enemy attribute", enemy_attribute);
    });
}

pub fn result_panel(
    ui: &mut egui::Ui,
    servant: &ServantRecord,
    selection: &TurnSelection,
    result: &TurnDamageResult,
    announce_result: bool,
) {
    section_frame(ui, |ui| {
        ui.heading(RichText::new("Damage by card").size(18.0));
        ui.label(
            RichText::new(
                "Non-critical damage to the selected enemy; each card has its own range.",
            )
            .color(TEXT_MUTED),
        );
        ui.add_space(14.0);
        for (index, card) in selection.slots.iter().enumerate() {
            ui.push_id(index, |ui| {
                ui.label(
                    RichText::new(format!(
                        "{}. {}",
                        index + 1,
                        selected_card_label(servant, *card)
                    ))
                    .strong(),
                );
                if let SelectedCard::NoblePhantasm(np_index) = card {
                    if let Some(np) = servant.noble_phantasms.get(*np_index) {
                        let multiplier = np.multipliers[selection.np_level as usize - 1];
                        ui.label(
                            RichText::new(format!(
                                "NP level {} · base damage ×{multiplier:.2}",
                                selection.np_level
                            ))
                            .size(12.0)
                            .color(TEXT_MUTED),
                        );
                        if let Some(scale) = &np.affection {
                            ui.label(
                                RichText::new(format!(
                                    "Affection level {} · damage ×{:.2}",
                                    selection.affection_level,
                                    scale.multiplier(selection.affection_level)
                                ))
                                .size(12.0)
                                .color(TEXT_MUTED),
                            );
                        }
                    }
                }
                damage_result(ui, result.cards[index], announce_result);
                breakdown(ui, result.cards[index]);
                ui.add_space(8.0);
            });
        }
        if let Some(extra) = result.extra {
            ui.separator();
            ui.label(RichText::new("Extra attack · Brave Chain").strong());
            damage_result(ui, extra, announce_result);
            breakdown(ui, extra);
        }
        for note in &result.notes {
            ui.label(RichText::new(note).size(12.0).color(TEXT_MUTED));
        }
    });
}

#[derive(Debug, Clone)]
pub struct PercentInput {
    text: String,
    value: f64,
}

impl Default for PercentInput {
    fn default() -> Self {
        Self {
            text: "0".into(),
            value: 0.0,
        }
    }
}

impl PercentInput {
    pub fn value(&self) -> f64 {
        self.value
    }
}

#[derive(Debug, Clone, Default)]
pub struct TurnBuffInputs {
    pub attack: PercentInput,
    pub buster: PercentInput,
    pub arts: PercentInput,
    pub quick: PercentInput,
    pub np_damage: PercentInput,
    pub enemy_defense: PercentInput,
}

impl TurnBuffInputs {
    pub fn values(&self) -> TurnBuffs {
        TurnBuffs {
            attack_buff: percent_to_modifier(self.attack.value()),
            buster_buff: percent_to_modifier(self.buster.value()),
            arts_buff: percent_to_modifier(self.arts.value()),
            quick_buff: percent_to_modifier(self.quick.value()),
            np_damage_buff: percent_to_modifier(self.np_damage.value()),
            enemy_defense: percent_to_modifier(self.enemy_defense.value()),
        }
    }
}

pub fn status_message(ui: &mut egui::Ui, text: &str) {
    let response = ui.label(text);
    mark_live_status(ui.ctx(), response.id);
}

pub fn sticky_result_summary(
    ui: &mut egui::Ui,
    servant: &ServantRecord,
    selection: &TurnSelection,
    result: &TurnDamageResult,
) {
    egui::Frame::new()
        .fill(RESULT_PANEL)
        .stroke(egui::Stroke::new(1.5, ACCENT))
        .corner_radius(10)
        .inner_margin(12)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("DAMAGE BY CARD").size(12.0).color(ACCENT));
            for (index, card) in selection.slots.iter().enumerate() {
                let response = ui.label(format!(
                    "{} · {}: {}",
                    index + 1,
                    selected_card_short_label(servant, *card),
                    format_damage_range(result.cards[index])
                ));
                mark_live_status(ui.ctx(), response.id);
            }
            if let Some(extra) = result.extra {
                ui.label(format!("Extra: {}", format_damage_range(extra)));
            }
        });
}

fn mark_live_status(context: &egui::Context, id: egui::Id) {
    context.accesskit_node_builder(id, |node| {
        node.set_role(egui::accesskit::Role::Status);
        node.set_live(egui::accesskit::Live::Polite);
        node.set_live_atomic();
    });
}

fn section_frame(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(PANEL)
        .corner_radius(12)
        .inner_margin(16)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add_contents(ui);
        });
}

fn turn_selector(
    ui: &mut egui::Ui,
    servant: &ServantRecord,
    selection: &mut Option<TurnSelection>,
) {
    ui.label(RichText::new("SERVANT CARDS").size(13.0).color(TEXT_MUTED));
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
    ui.horizontal_wrapped(|ui| {
        for (index, card) in servant.deck.iter().enumerate() {
            let used = selection
                .as_ref()
                .is_some_and(|turn| turn.slots.contains(&SelectedCard::Normal(index)));
            let color = card_color(*card);
            egui::Frame::new()
                .fill(if used { color } else { PANEL_MUTED })
                .corner_radius(7)
                .inner_margin(egui::Margin::symmetric(9, 7))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(format!("{} {}", card_name(*card), index + 1))
                            .color(Color32::WHITE),
                    );
                });
        }
    });
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
    ui.add_space(8.0);
    for slot in 0..3 {
        let slot_label = ui.label(format!("Card {}", slot + 1));
        let selected_text = selected_card_label(servant, turn.slots[slot]);
        let combo = egui::ComboBox::from_id_salt(("turn_slot", slot))
            .selected_text(selected_text)
            .width(ui.available_width().min(350.0))
            .wrap_mode(egui::TextWrapMode::Truncate)
            .show_ui(ui, |ui| {
                for (index, card) in servant.deck.iter().enumerate() {
                    let candidate = SelectedCard::Normal(index);
                    let available = !turn
                        .slots
                        .iter()
                        .enumerate()
                        .any(|(other, selected)| other != slot && *selected == candidate);
                    if ui
                        .add_enabled(
                            available,
                            egui::Button::new(format!(
                                "{} · deck card {}",
                                card_name(*card),
                                index + 1
                            )),
                        )
                        .clicked()
                    {
                        turn.slots[slot] = candidate;
                    }
                }
                if servant.np_status == NpStatus::Damaging {
                    ui.separator();
                    let np_used_elsewhere = turn.slots.iter().enumerate().any(|(other, card)| {
                        other != slot && matches!(card, SelectedCard::NoblePhantasm(_))
                    });
                    for (index, np) in servant.noble_phantasms.iter().enumerate() {
                        if ui
                            .add_enabled(
                                !np_used_elsewhere,
                                egui::Button::new(format!("NP · {}", np.name)),
                            )
                            .clicked()
                        {
                            turn.slots[slot] = SelectedCard::NoblePhantasm(index);
                        }
                    }
                }
            });
        combo.response.labelled_by(slot_label.id);
    }
    if turn
        .slots
        .iter()
        .any(|card| matches!(card, SelectedCard::NoblePhantasm(_)))
    {
        let level_label = ui.label("NP level");
        let combo = egui::ComboBox::from_id_salt("np_level")
            .selected_text(turn.np_level.to_string())
            .width(100.0)
            .show_ui(ui, |ui| {
                for level in 1..=5 {
                    ui.selectable_value(&mut turn.np_level, level, level.to_string());
                }
            });
        combo.response.labelled_by(level_label.id);
        if let Some(scale) = turn.slots.iter().find_map(|card| match card {
            SelectedCard::NoblePhantasm(index) => servant
                .noble_phantasms
                .get(*index)
                .and_then(|np| np.affection.as_ref()),
            SelectedCard::Normal(_) => None,
        }) {
            let affection_label = ui.label("Affection level at NP damage");
            let combo = egui::ComboBox::from_id_salt("affection_level")
                .selected_text(turn.affection_level.to_string())
                .width(100.0)
                .show_ui(ui, |ui| {
                    for level in 0..=scale.max_level {
                        ui.selectable_value(&mut turn.affection_level, level, level.to_string());
                    }
                });
            combo.response.labelled_by(affection_label.id);
            ui.label(
                RichText::new("Each level adds 10% NP damage; level 0 has no affection bonus. At level 7+, this NP ignores enemy defense. Enter the level after any Overcharge gain.")
                    .size(12.0)
                    .color(TEXT_MUTED),
            );
        }
    }
}

fn card_name(card: CardType) -> &'static str {
    match card {
        CardType::Buster => "Buster",
        CardType::Arts => "Arts",
        CardType::Quick => "Quick",
        CardType::Extra => "Extra",
    }
}

fn card_color(card: CardType) -> Color32 {
    match card {
        CardType::Buster => BUSTER,
        CardType::Arts => ARTS,
        CardType::Quick => QUICK,
        CardType::Extra => ACCENT,
    }
}

fn selected_card_label(servant: &ServantRecord, card: SelectedCard) -> String {
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

fn selected_card_short_label(servant: &ServantRecord, card: SelectedCard) -> String {
    match card {
        SelectedCard::Normal(index) => servant
            .deck
            .get(index)
            .map_or_else(|| "Card".to_owned(), |color| card_name(*color).to_owned()),
        SelectedCard::NoblePhantasm(_) => "NP".to_owned(),
    }
}

fn percent_input(ui: &mut egui::Ui, label: &str, value: &mut PercentInput) {
    let label_response = ui.label(RichText::new(label).size(14.0).color(TEXT_MUTED));
    let input_width = ui.available_width().min(220.0);
    let response = ui.add_sized(
        [input_width, 44.0],
        egui::TextEdit::singleline(&mut value.text)
            .id_salt(("percent_input", label))
            .desired_width(input_width)
            .hint_text("0")
            .suffix("%")
            .text_color(Color32::WHITE)
            .frame(
                egui::Frame::new()
                    .fill(PANEL_MUTED)
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(64, 75, 97)))
                    .corner_radius(6)
                    .inner_margin(egui::Margin::symmetric(10, 6)),
            ),
    );
    let response = response.labelled_by(label_response.id);

    let trimmed = value.text.trim();
    let invalid = if trimmed.is_empty() {
        value.value = 0.0;
        false
    } else {
        match trimmed.parse::<f64>() {
            Ok(parsed) if parsed.is_finite() => {
                value.value = parsed.clamp(-100.0, 999.0);
                if response.lost_focus() {
                    value.text = value.value.to_string();
                }
                false
            }
            _ => true,
        }
    };
    if invalid {
        ui.colored_label(ERROR, "Enter a number from −100 to 999.");
    }
}

fn servant_selector(
    ui: &mut egui::Ui,
    data: &GameData,
    selected_id: &mut u32,
    search: &mut String,
    portrait: Option<&egui::TextureHandle>,
    portrait_message: &str,
    portrait_error: Option<&str>,
) {
    let selected = data.servant(*selected_id).or_else(|| data.servants.first());
    let selected_name = selected.map_or("No servants loaded", |servant| servant.name.as_str());

    ui.label(RichText::new("SERVANT").size(13.0).color(TEXT_MUTED));
    ui.add_space(6.0);
    ui.horizontal_top(|ui| {
        portrait_window(ui, portrait, portrait_message, portrait_error);
        ui.vertical(|ui| {
            ui.label(RichText::new(selected_name).strong());
            let search_label = ui.label("Search servants");
            let search_response = ui
                .add(
                    egui::TextEdit::singleline(search)
                        .id_salt("servant_search")
                        .desired_width(ui.available_width())
                        .hint_text("Type a servant name"),
                )
                .labelled_by(search_label.id);
            if search_response.has_focus() || !search.is_empty() {
                let query = search.trim().to_lowercase();
                let mut matches = 0;
                egui::ScrollArea::vertical()
                    .id_salt("servant_search_results")
                    .max_height(180.0)
                    .show(ui, |ui| {
                        for servant in &data.servants {
                            if servant.name.to_lowercase().contains(&query) {
                                matches += 1;
                                if ui
                                    .selectable_label(*selected_id == servant.id, &servant.name)
                                    .clicked()
                                {
                                    *selected_id = servant.id;
                                    search.clear();
                                    ui.memory_mut(|memory| {
                                        memory.surrender_focus(search_response.id)
                                    });
                                }
                            }
                        }
                    });
                if matches == 0 {
                    ui.label("No servants match that search.");
                }
            }

            if let Some(servant) = selected {
                ui.add_space(4.0);
                let attack_label = servant.level.map_or_else(
                    || format!("Maximum ATK {}", servant.attack),
                    |level| format!("Level {level} · {} ATK", servant.attack),
                );
                ui.label(RichText::new(attack_label).size(12.0).color(TEXT_MUTED));
                ui.label(
                    RichText::new(format!(
                        "{} · {}",
                        servant.class.label(),
                        servant.attribute.label()
                    ))
                    .size(12.0)
                    .color(TEXT_MUTED),
                );
            }
        });
    });
}

fn portrait_window(
    ui: &mut egui::Ui,
    portrait: Option<&egui::TextureHandle>,
    message: &str,
    error: Option<&str>,
) {
    let size = egui::vec2(100.0, 136.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
    ui.painter().rect_filled(rect, 8.0, PANEL_MUTED);
    if let Some(texture) = portrait {
        let source_size = texture.size_vec2();
        let scale = (size.x / source_size.x).min(size.y / source_size.y);
        let image_rect = egui::Rect::from_center_size(rect.center(), source_size * scale);
        ui.painter().image(
            texture.id(),
            image_rect,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            message,
            egui::FontId::proportional(11.0),
            TEXT_MUTED,
        );
    }
    ui.painter().rect_stroke(
        rect,
        8.0,
        egui::Stroke::new(1.0, Color32::from_rgb(59, 69, 91)),
        egui::StrokeKind::Inside,
    );
    if let Some(error) = error {
        response.on_hover_text(error);
    }
}

fn class_selector(ui: &mut egui::Ui, label: &str, selected: &mut ClassType) {
    let label_response = ui.label(RichText::new(label).size(14.0).color(TEXT_MUTED));
    let combo = egui::ComboBox::from_id_salt(label)
        .selected_text(selected.label())
        .width(ui.available_width().min(220.0))
        .show_ui(ui, |ui| {
            for class in ClassType::SELECTABLE {
                ui.selectable_value(selected, class, class.label());
            }
        });
    combo.response.labelled_by(label_response.id);
}

fn attribute_selector(ui: &mut egui::Ui, label: &str, selected: &mut AttributeType) {
    let label_response = ui.label(RichText::new(label).size(14.0).color(TEXT_MUTED));
    let combo = egui::ComboBox::from_id_salt(label)
        .selected_text(selected.label())
        .width(ui.available_width().min(220.0))
        .show_ui(ui, |ui| {
            for attribute in AttributeType::ALL {
                ui.selectable_value(selected, attribute, attribute.label());
            }
        });
    combo.response.labelled_by(label_response.id);
}

fn damage_result(ui: &mut egui::Ui, result: DamageResult, announce_result: bool) {
    egui::Frame::new()
        .fill(RESULT_PANEL)
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(48, 95, 133)))
        .corner_radius(12)
        .inner_margin(18)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("ESTIMATED DAMAGE").size(13.0).color(ACCENT));
            ui.add_space(5.0);
            let damage_response = ui.label(
                RichText::new(format_damage_range(result))
                    .size((ui.available_width() / 8.0).clamp(28.0, 42.0))
                    .strong()
                    .color(Color32::WHITE),
            );
            if announce_result {
                mark_live_status(ui.ctx(), damage_response.id);
            }
            ui.label(
                RichText::new(format!(
                    "Damage before random variance: {}",
                    format_integer(result.damage_before_random.round() as u32)
                ))
                .size(13.0)
                .color(TEXT_MUTED),
            );
        });
}

fn format_damage_range(result: DamageResult) -> String {
    format!(
        "{}–{}",
        format_integer(result.minimum_damage),
        format_integer(result.maximum_damage)
    )
}

fn format_integer(value: u32) -> String {
    let digits = value.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(character);
    }
    grouped
}

fn breakdown(ui: &mut egui::Ui, result: DamageResult) {
    ui.collapsing(RichText::new("Calculation breakdown").strong(), |ui| {
        ui.add_space(8.0);
        egui::Grid::new("damage_breakdown")
            .num_columns(2)
            .spacing([40.0, 8.0])
            .show(ui, |ui| {
                for (label, value) in [
                    ("Base card", format!("×{:.2}", result.base_card_multiplier)),
                    ("Card buff", format!("×{:.2}", result.card_buff_multiplier)),
                    (
                        "First card bonus",
                        format!("+{:.2}", result.first_card_bonus),
                    ),
                    (
                        "Card total",
                        format!("×{:.2}", result.card_damage_multiplier),
                    ),
                    (
                        "Class attack",
                        format!("×{:.2}", result.class_attack_multiplier),
                    ),
                    ("Class affinity", format!("×{:.2}", result.class_multiplier)),
                    (
                        "Attribute affinity",
                        format!("×{:.2}", result.attribute_multiplier),
                    ),
                    (
                        "Attack − defense",
                        format!("×{:.2}", result.attack_defense_multiplier),
                    ),
                ] {
                    ui.label(RichText::new(label).color(TEXT_MUTED));
                    ui.label(value);
                    ui.end_row();
                }
            });
    });
}
