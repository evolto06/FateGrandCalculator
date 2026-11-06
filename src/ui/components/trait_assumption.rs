//! The manual enemy-trait assumption for audited NPs, and keeping it in sync with the selected NP.

use super::*;

#[derive(Clone, PartialEq)]
pub(super) struct TraitBonusBinding {
    pub(super) servant_id: u32,
    pub(super) np_id: u32,
    pub(super) condition: Option<TraitCondition>,
}

#[derive(Clone, Default)]
pub(super) struct TraitBonusUiState {
    pub(super) initialized: bool,
    pub(super) binding: Option<TraitBonusBinding>,
    pub(super) reset_feedback: Option<String>,
}

pub(super) fn trait_bonus_ui_id() -> egui::Id {
    egui::Id::new("enemy_trait_assumption")
}

// Only the desktop servant-data updater resets these.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn reset_trait_bonus_for_catalog(context: &egui::Context) {
    context.data_mut(|data| {
        data.insert_temp(
            trait_bonus_ui_id(),
            TraitBonusUiState {
                reset_feedback: Some(
                    "Enemy trait assumption reset after the servant data update.".into(),
                ),
                ..Default::default()
            },
        )
    });
}

pub(super) fn sync_trait_bonus(
    context: &egui::Context,
    servant: &ServantRecord,
    turn: &mut TurnSelection,
) {
    let binding = turn.slots.iter().find_map(|card| {
        let SelectedCard::NoblePhantasm(index) = card else {
            return None;
        };
        let np = servant.noble_phantasms.get(*index)?;
        Some(TraitBonusBinding {
            servant_id: servant.id,
            np_id: np.id,
            condition: np.trait_bonus_condition(),
        })
    });
    let mut state = context
        .data(|data| data.get_temp::<TraitBonusUiState>(trait_bonus_ui_id()))
        .unwrap_or_default();
    if !state.initialized || state.binding != binding {
        if state.initialized {
            state.reset_feedback = Some(
                "Enemy trait assumption reset because the servant, NP, or condition changed."
                    .into(),
            );
        }
        turn.trait_bonus = None;
        state.binding = binding;
    }
    state.initialized = true;
    context.data_mut(|data| data.insert_temp(trait_bonus_ui_id(), state));
}

pub(super) fn trait_bonus_settings(
    ui: &mut egui::Ui,
    servant: &ServantRecord,
    turn: &mut TurnSelection,
) -> Option<egui::Response> {
    sync_trait_bonus(ui.ctx(), servant, turn);
    let state = ui
        .ctx()
        .data(|data| data.get_temp::<TraitBonusUiState>(trait_bonus_ui_id()))
        .unwrap_or_default();
    if let Some(message) = state.reset_feedback {
        status_message(ui, &message);
    }
    let condition = state.binding?.condition?;
    ui.label(RichText::new("Enemy trait at NP damage time").strong());
    let mut matching = turn.trait_bonus == Some(condition);
    let response = ui.checkbox(&mut matching, condition.assumption_label());
    if response.changed() {
        turn.trait_bonus = matching.then_some(condition);
        ui.ctx().data_mut(|data| {
            let mut state = data
                .get_temp::<TraitBonusUiState>(trait_bonus_ui_id())
                .unwrap_or_default();
            state.reset_feedback = None;
            data.insert_temp(trait_bonus_ui_id(), state);
        });
    }
    ui.label(
        RichText::new(if matching {
            "Matching assumed: conditional NP multiplier applies."
        } else {
            "Matching not assumed: base NP damage applies."
        })
        .size(12.0)
        .color(TEXT_MUTED),
    );
    ui.label(RichText::new("Set the condition at damage time, including any traits granted before damage. Trait-granting effects are not simulated; enemy class and attribute do not set this assumption.").size(12.0).color(TEXT_MUTED));
    Some(response)
}
