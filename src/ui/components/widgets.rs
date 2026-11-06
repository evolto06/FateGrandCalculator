//! Shared layout helpers and small selectors.

use super::*;

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

pub fn status_message(ui: &mut egui::Ui, text: &str) {
    let response = ui.label(text);
    mark_live_status(ui.ctx(), response.id);
}

pub(super) fn mark_live_status(context: &egui::Context, id: egui::Id) {
    context.accesskit_node_builder(id, |node| {
        node.set_role(egui::accesskit::Role::Status);
        node.set_live(egui::accesskit::Live::Polite);
        node.set_live_atomic();
    });
}

pub(super) fn section_frame(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(PANEL)
        .corner_radius(12)
        .inner_margin(16)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add_contents(ui);
        });
}

pub(super) fn class_selector(ui: &mut egui::Ui, label: &str, selected: &mut ClassType) {
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

pub(super) fn attribute_selector(ui: &mut egui::Ui, label: &str, selected: &mut AttributeType) {
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
