//! The contextual panel beside the rail: Texto (Caractere/Parágrafo), the adjustment editor
//! (Ajuste/Máscara), Efeitos, the layer or document Properties, and the panel picked on the rail.
//!
//! It reuses the existing panels' bodies (`panels::dock_body`, the type sections, the adjustment
//! editors, the mask properties) and the Layer Style dialog's model (`layer_style::panel`, the
//! inline Efeitos editor) in the studio card chrome, so every editor stays one. Sections the
//! mockups fold (Transformar, Avançado, Máscara) are [`fold`] rows.

use egui::{Align, CornerRadius, Layout, Rect, Response, RichText, Sense, Stroke, StrokeKind, pos2, vec2};
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
    /// The Efeitos editor's session (the style being edited, what Reverter brings back).
    pub(crate) fx: Option<crate::layer_style::panel::FxEdit>,
    /// The card's rect last frame (what its content must stay inside).
    pub(crate) card: Option<Rect>,
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
    let card = crate::direction::end_panel("studio-right-panel")
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
                    // Nothing draws past the card: long rows are cut at its padding.
                    ui.set_clip_rect(ui.clip_rect().intersect(ui.max_rect()));
                    header(app, ui, &t, view);
                    if !app.studio.panel.collapsed {
                        ui.add_space(6.0);
                        widgets::hairline(ui);
                        ui.add_space(8.0);
                        let tab = app.studio.panel.tab;
                        ui.push_id(("studio-panel", view, tab), |ui| body(app, ui, view, tab));
                    }
                })
                .response
                .rect
        });
    app.studio.panel.card = Some(card.inner);
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
        (View::Text, _) => scrolled(ui, |ui| text(app, ui)),
        (View::Adjustment, 1) => scrolled(ui, |ui| mask(app, ui)),
        (View::Adjustment, _) if selected(app) == Selected::Adjustment => scrolled(ui, |ui| adjustment(app, ui)),
        (View::AddAdjustment, _) => dock_panel(app, ui, Group::Properties, "Adjustments"),
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

/// Efeitos: the inline Layer Style editor (every effect kind, its parameters, Preview, Reverter
/// and Aplicar); the full dialog stays one button away.
fn effects(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let mut fx = app.studio.panel.fx.take();
    crate::layer_style::panel::show(app, ui, &mut fx);
    app.studio.panel.fx = fx;
}

/// Texto: Character, Paragraph and Type Options of the selected type layer, then the folded
/// Transform (position, size, opacity, fill) and Advanced (the type layer's quick actions). With no
/// type layer, Character and Paragraph edit the next type layer's defaults.
fn text(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let Some((doc, Some(id))) = app.session.active().map(|st| (st.doc.clone(), st.active_layer)) else { return defaults_text(app, ui) };
    let Some(layer) = doc.layer(id).filter(|l| matches!(l.content, LayerContent::Text(_))) else { return defaults_text(app, ui) };
    crate::type_tool::type_properties(app, ui);
    if fold(ui, "text-transform", tl!("Transform"), None) {
        crate::panels::layer_controls(app, ui, layer);
    }
    if fold(ui, "text-advanced", tl!("Advanced"), Some(tl!("Quick actions for the type layer"))) {
        quick_actions(app, ui, &layer.content);
    }
}

fn defaults_text(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    crate::panels::dock_body(app, ui, Group::Character, 0);
    crate::panels::dock_body(app, ui, Group::Character, 1);
}

/// The layer kind's Quick Actions (Convert to Shape, Rasterize Type…), one outlined button each.
fn quick_actions(app: &mut PhotocraftApp, ui: &mut egui::Ui, content: &LayerContent) {
    let ctx = ui.ctx().clone();
    for &(label, id) in crate::props_layout::quick_actions(content) {
        if outlined_button(ui, None, tl!(label), ui.available_width()).clicked() {
            let result = crate::menus::invoke(app, &ctx, id, json!({}));
            report(app, result);
        }
        ui.add_space(4.0);
    }
}

