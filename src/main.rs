mod app;
mod ui;

use app::CalculatorApp;
use eframe::egui;

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
