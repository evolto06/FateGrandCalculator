use eframe::egui;
use fate_grand_calculator::damage::{CardType, DamageInput, calculate};

use crate::ui;

pub struct CalculatorApp {
    attack: u32,
    card_type: CardType,
    attack_buff_percent: f64,
    card_buff_percent: f64,
    enemy_defense_percent: f64,
    class_multiplier: f64,
    attribute_multiplier: f64,
}

impl Default for CalculatorApp {
    fn default() -> Self {
        Self {
            attack: 12_000,
            card_type: CardType::Buster,
            attack_buff_percent: 0.0,
            card_buff_percent: 0.0,
            enemy_defense_percent: 0.0,
            class_multiplier: 1.0,
            attribute_multiplier: 1.0,
        }
    }
}

impl CalculatorApp {
    fn damage_input(&self) -> DamageInput {
        DamageInput {
            attack: self.attack,
            card_type: self.card_type,
            attack_buff: self.attack_buff_percent / 100.0,
            card_buff: self.card_buff_percent / 100.0,
            enemy_defense: self.enemy_defense_percent / 100.0,
            class_multiplier: self.class_multiplier,
            attribute_multiplier: self.attribute_multiplier,
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

                    let result = calculate(self.damage_input());
                    ui.columns(2, |columns| {
                        ui::attack_panel(
                            &mut columns[0],
                            &mut self.attack,
                            &mut self.card_type,
                            &mut self.attack_buff_percent,
                            &mut self.card_buff_percent,
                            &mut self.enemy_defense_percent,
                        );
                        ui::result_panel(
                            &mut columns[1],
                            result,
                            &mut self.class_multiplier,
                            &mut self.attribute_multiplier,
                        );
                    });
                });
            });
    }
}
