//! Bottom area: the layer properties bar (blend mode, opacity, fill, locks), the layer strip
//! (Camadas / Canais / Caminhos, layers as cards with thumbnail and mask), and the status bar
//! (zoom, mode, depth, size, layer count, file name).
//!
//! Everything runs the classic Layers panel's commands (`layer.setProps`, `layer.select`,
//! `layer.moveTo`, …) and reuses its widgets, thumbnails, menus and rename field.

use egui::{RichText, vec2};
use photocraft_color::BlendMode;
use photocraft_doc::{Layer, LayerId};
use serde_json::{Value, json};

use crate::theme::Tokens;
use crate::{PhotocraftApp, icons, widgets};

mod strip;

/// Which list the layer strip shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Layers,
    Channels,
    Paths,
}

#[derive(Default)]
pub struct State {
    pub tab: Tab,
}

const STATUS_H: f32 = 34.0;
const STRIP_H: f32 = 150.0;
const PROPS_H: f32 = 42.0;
/// Width of the Opacity / Fill pop-up fields (the classic Layers panel's).
const PCT_W: f32 = 66.0 + widgets::POPUP_ARROW_W;

/// Draw the three rows. Bottom panels claim the edge in call order: the first is the lowest.
pub fn show(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let row = |id: &'static str, h: f32, fill| {
        egui::Panel::bottom(id).exact_size(h).frame(egui::Frame::NONE.fill(fill).inner_margin(egui::Margin::symmetric(12, 0)))
    };
    row("studio_status", STATUS_H, t.chrome).show(ui, |ui| status_line(app, ui));
    row("studio_layer_strip", STRIP_H, t.dock).show(ui, |ui| strip::show(app, ui));
    row("studio_layer_props", PROPS_H, t.chrome).show(ui, |ui| props_bar(app, ui));
}

// ------------------------------------------------------------------------------- status line

/// "RGB · 8 bits/channel · 2400 × 1500 px · 4 layers · name".
fn status_text(mode: &str, bits: u32, (w, h): (u32, u32), layers: usize, name: &str) -> String {
    let depth = crate::i18n::fmt(tl!("{bits} bits/channel"), &[("bits", &bits.to_string())]);
    let count = crate::i18n::trn(crate::i18n::current(), layers as u64, "{n} layer", "{n} layers");
    [mode, &depth, &format!("{w} × {h} px"), &count, name].join("  ·  ")
}

/// Zoom presets of the zoom field's menu, in percent.
const ZOOM_PRESETS: [f32; 8] = [12.5, 25.0, 33.33, 50.0, 66.67, 100.0, 200.0, 400.0];

fn status_line(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let ctx = ui.ctx().clone();
    crate::direction::horizontal_centered(ui, |ui| {
        if let (Some(st), Some(i)) = (app.session.active(), app.session.active_index())
            && let Some(view) = app.ui.views.get(i)
        {
            let text = status_text(
                crate::canvas::mode_label(&st.doc),
                st.doc.depth.bits(),
                (st.doc.size.width, st.doc.size.height),
                st.doc.layer_count(),
                &st.doc.name,
            );
            // The classic status bar's zoom field: type, scrub or step a percentage.
            let mut pct = view.zoom * 100.0;
            let range = crate::zoom_levels::percent_range(view);
            let field = widgets::compact_wrapping_value_field(ui, &mut pct, range, "%", vec2(70.0, 24.0)).accessible_name(tl!("Zoom"));
            let animated = app.session.prefs().general.animated_zoom;
            if field.changed()
                && let Some(view) = app.ui.views.get_mut(i)
            {
                crate::zoom_levels::set_from_field(view, field.dragged() || field.drag_started(), pct, animated);
            }
            let presets = icons::button(ui, "chevron-down", 22.0, false, tl!("Zoom"));
            egui::Popup::menu(&presets).show(|ui| {
                for p in ZOOM_PRESETS {
                    if ui.button(format!("{} %", crate::canvas::fmt_zoom(p))).clicked()
                        && let Some(view) = app.ui.views.get_mut(i)
                    {
                        crate::zoom_levels::set_from_field(view, false, p, animated);
                    }
                }
            });
            ui.add_space(6.0);
            for (icon, id, tip) in [("minus", "view.zoomOut", tl!("Zoom Out")), ("plus", "view.zoomIn", tl!("Zoom In"))] {
                if icons::button(ui, icon, 24.0, false, tip).clicked() {
                    run(app, &ctx, id, json!({}));
                }
            }
            if icons::button(ui, "maximize-2", 24.0, false, tl!("Fit on Screen")).clicked()
                && let Some(view) = app.ui.views.get_mut(i)
            {
                view.fit_pending = true;
            }
            widgets::vline(ui, 16.0);
            ui.label(RichText::new(text).color(t.text_dim));
        } else {
            ui.label(RichText::new(tl!("No document")).color(t.text_faint));
        }
        if !app.ui.status.is_empty() {
            widgets::vline(ui, 16.0);
            let is_err = app.ui.status_error || app.ui.status.starts_with("Couldn");
            ui.label(RichText::new(&app.ui.status).color(if is_err { t.warning } else { t.text_faint }));
        }
        // Background jobs keep their progress and Cancel at the far end (#210).
        ui.with_layout(crate::direction::trailing(), |ui| {
            if app.session.has_jobs() {
                crate::jobs_ui::status_progress(app, ui);
            }
            crate::background_redraw::status(app, ui);
            crate::chrome_ui::render_path_status(app, ui);
        });
    });
}

