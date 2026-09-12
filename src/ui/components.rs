use eframe::egui::{self, Color32, RichText};
use fate_grand_calculator::damage::{CardType, DamageResult};

use super::theme::{
    ACCENT, ARTS, BACKGROUND, BUSTER, PANEL, PANEL_MUTED, QUICK, RESULT_PANEL, TEXT_MUTED,
};

pub fn apply_canvas(ui: &mut egui::Ui) {
    ui.painter().rect_filled(ui.max_rect(), 0.0, BACKGROUND);
    ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);
}

pub fn header(ui: &mut egui::Ui) {
    ui.add_space(14.0);
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.heading(RichText::new("FateGrandCalculator").size(27.0).strong());
            ui.label(
                RichText::new("Unofficial FGO damage calculator")
                    .size(13.0)
                    .color(TEXT_MUTED),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new("NORMAL CARD · EARLY BUILD")
                    .size(10.0)
                    .color(ACCENT),
            );
        });
    });
}

pub fn attack_panel(
    ui: &mut egui::Ui,
    attack: &mut u32,
    card_type: &mut CardType,
    attack_buff_percent: &mut f64,
    card_buff_percent: &mut f64,
    enemy_defense_percent: &mut f64,
) {
    section_frame(ui, |ui| {
        ui.heading(RichText::new("Attack setup").size(18.0));
        ui.label(RichText::new("Configure the attack and modifiers.").color(TEXT_MUTED));
        ui.add_space(14.0);

        ui.label(RichText::new("ATTACK").size(11.0).color(TEXT_MUTED));
        ui.add_sized(
            [180.0, 30.0],
            egui::DragValue::new(attack)
                .speed(100.0)
                .range(0..=999_999)
                .suffix(" ATK"),
        );
        ui.add_space(12.0);

        card_selector(ui, card_type);
        ui.add_space(14.0);
        ui.separator();
        ui.add_space(8.0);
        ui.label(RichText::new("BUFFS & TARGET").size(11.0).color(TEXT_MUTED));
        ui.add_space(4.0);
        percent_input(ui, "Attack buff", attack_buff_percent);
        percent_input(ui, "Card buff", card_buff_percent);
        percent_input(ui, "Enemy defense", enemy_defense_percent);
    });
}

pub fn result_panel(
    ui: &mut egui::Ui,
    result: DamageResult,
    class_multiplier: &mut f64,
    attribute_multiplier: &mut f64,
) {
    section_frame(ui, |ui| {
        ui.heading(RichText::new("Result").size(18.0));
        ui.label(RichText::new("Live calculation preview.").color(TEXT_MUTED));
        ui.add_space(14.0);

        damage_result(ui, result);
        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);
        ui.label(RichText::new("MATCHUPS").size(11.0).color(TEXT_MUTED));
        ui.add_space(4.0);
        multiplier_input(ui, "Class multiplier", class_multiplier);
        multiplier_input(ui, "Attribute multiplier", attribute_multiplier);
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);
        breakdown(ui, result);
    });
}

fn section_frame(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(PANEL)
        .corner_radius(12)
        .inner_margin(18)
        .show(ui, add_contents);
}

fn card_selector(ui: &mut egui::Ui, card_type: &mut CardType) {
    ui.label(RichText::new("COMMAND CARD").size(11.0).color(TEXT_MUTED));
    ui.add_space(6.0);
    ui.horizontal(|ui| {
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

            if ui.add_sized([84.0, 34.0], button).clicked() {
                *card_type = candidate;
            }
        }
    });
}

fn percent_input(ui: &mut egui::Ui, label: &str, value: &mut f64) {
    ui.label(RichText::new(label).size(12.0).color(TEXT_MUTED));
    ui.add_sized(
        [150.0, 28.0],
        egui::DragValue::new(value)
            .speed(0.5)
            .range(-100.0..=999.0)
            .suffix("%"),
    );
}

fn multiplier_input(ui: &mut egui::Ui, label: &str, value: &mut f64) {
    ui.label(RichText::new(label).size(12.0).color(TEXT_MUTED));
    ui.add_sized(
        [150.0, 28.0],
        egui::DragValue::new(value).speed(0.05).range(0.0..=10.0),
    );
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
                .size(32.0)
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
                for (label, multiplier) in [
                    ("Base card", result.base_card_multiplier),
                    ("Attack", result.attack_multiplier),
                    ("Card buff", result.card_buff_multiplier),
                    ("Defense", result.defense_multiplier),
                    ("Class", result.class_multiplier),
                    ("Attribute", result.attribute_multiplier),
                ] {
                    ui.label(RichText::new(label).color(TEXT_MUTED));
                    ui.label(format!("×{multiplier:.2}"));
                    ui.end_row();
                }
            });
    });
}
