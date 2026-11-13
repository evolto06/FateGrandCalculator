//! The manual enemy-status assumption for audited NPs, and keeping it in sync with the selected NP.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct EnemyStatusBinding {
    pub(super) servant_id: u32,
    pub(super) np_id: u32,
    pub(super) condition: Option<EnemyStatus>,
}

#[derive(Clone, Default)]
pub(super) struct EnemyStatusUiState {
    pub(super) initialized: bool,
    pub(super) binding: Option<EnemyStatusBinding>,
    pub(super) reset_feedback: Option<String>,
}

pub(super) fn enemy_status_ui_id() -> egui::Id {
    egui::Id::new("enemy_status_assumption")
}

// Only the desktop servant-data updater resets these.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn reset_enemy_status_for_catalog(context: &egui::Context) {
    context.data_mut(|data| {
        data.insert_temp(
            enemy_status_ui_id(),
            EnemyStatusUiState {
                initialized: false,
                binding: None,
                reset_feedback: Some(
                    "Enemy status reset to absent after the servant data update.".into(),
                ),
            },
        );
    });
}

pub(super) fn sync_enemy_status(
    context: &egui::Context,
    servant: &ServantRecord,
    turn: &mut TurnSelection,
) {
    let binding = turn.slots.iter().find_map(|card| {
        let SelectedCard::NoblePhantasm(index) = card else {
            return None;
        };
        let np = servant.noble_phantasms.get(*index)?;
        Some(EnemyStatusBinding {
            servant_id: servant.id,
            np_id: np.id,
            condition: np.enemy_status_condition(),
        })
    });
    let mut state = context
        .data(|data| data.get_temp::<EnemyStatusUiState>(enemy_status_ui_id()))
        .unwrap_or_default();
    if state.binding != binding {
        if state.initialized {
            state.reset_feedback = Some(
                "Enemy status reset to absent because the servant, NP, or condition changed."
                    .into(),
            );
        }
        turn.enemy_status = None;
        state.binding = binding;
    }
    state.initialized = true;
    context.data_mut(|data| data.insert_temp(enemy_status_ui_id(), state));
}

pub(super) fn enemy_status_settings(
    ui: &mut egui::Ui,
    servant: &ServantRecord,
    turn: &mut TurnSelection,
) -> Option<egui::Response> {
    sync_enemy_status(ui.ctx(), servant, turn);
    let state = ui
        .ctx()
        .data(|data| data.get_temp::<EnemyStatusUiState>(enemy_status_ui_id()))
        .unwrap_or_default();
    if let Some(message) = state.reset_feedback {
        status_message(ui, &message);
    }
    let binding = state.binding?;
    let condition = binding.condition?;
    ui.label(RichText::new("Enemy status at NP damage time").strong());
    let mut present = turn.enemy_status == Some(condition);
    let response = ui.checkbox(
        &mut present,
        format!("Matching {} present", condition.label()),
    );
    if response.changed() {
        turn.enemy_status = present.then_some(condition);
        ui.ctx().data_mut(|data| {
            let mut state = data
                .get_temp::<EnemyStatusUiState>(enemy_status_ui_id())
                .unwrap_or_default();
            state.reset_feedback = None;
            data.insert_temp(enemy_status_ui_id(), state);
        });
    }
    note(
        ui,
        if present {
            "Condition present: conditional NP multiplier applies."
        } else {
            "Condition absent: base NP damage applies."
        },
    );
    if condition == EnemyStatus::DefenseUp {
        note(
            ui,
            "Defense Up presence is separate from Enemy defense (%). This checkbox does not change the numeric defense value; enter it separately.",
        );
    }
    ui.collapsing(
        RichText::new("About the status assumption").size(TEXT_SMALL),
        |ui| {
            note(ui, "Set present only if the status actually landed before damage. Status application and success probability are not simulated; effects after damage do not activate this bonus.");
            note(
                ui,
                if condition.includes_ignored_individuality() {
                    "Also counts effects excluded from normal trait matching."
                } else {
                    "Effects excluded from trait matching do not count."
                },
            );
        },
    );
    Some(response)
}
