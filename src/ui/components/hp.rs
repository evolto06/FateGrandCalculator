//! Attacker HP inputs for low-HP NPs, kept per servant in egui temp data.

use super::*;

pub(super) fn hp_inputs_id(servant_id: u32) -> egui::Id {
    egui::Id::new(("attacker_hp_inputs", servant_id))
}

pub(crate) fn clear_hp_inputs(context: &egui::Context, servant_id: u32) {
    context.data_mut(|data| {
        data.remove::<HpInputs>(hp_inputs_id(servant_id));
    });
}

// Only the desktop servant-data updater resets these.
#[cfg(not(target_arch = "wasm32"))]
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
pub(super) struct HpInputs {
    pub(super) current: String,
    pub(super) maximum: String,
    pub(super) reset_feedback: Option<String>,
}

impl HpInputs {
    pub(super) fn for_servant(servant: &ServantRecord, turn: &TurnSelection) -> Self {
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

    pub(super) fn integer(text: &str, label: &str) -> Result<u32, String> {
        if text.is_empty() {
            return Err(format!("Enter {label} as a positive whole number."));
        }
        if !text.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(format!("{label} must contain whole-number digits only."));
        }
        text.parse::<u32>()
            .map_err(|_| format!("{label} is too large."))
    }

    pub(super) fn value(&self) -> Result<AttackerHp, String> {
        AttackerHp::new(
            Self::integer(&self.current, "current HP")?,
            Self::integer(&self.maximum, "maximum HP")?,
        )
    }

    pub(super) fn maximum_changed(&mut self) {
        if let Ok(max) = Self::integer(&self.maximum, "maximum HP")
            && max > 0
        {
            self.current = max.to_string();
            self.reset_feedback =
                Some("Current HP reset to full HP because maximum HP changed.".into());
            return;
        }
        self.reset_feedback = None;
    }
}

pub(super) fn hp_settings(ui: &mut egui::Ui, servant: &ServantRecord, turn: &mut TurnSelection) {
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