// ------------------------------------------------------------------------ layer properties bar

/// The layers a properties change applies to: the whole selection when the active layer is in
/// it (without the Background, as the classic panel does, #5856), else the active layer alone.
fn targets(doc: &photocraft_doc::Document, l: &Layer, selection: &[LayerId]) -> (Vec<LayerId>, Value) {
    let ids: Vec<LayerId> = if selection.len() > 1 && selection.contains(&l.id) {
        selection.iter().copied().filter(|id| *id == l.id || !doc.layer(*id).is_some_and(|x| crate::doc_props_ui::is_background(doc, x))).collect()
    } else {
        vec![l.id]
    };
    let v = if ids.len() > 1 { json!({"layers": ids.iter().map(|id| id.0).collect::<Vec<_>>()}) } else { json!({"layer": l.id.0}) };
    (ids, v)
}

/// The `layer.setProps` edit of Opacity or Fill (`key`); a slider drag coalesces into one step.
fn pct_action(target: &Value, key: &str, pct: f32, drag: Option<u64>) -> (String, Value) {
    let mut p = target.clone();
    p[key] = json!(pct / 100.0);
    if let Some(n) = drag {
        p["coalesce"] = json!(format!("layer-{key}-slider:{n}"));
    }
    ("layer.setProps".into(), p)
}

fn caption(ui: &mut egui::Ui, text: &str) {
    let t = Tokens::get(ui.ctx());
    ui.label(RichText::new(text).color(t.text_dim));
}

/// Blend mode, Opacity, Fill and the locks of the active layer (greyed at their defaults with
/// none), the classic Layers panel's controls in one row.
fn props_bar(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let mut actions: Vec<(String, Value)> = Vec::new();
    let st = app.session.active();
    let doc = st.map(|s| s.doc.clone());
    let selection = st.map(|s| s.selected_layers()).unwrap_or_default();
    let active = doc.as_ref().zip(st.and_then(|s| s.active_layer)).and_then(|(d, id)| d.layer(id).cloned());
    let lang = crate::i18n::current();
    egui::ScrollArea::horizontal().id_salt("studio-props-bar").auto_shrink([false, false]).show(ui, |ui| {
        ui.set_min_height(PROPS_H);
        crate::direction::horizontal_centered(ui, |ui| {
            let (ids, target) = match (&doc, &active) {
                (Some(d), Some(l)) => targets(d, l, &selection),
                _ => (Vec::new(), Value::Null),
            };
            let bg = doc.as_ref().zip(active.as_ref()).is_some_and(|(d, l)| crate::doc_props_ui::is_background(d, l));
            let all_groups = !ids.is_empty() && ids.iter().all(|id| doc.as_ref().and_then(|d| d.layer(*id)).is_some_and(|x| x.is_group()));
            ui.add_enabled_ui(active.is_some() && !bg, |ui| {
                caption(ui, tl!("Blend Mode:"));
                let current = active.as_ref().map_or(BlendMode::Normal, |l| l.blend);
                let mut m = current;
                let options = crate::panels::blend_options(all_groups, current);
                let (chosen, hovered) =
                    widgets::dropdown_wheel_grouped(ui, "studio-blend", tl!("Blend Mode"), &mut m, &options, 130.0, |m| crate::panels::starts_blend_group(*m));
                for m in &chosen {
                    let mut p = target.clone();
                    p["blend"] = json!(m.label());
                    actions.push(("layer.setProps".into(), p));
                }
                // Hovering a mode previews it on the canvas (#970).
                if !ids.is_empty() {
                    crate::blend_preview::hover(app, &ids, hovered.filter(|_| chosen.is_empty()));
                }
                ui.add_space(14.0);
                for (key, label, value) in [
                    ("opacity", tl!("Opacity:"), active.as_ref().map_or(1.0, |l| l.opacity)),
                    ("fill", crate::i18n::tr_ctx(lang, "layers", "Fill:"), active.as_ref().map_or(1.0, |l| l.fill_opacity)),
                ] {
                    caption(ui, label);
                    let mut v = value * 100.0;
                    let field = widgets::popup_value_field(ui, label, &mut v, 0.0..=100.0, "%", PCT_W);
                    if field.changed {
                        actions.push(pct_action(&target, key, v, field.drag));
                    }
                    ui.add_space(14.0);
                }
            });
            ui.add_enabled_ui(active.is_some(), |ui| {
                caption(ui, tl!("Lock:"));
                ui.spacing_mut().item_spacing.x = 2.0;
                let lk = active.as_ref().map(|l| l.locks).unwrap_or_default();
                for (key, icon, tip, on) in [
                    ("transparency", "grid-3x3", tl!("Lock transparent pixels"), lk.transparency),
                    ("pixels", "brush", tl!("Lock image pixels"), lk.pixels),
                    ("position", "move", tl!("Lock position"), lk.position),
                    ("artboard", "scan", tl!("Prevent auto-nesting in and out of Artboards and Frames"), lk.artboard),
                    ("all", "lock", tl!("Lock all"), lk.all),
                ] {
                    // Lock Image Pixels includes Lock Transparent Pixels, which waits for it (#4892).
                    let enabled = key != "transparency" || !lk.pixels;
                    if ui.add_enabled_ui(enabled, |ui| icons::button(ui, icon, 26.0, on, tip)).inner.clicked() {
                        let mut p = target.clone();
                        p["locks"] = json!({ key: !on });
                        actions.push(("layer.setProps".into(), p));
                    }
                }
            });
        });
    });
    let ctx = ui.ctx().clone();
    run_actions(app, &ctx, actions);
}

