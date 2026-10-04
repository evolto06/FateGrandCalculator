use eframe::egui;
use fate_grand_calculator::damage::{TurnBuffs, calculate_turn};
use fate_grand_calculator::loader::GameData;
use fate_grand_calculator::model::{AttributeType, ClassType, TurnSelection};
#[cfg(not(target_arch = "wasm32"))]
use fate_grand_calculator::servant_data::{ServantDataService, UpdateReport, format_retrieved_at};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc::{self, Receiver, Sender};

#[cfg(not(target_arch = "wasm32"))]
use crate::portrait::PortraitStream;
use crate::ui;

pub struct CalculatorApp {
    game_data: Result<GameData, String>,
    selected_servant_id: u32,
    servant_search: String,
    turn_selection: Option<TurnSelection>,
    active_attack_slot: usize,
    buffs: ui::TurnBuffInputs,
    enemy_class: ClassType,
    enemy_attribute: AttributeType,
    data_status: String,
    #[cfg(not(target_arch = "wasm32"))]
    portrait: PortraitStream,
    #[cfg(not(target_arch = "wasm32"))]
    update_in_progress: bool,
    #[cfg(not(target_arch = "wasm32"))]
    update_sender: Sender<Result<UpdateReport, String>>,
    #[cfg(not(target_arch = "wasm32"))]
    update_receiver: Receiver<Result<UpdateReport, String>>,
}

impl Default for CalculatorApp {
    fn default() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let startup = ServantDataService::load_local_or_bundled();
        #[cfg(target_arch = "wasm32")]
        let startup = (
            GameData::bundled(),
            "Using bundled servant data.".to_owned(),
        );

        #[cfg(not(target_arch = "wasm32"))]
        let (game_data, data_status) = (startup.game_data, startup.status);
        #[cfg(target_arch = "wasm32")]
        let (game_data, data_status) = startup;

        let selected_servant_id = game_data
            .as_ref()
            .ok()
            .and_then(|data| data.servants.first())
            .map_or(0, |servant| servant.id);
        let turn_selection = game_data
            .as_ref()
            .ok()
            .and_then(|data| data.servant(selected_servant_id))
            .and_then(TurnSelection::default_for);
        #[cfg(not(target_arch = "wasm32"))]
        let (update_sender, update_receiver) = mpsc::channel();

        Self {
            game_data,
            selected_servant_id,
            servant_search: String::new(),
            turn_selection,
            active_attack_slot: 0,
            buffs: ui::TurnBuffInputs::default(),
            enemy_class: ClassType::Lancer,
            enemy_attribute: AttributeType::Sky,
            data_status,
            #[cfg(not(target_arch = "wasm32"))]
            portrait: PortraitStream::default(),
            #[cfg(not(target_arch = "wasm32"))]
            update_in_progress: false,
            #[cfg(not(target_arch = "wasm32"))]
            update_sender,
            #[cfg(not(target_arch = "wasm32"))]
            update_receiver,
        }
    }
}

