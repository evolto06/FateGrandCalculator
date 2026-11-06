//! The attack setup and matchup panels: servant choice and portrait.

use super::*;

/// What the portrait window shows for the selected servant.
#[derive(Clone, Copy)]
pub struct PortraitDisplay<'a> {
    pub texture: Option<&'a egui::TextureHandle>,
    pub message: &'a str,
    pub error: Option<&'a str>,
}

impl PortraitDisplay<'static> {
    /// Shown by builds without portrait streaming, and by UI tests.
    #[cfg(any(test, target_arch = "wasm32"))]
    pub const MISSING: Self = Self {
        texture: None,
        message: "No portrait
available",
        error: None,
    };
}

/// The servant choice and attack order that the attack panel edits together.
pub struct ServantPick<'a> {
    pub selected_id: &'a mut u32,
    pub search: &'a mut String,
    pub selection: &'a mut Option<TurnSelection>,
    pub active_slot: &'a mut usize,
}

pub fn attack_panel(
    ui: &mut egui::Ui,
    data: &GameData,
    portrait: PortraitDisplay<'_>,
    mut pick: ServantPick<'_>,
    buffs: &mut TurnBuffInputs,
) {
    section_frame(ui, |ui| {
        section_heading(
            ui,
            "Attack setup",
            "Pick an attack position, then choose its card.",
        );

        let width = ui.available_width();
        if width >= 450.0 {
            let servant_width = 220.0;
            let cards_width = width - servant_width - ui.spacing().item_spacing.x;
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(servant_width, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(servant_width);
                        servant_setup(ui, data, portrait, &mut pick);
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(cards_width, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(cards_width);
                        if let Some(servant) = data.servant(*pick.selected_id) {
                            turn_selector(ui, servant, pick.selection, pick.active_slot);
                        }
                    },
                );
            });
        } else {
            ui.vertical_centered(|ui| {
                ui.set_width(width.min(300.0));
                servant_setup(ui, data, portrait, &mut pick);
            });
            ui.add_space(12.0);
            if let Some(servant) = data.servant(*pick.selected_id) {
                turn_selector(ui, servant, pick.selection, pick.active_slot);
            }
        }
        ui.add_space(14.0);
        ui.separator();
        ui.add_space(8.0);
        buff_inputs(ui, buffs);
    });
}

pub(super) fn servant_setup(
    ui: &mut egui::Ui,
    data: &GameData,
    portrait: PortraitDisplay<'_>,
    pick: &mut ServantPick<'_>,
) -> egui::Response {
    let ServantPick {
        selected_id,
        search,
        selection,
        active_slot,
    } = pick;
    let previous_id = **selected_id;
    let response = servant_selector(ui, data, selected_id, search, portrait);
    if **selected_id != previous_id {
        clear_overcharge_feedback(ui.ctx(), previous_id);
        clear_overcharge_feedback(ui.ctx(), **selected_id);
        clear_hp_inputs(ui.ctx(), previous_id);
        clear_hp_inputs(ui.ctx(), **selected_id);
        **selection = data
            .servant(**selected_id)
            .and_then(TurnSelection::default_for);
        if let (Some(servant), Some(turn)) = (data.servant(**selected_id), selection.as_ref()) {
            let mut inputs = HpInputs::for_servant(servant, turn);
            inputs.reset_feedback = Some(if turn.attacker_hp.is_some() {
                "HP reset to full HP for the selected servant.".into()
            } else {
                "HP reset for the selected servant. Enter maximum HP to start at full HP.".into()
            });
            ui.ctx()
                .data_mut(|data| data.insert_temp(hp_inputs_id(servant.id), inputs));
        }
        **active_slot = 0;
        ui.ctx().request_repaint();
    } else if let Some(error) = portrait.error {
        let response =
            ui.colored_label(ERROR, "Portrait unavailable. Change servant to try again.");
        mark_live_status(ui.ctx(), response.id);
        note(ui, error);
    }
    response
}

