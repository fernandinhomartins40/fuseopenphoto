//! Bottom area: the layer properties bar (blend mode, opacity, fill, locks), the layer strip
//! (Camadas / Canais / Caminhos, layers as cards with thumbnail and mask), and the status bar
//! (zoom, mode, depth, size, layer count, file name).

use crate::PhotocraftApp;

#[derive(Default)]
pub struct State {}

pub fn show(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    // Placeholder: the classic status bar.
    crate::panels::status_bar(app, ui);
}