/// An adjustment layer (Níveis, Matiz/Saturação…): its name, the shared editor, Preview (the
/// layer's visibility) and Clip to layer, Reset, then the folded Mask and Advanced (opacity, fill).
fn adjustment(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let Some((doc, Some(id))) = app.session.active().map(|st| (st.doc.clone(), st.active_layer)) else { return };
    let Some(layer) = doc.layer(id) else { return };
    let LayerContent::Adjustment(adj) = &layer.content else { return };
    // A targeted mask thumbnail shows the mask, as the classic Properties panel does.
    if crate::mask_props_ui::targeted(app, layer) || crate::mask_props_ui::vector_targeted(app, layer) {
        return mask(app, ui);
    }
    ui.horizontal(|ui| {
        ui.set_min_height(28.0);
        let (r, _) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::hover());
        icons::paint(ui, r, crate::props_layout::kind_icon(&layer.content), 18.0, t.icon);
        ui.label(RichText::new(tl!(adj.label())).size(15.0).color(t.text));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let (r, resp) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::hover());
            icons::paint(ui, r, "info", 16.0, t.text_faint);
            resp.on_hover_text(&layer.name);
        });
    });
    ui.add_space(6.0);
    crate::adjust_editors::layer_editor(app, ui, id, adj);
    ui.add_space(10.0);
    let (mut visible, mut clipped) = (layer.visible, layer.clipped);
    let mut run: Option<Value> = None;
    ui.horizontal(|ui| {
        let preview = crate::i18n::tr_ctx(crate::i18n::current(), "studio", "Preview");
        if widgets::checkbox(ui, &mut visible, preview).changed() {
            run = Some(json!({"layer": id.0, "visible": visible}));
        }
        ui.add_space(16.0);
        if widgets::checkbox(ui, &mut clipped, tl!("Clip to layer")).changed() {
            run = Some(json!({"layer": id.0, "clipped": clipped}));
        }
    });
    if let Some(p) = run {
        let result = app.run("layer.setProps", p);
        report(app, result);
    }
    ui.add_space(8.0);
    if outlined_button(ui, Some("rotate-ccw"), tl!("Reset to defaults"), ui.available_width()).clicked() {
        // Photoshop's two-step Reset where measured (Color Balance), else the defaults.
        let params = crate::adjust_editors::properties_reset(ui.ctx(), doc.id.0, id, adj).unwrap_or_else(|| json!({"layer": id.0}));
        let result = app.run("layer.setAdjustment", params);
        report(app, result);
        app.live_adjust = None;
    }
    if fold(ui, "adjustment-mask", tl!("Mask"), Some(tl!("The layer's mask"))) {
        mask(app, ui);
    }
    if fold(ui, "adjustment-advanced", tl!("Advanced"), Some(tl!("Opacity and fill of the layer"))) {
        crate::panels::layer_controls(app, ui, layer);
    }
}

/// A folding section row: a hairline, then the chevron, the title and an optional ⓘ with `tip`
/// (closed until opened, as the mockups show Transformar, Avançado and Máscara). Returns whether
/// it is open.
fn fold(ui: &mut egui::Ui, id: &str, title: &str, tip: Option<&str>) -> bool {
    let t = Tokens::get(ui.ctx());
    let key = egui::Id::unique(("studio-fold", id));
    let mut open = ui.data(|d| d.get_temp::<bool>(key)).unwrap_or(false);
    ui.add_space(6.0);
    widgets::hairline(ui);
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 36.0), Sense::click());
    if resp.clicked() {
        open = !open;
        ui.data_mut(|d| d.insert_temp(key, open));
    }
    if resp.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(6), t.hover);
    }
    let chevron = if open { "chevron-down" } else { "chevron-right" };
    icons::paint(ui, Rect::from_center_size(pos2(rect.left() + 10.0, rect.center().y), vec2(16.0, 16.0)), chevron, 15.0, t.text_dim);
    ui.painter().text(pos2(rect.left() + 28.0, rect.center().y), egui::Align2::LEFT_CENTER, title, egui::FontId::proportional(14.0), t.text);
    if tip.is_some() {
        icons::paint(ui, Rect::from_center_size(pos2(rect.right() - 12.0, rect.center().y), vec2(16.0, 16.0)), "info", 15.0, t.text_faint);
    }
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::DisclosureTriangle, true, open, title));
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    if let Some(tip) = tip {
        resp.on_hover_text(tip);
    }
    if open {
        ui.add_space(4.0);
    }
    open
}

