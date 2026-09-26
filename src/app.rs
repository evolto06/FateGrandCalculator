use eframe::egui;
use fate_grand_calculator::damage::{calculate, percent_to_modifier};
use fate_grand_calculator::loader::GameData;
use fate_grand_calculator::model::{AttributeType, CardType, ClassType};

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
}

impl Default for CalculatorApp {
    fn default() -> Self {
        let game_data = GameData::bundled();
        let selected_servant_id = game_data
            .as_ref()
            .ok()
            .and_then(|data| data.servants.first())
            .map_or(0, |servant| servant.id);

        Self {
            game_data,
            selected_servant_id,
            card_type: CardType::Buster,
            attack_buff_percent: 0.0,
            card_buff_percent: 0.0,
            enemy_defense_percent: 0.0,
            enemy_class: ClassType::Lancer,
            enemy_attribute: AttributeType::Sky,
        }
    }
}

impl eframe::App for CalculatorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        ui::apply_canvas(ui);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.vertical(|ui| {
                    ui::header(ui);
                    ui.add_space(16.0);

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
            "The selected servant is missing from the bundled data.",
        );
    }
}
