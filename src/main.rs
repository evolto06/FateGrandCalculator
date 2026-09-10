use eframe::egui::{self, Color32, RichText};
use fate_grand_calculator::damage::{CardType, DamageInput, DamageResult, calculate};

const BACKGROUND: Color32 = Color32::from_rgb(16, 19, 28);
const PANEL: Color32 = Color32::from_rgb(27, 32, 46);
const PANEL_MUTED: Color32 = Color32::from_rgb(34, 40, 56);
const RESULT_PANEL: Color32 = Color32::from_rgb(25, 42, 61);
const TEXT_MUTED: Color32 = Color32::from_rgb(159, 170, 189);
const ACCENT: Color32 = Color32::from_rgb(99, 194, 255);
const BUSTER: Color32 = Color32::from_rgb(200, 75, 81);
const ARTS: Color32 = Color32::from_rgb(61, 142, 218);
const QUICK: Color32 = Color32::from_rgb(65, 177, 116);

struct CalculatorApp {
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

    fn card_selector(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("COMMAND CARD").size(11.0).color(TEXT_MUTED));
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            for (card_type, label, color) in [
                (CardType::Buster, "Buster", BUSTER),
                (CardType::Arts, "Arts", ARTS),
                (CardType::Quick, "Quick", QUICK),
            ] {
                let selected = self.card_type == card_type;
                let button = egui::Button::new(RichText::new(label).strong().color(if selected {
                    Color32::WHITE
                } else {
                    TEXT_MUTED
                }))
                .fill(if selected { color } else { PANEL_MUTED })
                .stroke(egui::Stroke::new(
                    1.0,
                    if selected {
                        color
                    } else {
                        Color32::TRANSPARENT
                    },
                ))
                .corner_radius(7);

                if ui.add_sized([84.0, 34.0], button).clicked() {
                    self.card_type = card_type;
                }
            }
        });
    }

    fn percent_input(ui: &mut egui::Ui, label: &str, value: &mut f64) {
        ui.label(RichText::new(label).size(12.0).color(TEXT_MUTED));
        ui.add_sized(
            [150.0, 28.0],
            egui::DragValue::new(value)
                .speed(0.5)
                .range(-100.0..=999.0)
                .suffix("%"),
        );
    }

    fn multiplier_input(ui: &mut egui::Ui, label: &str, value: &mut f64) {
        ui.label(RichText::new(label).size(12.0).color(TEXT_MUTED));
        ui.add_sized(
            [150.0, 28.0],
            egui::DragValue::new(value).speed(0.05).range(0.0..=10.0),
        );
    }

    fn result_panel(ui: &mut egui::Ui, result: DamageResult) {
        egui::Frame::new()
            .fill(RESULT_PANEL)
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(48, 95, 133)))
            .corner_radius(12)
            .inner_margin(18)
            .show(ui, |ui| {
                ui.label(RichText::new("ESTIMATED DAMAGE").size(11.0).color(ACCENT));
                ui.add_space(5.0);
                ui.label(
                    RichText::new(format!(
                        "{}–{}",
                        result.minimum_damage, result.maximum_damage
                    ))
                    .size(32.0)
                    .strong()
                    .color(Color32::WHITE),
                );
                ui.label(
                    RichText::new(format!(
                        "Before random range: {:.0}",
                        result.damage_before_random
                    ))
                    .size(12.0)
                    .color(TEXT_MUTED),
                );
            });
    }

    fn breakdown(ui: &mut egui::Ui, result: DamageResult) {
        ui.collapsing(RichText::new("Calculation breakdown").strong(), |ui| {
            ui.add_space(8.0);
            egui::Grid::new("damage_breakdown")
                .num_columns(2)
                .spacing([40.0, 8.0])
                .show(ui, |ui| {
                    for (label, multiplier) in [
                        ("Base card", result.base_card_multiplier),
                        ("Attack", result.attack_multiplier),
                        ("Card buff", result.card_buff_multiplier),
                        ("Defense", result.defense_multiplier),
                        ("Class", result.class_multiplier),
                        ("Attribute", result.attribute_multiplier),
                    ] {
                        ui.label(RichText::new(label).color(TEXT_MUTED));
                        ui.label(format!("×{multiplier:.2}"));
                        ui.end_row();
                    }
                });
        });
    }
}

impl eframe::App for CalculatorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        ui.painter().rect_filled(ui.max_rect(), 0.0, BACKGROUND);
        ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.vertical(|ui| {
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.heading(RichText::new("FateGrandCalculator").size(27.0).strong());
                            ui.label(
                                RichText::new("Unofficial FGO damage calculator")
                                    .size(13.0)
                                    .color(TEXT_MUTED),
                            );
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new("NORMAL CARD · EARLY BUILD")
                                    .size(10.0)
                                    .color(ACCENT),
                            );
                        });
                    });

                    ui.add_space(16.0);
                    let result = calculate(self.damage_input());

                    ui.columns(2, |columns| {
                        egui::Frame::new()
                            .fill(PANEL)
                            .corner_radius(12)
                            .inner_margin(18)
                            .show(&mut columns[0], |ui| {
                                ui.heading(RichText::new("Attack setup").size(18.0));
                                ui.label(
                                    RichText::new("Configure the attack and modifiers.")
                                        .color(TEXT_MUTED),
                                );
                                ui.add_space(14.0);

                                ui.label(RichText::new("ATTACK").size(11.0).color(TEXT_MUTED));
                                ui.add_sized(
                                    [180.0, 30.0],
                                    egui::DragValue::new(&mut self.attack)
                                        .speed(100.0)
                                        .range(0..=999_999)
                                        .suffix(" ATK"),
                                );
                                ui.add_space(12.0);

                                self.card_selector(ui);
                                ui.add_space(14.0);
                                ui.separator();
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new("BUFFS & TARGET").size(11.0).color(TEXT_MUTED),
                                );
                                ui.add_space(4.0);
                                Self::percent_input(
                                    ui,
                                    "Attack buff",
                                    &mut self.attack_buff_percent,
                                );
                                Self::percent_input(ui, "Card buff", &mut self.card_buff_percent);
                                Self::percent_input(
                                    ui,
                                    "Enemy defense",
                                    &mut self.enemy_defense_percent,
                                );
                            });

                        egui::Frame::new()
                            .fill(PANEL)
                            .corner_radius(12)
                            .inner_margin(18)
                            .show(&mut columns[1], |ui| {
                                ui.heading(RichText::new("Result").size(18.0));
                                ui.label(
                                    RichText::new("Live calculation preview.").color(TEXT_MUTED),
                                );
                                ui.add_space(14.0);

                                Self::result_panel(ui, result);
                                ui.add_space(16.0);
                                ui.separator();
                                ui.add_space(8.0);
                                ui.label(RichText::new("MATCHUPS").size(11.0).color(TEXT_MUTED));
                                ui.add_space(4.0);
                                Self::multiplier_input(
                                    ui,
                                    "Class multiplier",
                                    &mut self.class_multiplier,
                                );
                                Self::multiplier_input(
                                    ui,
                                    "Attribute multiplier",
                                    &mut self.attribute_multiplier,
                                );
                                ui.add_space(12.0);
                                ui.separator();
                                ui.add_space(8.0);
                                Self::breakdown(ui, result);
                            });
                    });
                });
            });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "FateGrandCalculator",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([960.0, 650.0])
                .with_min_inner_size([760.0, 560.0]),
            ..Default::default()
        },
        Box::new(|_| Ok(Box::new(CalculatorApp::default()))),
    )
}