// ----------------------------------------------------------------------------- dispatch

/// Run a command like its menu item, reporting a failure in the status line.
fn run(app: &mut PhotocraftApp, ctx: &egui::Context, id: &str, params: Value) {
    if let Err(e) = crate::menus::invoke(app, ctx, id, params) {
        app.ui.status = e;
        app.ui.status_error = true;
    }
}

/// Run what the bar and the cards queued, the way the classic Layers panel does: the `ui.*`
/// pseudo-commands retarget pixels / masks, null params open the command's dialog.
fn run_actions(app: &mut PhotocraftApp, ctx: &egui::Context, actions: Vec<(String, Value)>) {
    for (id, p) in actions {
        // A click commits a type session before the selection changes (#5584).
        if id == "layer.select" && app.ui.text_edit.is_some() {
            crate::type_tool::commit(app);
        }
        match id.as_str() {
            "ui.maskTarget" => {
                app.end_filter_mask_view();
                app.ui.mask_target = p.as_bool().unwrap_or(false);
                app.ui.vector_mask_target = false;
                app.ui.filter_mask_target = None;
                if app.ui.mask_target {
                    app.target_mask_over_colour_rows();
                }
                app.sync_mask_targets();
            }
            "ui.vectorMaskTarget" => {
                app.end_filter_mask_view();
                app.ui.vector_mask_target = p.as_bool().unwrap_or(false);
                app.ui.mask_target = false;
                app.ui.filter_mask_target = None;
                app.sync_mask_targets();
            }
            "ui.filterMaskTarget" => app.select_filter_mask_target(p.as_u64()),
            // A mask's settings live in Properties.
            "ui.properties" => {
                app.ui.panels.properties = true;
                app.ui.dock_tabs.properties = 0;
                app.ui.dock.collapsed.retain(|g| *g != crate::dock::Group::Properties);
            }
            _ if p.is_null() || id == "layer.newFillLayer.solidColor" || id == photocraft_engine::pattern_fill_cmds::CREATE => {
                run(app, ctx, &id, if p.is_null() { json!({}) } else { p });
            }
            _ => {
                if let Err(e) = app.run(&id, p) {
                    app.ui.status = e;
                    app.ui.status_error = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_text_lists_mode_depth_size_layers_and_name() {
        assert_eq!(status_text("RGB", 8, (2400, 1500), 4, "Untitled"), "RGB  ·  8 bits/channel  ·  2400 × 1500 px  ·  4 layers  ·  Untitled");
        assert!(status_text("Gray", 16, (1, 1), 1, "a").contains("1 layer  ·"));
    }

    #[test]
    fn several_selected_layers_are_targeted_without_the_background() {
        let mut session = photocraft_engine::Session::new();
        session.execute("file.new", json!({"width": 8, "height": 8})).unwrap();
        session.execute("layer.new.layer", json!({})).unwrap();
        let st = session.active().unwrap();
        let doc = &st.doc;
        let (bg, top) = (doc.layers[0].id, doc.layers[1].id);
        let l = doc.layer(top).unwrap();
        assert_eq!(targets(doc, l, &[top]).1, json!({"layer": top.0}));
        let (ids, v) = targets(doc, l, &[bg, top]);
        assert_eq!(ids, vec![top]);
        assert_eq!(v, json!({"layer": top.0}));
        assert_eq!(pct_action(&v, "opacity", 50.0, Some(3)).1, json!({"layer": top.0, "opacity": 0.5, "coalesce": "layer-opacity-slider:3"}));
    }
}
