//! Damage results, the sticky summary and the calculation breakdown.

use super::*;

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
                if let SelectedCard::NoblePhantasm(np_index) = card
                    && let Some(np) = servant.noble_phantasms.get(*np_index) {
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
                        for (component_index, part) in np.components.iter().enumerate() {
                            if let Some(values) = part.trait_bonus_breakdown(selection.np_level, selection.overcharge_level, selection.trait_bonus) {
                                ui.label(RichText::new(format!(
                                    "Component {} · {} · {} · base ×{:.3} · conditional ×{:.3} · applied ×{:.3} · effective ×{:.3}",
                                    component_index + 1, values.condition.assumption_label(), if values.active { "matching assumed" } else { "matching not assumed" },
                                    values.base_multiplier, values.conditional_multiplier, values.applied_multiplier, values.effective_multiplier,
                                )).size(12.0).color(TEXT_MUTED));
                            }
                            if let Some(values) = part.enemy_status_breakdown(selection.np_level, selection.overcharge_level, selection.enemy_status) {
                                ui.label(RichText::new(format!(
                                    "Component {} · {} {} · base ×{:.3} · conditional ×{:.3} · applied ×{:.3} · effective ×{:.3}",
                                    component_index + 1, values.condition.label(), if values.active { "present" } else { "absent" },
                                    values.base_multiplier, values.conditional_multiplier, values.applied_multiplier, values.effective_multiplier,
                                )).size(12.0).color(TEXT_MUTED));
                            }
                        }
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
                                if let SelectedCard::NoblePhantasm(np_index) = card
                                    && let Some(multiplier) = servant
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

pub(super) fn damage_result(ui: &mut egui::Ui, result: DamageResult, announce_result: bool) {
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
                    "Damage subject to random variance: {}",
                    format_integer(result.damage_before_random.round() as u32)
                ))
                .size(13.0)
                .color(TEXT_MUTED),
            );
        });
}

pub(super) fn format_damage_range(result: DamageResult) -> String {
    format!(
        "{}–{}",
        format_integer(result.minimum_damage),
        format_integer(result.maximum_damage)
    )
}

pub(super) fn format_integer(value: u32) -> String {
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

pub(super) fn breakdown(ui: &mut egui::Ui, result: DamageResult) {
    ui.collapsing(RichText::new("Calculation breakdown").strong(), |ui| {
        ui.add_space(8.0);
        egui::Grid::new("damage_breakdown")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .max_col_width((ui.available_width() - 12.0) * 0.65)
            .show(ui, |ui| {
                for (label, value) in [
                    (
                        "Extra attack factor",
                        format!("×{:.2}", result.extra_attack_multiplier),
                    ),
                    (
                        "Flat addition after random",
                        format!("+{:.1}", result.flat_damage_addition),
                    ),
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
