//! The contextual panel beside the rail: Texto (Caractere/Parágrafo), the adjustment editor
//! (Ajuste/Máscara), Efeitos, the layer or document Properties, and the panel picked on the rail.
//!
//! It only wraps the existing panels' bodies (`panels::dock_body`, the type sections, the mask
//! properties, the Layer Style dialog) in the studio card chrome, so every editor stays one.

use egui::{Align, CornerRadius, Layout, Rect, Response, RichText, Sense, Stroke, pos2, vec2};
use photocraft_doc::LayerContent;
use serde_json::{Value, json};

use crate::PhotocraftApp;
use crate::dock::Group;
use crate::studio::Mode;
use crate::studio::right_rail::PANELS;
use crate::theme::Tokens;
use crate::{icons, widgets};

/// The card's width (points).
const WIDTH: f32 = 320.0;
/// Gap between the canvas and the card.
const GAP: i8 = 12;

#[derive(Default)]
pub struct State {
    /// Only the header shows.
    pub collapsed: bool,
    /// Pinned: this view stays while the mode or the selected layer changes.
    pub pinned: Option<View>,
    /// The selected header tab, and the view it was picked in (a new view starts on its first).
    tab: usize,
    tab_view: Option<View>,
}

/// What the card shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum View {
    /// A dock panel picked on the rail (its key), or [`PANELS`]: the list of every panel.
    Panel(&'static str),
    /// Texto: Character and Paragraph | Effects.
    Text,
    /// An adjustment layer: its editor | its mask.
    Adjustment,
    /// Cor e luz with no adjustment layer selected: the Adjustments grid.
    AddAdjustment,
    /// The layer's or document's Properties | Effects.
    Properties,
}

/// The kind of the selected layer, as far as the card cares.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selected {
    Nothing,
    Text,
    Adjustment,
    Other,
}

/// The dock panels the card can draw: (key, English label as in `Group::tabs`, group).
pub(crate) const DOCK_PANELS: [(&str, &str, Group); 18] = [
    ("properties", "Properties", Group::Properties),
    ("adjustments", "Adjustments", Group::Properties),
    ("character", "Character", Group::Character),
    ("paragraph", "Paragraph", Group::Character),
    ("color", "Color", Group::Color),
    ("swatches", "Swatches", Group::Color),
    ("gradients", "Gradients", Group::Color),
    ("patterns", "Patterns", Group::Color),
    ("navigator", "Navigator", Group::Navigator),
    ("histogram", "Histogram", Group::Navigator),
    ("info", "Info", Group::Navigator),
    ("history", "History", Group::History),
    ("actions", "Actions", Group::History),
    ("layerComps", "Layer Comps", Group::History),
    ("layers", "Layers", Group::Layers),
    ("channels", "Channels", Group::Layers),
    ("paths", "Paths", Group::Layers),
    ("brushes", "Brushes", Group::Brushes),
];

/// Window › panel commands that do nothing in the studio shell (they show classic chrome).
const CLASSIC_ONLY: [&str; 3] = ["window.panel.options", "window.panel.tools", "window.panel.contextualTaskBar"];

/// The card's view by priority: the rail's pick, the pinned view, then the mode and the layer.
pub fn view(pick: Option<&'static str>, pinned: Option<View>, mode: Mode, selected: Selected) -> View {
    if let Some(key) = pick {
        return View::Panel(key);
    }
    if let Some(v) = pinned {
        return v;
    }
    match (mode, selected) {
        (Mode::Text, _) | (_, Selected::Text) => View::Text,
        (_, Selected::Adjustment) => View::Adjustment,
        (Mode::ColorLight, _) => View::AddAdjustment,
        _ => View::Properties,
    }
}

/// The header's tabs (English); none means the header shows [`title`].
fn tabs(view: View) -> &'static [&'static str] {
    match view {
        View::Text => &["Text", "Effects"],
        View::Adjustment => &["Adjustment", "Mask"],
        View::Properties => &["Properties", "Effects"],
        View::Panel(_) | View::AddAdjustment => &[],
    }
}

fn title(view: View) -> &'static str {
    match view {
        View::Panel(key) => DOCK_PANELS.iter().find(|p| p.0 == key).map_or("Panels", |p| p.1),
        View::AddAdjustment => "Adjustments",
        View::Text => "Text",
        View::Adjustment => "Adjustment",
        View::Properties => "Properties",
    }
}

