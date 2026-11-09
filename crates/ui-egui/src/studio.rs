//! The studio shell: the house layout that replaces the Photoshop-style chrome.
//!
//! ```text
//! ┌ top_bar ── ☰ · name · document · undo/redo · command search · saved · Export ─────────────┐
//! ├ mode_bar ─ Compor · Selecionar · Retocar · Cor e luz · Texto ─────────────────────────────┤
//! │ tools_menu (floating)   context_bars + options_pill  │ right_panel │ right_rail │
//! │                 canvas (CentralPanel)                 │ (contextual)│ Histórico… │
//! ├ bottom ─ layer properties bar · layer strip (Camadas/Canais/Caminhos) · status/zoom ─────┤
//! ```
//!
//! Every region only draws and dispatches the same commands as before (`app.run`,
//! `menus::invoke`), so no feature is lost; Window › Classic Layout brings the old shell back.
//! Each region owns its file and its `State` (in [`StudioState`]); `docs/redesign.md` is the spec.

use crate::PhotocraftApp;
use crate::theme::Tokens;

pub mod bottom;
pub mod context_bars;
pub mod mode_bar;
pub mod options_pill;
pub mod right_panel;
pub mod right_rail;
pub mod style;
pub mod tools_menu;
pub mod top_bar;

/// The five work modes of the mode bar. Each groups tools, the floating context bar and the
/// contextual panel; nothing is reachable only through one mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Compose,
    Select,
    Retouch,
    ColorLight,
    Text,
}

/// The studio shell's view state (not persisted).
#[derive(Default)]
pub struct StudioState {
    /// Window › Classic Layout: draw the previous Photoshop-style shell instead.
    pub classic: bool,
    pub mode: Mode,
    pub top: top_bar::State,
    pub modes: mode_bar::State,
    pub tools: tools_menu::State,
    pub bottom: bottom::State,
    pub rail: right_rail::State,
    pub panel: right_panel::State,
    pub context: context_bars::State,
}

/// Draw the studio shell around the canvas. Order matters: egui panels claim their edges in call
/// order, and the canvas takes what is left.
pub fn frame(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    top_bar::show(app, ui);
    mode_bar::show(app, ui);
    bottom::status(app, ui);
    if app.codex.open {
        // Codex runs down to the status line and the layer rows stay under the canvas (mockup 1):
        // the chat needs the height, above all on a laptop.
        right_rail::show(app, ui);
        crate::codex_panel::dock(app, ui);
        bottom::rows(app, ui);
        // Codex replaces the contextual panel; a panel the rail picks still shows beside it.
        if app.studio.rail.pick.is_some() {
            right_panel::show(app, ui);
        }
    } else {
        bottom::rows(app, ui);
        right_rail::show(app, ui);
        right_panel::show(app, ui);
    }
    let t = Tokens::get(ui.ctx());
    let backdrop = crate::prefs_ui::pasteboard_color(app).unwrap_or(t.canvas);
    egui::CentralPanel::default().frame(egui::Frame::NONE.fill(backdrop)).show(ui, |ui| {
        crate::canvas::document_area(app, ui);
        tools_menu::show(app, ui);
        context_bars::show(app, ui);
    });
}

/// Window › Classic Layout.
pub fn toggle_classic(app: &mut PhotocraftApp) {
    app.studio.classic = !app.studio.classic;
}
