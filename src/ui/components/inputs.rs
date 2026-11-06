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
    pub fn value(&self) -> Result<f64, &'static str> {
        let text = self.text.trim();
        if text.is_empty() {
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

pub(super) fn percent_input(ui: &mut egui::Ui, label: &str, value: &mut PercentInput) {
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
    response.labelled_by(label_response.id);

    if let Err(error) = value.value() {
        let error_response = ui.colored_label(ERROR, error);
        mark_live_status(ui.ctx(), error_response.id);
    } else if label == "NP damage buff (%)" && value.value().is_ok_and(|value| value > 500.0) {
        ui.label(RichText::new("NP damage buff capped at 500%.").color(TEXT_MUTED));
    }
}
