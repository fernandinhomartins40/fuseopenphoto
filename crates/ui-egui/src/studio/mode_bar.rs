//! Mode bar: Compor · Selecionar · Retocar · Cor e luz · Texto (`studio::Mode`), centred under
//! the top bar; picking a mode selects its default tool.

use crate::PhotocraftApp;

#[derive(Default)]
pub struct State {}

pub fn show(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    // Placeholder: the classic tool options bar.
    crate::panels::options_bar(app, ui);
}
