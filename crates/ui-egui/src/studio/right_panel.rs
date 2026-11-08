//! The contextual panel beside the rail: Texto (Caractere/Parágrafo), Níveis/Curvas…, Efeitos,
//! Ajuste/Máscara, and the panel picked on the rail.

use crate::PhotocraftApp;

#[derive(Default)]
pub struct State {}

pub fn show(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    // Placeholder: the classic floating Properties window.
    let ctx = ui.ctx().clone();
    crate::panels::properties_window(app, &ctx);
}