impl eframe::App for CalculatorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))]
        self.poll_update_result(ui.ctx());

        #[cfg(not(target_arch = "wasm32"))]
        {
            let selection = self.game_data.as_ref().ok().and_then(|data| {
                data.servant(self.selected_servant_id)
                    .or_else(|| data.servants.first())
                    .map(|servant| (data.region.clone(), servant.id))
            });
            self.portrait.sync_selection(
                selection
                    .as_ref()
                    .map(|(region, id)| (region.as_str(), *id)),
                ui.ctx(),
            );
        }

        ui::apply_canvas(ui);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let viewport_width = ui.available_width();
                let content_width = viewport_width.min(1440.0);
                let narrow_layout = content_width < 820.0;
                ui.set_min_width(viewport_width);
                ui.horizontal(|ui| {
                    if viewport_width > content_width {
                        let side_padding = (viewport_width - content_width) / 2.0;
                        let layout_gap = ui.spacing().item_spacing.x;
                        ui.add_space((side_padding - layout_gap).max(0.0));
                    }
                    ui.allocate_ui_with_layout(
                        egui::vec2(content_width, 0.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_width(content_width);
                            ui.vertical(|ui| {
                                ui::header(ui);
                                ui.add_space(16.0);

                                #[cfg(not(target_arch = "wasm32"))]
                                {
                                    let update_clicked = ui.horizontal_wrapped(|ui| {
                                        let clicked = ui
                                            .add_enabled(
                                                !self.update_in_progress,
                                                egui::Button::new(if self.update_in_progress {
                                                    "Updating servant data…"
                                                } else {
                                                    "Update servant data"
                                                }),
                                            )
                                            .clicked();
                                        ui::status_message(ui, &self.data_status);
                                        clicked
                                    });
                                    if update_clicked.inner {
                                        self.start_update(ui.ctx());
                                    }
                                    ui.add_space(8.0);
                                }

                                #[cfg(target_arch = "wasm32")]
                                ui::status_message(ui, &self.data_status);

                                let data = match &self.game_data {
                                    Ok(data) => data,
                                    Err(error) => {
                                        ui.colored_label(
                                            egui::Color32::LIGHT_RED,
                                            format!("Game data could not be loaded: {error}"),
                                        );
                                        return;
                                    }
                                };
                                #[cfg(not(target_arch = "wasm32"))]
                                let (portrait_texture, portrait_message, portrait_error) = (
                                    self.portrait.texture(),
                                    self.portrait.message(),
                                    self.portrait.error(),
                                );
                                #[cfg(target_arch = "wasm32")]
                    let (portrait_texture, portrait_message, portrait_error): (
                        Option<&egui::TextureHandle>,
                        &str,
                        Option<&str>,
                    ) = (None, "No portrait\navailable", None);

                                let selected_servant_id = &mut self.selected_servant_id;
                                let servant_search = &mut self.servant_search;
                                let turn_selection = &mut self.turn_selection;
                                let active_attack_slot = &mut self.active_attack_slot;
                                let buffs = &mut self.buffs;
                                let enemy_class = &mut self.enemy_class;
                                let enemy_attribute = &mut self.enemy_attribute;

                                if !narrow_layout {
                                    let available_width = ui.available_width();
                                    let column_gap = ui.spacing().item_spacing.x;
                                    let columns_width = available_width - column_gap;
                                    let attack_width = columns_width * 0.56;
                                    let result_width = columns_width - attack_width;
                                    ui.horizontal_top(|ui| {
                                        ui.allocate_ui_with_layout(
                                            egui::vec2(attack_width, 0.0),
                                            egui::Layout::top_down(egui::Align::Min),
                                            |ui| {
                                                ui::attack_panel(
                                                    ui,
                                                    data,
                                                    selected_servant_id,
                                                    servant_search,
                                                    portrait_texture,
                                                    portrait_message,
                                                    portrait_error,
                                                    turn_selection,
                                                    active_attack_slot,
                                                    buffs,
                                                )
                                            },
                                        );
                                        ui.allocate_ui_with_layout(
                                            egui::vec2(result_width, 0.0),
                                            egui::Layout::top_down(egui::Align::Min),
                                            |ui| {
                                                ui::matchup_panel(ui, enemy_class, enemy_attribute);
                                                ui.add_space(10.0);
                                                show_result(
                                                    ui,
                                                    data,
                                                    *selected_servant_id,
                                                    turn_selection.as_ref(),
                                                    buffs.values(),
                                                    *enemy_class,
                                                    *enemy_attribute,
                                                    !narrow_layout,
                                                );
                                            },
                                        );
                                    });
                                } else {
                                    ui::attack_panel(
                                        ui,
                                        data,
                                        selected_servant_id,
                                        servant_search,
                                        portrait_texture,
                                        portrait_message,
                                        portrait_error,
                                        turn_selection,
                                        active_attack_slot,
                                        buffs,
                                    );
                                    ui.add_space(10.0);
                                    ui::matchup_panel(ui, enemy_class, enemy_attribute);
                                    ui.add_space(10.0);
                                    show_result(
                                        ui,
                                        data,
                                        *selected_servant_id,
                                        turn_selection.as_ref(),
                                        buffs.values(),
                                        *enemy_class,
                                        *enemy_attribute,
                                        !narrow_layout,
                                    );
                                }
                                if narrow_layout {
                                    ui.add_space(150.0);
                                }
                            });
                        },
                    );
                });
            });

        if ui.available_width() < 820.0 {
            let result = self.game_data.as_ref().ok().and_then(|data| {
                let servant = data.servant(self.selected_servant_id)?;
                let selection = self.turn_selection.as_ref()?;
                let result = calculate_turn(
                    servant,
                    selection,
                    self.buffs.values(),
                    self.enemy_class,
                    self.enemy_attribute,
                )
                .ok()?;
                Some((servant, selection, result))
            });
            if let Some((servant, selection, result)) = result {
                let summary_width = (ui.ctx().content_rect().width() - 24.0).clamp(280.0, 520.0);
                egui::Area::new(egui::Id::new("sticky_damage_summary"))
                    .order(egui::Order::Foreground)
                    .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -8.0))
                    .interactable(false)
                    .show(ui.ctx(), |ui| {
                        ui.set_width(summary_width);
                        ui::sticky_result_summary(ui, servant, selection, &result);
                    });
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl CalculatorApp {
    fn start_update(&mut self, context: &egui::Context) {
        self.update_in_progress = true;
        self.data_status = "Contacting Atlas Academy…".into();

        let sender = self.update_sender.clone();
        let context = context.clone();
        let worker = std::thread::Builder::new()
            .name("servant-data-update".into())
            .spawn(move || {
                let result =
                    std::panic::catch_unwind(ServantDataService::update).unwrap_or_else(|_| {
                        Err(
                            "The servant data updater stopped unexpectedly; current data was kept."
                                .into(),
                        )
                    });
                let _ = sender.send(result);
                context.request_repaint();
            });

        if let Err(error) = worker {
            self.update_in_progress = false;
            self.data_status = format!("Could not start the update: {error}");
        }
    }

    fn poll_update_result(&mut self, context: &egui::Context) {
        match self.update_receiver.try_recv() {
            Ok(Ok(report)) => {
                ui::reset_enemy_status_for_catalog(context);
                let resets_hp = ui::has_hp_inputs(context, self.selected_servant_id)
                    || self
                        .game_data
                        .as_ref()
                        .ok()
                        .and_then(|data| data.servant(self.selected_servant_id))
                        .zip(self.turn_selection.as_ref())
                        .is_some_and(|(servant, turn)| {
                            turn.slots.iter().any(|card| {
                                if let fate_grand_calculator::model::SelectedCard::NoblePhantasm(
                                    index,
                                ) = card
                                {
                                    servant
                                        .noble_phantasms
                                        .get(*index)
                                        .is_some_and(|np| np.requires_attacker_hp())
                                } else {
                                    false
                                }
                            })
                        });
                ui::clear_overcharge_feedback(context, self.selected_servant_id);
                ui::clear_hp_inputs(context, self.selected_servant_id);
                if report.game_data.servant(self.selected_servant_id).is_none() {
                    self.selected_servant_id = report
                        .game_data
                        .servants
                        .first()
                        .map_or(0, |servant| servant.id);
                }
                ui::clear_overcharge_feedback(context, self.selected_servant_id);
                ui::clear_hp_inputs(context, self.selected_servant_id);
                let total = report.game_data.servants.len();
                let skipped = report.skipped_rows;
                let updated_at = format_retrieved_at(&report.game_data.retrieved_at);
                self.game_data = Ok(report.game_data);
                self.turn_selection = self
                    .game_data
                    .as_ref()
                    .ok()
                    .and_then(|data| data.servant(self.selected_servant_id))
                    .and_then(TurnSelection::default_for);
                self.active_attack_slot = 0;
                self.update_in_progress = false;
                self.data_status = if skipped == 0 {
                    format!("Updated {total} servants · saved locally at {updated_at}.")
                } else {
                    format!(
                        "Updated {total} servants · saved locally at {updated_at}. Skipped {skipped} incomplete entries."
                    )
                };
                if resets_hp {
                    self.data_status.push_str(if self.turn_selection.as_ref().is_some_and(|turn| turn.attacker_hp.is_some()) {
                        " HP inputs reset to full HP from the updated servant data."
                    } else {
                        " HP inputs reset. Enter maximum HP manually if the selected NP requires HP."
                    });
                }
                self.data_status
                    .push_str(" Enemy status reset to absent after the servant data update.");
            }
            Ok(Err(error)) => {
                self.update_in_progress = false;
                self.data_status = format!("Update failed; current data was kept. {error}");
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                self.update_in_progress = false;
                self.data_status = "The servant data updater stopped unexpectedly.".into();
            }
        }
    }
}