/// The mockups' outlined pill button (Redefinir…, Reverter): an optional leading icon, the label
/// centred, at least `width` wide.
pub(crate) fn outlined_button(ui: &mut egui::Ui, icon: Option<&str>, label: &str, width: f32) -> Response {
    let t = Tokens::get(ui.ctx());
    let enabled = ui.is_enabled();
    let color = if enabled { t.text } else { t.text_faint };
    let galley = ui.painter().layout_no_wrap(label.to_owned(), egui::FontId::proportional(13.0), color);
    let icon_w = if icon.is_some() { 22.0 } else { 0.0 };
    let content = galley.size().x + icon_w;
    let (rect, resp) = ui.allocate_exact_size(vec2(width.max(content + 28.0), 30.0), Sense::click());
    if resp.hovered() && enabled {
        ui.painter().rect_filled(rect, CornerRadius::same(15), t.hover);
    }
    ui.painter().rect_stroke(rect, CornerRadius::same(15), Stroke::new(1.0, if enabled { t.text_dim } else { t.card_border }), StrokeKind::Inside);
    let x = rect.center().x - content / 2.0;
    if let Some(icon) = icon {
        icons::paint(ui, Rect::from_center_size(pos2(x + 8.0, rect.center().y), vec2(16.0, 16.0)), icon, 15.0, color);
    }
    ui.painter().galley(pos2(x + icon_w, rect.center().y - galley.size().y / 2.0), galley, color);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, label));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
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

    /// Every widget of every view the card shows, in pt-BR (long labels), at the card's width,
    /// stays within the card left to right: nothing is cut at its edges ("enchimento").
    #[test]
    fn every_view_fits_the_card_in_portuguese() {
        use egui_kittest::{Harness, kittest::NodeT, kittest::Queryable};
        let mut h = Harness::builder().with_size(vec2(1600.0, 900.0)).with_step_dt(1.0 / 60.0).build_eframe(|cc| {
            crate::PhotocraftApp::setup_context(&cc.egui_ctx, Default::default());
            let mut app = crate::PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
            app.session.edit_prefs(|p| p.interface.language = "pt-br".into());
            app.run("file.new", json!({"width": 1200, "height": 800})).unwrap();
            app
        });
        h.run_steps(5);
        let mut views: Vec<(&str, Option<&'static str>, Option<View>, usize)> = vec![("Painéis", Some(PANELS), None, 0)];
        views.extend(DOCK_PANELS.iter().map(|p| (p.1, Some(p.0), None, 0)));
        views.extend([("Properties", None, Some(View::Properties), 0), ("Properties/Effects", None, Some(View::Properties), 1)]);
        let mut bad = Vec::new();
        let mut check = |h: &mut Harness<'_, crate::PhotocraftApp>, name: &str, pick: Option<&'static str>, pinned: Option<View>, tab: usize| {
            h.state_mut().studio.rail.pick = pick;
            h.state_mut().studio.panel.pinned = pinned;
            h.run_steps(2);
            h.state_mut().studio.panel.tab = tab;
            h.run_steps(3);
            let card = h.state().studio.panel.card.expect("the card draws");
            // PHOTOCRAFT_PANEL_SNAPSHOTS=<dir> saves each view, to look at.
            if let Some(dir) = std::env::var_os("PHOTOCRAFT_PANEL_SNAPSHOTS") {
                let _gpu = photocraft_testkit::gpu_lock();
                let file = format!("{}-{tab}.png", name.replace(['/', ' '], "-"));
                h.render().expect("render").save(std::path::Path::new(&dir).join(file)).expect("save");
            }
            assert!(card.width() > 100.0, "{name}: no card");
            for node in h.query_all(egui_kittest::kittest::by().include_labels()) {
                let Some(b) = node.accesskit_node().bounding_box() else { continue };
                let r = Rect::from_min_max(pos2(b.x0 as f32, b.y0 as f32), pos2(b.x1 as f32, b.y1 as f32));
                if !r.intersects(card) || r.contains_rect(card) || r.width() < 1.0 {
                    continue;
                }
                if r.left() < card.left() - 0.5 || r.right() > card.right() + 0.5 {
                    bad.push(format!("{name}: {:?} {:?} at {r:?} outside {card:?}", node.accesskit_node().role(), node.accesskit_node().label()));
                }
            }
        };
        for (name, pick, pinned, tab) in views {
            check(&mut h, name, pick, pinned, tab);
        }
        // The Background showed the document's Properties; a pixel layer shows its own.
        h.state_mut().run("layer.new.layer", json!({})).unwrap();
        for tab in [0, 1] {
            check(&mut h, "Layer", None, Some(View::Properties), tab);
        }
        // A type layer (Texto | Efeitos) and an adjustment layer (Ajuste | Máscara).
        h.state_mut().run("type.create", json!({"text": "Preenchimento", "size": 40, "x": 10, "y": 60})).unwrap();
        for tab in [0, 1] {
            check(&mut h, "Text", None, Some(View::Text), tab);
        }
        h.state_mut().run("layer.newAdjustmentLayer.hueSaturation", json!({})).unwrap();
        for tab in [0, 1] {
            check(&mut h, "Adjustment", None, Some(View::Adjustment), tab);
        }
        h.state_mut().run("layer.newAdjustmentLayer.levels", json!({})).unwrap();
        check(&mut h, "Levels", None, Some(View::Adjustment), 0);
        assert!(bad.is_empty(), "{} widgets leave the card:
{}", bad.len(), bad.join("
"));
    }

    #[test]
    fn every_dock_panel_has_a_window_command() {
        // The list skips Window › panel commands whose panel the card draws, so each must exist.
        assert!(DOCK_PANELS.iter().all(|p| crate::menu_catalog::CATALOG.iter().any(|e| e.3 == format!("window.panel.{}", p.0))));
    }
}
