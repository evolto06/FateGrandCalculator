use eframe::egui::{self, Color32, RichText};
use fate_grand_calculator::damage::{
    DamageResult, TurnBuffs, TurnDamageResult, percent_to_modifier,
};
use fate_grand_calculator::loader::{GameData, NpStatus, ServantRecord};
use fate_grand_calculator::model::{
    AttackerHp, AttributeType, CardType, ClassType, SelectedCard, TurnSelection,
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
    active_slot: &mut usize,
    buffs: &mut TurnBuffInputs,
) {
    section_frame(ui, |ui| {
        ui.heading(RichText::new("Attack setup").size(18.0));
        ui.label(RichText::new("Select an attack card to change it.").color(TEXT_MUTED));
        ui.add_space(14.0);

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
                        servant_setup(
                            ui,
                            data,
                            selected_servant_id,
                            servant_search,
                            portrait,
                            portrait_message,
                            portrait_error,
                            selection,
                            active_slot,
                        );
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(cards_width, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(cards_width);
                        if let Some(servant) = data.servant(*selected_servant_id) {
                            turn_selector(ui, servant, selection, active_slot);
                        }
                    },
                );
            });
        } else {
            ui.vertical_centered(|ui| {
                ui.set_width(width.min(300.0));
                servant_setup(
                    ui,
                    data,
                    selected_servant_id,
                    servant_search,
                    portrait,
                    portrait_message,
                    portrait_error,
                    selection,
                    active_slot,
                );
            });
            ui.add_space(12.0);
            if let Some(servant) = data.servant(*selected_servant_id) {
                turn_selector(ui, servant, selection, active_slot);
            }
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

fn servant_setup(
    ui: &mut egui::Ui,
    data: &GameData,
    selected_id: &mut u32,
    search: &mut String,
    portrait: Option<&egui::TextureHandle>,
    portrait_message: &str,
    portrait_error: Option<&str>,
    selection: &mut Option<TurnSelection>,
    active_slot: &mut usize,
) -> egui::Response {
    let previous_id = *selected_id;
    let response = servant_selector(
        ui,
        data,
        selected_id,
        search,
        portrait,
        portrait_message,
        portrait_error,
    );
    if *selected_id != previous_id {
        clear_overcharge_feedback(ui.ctx(), previous_id);
        clear_overcharge_feedback(ui.ctx(), *selected_id);
        clear_hp_inputs(ui.ctx(), previous_id);
        clear_hp_inputs(ui.ctx(), *selected_id);
        *selection = data
            .servant(*selected_id)
            .and_then(TurnSelection::default_for);
        if let (Some(servant), Some(turn)) = (data.servant(*selected_id), selection.as_ref()) {
            let mut inputs = HpInputs::for_servant(servant, turn);
            inputs.reset_feedback = Some(if turn.attacker_hp.is_some() {
                "HP reset to full HP for the selected servant.".into()
            } else {
                "HP reset for the selected servant. Enter maximum HP to start at full HP.".into()
            });
            ui.ctx()
                .data_mut(|data| data.insert_temp(hp_inputs_id(servant.id), inputs));
        }
        *active_slot = 0;
        ui.ctx().request_repaint();
    } else if portrait_error.is_some() {
        let response =
            ui.colored_label(ERROR, "Portrait unavailable. Change servant to try again.");
        mark_live_status(ui.ctx(), response.id);
    }
    response
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
                        let multiplier =
                            np.base_multiplier(selection.np_level, selection.overcharge_level);
                        ui.label(
                            RichText::new(format!(
                                "NP level {} · Overcharge {}{}",
                                selection.np_level,
                                selection.overcharge_level,
                                multiplier.map_or_else(String::new, |value| format!(
                                    " · base damage ×{value:.2}"
                                ))
                            ))
                            .size(12.0)
                            .color(TEXT_MUTED),
                        );
                        if let Some(hp) = selection.attacker_hp {
                            for part in &np.components {
                                if let Some(values) = part.low_hp_breakdown(
                                    selection.np_level,
                                    selection.overcharge_level,
                                    hp,
                                ) {
                                    ui.label(
                                        RichText::new(format!(
                                            "Base ×{:.3} + missing-HP bonus ×{:.3} = effective ×{:.3}",
                                            values.base_multiplier,
                                            values.hp_contribution,
                                            values.effective_multiplier,
                                        ))
                                        .size(12.0)
                                        .color(TEXT_MUTED),
                                    );
                                }
                            }
                        }
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
                        if np.defense_pierce {
                            ui.label(
                                RichText::new("Ignores Defense Up · retains Defense Down")
                                    .size(12.0)
                                    .color(TEXT_MUTED),
                            );
                        }
                    }
                }
                damage_result(ui, result.cards[index], announce_result);
                if result.np_components[index].len() > 1 {
                    ui.label(
                        RichText::new("Total of separately calculated NP components")
                            .size(12.0)
                            .color(TEXT_MUTED),
                    );
                    ui.collapsing("NP component breakdown", |ui| {
                        for (component_index, component) in
                            result.np_components[index].iter().enumerate()
                        {
                            ui.push_id(component_index, |ui| {
                                ui.label(format!(
                                    "Component {} · {}",
                                    component_index + 1,
                                    format_damage_range(*component)
                                ));
                                if let SelectedCard::NoblePhantasm(np_index) = card {
                                    if let Some(multiplier) = servant
                                        .noble_phantasms
                                        .get(*np_index)
                                        .and_then(|np| np.components.get(component_index))
                                        .and_then(|part| {
                                            part.multiplier(
                                                selection.np_level,
                                                selection.overcharge_level,
                                            )
                                        })
                                    {
                                        ui.label(
                                            RichText::new(format!(
                                                "Base NP multiplier ×{multiplier:.2}"
                                            ))
                                            .size(12.0)
                                            .color(TEXT_MUTED),
                                        );
                                    }
                                }
                                breakdown(ui, *component);
                            });
                        }
                    });
                } else {
                    breakdown(ui, result.cards[index]);
                }
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