fn show_result(
    ui: &mut egui::Ui,
    data: &GameData,
    servant_id: u32,
    selection: Option<&TurnSelection>,
    buffs: TurnBuffs,
    enemy_class: ClassType,
    enemy_attribute: AttributeType,
    announce_result: bool,
) {
    let Some(servant) = data.servant(servant_id) else {
        ui.colored_label(
            egui::Color32::LIGHT_RED,
            "The selected servant is missing from the loaded servant data.",
        );
        return;
    };
    let Some(selection) = selection else {
        ui.colored_label(
            egui::Color32::LIGHT_RED,
            servant
                .deck_note
                .as_deref()
                .unwrap_or("Card data is unavailable. Update servant data to load it."),
        );
        return;
    };
    match calculate_turn(servant, selection, buffs, enemy_class, enemy_attribute) {
        Ok(result) => ui::result_panel(ui, servant, selection, &result, announce_result),
        Err(error) => {
            ui.colored_label(
                egui::Color32::LIGHT_RED,
                format!("Could not calculate this sequence: {error}"),
            );
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    fn app_with_bundled_data() -> CalculatorApp {
        let data = GameData::bundled().unwrap();
        let selected_servant_id = data.servants[0].id;
        let turn_selection = TurnSelection::default_for(&data.servants[0]);
        let (update_sender, update_receiver) = mpsc::channel();
        CalculatorApp {
            game_data: Ok(data),
            selected_servant_id,
            servant_search: String::new(),
            turn_selection,
            active_attack_slot: 2,
            buffs: ui::TurnBuffInputs::default(),
            enemy_class: ClassType::Lancer,
            enemy_attribute: AttributeType::Sky,
            data_status: String::new(),
            portrait: PortraitStream::default(),
            update_in_progress: true,
            update_sender,
            update_receiver,
        }
    }

    #[test]
    fn successful_data_update_clears_reset_feedback_and_resets_overcharge() {
        for removes_selected_servant in [false, true] {
            let mut app = app_with_bundled_data();
            let context = egui::Context::default();
            let previous_id = app.selected_servant_id;
            let mut updated = GameData::bundled().unwrap();
            if removes_selected_servant {
                updated.servants.remove(0);
            }
            let next_id = updated.servants[0].id;
            for id in [previous_id, next_id] {
                context.data_mut(|data| {
                    data.insert_temp(
                        egui::Id::new(("overcharge_reset_feedback", id)),
                        "Overcharge reset: update servant data.".to_owned(),
                    )
                });
            }
            app.turn_selection.as_mut().unwrap().overcharge_level = 5;
            app.update_sender
                .send(Ok(UpdateReport {
                    game_data: updated,
                    skipped_rows: 0,
                }))
                .unwrap();
            app.poll_update_result(&context);
            assert_eq!(app.selected_servant_id, next_id);
            assert_eq!(app.turn_selection.as_ref().unwrap().overcharge_level, 1);
            assert_eq!(app.active_attack_slot, 0);
            for id in [previous_id, next_id] {
                assert!(
                    context
                        .data(|data| data
                            .get_temp::<String>(egui::Id::new(("overcharge_reset_feedback", id))))
                        .is_none()
                );
            }
        }
    }

    #[test]
    fn failed_data_update_keeps_current_reset_feedback() {
        let mut app = app_with_bundled_data();
        let context = egui::Context::default();
        let id = egui::Id::new(("overcharge_reset_feedback", app.selected_servant_id));
        context.data_mut(|data| data.insert_temp(id, "Current reset feedback".to_owned()));
        app.update_sender
            .send(Err("Test network failure".to_owned()))
            .unwrap();
        app.poll_update_result(&context);
        assert_eq!(
            context.data(|data| data.get_temp::<String>(id)).as_deref(),
            Some("Current reset feedback")
        );
        assert!(
            app.data_status
                .starts_with("Update failed; current data was kept.")
        );
    }

    #[test]
    fn catalog_update_resets_status_and_failed_update_keeps_assumption() {
        use fate_grand_calculator::np_mechanics::enemy_status::EnemyStatus;
        for succeeds in [false, true] {
            let mut app = app_with_bundled_data();
            let context = egui::Context::default();
            app.turn_selection.as_mut().unwrap().enemy_status = Some(EnemyStatus::Poison);
            let update = if succeeds {
                Ok(UpdateReport {
                    game_data: GameData::bundled().unwrap(),
                    skipped_rows: 0,
                })
            } else {
                Err("Test failure".into())
            };
            app.update_sender.send(update).unwrap();
            app.poll_update_result(&context);
            assert_eq!(
                app.turn_selection.as_ref().unwrap().enemy_status,
                if succeeds {
                    None
                } else {
                    Some(EnemyStatus::Poison)
                }
            );
            assert_eq!(
                app.data_status.contains("Enemy status reset to absent"),
                succeeds
            );
        }
    }

    #[test]
    fn data_update_reports_hp_resets_and_failure_preserves_hp_edits() {
        use fate_grand_calculator::model::AttackerHp;
        for invalid_edit in [false, true] {
            for succeeds in [false, true] {
                let mut app = app_with_bundled_data();
                let context = egui::Context::default();
                let servant_id = app.selected_servant_id;
                let hp = if invalid_edit {
                    None
                } else {
                    Some(AttackerHp::new(33, 100).unwrap())
                };
                app.turn_selection.as_mut().unwrap().attacker_hp = hp;
                let current_text = if invalid_edit { "not-an-integer" } else { "33" };
                ui::test_hp_inputs(&context, servant_id, Some((current_text, "100")));
                if succeeds {
                    let mut updated = GameData::bundled().unwrap();
                    updated.servants[0].max_hp = Some(200);
                    app.update_sender
                        .send(Ok(UpdateReport {
                            game_data: updated,
                            skipped_rows: 0,
                        }))
                        .unwrap();
                } else {
                    app.update_sender
                        .send(Err("Test network failure".into()))
                        .unwrap();
                }
                app.poll_update_result(&context);
                if succeeds {
                    assert_eq!(
                        app.turn_selection.as_ref().unwrap().attacker_hp,
                        Some(AttackerHp::new(200, 200).unwrap())
                    );
                    assert!(ui::test_hp_inputs(&context, servant_id, None).is_none());
                    assert!(
                        app.data_status
                            .contains("HP inputs reset to full HP from the updated servant data.")
                    );
                } else {
                    assert_eq!(app.turn_selection.as_ref().unwrap().attacker_hp, hp);
                    assert_eq!(
                        ui::test_hp_inputs(&context, servant_id, None),
                        Some((current_text.into(), "100".into()))
                    );
                    assert!(!app.data_status.contains("HP inputs reset"));
                }
            }
        }
    }
}
