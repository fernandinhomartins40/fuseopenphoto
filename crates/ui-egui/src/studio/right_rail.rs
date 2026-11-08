//! Right icon rail: Histórico, Navegador, Info, Painéis (every other panel).

use crate::PhotocraftApp;

#[derive(Default)]
pub struct State {}

pub fn show(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    // Placeholder: the classic toolbar, so every tool stays reachable until the studio tools land.
    crate::panels::toolbar(app, ui);
}
