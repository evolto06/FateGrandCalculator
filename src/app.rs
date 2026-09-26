use eframe::egui;
use fate_grand_calculator::damage::{calculate, percent_to_modifier};
use fate_grand_calculator::loader::GameData;
use fate_grand_calculator::model::{AttributeType, CardType, ClassType};
#[cfg(not(target_arch = "wasm32"))]
use fate_grand_calculator::servant_data::{ServantDataService, UpdateReport};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc::{self, Receiver, Sender};

use crate::ui;

pub struct CalculatorApp {
    game_data: Result<GameData, String>,
    selected_servant_id: u32,
    card_type: CardType,
    attack_buff_percent: f64,
    card_buff_percent: f64,
    enemy_defense_percent: f64,
    enemy_class: ClassType,
    enemy_attribute: AttributeType,
    data_status: String,
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
        #[cfg(not(target_arch = "wasm32"))]
        let (update_sender, update_receiver) = mpsc::channel();

        Self {
            game_data,
            selected_servant_id,
            card_type: CardType::Buster,
            attack_buff_percent: 0.0,
            card_buff_percent: 0.0,
            enemy_defense_percent: 0.0,
            enemy_class: ClassType::Lancer,
            enemy_attribute: AttributeType::Sky,
            data_status,
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

        ui::apply_canvas(ui);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
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
                            ui.label(&self.data_status);
                            clicked
                        });
                        if update_clicked.inner {
                            self.start_update(ui.ctx());
                        }
                        ui.add_space(8.0);
                    }

                    #[cfg(target_arch = "wasm32")]
                    ui.label(&self.data_status);

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
                    let selected_servant_id = &mut self.selected_servant_id;
                    let card_type = &mut self.card_type;
                    let attack_buff_percent = &mut self.attack_buff_percent;
                    let card_buff_percent = &mut self.card_buff_percent;
                    let enemy_defense_percent = &mut self.enemy_defense_percent;
                    let enemy_class = &mut self.enemy_class;
                    let enemy_attribute = &mut self.enemy_attribute;

                    if ui.available_width() >= 820.0 {
                        ui.columns(2, |columns| {
                            ui::attack_panel(
                                &mut columns[0],
                                data,
                                selected_servant_id,
                                card_type,
                                attack_buff_percent,
                                card_buff_percent,
                                enemy_defense_percent,
                            );

                            columns[1].vertical(|ui| {
                                ui::matchup_panel(ui, enemy_class, enemy_attribute);
                                ui.add_space(10.0);
                                show_result(
                                    ui,
                                    data,
                                    *selected_servant_id,
                                    *card_type,
                                    *attack_buff_percent,
                                    *card_buff_percent,
                                    *enemy_defense_percent,
                                    *enemy_class,
                                    *enemy_attribute,
                                );
                            });
                        });
                    } else {
                        ui::attack_panel(
                            ui,
                            data,
                            selected_servant_id,
                            card_type,
                            attack_buff_percent,
                            card_buff_percent,
                            enemy_defense_percent,
                        );
                        ui.add_space(10.0);
                        ui::matchup_panel(ui, enemy_class, enemy_attribute);
                        ui.add_space(10.0);
                        show_result(
                            ui,
                            data,
                            *selected_servant_id,
                            *card_type,
                            *attack_buff_percent,
                            *card_buff_percent,
                            *enemy_defense_percent,
                            *enemy_class,
                            *enemy_attribute,
                        );
                    }
                });
            });
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
                self.game_data = Ok(report.game_data);
                self.update_in_progress = false;
                self.data_status = if skipped == 0 {
                    format!("Updated {total} servants from Atlas Academy and saved them locally.")
                } else {
                    format!(
                        "Updated {total} servants. Skipped {skipped} unsupported or incomplete entries."
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
    card_type: CardType,
    attack_buff_percent: f64,
    card_buff_percent: f64,
    enemy_defense_percent: f64,
    enemy_class: ClassType,
    enemy_attribute: AttributeType,
) {
    if let Some(servant) = data.servant(servant_id) {
        let input = servant.normal_card_input(
            card_type,
            percent_to_modifier(attack_buff_percent),
            percent_to_modifier(card_buff_percent),
            percent_to_modifier(enemy_defense_percent),
            enemy_class,
            enemy_attribute,
        );
        ui::result_panel(ui, calculate(input));
    } else {
        ui.colored_label(
            egui::Color32::LIGHT_RED,
            "The selected servant is missing from the loaded servant data.",
        );
    }
}
