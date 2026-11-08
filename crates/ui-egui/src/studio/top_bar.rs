//! Top bar: ☰ (the whole menu tree), app name, document title, undo/redo, command search (⌘K),
//! save status and the Export button; also the window's drag area and caption buttons.

use crate::PhotocraftApp;

#[derive(Default)]
pub struct State {}

pub fn show(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    // Placeholder until the studio top bar lands: the classic title bar with menus.
    crate::panels::title_bar(app, ui);
}
