mod app;
#[cfg(not(target_arch = "wasm32"))]
mod portrait;
mod ui;

use app::CalculatorApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "FateGrandCalculator",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([960.0, 650.0])
                .with_min_inner_size([360.0, 520.0]),
            ..Default::default()
        },
        Box::new(|_| Ok(Box::new(CalculatorApp::default()))),
    )
}
