mod app;
#[cfg(not(target_arch = "wasm32"))]
mod portrait;
mod ui;

use app::CalculatorApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    use eframe::egui;

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

/// Browser entry point: draws into the `<canvas id="app">` in `index.html`.
/// Build and serve it with `trunk serve` (see the README).
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    wasm_bindgen_futures::spawn_local(async {
        let Some(document) = eframe::web_sys::window().and_then(|window| window.document()) else {
            eframe::web_sys::console::error_1(&"No browser document is available.".into());
            return;
        };
        let Some(canvas) = document.get_element_by_id("app").and_then(|element| {
            element
                .dyn_into::<eframe::web_sys::HtmlCanvasElement>()
                .ok()
        }) else {
            eframe::web_sys::console::error_1(&"The page has no <canvas id=\"app\">.".into());
            return;
        };

        let started = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|_| Ok(Box::new(CalculatorApp::default()))),
            )
            .await;

        if let Some(loading) = document.get_element_by_id("loading") {
            match started {
                Ok(()) => loading.remove(),
                Err(error) => {
                    loading.set_text_content(Some("FateGrandCalculator could not start."));
                    eframe::web_sys::console::error_1(&error);
                }
            }
        }
    });
}