/// Everything about the target: class, attribute and defense.
pub fn matchup_panel(
    ui: &mut egui::Ui,
    enemy_class: &mut ClassType,
    enemy_attribute: &mut AttributeType,
    enemy_defense: &mut PercentInput,
) {
    section_frame(ui, |ui| {
        section_heading(
            ui,
            "Enemy",
            "Set the target's class, attribute and defense.",
        );

        class_selector(ui, "Enemy class", enemy_class);
        if *enemy_class == ClassType::Beast {
            note(ui, "Beast affinity depends on the specific encounter.");
        }
        attribute_selector(ui, "Enemy attribute", enemy_attribute);
        percent_input(ui, "Enemy defense (%)", enemy_defense);
    });
}

pub(super) fn servant_selector(
    ui: &mut egui::Ui,
    data: &GameData,
    selected_id: &mut u32,
    search: &mut String,
    portrait: PortraitDisplay<'_>,
) -> egui::Response {
    let selected = data.servant(*selected_id).or_else(|| data.servants.first());
    let selected_name = selected.map_or("No servants loaded", |servant| servant.name.as_str());

    group_label(ui, "SERVANT");
    ui.add_space(6.0);
    portrait_window(ui, portrait);
    ui.add_space(6.0);
    ui.add(egui::Label::new(RichText::new(selected_name).strong()).wrap());
    if let Some(servant) = selected {
        let attack_label = servant.level.map_or_else(
            || format!("Maximum ATK {}", servant.attack),
            |level| format!("Level {level} · {} ATK", servant.attack),
        );
        note(ui, &attack_label);
        note(
            ui,
            &format!("{} · {}", servant.class.label(), servant.attribute.label()),
        );
    }
    ui.add_space(8.0);
    let label = ui.label("Servant");
    let combo = egui::ComboBox::from_id_salt("servant_selector")
        .selected_text(selected_name)
        .width(ui.available_width())
        .height(280.0)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .wrap_mode(egui::TextWrapMode::Truncate)
        .show_ui(ui, |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
            let search_label = ui.label("Search servants");
            let search_response = ui
                .add(
                    egui::TextEdit::singleline(search)
                        .id_salt("servant_search")
                        .desired_width(ui.available_width())
                        .hint_text("Type a servant name or class"),
                )
                .labelled_by(search_label.id);
            let query = search.trim().to_lowercase();
            let mut matches = 0;
            egui::ScrollArea::vertical()
                .id_salt("servant_search_results")
                .max_height(180.0)
                .show(ui, |ui| {
                    for servant in &data.servants {
                        if servant.name.to_lowercase().contains(&query)
                            || servant.class.label().to_lowercase().contains(&query)
                        {
                            matches += 1;
                            if ui
                                .selectable_label(*selected_id == servant.id, &servant.name)
                                .clicked()
                            {
                                *selected_id = servant.id;
                                search.clear();
                                ui.memory_mut(|memory| memory.surrender_focus(search_response.id));
                                ui.close();
                            }
                        }
                    }
                });
            if matches == 0 {
                ui.label("No servants match that search.");
            }
        });
    combo.response.labelled_by(label.id)
}

pub(super) fn portrait_window(ui: &mut egui::Ui, portrait: PortraitDisplay<'_>) {
    let width = ui.available_width().min(220.0);
    let size = egui::vec2(width, width * (300.0 / 220.0));
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
    ui.painter().rect_filled(rect, 8.0, PANEL_MUTED);
    if let Some(texture) = portrait.texture {
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
        let mut job = egui::text::LayoutJob::simple(
            portrait.message.to_owned(),
            egui::FontId::proportional(TEXT_SMALL),
            TEXT_MUTED,
            (rect.width() - 16.0).max(1.0),
        );
        job.halign = egui::Align::Center;
        let galley = ui.painter().layout_job(job);
        let position = egui::pos2(rect.center().x, rect.center().y - galley.size().y / 2.0);
        ui.painter().galley(position, galley, TEXT_MUTED);
    }
    ui.painter().rect_stroke(
        rect,
        8.0,
        egui::Stroke::new(1.0, BORDER),
        egui::StrokeKind::Inside,
    );
    if let Some(error) = portrait.error {
        response.on_hover_text(error);
    }
}