fn selected(app: &PhotocraftApp) -> Selected {
    let Some(layer) = app.session.active().and_then(|st| st.doc.layer(st.active_layer?)) else { return Selected::Nothing };
    match layer.content {
        LayerContent::Text(_) => Selected::Text,
        LayerContent::Adjustment(_) => Selected::Adjustment,
        _ => Selected::Other,
    }
}

pub fn show(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let backdrop = crate::prefs_ui::pasteboard_color(app).unwrap_or(t.canvas);
    let view = view(app.studio.rail.pick, app.studio.panel.pinned, app.studio.mode, selected(app));
    let st = &mut app.studio.panel;
    if st.tab_view != Some(view) {
        st.tab = 0;
        st.tab_view = Some(view);
    }
    crate::direction::end_panel("studio-right-panel")
        .resizable(false)
        .exact_size(WIDTH + f32::from(GAP))
        .frame(egui::Frame::NONE.fill(backdrop).inner_margin(egui::Margin { left: GAP, right: 0, top: 12, bottom: 12 }))
        .show(ui, |ui| {
            egui::Frame::NONE
                .fill(t.card)
                .stroke(Stroke::new(1.0, t.card_border))
                .corner_radius(CornerRadius::same(12))
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    header(app, ui, &t, view);
                    if !app.studio.panel.collapsed {
                        ui.add_space(6.0);
                        widgets::hairline(ui);
                        ui.add_space(8.0);
                        let tab = app.studio.panel.tab;
                        ui.push_id(("studio-panel", view, tab), |ui| body(app, ui, view, tab));
                    }
                });
        });
}

/// Grip, title or tabs, then pin (or close for a rail pick) and the collapse chevron.
fn header(app: &mut PhotocraftApp, ui: &mut egui::Ui, t: &Tokens, view: View) {
    ui.horizontal(|ui| {
        ui.set_min_height(28.0);
        grip(ui, t);
        let names = tabs(view);
        if names.is_empty() {
            ui.label(RichText::new(tl!(title(view))).size(15.0).strong().color(t.text));
        }
        for (i, name) in names.iter().enumerate() {
            if widgets::pill_tab(ui, tl!(name), i == app.studio.panel.tab).clicked() {
                app.studio.panel.tab = i;
            }
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let collapsed = app.studio.panel.collapsed;
            let (icon, tip) = if collapsed { ("chevron-down", "Expand") } else { ("chevron-up", "Collapse") };
            if icons::button(ui, icon, 24.0, false, tip).clicked() {
                app.studio.panel.collapsed = !collapsed;
            }
            if let View::Panel(_) = view {
                if icons::button(ui, "x", 24.0, false, "Close").clicked() {
                    app.studio.rail.pick = None;
                }
            } else {
                let pinned = app.studio.panel.pinned.is_some();
                let tip = if pinned { "Unpin panel" } else { "Pin panel" };
                if icons::button(ui, "pin", 24.0, pinned, tip).clicked() {
                    app.studio.panel.pinned = if pinned { None } else { Some(view) };
                }
            }
        });
    });
}

/// The ⋮⋮ grip: two columns of three dots.
fn grip(ui: &mut egui::Ui, t: &Tokens) {
    let (rect, _) = ui.allocate_exact_size(vec2(12.0, 28.0), Sense::hover());
    for col in 0..2u8 {
        for row in 0..3u8 {
            let p = pos2(rect.left() + 3.0 + f32::from(col) * 5.0, rect.center().y - 5.0 + f32::from(row) * 5.0);
            ui.painter().circle_filled(p, 1.3, t.text_faint);
        }
    }
}

