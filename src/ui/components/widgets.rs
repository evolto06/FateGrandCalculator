//! Shared layout helpers and small selectors.

use super::*;

/// Paints the page background and applies the theme. The context style is
/// updated too, so popups such as dropdown lists match from the next frame.
pub fn apply_canvas(ui: &mut egui::Ui) {
    let styled_id = egui::Id::new("fgc_theme_applied");
    if !ui
        .ctx()
        .data(|data| data.get_temp::<bool>(styled_id).unwrap_or(false))
    {
        ui.ctx().set_theme(egui::Theme::Dark);
        ui.ctx().all_styles_mut(theme::apply);
        ui.ctx().data_mut(|data| data.insert_temp(styled_id, true));
    }
    theme::apply(ui.style_mut());
    ui.painter().rect_filled(ui.max_rect(), 0.0, BACKGROUND);
}

pub fn header(ui: &mut egui::Ui) {
    ui.add_space(14.0);
    let title_size = if ui.available_width() >= 420.0 {
        TEXT_TITLE
    } else {
        TEXT_TITLE_COMPACT
    };
    ui.heading(
        RichText::new("FateGrandCalculator")
            .size(title_size)
            .strong(),
    );
    ui.label(
        RichText::new("Unofficial FGO damage calculator")
            .size(TEXT_BODY)
            .color(TEXT_MUTED),
    );
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

/// A panel title with a one-line description under it.
pub(super) fn section_heading(ui: &mut egui::Ui, title: &str, description: &str) {
    ui.heading(RichText::new(title).size(TEXT_HEADING));
    note(ui, description);
    ui.add_space(8.0);
}

/// A small upper-case label that names a group of controls.
pub(super) fn group_label(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).size(TEXT_LABEL).color(TEXT_MUTED));
}

/// Secondary explanatory text.
pub(super) fn note(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.label(RichText::new(text).size(TEXT_SMALL).color(TEXT_MUTED))
}

pub(super) fn class_selector(ui: &mut egui::Ui, label: &str, selected: &mut ClassType) {
    let label_response = ui.label(RichText::new(label).size(TEXT_BODY).color(TEXT_MUTED));
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
    let label_response = ui.label(RichText::new(label).size(TEXT_BODY).color(TEXT_MUTED));
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
