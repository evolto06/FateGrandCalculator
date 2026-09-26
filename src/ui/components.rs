use eframe::egui::{self, Color32, RichText};
use fate_grand_calculator::damage::DamageResult;
use fate_grand_calculator::loader::GameData;
use fate_grand_calculator::model::{AttributeType, CardType, ClassType};

use super::theme::{
    ACCENT, ARTS, BACKGROUND, BUSTER, PANEL, PANEL_MUTED, QUICK, RESULT_PANEL, TEXT_MUTED,
};

pub fn apply_canvas(ui: &mut egui::Ui) {
    ui.painter().rect_filled(ui.max_rect(), 0.0, BACKGROUND);
    ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);
    ui.spacing_mut().interact_size = egui::vec2(44.0, 44.0);
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
                    .size(13.0)
                    .color(TEXT_MUTED),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new("FIRST CARD · EARLY BUILD")
                    .size(10.0)
                    .color(ACCENT),
            );
        });
    });
}

pub fn attack_panel(
    ui: &mut egui::Ui,
    data: &GameData,
    selected_servant_id: &mut u32,
    card_type: &mut CardType,
    attack_buff_percent: &mut f64,
    card_buff_percent: &mut f64,
    enemy_defense_percent: &mut f64,
) {
    section_frame(ui, |ui| {
        ui.heading(RichText::new("Attack setup").size(18.0));
        ui.label(RichText::new("One non-critical first command card.").color(TEXT_MUTED));
        ui.add_space(14.0);

        servant_selector(ui, data, selected_servant_id);
        ui.add_space(12.0);

        card_selector(ui, card_type);
        ui.add_space(14.0);
        ui.separator();
        ui.add_space(8.0);
        ui.label(RichText::new("BUFFS & TARGET").size(11.0).color(TEXT_MUTED));
        ui.small("Enter 20 for a 20% buff or defense value.");
        ui.add_space(4.0);
        percent_input(ui, "Attack buff", attack_buff_percent);
        percent_input(ui, "Card buff", card_buff_percent);
        percent_input(ui, "Enemy defense", enemy_defense_percent);
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
        ui.small("Beast class affinity depends on the specific encounter.");
        attribute_selector(ui, "Enemy attribute", enemy_attribute);
    });
}

pub fn result_panel(ui: &mut egui::Ui, result: DamageResult) {
    section_frame(ui, |ui| {
        ui.heading(RichText::new("First card result").size(18.0));
        ui.label(RichText::new("No critical hit or chain bonus.").color(TEXT_MUTED));
        ui.add_space(14.0);

        damage_result(ui, result);
        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);
        breakdown(ui, result);
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
    ui.label(RichText::new("COMMAND CARD").size(11.0).color(TEXT_MUTED));
    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        let button_width = ((ui.available_width() - 20.0) / 3.0).max(68.0);
        for (candidate, label, color) in [
            (CardType::Buster, "Buster", BUSTER),
            (CardType::Arts, "Arts", ARTS),
            (CardType::Quick, "Quick", QUICK),
        ] {
            let selected = *card_type == candidate;
            let button = egui::Button::new(RichText::new(label).strong().color(if selected {
                Color32::WHITE
            } else {
                TEXT_MUTED
            }))
            .fill(if selected { color } else { PANEL_MUTED })
            .stroke(egui::Stroke::new(
                1.0,
                if selected {
                    color
                } else {
                    Color32::TRANSPARENT
                },
            ))
            .corner_radius(7);

            if ui.add_sized([button_width, 44.0], button).clicked() {
                *card_type = candidate;
            }
        }
    });
}

fn percent_input(ui: &mut egui::Ui, label: &str, value: &mut f64) {
    ui.label(RichText::new(label).size(12.0).color(TEXT_MUTED));
    let input_width = ui.available_width().min(190.0);
    ui.add_sized(
        [input_width, 44.0],
        egui::DragValue::new(value)
            .speed(0.5)
            .range(-100.0..=999.0)
            .suffix("%"),
    );
}

fn servant_selector(ui: &mut egui::Ui, data: &GameData, selected_id: &mut u32) {
    let selected = data.servant(*selected_id).or_else(|| data.servants.first());
    let selected_name = selected.map_or("No servants loaded", |servant| servant.name.as_str());

    ui.label(RichText::new("SERVANT").size(11.0).color(TEXT_MUTED));
    ui.add_space(6.0);
    ui.horizontal_top(|ui| {
        portrait_placeholder(ui);
        ui.vertical(|ui| {
            egui::ComboBox::from_id_salt("servant_selector")
                .selected_text(selected_name)
                .width(ui.available_width())
                .show_ui(ui, |ui| {
                    for servant in &data.servants {
                        ui.selectable_value(selected_id, servant.id, &servant.name);
                    }
                });

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

fn portrait_placeholder(ui: &mut egui::Ui) {
    let size = egui::vec2(82.0, 108.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    ui.painter().rect_filled(rect, 8.0, PANEL_MUTED);
    ui.painter().rect_stroke(
        rect,
        8.0,
        egui::Stroke::new(1.0, Color32::from_rgb(59, 69, 91)),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "No portrait\navailable yet",
        egui::FontId::proportional(11.0),
        TEXT_MUTED,
    );
}

fn class_selector(ui: &mut egui::Ui, label: &str, selected: &mut ClassType) {
    ui.label(RichText::new(label).size(12.0).color(TEXT_MUTED));
    egui::ComboBox::from_id_salt(label)
        .selected_text(selected.label())
        .width(ui.available_width().min(220.0))
        .show_ui(ui, |ui| {
            for class in ClassType::SELECTABLE {
                ui.selectable_value(selected, class, class.label());
            }
        });
}

fn attribute_selector(ui: &mut egui::Ui, label: &str, selected: &mut AttributeType) {
    ui.label(RichText::new(label).size(12.0).color(TEXT_MUTED));
    egui::ComboBox::from_id_salt(label)
        .selected_text(selected.label())
        .width(ui.available_width().min(220.0))
        .show_ui(ui, |ui| {
            for attribute in AttributeType::ALL {
                ui.selectable_value(selected, attribute, attribute.label());
            }
        });
}

fn damage_result(ui: &mut egui::Ui, result: DamageResult) {
    egui::Frame::new()
        .fill(RESULT_PANEL)
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(48, 95, 133)))
        .corner_radius(12)
        .inner_margin(18)
        .show(ui, |ui| {
            ui.label(RichText::new("ESTIMATED DAMAGE").size(11.0).color(ACCENT));
            ui.add_space(5.0);
            ui.label(
                RichText::new(format!(
                    "{}–{}",
                    result.minimum_damage, result.maximum_damage
                ))
                .size((ui.available_width() / 8.5).clamp(24.0, 32.0))
                .strong()
                .color(Color32::WHITE),
            );
            ui.label(
                RichText::new(format!(
                    "Before random range: {:.0}",
                    result.damage_before_random
                ))
                .size(12.0)
                .color(TEXT_MUTED),
            );
        });
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