fn body(app: &mut PhotocraftApp, ui: &mut egui::Ui, view: View, tab: usize) {
    match (view, tab) {
        (View::Panel(PANELS), _) => scrolled(ui, |ui| panel_list(app, ui)),
        (View::Panel(key), _) => {
            if let Some(&(_, label, group)) = DOCK_PANELS.iter().find(|p| p.0 == key) {
                dock_panel(app, ui, group, label);
            }
        }
        (View::Text | View::Properties, 1) => scrolled(ui, |ui| effects(app, ui)),
        (View::Text, _) if selected(app) != Selected::Text => {
            // No type layer: Character and Paragraph edit the next type layer's defaults.
            scrolled(ui, |ui| {
                crate::panels::dock_body(app, ui, Group::Character, 0);
                crate::panels::dock_body(app, ui, Group::Character, 1);
            });
        }
        (View::Adjustment, 1) => scrolled(ui, |ui| mask(app, ui)),
        (View::AddAdjustment, _) => dock_panel(app, ui, Group::Properties, "Adjustments"),
        // A type layer's Properties are its Character and Paragraph sections; an adjustment
        // layer's are its editor.
        _ => dock_panel(app, ui, Group::Properties, "Properties"),
    }
}

/// Content as tall as it is, up to the space left, then scrolling.
fn scrolled(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    ui.spacing_mut().scroll.fade.strength = 0.0;
    egui::ScrollArea::vertical().auto_shrink([false, true]).max_height(ui.available_height().max(0.0)).show(ui, add);
}

/// One dock panel's body, drawn as the dock draws it (`panel_docking::body`): its group's tab is
/// this panel while it draws, and panels that lay out their own list get the whole height.
fn dock_panel(app: &mut PhotocraftApp, ui: &mut egui::Ui, group: Group, label: &str) {
    let pro = Tokens::get(ui.ctx()).pro;
    let Some(tab) = group.tabs(pro).iter().position(|name| *name == label) else { return };
    let shown = std::mem::replace(group.tab_mut(&mut app.ui.dock_tabs), tab);
    if group.scrolls_itself(tab) {
        let size = vec2(ui.available_width(), ui.available_height().max(0.0));
        ui.allocate_ui(size, |ui| {
            ui.set_min_height(size.y);
            crate::panels::dock_body(app, ui, group, tab);
        });
    } else {
        scrolled(ui, |ui| crate::panels::dock_body(app, ui, group, tab));
    }
    *group.tab_mut(&mut app.ui.dock_tabs) = shown;
}

/// Painéis: every dock panel (drawn in the card) and every other Window panel (its own window).
fn panel_list(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    for (key, label, _) in DOCK_PANELS {
        if row(ui, &t, tl!(label), "chevron-right").clicked() {
            app.studio.rail.pick = Some(key);
        }
    }
    ui.add_space(4.0);
    widgets::hairline(ui);
    ui.add_space(4.0);
    let lang = crate::i18n::current();
    let ctx = ui.ctx().clone();
    for (path, label, _, id) in crate::menu_catalog::CATALOG {
        let docked = id.strip_prefix("window.panel.").is_some_and(|key| DOCK_PANELS.iter().any(|p| p.0 == key));
        if *path != ["Window"] || !id.starts_with("window.panel.") || docked || CLASSIC_ONLY.contains(id) {
            continue;
        }
        if row(ui, &t, crate::i18n::tr_id(lang, id, label), "panel-right").clicked() {
            let result = crate::menus::invoke(app, &ctx, id, json!({}));
            report(app, result);
        }
    }
}

/// A full-width list row: label and a trailing icon.
fn row(ui: &mut egui::Ui, t: &Tokens, label: &str, icon: &str) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 28.0), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(6), t.hover);
    }
    ui.painter().text(pos2(rect.left() + 8.0, rect.center().y), egui::Align2::LEFT_CENTER, label, egui::FontId::proportional(13.0), t.text);
    icons::paint(ui, Rect::from_center_size(pos2(rect.right() - 14.0, rect.center().y), vec2(16.0, 16.0)), icon, 14.0, t.text_faint);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), label));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Efeitos: Blending Options and each effect kind with its on/off box; a row's chevron opens the
