//! Damage results, the turn summary and the calculation breakdown.

use super::*;

/// `announce_result` is true when the results sit beside the inputs (wide
/// layout). The panel then starts with the turn summary, which is the one
/// place screen readers hear updated damage from.
pub fn result_panel(
    ui: &mut egui::Ui,
    servant: &ServantRecord,
    selection: &TurnSelection,
    result: &TurnDamageResult,
    announce_result: bool,
) {
    section_frame(ui, |ui| {
        section_heading(
            ui,
            "Damage by card",
            "Non-critical damage to the selected enemy; each card has its own range.",
        );
        if announce_result {
            summary_frame(ui, "TURN SUMMARY", servant, selection, result);
            ui.add_space(8.0);
        }
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
                    && let Some(np) = servant.noble_phantasms.get(*np_index)
                {
                    np_summary(ui, np, selection);
                }
                damage_result(ui, result.cards[index]);
                if result.np_components[index].len() > 1 {
                    note(ui, "Total of separately calculated NP components");
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
                                    note(ui, &format!("Base NP multiplier ×{multiplier:.2}"));
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
            damage_result(ui, extra);
            breakdown(ui, extra);
        }
        if !result.notes.is_empty() {
            ui.add_space(4.0);
            group_label(ui, "NOTES");
            for note_text in &result.notes {
                note(ui, note_text);
            }
        }
    });
}

/// The NP's level and Overcharge, with its conditional multipliers folded
/// into an "NP details" section.
fn np_summary(
    ui: &mut egui::Ui,
    np: &fate_grand_calculator::loader::NoblePhantasmRecord,
    selection: &TurnSelection,
) {
    let multiplier = np.base_multiplier(selection.np_level, selection.overcharge_level);
    note(
        ui,
        &format!(
            "NP level {} · Overcharge {}{}",
            selection.np_level,
            selection.overcharge_level,
            multiplier.map_or_else(String::new, |value| format!(" · base damage ×{value:.2}"))
        ),
    );
    let mut details = Vec::new();
    for (component_index, part) in np.components.iter().enumerate() {
        if let Some(values) = part.trait_bonus_breakdown(
            selection.np_level,
            selection.overcharge_level,
            selection.trait_bonus,
        ) {
            details.push(format!(
                "Component {} · {} · {} · base ×{:.3} · conditional ×{:.3} · applied ×{:.3} · effective ×{:.3}",
                component_index + 1,
                values.condition.assumption_label(),
                if values.active { "matching assumed" } else { "matching not assumed" },
                values.base_multiplier,
                values.conditional_multiplier,
                values.applied_multiplier,
                values.effective_multiplier,
            ));
        }
        if let Some(values) = part.enemy_status_breakdown(
            selection.np_level,
            selection.overcharge_level,
            selection.enemy_status,
        ) {
            details.push(format!(
                "Component {} · {} {} · base ×{:.3} · conditional ×{:.3} · applied ×{:.3} · effective ×{:.3}",
                component_index + 1,
                values.condition.label(),
                if values.active { "present" } else { "absent" },
                values.base_multiplier,
                values.conditional_multiplier,
                values.applied_multiplier,
                values.effective_multiplier,
            ));
        }
        if let Some(hp) = selection.attacker_hp
            && let Some(values) =
                part.low_hp_breakdown(selection.np_level, selection.overcharge_level, hp)
        {
            details.push(format!(
                "Base ×{:.3} + missing-HP bonus ×{:.3} = effective ×{:.3}",
                values.base_multiplier, values.hp_contribution, values.effective_multiplier,
            ));
        }
    }
    if let Some(scale) = &np.affection {
        details.push(format!(
            "Affection level {} · damage ×{:.2}",
            selection.affection_level,
            scale.multiplier(selection.affection_level)
        ));
    }
    if np.defense_pierce {
        details.push("Ignores Defense Up · retains Defense Down".into());
    }
    if !details.is_empty() {
        ui.collapsing(RichText::new("NP details").size(TEXT_SMALL), |ui| {
            for line in &details {
                note(ui, line);
            }
        });
    }
}

/// Floating summary shown over narrow layouts, where the result panel is
/// below the inputs.
pub fn sticky_result_summary(
    ui: &mut egui::Ui,
    servant: &ServantRecord,
    selection: &TurnSelection,
    result: &TurnDamageResult,
) {
    summary_frame(ui, "DAMAGE BY CARD", servant, selection, result);
}

fn summary_frame(
    ui: &mut egui::Ui,
    title: &str,
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
            ui.label(RichText::new(title).size(TEXT_SMALL).color(ACCENT));
            let mut lines: Vec<String> = selection
                .slots
                .iter()
                .enumerate()
                .map(|(index, card)| {
                    format!(
                        "{} · {}: {}",
                        index + 1,
                        selected_card_short_label(servant, *card),
                        format_damage_range(result.cards[index])
                    )
                })
                .collect();
            if let Some(extra) = result.extra {
                lines.push(format!("Extra: {}", format_damage_range(extra)));
            }
            // One live region, so a change is announced once, not per line.
            let response = ui.label(RichText::new(lines.join("\n")).size(TEXT_BODY));
            mark_live_status(ui.ctx(), response.id);
        });
}

pub(super) fn damage_result(ui: &mut egui::Ui, result: DamageResult) {
    egui::Frame::new()
        .fill(RESULT_PANEL)
        .stroke(egui::Stroke::new(1.0, RESULT_BORDER))
        .corner_radius(10)
        .inner_margin(14)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new("ESTIMATED DAMAGE")
                    .size(TEXT_SMALL)
                    .color(ACCENT),
            );
            let range = format_damage_range(result);
            let width = ui.available_width();
            let mut font_size = (width / 9.0).clamp(TEXT_DAMAGE_MIN, TEXT_DAMAGE);
            let measured_width = ui
                .painter()
                .layout_no_wrap(range.clone(), egui::FontId::proportional(font_size), TEXT)
                .size()
                .x;
            if measured_width > width {
                font_size *= width / measured_width;
            }
            ui.label(RichText::new(range).size(font_size).strong().color(TEXT));
            note(
                ui,
                &format!(
                    "Before the random roll (×0.900–1.099): {}",
                    format_integer(result.damage_before_random.round() as u32)
                ),
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

/// The formula's terms in the order they are applied. Terms that cannot
/// affect this card (an Extra factor of ×1, a flat addition of 0) are left out.
pub(super) fn breakdown_rows(result: DamageResult) -> Vec<(&'static str, String)> {
    let mut rows = vec![
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
    ];
    if result.extra_attack_multiplier != 1.0 {
        rows.push((
            "Extra attack factor",
            format!("×{:.2}", result.extra_attack_multiplier),
        ));
    }
    if result.flat_damage_addition != 0.0 {
        rows.push((
            "Flat addition after random",
            format!("+{:.1}", result.flat_damage_addition),
        ));
    }
    rows
}

pub(super) fn breakdown(ui: &mut egui::Ui, result: DamageResult) {
    ui.collapsing(RichText::new("Calculation breakdown").strong(), |ui| {
        ui.add_space(8.0);
        egui::Grid::new("damage_breakdown")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .max_col_width((ui.available_width() - 12.0) * 0.65)
            .show(ui, |ui| {
                for (label, value) in breakdown_rows(result) {
                    ui.label(RichText::new(label).color(TEXT_MUTED));
                    ui.label(value);
                    ui.end_row();
                }
            });
    });
}
