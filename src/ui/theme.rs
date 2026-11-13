//! Colors, type sizes and the shared egui style.
//!
//! Every panel takes its colors and font sizes from here, so contrast and the
//! type scale can be checked in one place.

use eframe::egui::{self, Color32, Stroke};

pub const BACKGROUND: Color32 = Color32::from_rgb(16, 19, 28);
pub const PANEL: Color32 = Color32::from_rgb(27, 32, 46);
pub const PANEL_MUTED: Color32 = Color32::from_rgb(34, 40, 56);
pub const PANEL_HOVER: Color32 = Color32::from_rgb(44, 52, 72);
pub const RESULT_PANEL: Color32 = Color32::from_rgb(25, 42, 61);
pub const BORDER: Color32 = Color32::from_rgb(59, 69, 91);
pub const INPUT_BORDER: Color32 = Color32::from_rgb(64, 75, 97);
pub const RESULT_BORDER: Color32 = Color32::from_rgb(48, 95, 133);
pub const TEXT: Color32 = Color32::WHITE;
/// 6.2:1 or better on every panel color.
pub const TEXT_MUTED: Color32 = Color32::from_rgb(159, 170, 189);
pub const ACCENT: Color32 = Color32::from_rgb(99, 194, 255);
pub const ERROR: Color32 = Color32::from_rgb(255, 125, 125);

/// Card outline colors (at least 3:1 against `PANEL_MUTED`).
pub const BUSTER: Color32 = Color32::from_rgb(200, 75, 81);
pub const ARTS: Color32 = Color32::from_rgb(61, 142, 218);
pub const QUICK: Color32 = Color32::from_rgb(65, 177, 116);
pub const NP: Color32 = Color32::from_rgb(140, 130, 220);

/// Selected card fills. White captions on these are at least 4.5:1.
pub const BUSTER_FILL: Color32 = Color32::from_rgb(170, 64, 69);
pub const ARTS_FILL: Color32 = Color32::from_rgb(49, 114, 174);
pub const QUICK_FILL: Color32 = Color32::from_rgb(46, 124, 81);
pub const NP_FILL: Color32 = Color32::from_rgb(85, 83, 148);

/// The type scale. Nothing is drawn smaller than `TEXT_SMALL`.
pub const TEXT_SMALL: f32 = 12.0;
pub const TEXT_LABEL: f32 = 13.0;
pub const TEXT_BODY: f32 = 14.0;
pub const TEXT_SUBHEADING: f32 = 16.0;
pub const TEXT_HEADING: f32 = 18.0;
pub const TEXT_TITLE: f32 = 26.0;
pub const TEXT_TITLE_COMPACT: f32 = 22.0;
pub const TEXT_CARD_SYMBOL: f32 = 28.0;
pub const TEXT_DAMAGE: f32 = 30.0;
pub const TEXT_DAMAGE_MIN: f32 = 22.0;

/// Applies the dark theme to a style: widget fills, borders and spacing.
pub fn apply(style: &mut egui::Style) {
    let visuals = &mut style.visuals;
    visuals.dark_mode = true;
    visuals.panel_fill = BACKGROUND;
    visuals.window_fill = PANEL;
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.extreme_bg_color = PANEL_MUTED;
    visuals.faint_bg_color = PANEL_MUTED;
    visuals.override_text_color = None;
    visuals.selection.bg_fill = RESULT_BORDER;
    visuals.selection.stroke = Stroke::new(1.0, TEXT);

    let widgets = &mut visuals.widgets;
    for state in [&mut widgets.inactive, &mut widgets.open] {
        state.bg_fill = PANEL_MUTED;
        state.weak_bg_fill = PANEL_MUTED;
        state.bg_stroke = Stroke::new(1.0, INPUT_BORDER);
        state.fg_stroke = Stroke::new(1.0, TEXT);
    }
    widgets.hovered.bg_fill = PANEL_HOVER;
    widgets.hovered.weak_bg_fill = PANEL_HOVER;
    widgets.hovered.bg_stroke = Stroke::new(1.5, ACCENT);
    widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT);
    widgets.active.bg_fill = PANEL_HOVER;
    widgets.active.weak_bg_fill = PANEL_HOVER;
    widgets.active.bg_stroke = Stroke::new(2.0, ACCENT);
    widgets.active.fg_stroke = Stroke::new(1.0, TEXT);
    widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_MUTED);

    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.interact_size = egui::vec2(44.0, 44.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
}