/// Layer Style dialog on it, as does checking a kind the layer doesn't have yet.
fn effects(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    use crate::layer_style::{BLENDING, KINDS};
    let t = Tokens::get(ui.ctx());
    let Some((layer, items)) = app.session.active().and_then(|st| {
        let layer = st.doc.layer(st.active_layer?)?;
        Some((layer.id.0, layer.effects.items.iter().map(|e| (photocraft_engine::layer_style::kind_of(e), e.enabled())).collect::<Vec<_>>()))
    }) else {
        ui.label(RichText::new(tl!("Select a layer to edit its effects.")).color(t.text_faint));
        return;
    };
    let mut open: Option<Option<&str>> = None;
    let mut runs: Vec<(&str, Value)> = Vec::new();
    if row(ui, &t, tl!("Blending Options"), "chevron-right").clicked() {
        open = Some(Some(BLENDING));
    }
    for &(kind, name) in KINDS {
        let mine: Vec<usize> = items.iter().enumerate().filter(|(_, (k, _))| *k == kind).map(|(i, _)| i).collect();
        let mut on = mine.iter().any(|&i| items.get(i).is_some_and(|e| e.1));
        ui.horizontal(|ui| {
            ui.set_min_height(26.0);
            if widgets::checkbox(ui, &mut on, name).changed() {
                if mine.is_empty() {
                    open = Some(Some(kind));
                }
                for &index in &mine {
                    runs.push(("layer.setEffectsVisible", json!({"layer": layer, "visible": on, "index": index})));
                }
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if icons::button(ui, "chevron-right", 22.0, false, "Edit Layer Style…").clicked() {
                    open = Some(Some(kind));
                }
            });
        });
    }
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if widgets::secondary_button(ui, tl!("Layer Style…"), 0.0).clicked() {
            open = Some(None);
        }
        if !items.is_empty() && widgets::secondary_button(ui, tl!("Clear Layer Style"), 0.0).clicked() {
            runs.push(("layer.layerStyle.clear", json!({"layer": layer})));
        }
    });
    for (id, params) in runs {
        let result = app.run(id, params);
        report(app, result);
    }
    if let Some(select) = open {
        crate::layer_style::open(app, select);
    }
}

/// Máscara: the layer's pixel and vector mask properties, or Add Mask.
fn mask(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let Some(doc) = app.session.active().map(|st| (st.doc.clone(), st.active_layer)) else { return };
    let Some(layer) = doc.1.and_then(|id| doc.0.layer(id)) else { return };
    if layer.mask.is_some() {
        crate::mask_props_ui::properties(app, ui, layer);
    }
    if layer.vector_mask.is_some() {
        crate::mask_props_ui::vector_properties(app, ui, layer);
    }
    if layer.mask.is_none() && layer.vector_mask.is_none() && widgets::secondary_button(ui, tl!("Add Mask"), ui.available_width()).clicked() {
        let result = app.run("layer.layerMask.revealAll", json!({}));
        report(app, result);
    }
}

/// A refused command shows its reason in the status line, as the panels do.
fn report(app: &mut PhotocraftApp, result: Result<Value, String>) {
    if let Err(e) = result {
        app.ui.status = e;
        app.ui.status_error = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rail_pick_wins_then_the_pin_then_the_context() {
        assert_eq!(view(Some("history"), Some(View::Text), Mode::Text, Selected::Text), View::Panel("history"));
        assert_eq!(view(None, Some(View::Adjustment), Mode::Text, Selected::Text), View::Adjustment);
        assert_eq!(view(None, None, Mode::Text, Selected::Other), View::Text);
        assert_eq!(view(None, None, Mode::Compose, Selected::Text), View::Text);
        assert_eq!(view(None, None, Mode::Select, Selected::Adjustment), View::Adjustment);
        assert_eq!(view(None, None, Mode::ColorLight, Selected::Adjustment), View::Adjustment);
        assert_eq!(view(None, None, Mode::ColorLight, Selected::Other), View::AddAdjustment);
        assert_eq!(view(None, None, Mode::Retouch, Selected::Other), View::Properties);
        assert_eq!(view(None, None, Mode::Compose, Selected::Nothing), View::Properties);
    }

    #[test]
    fn every_dock_panel_is_a_tab_of_its_group_in_every_theme() {
        for (key, label, group) in DOCK_PANELS {
            for pro in [false, true] {
                assert!(group.tabs(pro).contains(&label), "{key}: {label} not in {group:?}");
            }
        }
    }

    #[test]
    fn every_dock_panel_has_a_window_command() {
        // The list skips Window › panel commands whose panel the card draws, so each must exist.
        assert!(DOCK_PANELS.iter().all(|p| crate::menu_catalog::CATALOG.iter().any(|e| e.3 == format!("window.panel.{}", p.0))));
    }
}
