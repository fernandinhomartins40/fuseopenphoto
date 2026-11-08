//! Right icon rail: Codex (opens the docked chat), then Histórico, Navegador, Info, Painéis (every
//! other panel). Those tiles pick what the right panel shows; clicking the lit tile again gives the panel back to the current context.

use egui::{Align2, CornerRadius, FontId, Rect, Response, Sense, Stroke, StrokeKind, pos2, vec2};

use crate::PhotocraftApp;
use crate::theme::Tokens;

#[derive(Default)]
pub struct State {
    /// The panel the rail picked: a dock panel key ("history", …) or [`PANELS`]; `None` leaves the
    /// right panel to the mode and the selected layer.
    pub pick: Option<&'static str>,
}

/// The Painéis tile's pick: the right panel lists every panel the app has.
pub const PANELS: &str = "panels";

/// (icon, English label, pick) top to bottom.
const TILES: [(&str, &str, &str); 4] = [("clock", "History", "history"), ("scan", "Navigator", "navigator"), ("info", "Info", "info"), ("layers", "Panels", PANELS)];

/// The Codex tile's command: it opens and closes the docked Codex chat.
const CODEX: &str = "window.toggle.codex";

const RAIL_W: f32 = 80.0;
const TILE_W: f32 = 64.0;
const TILE_H: f32 = 70.0;

pub fn show(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let backdrop = crate::prefs_ui::pasteboard_color(app).unwrap_or(t.canvas);
    crate::direction::end_panel("studio-rail")
        .resizable(false)
        .exact_size(RAIL_W)
        .frame(egui::Frame::NONE.fill(backdrop).inner_margin(egui::Margin { left: 8, right: 8, top: 12, bottom: 12 }))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            if tile(ui, &t, "wand-sparkles", tl!("Codex"), app.codex.open).clicked() {
                crate::menus::run_clicked(app, &ui.ctx().clone(), CODEX.to_string());
            }
            for (icon, label, key) in TILES {
                // "Info" stays short on the narrow tile (the plain row reads "Informações").
                let name = crate::i18n::tr_ctx(crate::i18n::current(), "studio-rail", label);
                let on = app.studio.rail.pick == Some(key);
                if tile(ui, &t, icon, name, on).clicked() {
                    app.studio.rail.pick = toggle(app.studio.rail.pick, key);
                    app.studio.panel.collapsed = false;
                }
            }
        });
}

/// A tile click: pick `key`, or drop the pick when it is already picked.
pub fn toggle(current: Option<&'static str>, key: &'static str) -> Option<&'static str> {
    if current == Some(key) { None } else { Some(key) }
}

/// A rounded icon-over-label tile, amber when its panel is showing.
fn tile(ui: &mut egui::Ui, t: &Tokens, icon: &str, label: &str, on: bool) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(TILE_W, TILE_H), Sense::click());
    let fill = if on {
        t.accent_soft
    } else if resp.hovered() {
        t.hover
    } else {
        t.card
    };
    let border = if on { t.accent_border } else { t.card_border };
    ui.painter().rect(rect, CornerRadius::same(12), fill, Stroke::new(1.0, border), StrokeKind::Inside);
    let tint = if on { t.accent } else { t.icon };
    crate::icons::paint(ui, Rect::from_center_size(pos2(rect.center().x, rect.top() + 26.0), vec2(24.0, 24.0)), icon, 22.0, tint);
    let text = if on { t.accent_text } else { t.text };
    ui.painter().text(pos2(rect.center().x, rect.bottom() - 16.0), Align2::CENTER_CENTER, label, FontId::proportional(11.0), text);
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, ui.is_enabled(), on, label));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tile_toggles_its_pick() {
        assert_eq!(toggle(None, "history"), Some("history"));
        assert_eq!(toggle(Some("history"), "history"), None);
        assert_eq!(toggle(Some("history"), PANELS), Some(PANELS));
    }

    /// The rail's Codex tile and the top bar's wand both open and close the Codex dock.
    #[test]
    fn the_codex_buttons_toggle_the_dock() {
        use egui_kittest::kittest::Queryable;
        let mut h = egui_kittest::Harness::builder().with_size(vec2(1600.0, 900.0)).build_eframe(|cc| {
            crate::PhotocraftApp::setup_context(&cc.egui_ctx, Default::default());
            let mut app = crate::PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
            app.run("file.new", serde_json::json!({"width": 64, "height": 48})).unwrap();
            app
        });
        h.run_steps(5);
        // Top bar's wand first (highest on screen), then the rail's tile.
        let click = |h: &mut egui_kittest::Harness<'_, crate::PhotocraftApp>, top: bool| {
            let mut all: Vec<_> = h.query_all_by_role_and_label(egui::accesskit::Role::Button, "Codex").collect();
            all.sort_by(|a, b| a.rect().min.y.total_cmp(&b.rect().min.y));
            assert!(all.len() >= 2, "a top-bar wand and a rail tile named Codex");
            let node = if top { all.first() } else { all.last() };
            node.unwrap().click();
            h.run_steps(3);
        };
        for top in [true, false] {
            assert!(!h.state().codex.open);
            click(&mut h, top);
            assert!(h.state().codex.open, "opens the dock (top bar: {top})");
            assert!(h.query_by_label("Conversation").is_some(), "the dock is drawn");
            click(&mut h, top);
            assert!(!h.state().codex.open, "closes it again (top bar: {top})");
            assert!(h.query_by_label("Conversation").is_none());
        }
    }

    #[test]
    fn every_tile_has_an_icon() {
        for (icon, ..) in TILES {
            assert!(crate::icons::exists(icon), "{icon}");
        }
    }
}
