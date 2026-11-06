//! NP level and Overcharge controls for the selected NP.

use super::*;

pub(super) fn overcharge_feedback_id(servant_id: u32) -> egui::Id {
    egui::Id::new(("overcharge_reset_feedback", servant_id))
}

pub(crate) fn clear_overcharge_feedback(context: &egui::Context, servant_id: u32) {
    context.data_mut(|data| {
        data.remove::<String>(overcharge_feedback_id(servant_id));
    });
}

pub(super) fn np_settings(ui: &mut egui::Ui, servant: &ServantRecord, turn: &mut TurnSelection) {
    trait_bonus_settings(ui, servant, turn);
    enemy_status_settings(ui, servant, turn);
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
        let Some(np) = turn.slots.iter().find_map(|card| match card {
            SelectedCard::NoblePhantasm(index) => servant.noble_phantasms.get(*index),
            SelectedCard::Normal(_) => None,
        }) else {
            ui.colored_label(ERROR, "Selected NP is unavailable. Choose another card.");
            return;
        };
        let coverage = np.available_overcharges();
        let previous_overcharge = turn.overcharge_level;
        let overcharge_label = ui.label("Overcharge level");
        let combo = egui::ComboBox::from_id_salt("overcharge_level")
            .selected_text(format!("OC {}", turn.overcharge_level))
            .width(100.0)
            .show_ui(ui, |ui| {
                for level in 1..=5 {
                    let available = coverage.contains(&level);
                    let response = ui.add_enabled(
                        available,
                        egui::Button::selectable(
                            turn.overcharge_level == level,
                            format!("OC {level}"),
                        ),
                    );
                    if !available {
                        response.clone().on_disabled_hover_text(
                            "Update servant data to load this Overcharge level.",
                        );
                    }
                    if response.clicked() {
                        turn.overcharge_level = level;
                        ui.close();
                    }
                }
            });
        combo.response.labelled_by(overcharge_label.id);
        ui.label(
            RichText::new("NP level and Overcharge are independent.")
                .size(12.0)
                .color(TEXT_MUTED),
        );
        if coverage.len() < 5 {
            ui.label(
                RichText::new("Update servant data to enable missing Overcharge levels.")
                    .size(12.0)
                    .color(TEXT_MUTED),
            );
        }
        if !coverage.contains(&turn.overcharge_level) {
            ui.colored_label(ERROR, "Selected Overcharge is unavailable. Choose an available level or update servant data.");
        }
        if previous_overcharge != turn.overcharge_level {
            ui.ctx().data_mut(|data| {
                data.remove::<String>(overcharge_feedback_id(servant.id));
            });
        }
        if let Some(message) = ui
            .ctx()
            .data(|data| data.get_temp::<String>(overcharge_feedback_id(servant.id)))
        {
            status_message(ui, &message);
        }
        if np.requires_attacker_hp() {
            hp_settings(ui, servant, turn);
        }
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
