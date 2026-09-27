use eframe::egui::{self, Color32, RichText};
use fate_grand_calculator::damage::DamageResult;
use fate_grand_calculator::loader::GameData;
use fate_grand_calculator::model::{AttributeType, CardType, ClassType};

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
            RichText::new("FIRST CARD · EARLY BUILD")
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
    card_type: &mut CardType,
    attack_buff_percent: &mut PercentInput,
    card_buff_percent: &mut PercentInput,
    enemy_defense_percent: &mut PercentInput,
) {
    section_frame(ui, |ui| {
        ui.heading(RichText::new("Attack setup").size(18.0));
        ui.label(RichText::new("One non-critical first command card.").color(TEXT_MUTED));
        ui.add_space(14.0);

        servant_selector(
            ui,
            data,
            selected_servant_id,
            servant_search,
            portrait,
            portrait_message,
            portrait_error,
        );
        if portrait_error.is_some() {
            let response =
                ui.colored_label(ERROR, "Portrait unavailable. Change servant to try again.");
            mark_live_status(ui.ctx(), response.id);
        }
        ui.add_space(12.0);

        card_selector(ui, card_type);
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
        percent_input(ui, "Attack buff (%)", attack_buff_percent);
        percent_input(ui, "Card buff (%)", card_buff_percent);
        percent_input(ui, "Enemy defense (%)", enemy_defense_percent);
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

pub fn result_panel(ui: &mut egui::Ui, result: DamageResult, announce_result: bool) {
    section_frame(ui, |ui| {
        ui.heading(RichText::new("First card result").size(18.0));
        ui.label(RichText::new("No critical hit or chain bonus.").color(TEXT_MUTED));
        ui.add_space(14.0);

        damage_result(ui, result, announce_result);
        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);
        breakdown(ui, result);
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

pub fn status_message(ui: &mut egui::Ui, text: &str) {
    let response = ui.label(text);
    mark_live_status(ui.ctx(), response.id);
}

pub fn sticky_result_summary(ui: &mut egui::Ui, result: DamageResult) {
    egui::Frame::new()
        .fill(RESULT_PANEL)
        .stroke(egui::Stroke::new(1.5, ACCENT))
        .corner_radius(10)
        .inner_margin(12)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Estimated damage").size(14.0).color(ACCENT));
                let response = ui.label(
                    RichText::new(format_damage_range(result))
                        .size(21.0)
                        .strong()
                        .color(Color32::WHITE),
                );
                mark_live_status(ui.ctx(), response.id);
            });
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

fn card_selector(ui: &mut egui::Ui, card_type: &mut CardType) {
    ui.label(RichText::new("COMMAND CARD").size(13.0).color(TEXT_MUTED));
    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        let button_width = ((ui.available_width() - 20.0) / 3.0).max(68.0);
        for (candidate, label, color) in [
            (CardType::Buster, "Buster", BUSTER),
            (CardType::Arts, "Arts", ARTS),
            (CardType::Quick, "Quick", QUICK),
        ] {
            let selected = *card_type == candidate;
            let label = if selected {
                format!("[x] {label}")
            } else {
                label.to_owned()
            };
            let button = egui::Button::selectable(
                selected,
                RichText::new(label).strong().color(if selected {
                    Color32::WHITE
                } else {
                    TEXT_MUTED
                }),
            )
            .fill(if selected {
                color
            } else {
                Color32::from_rgb(43, 51, 69)
            })
            .stroke(egui::Stroke::new(
                1.0,
                if selected {
                    color
                } else {
                    Color32::from_rgb(64, 75, 97)
                },
            ))
            .corner_radius(7);

            if ui.add_sized([button_width, 44.0], button).clicked() {
                *card_type = candidate;
            }
        }
    });
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

    let servant_label = ui.label(RichText::new("SERVANT").size(13.0).color(TEXT_MUTED));
    ui.add_space(6.0);
    ui.horizontal_top(|ui| {
        portrait_window(ui, portrait, portrait_message, portrait_error);
        ui.vertical(|ui| {
            let combo = egui::ComboBox::from_id_salt("servant_selector")
                .selected_text(selected_name)
                .width(ui.available_width())
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                .show_ui(ui, |ui| {
                    let search_label = ui.label("Search servants");
                    let search_width = ui.available_width();
                    let search_response = ui.add(
                        egui::TextEdit::singleline(search)
                            .id_salt("servant_search")
                            .desired_width(search_width)
                            .hint_text("Type a servant name"),
                    );
                    search_response.labelled_by(search_label.id);
                    ui.separator();

                    let query = search.trim().to_lowercase();
                    let mut matches = 0;
                    for servant in &data.servants {
                        if servant.name.to_lowercase().contains(&query) {
                            matches += 1;
                            if ui
                                .selectable_value(selected_id, servant.id, &servant.name)
                                .clicked()
                            {
                                search.clear();
                                ui.close();
                            }
                        }
                    }
                    if matches == 0 {
                        ui.label(if data.servants.is_empty() {
                            "No servants are available."
                        } else {
                            "No servants match that search."
                        });
                    }
                });
            combo.response.labelled_by(servant_label.id);

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