fn overcharge_feedback_id(servant_id: u32) -> egui::Id {
    egui::Id::new(("overcharge_reset_feedback", servant_id))
}

pub(crate) fn clear_overcharge_feedback(context: &egui::Context, servant_id: u32) {
    context.data_mut(|data| {
        data.remove::<String>(overcharge_feedback_id(servant_id));
    });
}

fn hp_inputs_id(servant_id: u32) -> egui::Id {
    egui::Id::new(("attacker_hp_inputs", servant_id))
}

pub(crate) fn clear_hp_inputs(context: &egui::Context, servant_id: u32) {
    context.data_mut(|data| {
        data.remove::<HpInputs>(hp_inputs_id(servant_id));
    });
}

pub(crate) fn has_hp_inputs(context: &egui::Context, servant_id: u32) -> bool {
    context.data(|data| {
        data.get_temp::<HpInputs>(hp_inputs_id(servant_id))
            .is_some()
    })
}

#[cfg(test)]
pub(crate) fn test_hp_inputs(
    context: &egui::Context,
    servant_id: u32,
    set: Option<(&str, &str)>,
) -> Option<(String, String)> {
    context.data_mut(|data| {
        if let Some((current, maximum)) = set {
            data.insert_temp(
                hp_inputs_id(servant_id),
                HpInputs {
                    current: current.into(),
                    maximum: maximum.into(),
                    reset_feedback: None,
                },
            );
        }
        data.get_temp::<HpInputs>(hp_inputs_id(servant_id))
            .map(|inputs| (inputs.current, inputs.maximum))
    })
}

#[derive(Clone, Debug, Default)]
struct HpInputs {
    current: String,
    maximum: String,
    reset_feedback: Option<String>,
}

impl HpInputs {
    fn for_servant(servant: &ServantRecord, turn: &TurnSelection) -> Self {
        let hp = turn.attacker_hp.or_else(|| {
            servant
                .max_hp
                .and_then(|max| AttackerHp::new(max, max).ok())
        });
        Self {
            current: hp.map_or_else(String::new, |hp| hp.current.to_string()),
            maximum: hp.map_or_else(String::new, |hp| hp.max.to_string()),
            reset_feedback: None,
        }
    }

    fn integer(text: &str, label: &str) -> Result<u32, String> {
        if text.is_empty() {
            return Err(format!("Enter {label} as a positive whole number."));
        }
        if !text.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(format!("{label} must contain whole-number digits only."));
        }
        text.parse::<u32>()
            .map_err(|_| format!("{label} is too large."))
    }

    fn value(&self) -> Result<AttackerHp, String> {
        AttackerHp::new(
            Self::integer(&self.current, "current HP")?,
            Self::integer(&self.maximum, "maximum HP")?,
        )
    }

    fn maximum_changed(&mut self) {
        if let Ok(max) = Self::integer(&self.maximum, "maximum HP") {
            if max > 0 {
                self.current = max.to_string();
                self.reset_feedback =
                    Some("Current HP reset to full HP because maximum HP changed.".into());
                return;
            }
        }
        self.reset_feedback = None;
    }
}

