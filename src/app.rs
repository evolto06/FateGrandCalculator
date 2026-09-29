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
        self.poll_update_result();

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

    fn poll_update_result(&mut self) {
        match self.update_receiver.try_recv() {
            Ok(Ok(report)) => {
                if report.game_data.servant(self.selected_servant_id).is_none() {
                    self.selected_servant_id = report
                        .game_data
                        .servants
                        .first()
                        .map_or(0, |servant| servant.id);
                }
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
                self.update_in_progress = false;
                self.data_status = if skipped == 0 {
                    format!("Updated {total} servants · saved locally at {updated_at}.")
                } else {
                    format!(
                        "Updated {total} servants · saved locally at {updated_at}. Skipped {skipped} incomplete entries."
                    )
                };
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
