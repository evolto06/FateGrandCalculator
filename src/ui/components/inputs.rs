//! Buff percentage inputs and their parsing.

use super::*;

#[derive(Debug, Clone)]
pub struct PercentInput {
    pub(super) text: String,
}

impl Default for PercentInput {
    fn default() -> Self {
        Self { text: "0".into() }
    }
}

impl PercentInput {
    /// A blank field, or a sign or decimal point on its own while the user is
    /// still typing, counts as 0 rather than an error.
    pub fn value(&self) -> Result<f64, &'static str> {
        let text = self.text.trim();
        if matches!(text, "" | "-" | "+" | "." | "-." | "+.") {
            return Ok(0.0);
        }
        match text.parse::<f64>() {
            Ok(value) if value.is_finite() && (-100.0..=999.0).contains(&value) => Ok(value),
            _ => Err("Enter a number from −100 to 999."),
        }
    }

    #[cfg(test)]
    pub(crate) fn set_text(&mut self, text: &str) {
        self.text = text.into();
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
    pub fn values(&self) -> Result<TurnBuffs, String> {
        let parse = |label: &str, input: &PercentInput| {
            input
                .value()
                .map(percent_to_modifier)
                .map_err(|error| format!("{label}: {error}"))
        };
        Ok(TurnBuffs {
            attack_buff: parse("Attack buff", &self.attack)?,
            buster_buff: parse("Buster buff", &self.buster)?,
            arts_buff: parse("Arts buff", &self.arts)?,
            quick_buff: parse("Quick buff", &self.quick)?,
            np_damage_buff: parse("NP damage buff", &self.np_damage)?,
            enemy_defense: parse("Enemy defense", &self.enemy_defense)?,
        })
    }
}

/// The attacker's buffs, laid out in as many columns as fit.
pub(super) fn buff_inputs(ui: &mut egui::Ui, buffs: &mut TurnBuffInputs) {
    group_label(ui, "BUFFS");
    note(ui, "Enter 20 for a 20% value.");
    ui.add_space(4.0);
    let mut fields = [
        ("Attack buff (%)", &mut buffs.attack),
        ("NP damage buff (%)", &mut buffs.np_damage),
        ("Buster buff (%)", &mut buffs.buster),
        ("Arts buff (%)", &mut buffs.arts),
        ("Quick buff (%)", &mut buffs.quick),
    ];
    let width = ui.available_width();
    let columns = if width >= 600.0 {
        3
    } else if width >= 380.0 {
        2
    } else {
        1
    };
    for row in fields.chunks_mut(columns) {
        ui.columns(columns, |cells| {
            for (cell, (label, input)) in cells.iter_mut().zip(row.iter_mut()) {
                percent_input(cell, label, input);
            }
        });
    }
}

pub(super) fn percent_input(ui: &mut egui::Ui, label: &str, value: &mut PercentInput) {
    let label_response = ui.label(RichText::new(label).size(TEXT_BODY).color(TEXT_MUTED));
    let input_width = ui.available_width().min(220.0);
    let response = ui.add_sized(
        [input_width, 44.0],
        egui::TextEdit::singleline(&mut value.text)
            .id_salt(("percent_input", label))
            .desired_width(input_width)
            .vertical_align(egui::Align::Center)
            .hint_text("0")
            .suffix("%")
            .text_color(TEXT)
            .frame(
                egui::Frame::new()
                    .fill(PANEL_MUTED)
                    .stroke(egui::Stroke::new(1.0, INPUT_BORDER))
                    .corner_radius(6)
                    .inner_margin(egui::Margin::symmetric(10, 6)),
            ),
    );
    response.labelled_by(label_response.id);

    if let Err(error) = value.value() {
        let error_response = ui.colored_label(ERROR, error);
        mark_live_status(ui.ctx(), error_response.id);
    } else if label == "NP damage buff (%)" && value.value().is_ok_and(|value| value > 500.0) {
        note(ui, "NP damage buff capped at 500%.");
    }
}