fn hp_settings(ui: &mut egui::Ui, servant: &ServantRecord, turn: &mut TurnSelection) {
    let id = hp_inputs_id(servant.id);
    let mut inputs = ui
        .ctx()
        .data(|data| data.get_temp::<HpInputs>(id))
        .unwrap_or_else(|| HpInputs::for_servant(servant, turn));
    ui.add_space(8.0);
    ui.label(RichText::new("HP at NP damage time").strong());
    ui.label(
        RichText::new(
            "Enter HP after any preceding healing or HP loss. These effects are applied manually.",
        )
        .size(12.0)
        .color(TEXT_MUTED),
    );
    if servant.max_hp.is_none() {
        ui.label(
            RichText::new("Maximum HP is unavailable in this data. Enter it manually.")
                .size(12.0)
                .color(TEXT_MUTED),
        );
    }
    let maximum_label = ui.label("Maximum HP");
    let maximum = ui.add(
        egui::TextEdit::singleline(&mut inputs.maximum)
            .id_salt("maximum_hp")
            .desired_width(130.0)
            .hint_text("Whole number"),
    );
    if maximum.changed() {
        inputs.maximum_changed();
    }
    maximum.labelled_by(maximum_label.id);
    let current_label = ui.label("Current HP");
    let current = ui.add(
        egui::TextEdit::singleline(&mut inputs.current)
            .id_salt("current_hp")
            .desired_width(130.0)
            .hint_text("Whole number"),
    );
    if current.changed() {
        inputs.reset_feedback = None;
    }
    current.labelled_by(current_label.id);
    match inputs.value() {
        Ok(hp) => {
            turn.attacker_hp = Some(hp);
            ui.label(format!("HP remaining: {:.2}%", hp.ratio() * 100.0));
        }
        Err(error) => {
            turn.attacker_hp = None;
            ui.colored_label(ERROR, format!("{error} Damage calculation is paused."));
        }
    }
    if let Some(message) = &inputs.reset_feedback {
        status_message(ui, message);
    }
    ui.ctx().data_mut(|data| data.insert_temp(id, inputs));
}

fn np_settings(ui: &mut egui::Ui, servant: &ServantRecord, turn: &mut TurnSelection) {
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

fn card_owner(turn: &TurnSelection, candidate: SelectedCard) -> Option<usize> {
    turn.slots
        .iter()
        .position(|selected| match (candidate, *selected) {
            (SelectedCard::NoblePhantasm(_), SelectedCard::NoblePhantasm(_)) => true,
            _ => candidate == *selected,
        })
}

fn card_tile(
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
        if let SelectedCard::NoblePhantasm(index) = candidate {
            if let Some(np) = servant.noble_phantasms.get(index) {
                if !np.available_overcharges().contains(&turn.overcharge_level) {
                    let previous = turn.overcharge_level;
                    turn.overcharge_level = 1;
                    ui.ctx().data_mut(|data| data.insert_temp(overcharge_feedback_id(servant.id), format!("Overcharge reset from {previous} to 1: this NP needs updated servant data for higher levels.")));
                }
            }
        }
    }
    response
}

fn selected_card_symbol(servant: &ServantRecord, card: SelectedCard) -> &'static str {
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

fn selected_card_color(servant: &ServantRecord, card: SelectedCard) -> Color32 {
    match card {
        SelectedCard::NoblePhantasm(_) => Color32::from_rgb(85, 83, 148),
        SelectedCard::Normal(index) => card_color(servant.deck[index]),
    }
}

fn card_button(
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

fn card_focus_outline(ui: &egui::Ui, response: &egui::Response) {
    if response.enabled() && (response.has_focus() || response.hovered()) {
        ui.painter().rect_stroke(
            response.rect,
            8.0,
            egui::Stroke::new(if response.has_focus() { 3.0 } else { 1.5 }, ACCENT),
            egui::StrokeKind::Inside,
        );
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
) -> egui::Response {
    let selected = data.servant(*selected_id).or_else(|| data.servants.first());
    let selected_name = selected.map_or("No servants loaded", |servant| servant.name.as_str());

    ui.label(RichText::new("SERVANT").size(13.0).color(TEXT_MUTED));
    ui.add_space(6.0);
    portrait_window(ui, portrait, portrait_message, portrait_error);
    ui.add_space(6.0);
    ui.add(egui::Label::new(RichText::new(selected_name).strong()).wrap());
    if let Some(servant) = selected {
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
                        .hint_text("Type a servant name"),
                )
                .labelled_by(search_label.id);
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

fn portrait_window(
    ui: &mut egui::Ui,
    portrait: Option<&egui::TextureHandle>,
    message: &str,
    error: Option<&str>,
) {
    let width = ui.available_width().min(220.0);
    let size = egui::vec2(width, width * (300.0 / 220.0));
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
            let range = format_damage_range(result);
            let width = ui.available_width();
            let mut font_size = (width / 8.0).clamp(28.0, 42.0);
            let measured_width = ui
                .painter()
                .layout_no_wrap(
                    range.clone(),
                    egui::FontId::proportional(font_size),
                    Color32::WHITE,
                )
                .size()
                .x;
            if measured_width > width {
                font_size *= width / measured_width;
            }
            let damage_response = ui.label(
                RichText::new(range)
                    .size(font_size)
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

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
