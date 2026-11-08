//! The layer strip: tabs (Camadas / Canais / Caminhos), the layers as horizontal cards, and the
//! new layer / new group / delete / ⋯ column. Cards run the classic Layers rows' commands and
//! reuse their thumbnails, mask thumbnails, menus and in-place rename.

use egui::{Align, Align2, FontId, Layout, Pos2, Rect, Sense, Stroke, StrokeKind, pos2, vec2};
use photocraft_doc::{Document, Layer, LayerContent, LayerId};
use serde_json::{Value, json};

use super::{Tab, run_actions};
use crate::mask_thumbs_ui::MaskKind;
use crate::studio::style::RADIUS;
use crate::theme::Tokens;
use crate::{PhotocraftApp, icons, widgets};

const TABS_W: f32 = 132.0;
const SIDE_W: f32 = 40.0;
const THUMB_W: f32 = 150.0;
/// How much lower each nesting level sits, so a group's contents read as inside it.
const NEST_STEP: f32 = 6.0;

fn drag_key() -> egui::Id {
    egui::Id::unique("studio-layer-card-drag")
}

pub(super) fn show(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let h = ui.available_height() - 16.0;
    ui.add_space(8.0);
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        ui.allocate_ui_with_layout(vec2(TABS_W, h), Layout::top_down(Align::Min), |ui| tabs(app, ui, h));
        let tab = app.studio.bottom.tab;
        let side = if tab == Tab::Layers { SIDE_W + 10.0 } else { 0.0 };
        let w = (ui.available_width() - side).max(0.0);
        ui.allocate_ui_with_layout(vec2(w, h), Layout::left_to_right(Align::Min), |ui| {
            ui.set_min_size(vec2(w, h));
            match tab {
                Tab::Layers => cards(app, ui),
                Tab::Channels => list(ui, "studio-channels", |ui| crate::channels_panel::show(app, ui)),
                Tab::Paths => list(ui, "studio-paths", |ui| crate::vector_ui::paths_panel(app, ui)),
            }
        });
        if tab == Tab::Layers {
            ui.allocate_ui_with_layout(vec2(SIDE_W, h), Layout::top_down(Align::Center), side_column(app));
        }
    });
}

/// The Channels or Paths panel's own content, scrolling inside the strip.
fn list(ui: &mut egui::Ui, id: &str, add: impl FnOnce(&mut egui::Ui)) {
    let t = Tokens::get(ui.ctx());
    egui::Frame::NONE.fill(t.card).corner_radius(RADIUS).inner_margin(6).show(ui, |ui| {
        egui::ScrollArea::vertical().id_salt(id).auto_shrink([false, false]).show(ui, add);
    });
}

fn tabs(app: &mut PhotocraftApp, ui: &mut egui::Ui, h: f32) {
    let t = Tokens::get(ui.ctx());
    let lang = crate::i18n::current();
    egui::Frame::NONE.fill(t.card).corner_radius(RADIUS).inner_margin(4).show(ui, |ui| {
        ui.set_min_height(h - 8.0);
        ui.spacing_mut().item_spacing.y = 2.0;
        for (tab, icon, label) in [
            (Tab::Layers, "layers", tl!("Layers")),
            (Tab::Channels, "blend", tl!("Channels")),
            (Tab::Paths, "spline", crate::i18n::tr_ctx(lang, "studio", "Paths")),
        ] {
            let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 36.0), Sense::click());
            let on = app.studio.bottom.tab == tab;
            if on {
                ui.painter().rect_filled(r, RADIUS - 2.0, t.accent_soft);
            } else if resp.hovered() {
                ui.painter().rect_filled(r, RADIUS - 2.0, t.hover);
            }
            let color = if on { t.accent } else { t.text };
            icons::paint(ui, Rect::from_center_size(pos2(r.left() + 18.0, r.center().y), vec2(20.0, 20.0)), icon, 18.0, color);
            ui.painter().text(pos2(r.left() + 36.0, r.center().y), Align2::LEFT_CENTER, label, FontId::proportional(14.0), color);
            resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, on, label));
            if resp.clicked() {
                app.studio.bottom.tab = tab;
            }
        }
    });
}

/// The cards in strip order (top of the stack first, a group just before its contents, closed
/// groups' contents left out), without layers hidden by isolation or the kind filter.
fn card_rows<'a>(app: &PhotocraftApp, doc: &'a Document, selection: &[LayerId], isolated: &[LayerId]) -> Vec<(usize, &'a Layer)> {
    let filter = crate::layer_filter_ui::Filter::with_selection(app, doc, selection);
    let mut rows = crate::layer_tree_ui::panel_rows(app, doc);
    rows.retain(|(_, l)| photocraft_engine::select_extra_cmds::isolation_shows(doc, isolated, l.id) && filter.shows(l));
    rows
}

/// The part of `area` a `w`×`h` image fills (keeping its aspect ratio) and the matching UVs of
/// the letterboxed square thumbnail texture.
fn fit(area: Rect, w: u32, h: u32) -> (Rect, Rect) {
    let (_, uv) = crate::channels_panel::fit_thumb(area, w, h);
    let (w, h) = (w.max(1) as f32, h.max(1) as f32);
    let s = (area.width() / w).min(area.height() / h).max(0.0);
    (Rect::from_center_size(area.center(), vec2(w * s, h * s).max(vec2(1.0, 1.0))), uv)
}

/// Where a card dragged over `target` lands: above it in the stack (left half), below it (right
/// half), or into a group (its middle).
fn drop_position(frac: f32, group: bool) -> &'static str {
    if group && (0.3..0.7).contains(&frac) {
        "into"
    } else if frac < 0.5 {
        "above"
    } else {
        "below"
    }
}

/// A drag from inside a multi-selection moves the whole selection.
fn drop_payload(dragged: u64, target: LayerId, position: &str, selected: &[LayerId]) -> Value {
    if selected.len() > 1 && selected.contains(&LayerId(dragged)) {
        json!({"layers": selected.iter().map(|id| id.0).collect::<Vec<_>>(), "target": target.0, "position": position})
    } else {
        json!({"layer": dragged, "target": target.0, "position": position})
    }
}

/// The Background stays at the bottom (Photoshop refuses the drop; a copy may still go above).
fn background_refused(doc: &Document, payload: &Value, target: LayerId, position: &str, copy: bool) -> bool {
    let Some(bg) = doc.layers.first().filter(|b| crate::doc_props_ui::is_background(doc, b)).map(|b| b.id) else { return false };
    let moves_bg =
        !copy && (payload["layer"].as_u64() == Some(bg.0) || payload["layers"].as_array().is_some_and(|a| a.iter().any(|v| v.as_u64() == Some(bg.0))));
    moves_bg || (target == bg && position == "below")
}

/// The lock badge's click: the Background becomes a normal layer, any other layer is unlocked
/// (or locked all when it had no lock), as the classic row's lock does.
fn lock_toggle(doc: &Document, l: &Layer) -> (String, Value) {
    if crate::doc_props_ui::is_background(doc, l) {
        return ("layer.new.layerFromBackground".into(), json!({}));
    }
    let locks =
        if l.locks.any() { json!({"transparency": false, "pixels": false, "position": false, "artboard": false, "all": false}) } else { json!({"all": true}) };
    ("layer.setProps".into(), json!({"layer": l.id.0, "locks": locks}))
}

/// `layer.select` mode: ⌘/Ctrl toggles, ⇧ selects a range, a plain click selects one.
fn select_mode(m: egui::Modifiers) -> &'static str {
    if m.command {
        "toggle"
    } else if m.shift {
        "range"
    } else {
        "replace"
    }
}

fn kind_icon(l: &Layer) -> Option<&'static str> {
    match &l.content {
        _ if l.frame.is_some() => Some("frame-placeholder"),
        LayerContent::Group(g) => Some(if g.expanded { "folder-open" } else { "folder" }),
        LayerContent::Text(_) => Some("type"),
        LayerContent::Adjustment(_) => Some("contrast"),
        LayerContent::Fill(_) => Some("paint-bucket"),
        LayerContent::Shape(_) => Some("pentagon"),
        LayerContent::Smart(_) => Some("image"),
        LayerContent::Raster(_) => None,
    }
}

fn cards(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let ctx = ui.ctx().clone();
    let reveal = crate::layer_reveal::track(app, &ctx);
    let Some(st) = app.session.active() else {
        ui.label(egui::RichText::new(tl!("No document")).color(t.text_faint));
        return;
    };
    let doc = st.doc.clone();
    let (active, selection, isolated) = (st.active_layer, st.selected_layers(), st.isolated_layers.clone());
    let mut actions: Vec<(String, Value)> = Vec::new();
    app.reorder_preview_hit = false;
    if !ctx.input(|i| i.pointer.primary_down()) {
        app.reorder_preview = None;
    }
    // A rename nobody drew last frame (its card is gone) ends, committed.
    if crate::layer_row_ui::renaming(&ctx).is_some()
        && !crate::layer_row_ui::rename_active(&ctx)
        && let Some(done) = crate::layer_row_ui::end_rename(&ctx, true)
    {
        actions.push(done);
    }
    let rows = card_rows(app, &doc, &selection, &isolated);
    egui::ScrollArea::horizontal().id_salt(("studio-layer-cards", doc.id.0)).auto_shrink([false, false]).show(ui, |ui| {
        let dragging = ctx.data(|d| d.get_temp::<u64>(drag_key())).is_some() && ctx.input(|i| i.pointer.primary_down());
        // The vertical edge scroll of the classic list, turned on its side.
        let flip = |p: Pos2| pos2(p.y, p.x);
        let clip = ui.clip_rect();
        let viewport = Rect::from_min_max(flip(clip.min), flip(clip.max));
        let delta = crate::panels::layer_drag_edge_scroll(ctx.input(|i| i.pointer.interact_pos()).map(flip), viewport, dragging, ctx.input(|i| i.stable_dt));
        if delta != 0.0 {
            ui.scroll_with_delta(vec2(delta, 0.0));
            ctx.request_repaint();
        }
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            for &(depth, l) in &rows {
                let sel = Sel { selected: selection.contains(&l.id), primary: active == Some(l.id), multi: selection.len() > 1 };
                let rect = card(app, ui, &doc, l, depth, sel, &mut actions);
                if reveal == Some(l.id) {
                    ui.scroll_to_rect(rect, Some(Align::Center));
                }
            }
            // Empty space after the cards: a click deselects every layer, a right-click opens
            // the thumbnail size and clipping choices, as in the classic panel.
            let rest = ui.available_rect_before_wrap();
            if rest.width() > 0.0 {
                let resp = ui.interact(rest, ui.scope_id().with("studio-cards-empty"), Sense::click());
                if resp.clicked() && !selection.is_empty() {
                    actions.push(("select.deselectLayers".into(), json!({})));
                }
                resp.context_menu(|ui| {
                    crate::layer_row_ui::panel_menu(app, ui);
                });
            }
        });
    });
    if !app.reorder_preview_hit {
        app.reorder_preview = None;
    }
    if ctx.input(|i| i.pointer.any_released()) {
        // The side column reads the drag first (drops on its buttons), in this same frame.
        ctx.data_mut(|d| d.insert_temp(drag_key().with("released"), ctx.data(|d| d.get_temp::<u64>(drag_key()))));
        ctx.data_mut(|d| d.remove::<u64>(drag_key()));
    } else {
        ctx.data_mut(|d| d.remove::<Option<u64>>(drag_key().with("released")));
    }
    run_actions(app, &ctx, actions);
}

#[derive(Clone, Copy)]
struct Sel {
    selected: bool,
    primary: bool,
    multi: bool,
}

/// One layer card; returns its rect.
fn card(app: &mut PhotocraftApp, ui: &mut egui::Ui, doc: &Document, l: &Layer, depth: usize, sel: Sel, actions: &mut Vec<(String, Value)>) -> Rect {
    let ctx = ui.ctx().clone();
    let t = Tokens::get(&ctx);
    let full_h = ui.available_height();
    let inset = (depth.min(4) as f32) * NEST_STEP;
    let h = (full_h - inset).max(60.0);
    let ts = (h - 50.0).clamp(24.0, 80.0);
    let masks_w = crate::mask_thumbs_ui::width(l, ts.min(56.0));
    let w = 12.0 + THUMB_W + masks_w + 30.0 + 8.0;
    let (slot, _) = ui.allocate_exact_size(vec2(w, full_h), Sense::hover());
    let rect = Rect::from_min_size(slot.min + vec2(0.0, inset), vec2(w, h));
    let resp = ui.interact(rect, ui.scope_id().with(("studio-card", l.id.0)), Sense::click_and_drag());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, sel.selected, &l.name));
    if !ui.is_rect_visible(rect) && crate::layer_row_ui::renaming(&ctx) != Some(l.id.0) && !resp.context_menu_opened() {
        return rect;
    }
    let painter = ui.painter_at(rect.expand(6.0));
    if depth > 0 {
        // A band behind a group's contents; neighbouring bands meet across the gap between cards.
        painter.rect_filled(slot.expand2(vec2(5.0, 0.0)).with_min_y(rect.top() - 4.0), 0.0, t.accent_soft.gamma_multiply(0.35));
    }
    let hovered = resp.hovered() || app.canvas_hover_layer == Some(l.id);
    if resp.hovered() {
        app.hover_layer = Some(l.id);
    }
    painter.rect_filled(rect, RADIUS, if hovered && !sel.selected { t.hover } else { t.card });
    let border = match (sel.primary, sel.selected) {
        (true, _) => Stroke::new(2.0, t.accent),
        (false, true) => Stroke::new(1.5, t.accent.gamma_multiply(0.6)),
        _ => Stroke::new(1.0, t.card_border),
    };
    painter.rect_stroke(rect, RADIUS, border, StrokeKind::Inside);

    // Header: eye, (group disclosure), kind icon, name, fx.
    let header = Rect::from_min_size(rect.min + vec2(8.0, 6.0), vec2(w - 16.0, 24.0));
    let cell = |x: f32| Rect::from_min_size(pos2(x, header.top()), vec2(22.0, header.height()));
    let mut taken = false;
    let eye_rect = cell(header.left());
    let eye = ui.interact(eye_rect, ui.scope_id().with(("studio-eye", l.id.0)), Sense::click()).on_hover_text(tl!("Visibility"));
    let label_tint = t.layer_label_icon(l.label);
    icons::paint(ui, eye_rect, if l.visible { "eye" } else { "eye-off" }, 16.0, if l.visible { label_tint } else { t.text_faint });
    if eye.clicked() {
        taken = true;
        if ui.input(|i| i.modifiers.alt) {
            actions.push(("layer.showOnly".into(), json!({"layer": l.id.0})));
        } else {
            actions.push(("layer.setProps".into(), json!({"layer": l.id.0, "visible": !l.visible})));
        }
    }
    eye.context_menu(|ui| {
        widgets::style_spectrum_popup_menu(ui);
        crate::layer_menu_ui::color_choices(app, ui, l, sel.multi && sel.selected, actions);
    });
    let mut x = eye_rect.right() + 2.0;
    if let LayerContent::Group(g) = &l.content
        && l.frame.is_none()
    {
        let r = cell(x);
        let tip = if g.expanded { tl!("Collapse") } else { tl!("Expand") };
        let d = ui.interact(r, ui.scope_id().with(("studio-disclosure", l.id.0)), Sense::click()).on_hover_text(tip);
        icons::paint(ui, r, if g.expanded { "chevron-down" } else { "chevron-right" }, 14.0, if d.hovered() { t.text } else { t.icon });
        if d.clicked() {
            taken = true;
            let all = ui.input(|i| i.modifiers.alt);
            actions.push(("layer.setExpanded".into(), json!({"layer": l.id.0, "expanded": !g.expanded, "all": all})));
        }
        x += 20.0;
    }
    if l.clipped {
        icons::paint(ui, cell(x), "clip-below", 14.0, t.text_faint);
        x += 18.0;
    }
    if let Some(icon) = kind_icon(l) {
        icons::paint(ui, cell(x), icon, 16.0, t.icon);
        x += 24.0;
    }
    let has_fx = !l.effects.items.is_empty();
    let fx_rect = Rect::from_min_max(pos2(header.right() - 22.0, header.top()), header.max);
    let name_right = if has_fx { fx_rect.left() - 4.0 } else { header.right() };
    let name_color = if l.visible { t.text } else { t.text_faint };
    let name_rect = crate::layer_row_ui::truncated(&painter, &l.name, FontId::proportional(14.0), name_color, false, name_right - x).map(|g| {
        let p = pos2(x, header.center().y - g.size().y / 2.0);
        let r = Rect::from_min_size(p, g.size());
        painter.galley(p, g, name_color);
        r
    });
    if has_fx {
        let fx = ui.interact(fx_rect, ui.scope_id().with(("studio-fx", l.id.0)), Sense::click()).on_hover_text(tl!("Layer Style"));
        let mut job = egui::text::LayoutJob::default();
        let color = if l.effects.enabled { t.accent } else { t.text_faint };
        job.append("fx", 0.0, egui::TextFormat { font_id: FontId::proportional(16.0), color, italics: true, ..Default::default() });
        let g = painter.layout_job(job);
        painter.galley(fx_rect.center() - g.size() / 2.0, g, color);
        if fx.clicked() {
            taken = true;
            crate::layer_style::open_for_layer(app, l.id, None);
        }
    }

    // Body: thumbnail, masks (with their chains), then link and lock badges.
    let body = Rect::from_min_max(pos2(rect.left() + 12.0, header.bottom() + 6.0), pos2(rect.left() + 12.0 + THUMB_W, rect.bottom() - 10.0));
    let thumb = paint_thumb(app, &ctx, ui, &painter, doc, l, body, sel.primary);
    let mut mx = body.right() + 10.0;
    let masks = crate::mask_thumbs_ui::paint(app, &ctx, ui, &painter, doc, l, &mut mx, body.center().y, ts.min(56.0), actions);
    if sel.primary {
        // The targeted thumbnail (pixels, mask or vector mask) carries the corner brackets.
        let target = if app.ui.vector_mask_target {
            masks.thumb(MaskKind::Vector)
        } else if app.ui.mask_target {
            masks.thumb(MaskKind::Pixel)
        } else {
            None
        };
        if let Some(r) = target {
            crate::mask_thumbs_ui::paint_brackets(&painter, r, t.text);
        }
    }
    let badge_x = rect.right() - 24.0;
    let badge = |y: f32| Rect::from_center_size(pos2(badge_x, y), vec2(22.0, 22.0));
    if l.link_group.is_some() {
        icons::paint(ui, badge(body.center().y - 14.0), "link-2", 16.0, t.icon);
    }
    if l.locks.any() || doc.is_background(l.id) {
        let r = badge(body.center().y + if l.link_group.is_some() { 14.0 } else { 0.0 });
        let lock = ui.interact(r, ui.scope_id().with(("studio-lock", l.id.0)), Sense::click()).on_hover_text(tl!("Lock layer"));
        icons::paint(ui, r, "lock", 16.0, if lock.hovered() { t.text } else { t.icon });
        if lock.clicked() {
            taken = true;
            actions.push(lock_toggle(doc, l));
        }
    }

    let m = ui.input(|i| i.modifiers);
    let pos = resp.interact_pointer_pos();
    let on_thumb = pos.is_some_and(|p| thumb.expand(2.0).contains(p));
    let mask_hit = pos.and_then(|p| masks.hit(p));
    let bg = crate::doc_props_ui::is_background(doc, l);
    if resp.clicked() && !taken && !eye.clicked() && !masks.clicked {
        // ⌘/Ctrl-click a thumbnail loads it as a selection (⇧ add, ⌥ subtract).
        let load = if !m.command {
            None
        } else if let Some(kind) = mask_hit {
            Some(crate::mask_thumbs_ui::load_params(l, kind, m))
        } else {
            (on_thumb && !bg).then(|| json!({"channel": "transparency", "layer": l.id.0, "operation": crate::channels_panel::load_operation(m)}))
        };
        if let Some(p) = load {
            actions.push(("select.loadSelection".into(), p));
        } else if let Some(cmd) = mask_hit.and_then(|k| crate::mask_thumbs_ui::click_command(l, k, m)) {
            actions.push(cmd);
        } else {
            let mode = if m.command && on_thumb && bg { "replace" } else { select_mode(m) };
            actions.push(("layer.select".into(), json!({"layer": l.id.0, "mode": mode})));
            crate::smart_ui::deselect(&ctx, doc.id);
            let content_less = matches!(l.content, LayerContent::Adjustment(_) | LayerContent::Fill(_));
            if mask_hit == Some(MaskKind::Vector) {
                app.ui.selected_path = Some("layer".into());
                actions.push(("ui.vectorMaskTarget".into(), json!(true)));
            } else if mask_hit == Some(MaskKind::Pixel) || (content_less && l.mask.is_some() && !on_thumb) {
                if mask_hit.is_some() {
                    app.ui.properties_layer = None;
                }
                actions.push(("ui.maskTarget".into(), json!(true)));
            } else if on_thumb || !sel.primary {
                actions.push(("ui.maskTarget".into(), json!(false)));
            }
        }
    }
    // Double-click: the name renames in place, the Background becomes a normal layer, a mask
    // opens its settings, a content thumbnail its options; elsewhere Layer Style.
    if resp.double_clicked() && !taken {
        let on_name = name_rect.is_some_and(|n| pos.is_some_and(|p| Rect::from_x_y_ranges(n.x_range(), header.y_range()).expand(2.0).contains(p)));
        if bg {
            actions.push(("layer.new.layerFromBackground".into(), json!({})));
        } else if on_name {
            actions.extend(crate::layer_row_ui::start_rename(&ctx, l.id.0, &l.name));
        } else if let Some(kind) = mask_hit {
            actions.push((if kind == MaskKind::Pixel { "ui.maskTarget" } else { "ui.vectorMaskTarget" }.into(), json!(true)));
            match crate::mask_thumbs_ui::double_click_command(app, kind) {
                Some((cmd, v)) => actions.push((cmd.into(), v)),
                None => actions.push(("ui.properties".into(), json!(true))),
            }
        } else {
            let id = match &l.content {
                LayerContent::Adjustment(_) | LayerContent::Fill(_) | LayerContent::Shape(_) if on_thumb => "layer.layerContentOptions",
                LayerContent::Smart(_) if on_thumb => "layer.smartObjects.editContents",
                LayerContent::Text(_) if on_thumb => "type.editText",
                _ => "layer.layerStyle.blendingOptions",
            };
            if crate::menus::is_enabled(app, id) {
                actions.push((id.into(), Value::Null));
            }
        }
    }
    let edit = Rect::from_min_max(pos2(x - 3.0, header.top()), pos2(name_right.max(x + 60.0), header.bottom()));
    if let Some(done) = crate::layer_row_ui::rename_field(ui, l.id.0, edit) {
        actions.push(done);
    }
    // Right-click: the mask thumbnail's menu over a layer mask, the layer menu elsewhere.
    let on_mask_id = ui.scope_id().with(("studio-mask-menu", l.id.0));
    if resp.secondary_clicked() {
        let on_mask = resp.interact_pointer_pos().and_then(|p| masks.hit(p)) == Some(MaskKind::Pixel);
        ctx.data_mut(|d| d.insert_temp(on_mask_id, on_mask));
    }
    resp.context_menu(|ui| {
        widgets::style_spectrum_popup_menu(ui);
        if ui.ctx().data(|d| d.get_temp::<bool>(on_mask_id)).unwrap_or(false) {
            crate::layer_menu_ui::show_mask(app, ui, l, actions);
        } else if crate::layer_menu_ui::show(app, ui, l, sel.multi && sel.selected, actions) {
            actions.extend(crate::layer_row_ui::start_rename(ui.ctx(), l.id.0, &l.name));
        }
    });
    drag_and_drop(app, &ctx, ui, doc, l, rect, &resp, taken, actions);
    rect
}

/// The layer thumbnail inside `area`; returns the rect it covers.
#[allow(clippy::too_many_arguments)]
fn paint_thumb(app: &mut PhotocraftApp, ctx: &egui::Context, ui: &egui::Ui, painter: &egui::Painter, doc: &Document, l: &Layer, area: Rect, primary: bool) -> Rect {
    let t = Tokens::get(ctx);
    let (w, h) = (doc.size.width, doc.size.height);
    let (tile, _) = fit(area, w, h);
    let icon_tile = |icon: &str| {
        painter.rect_filled(tile, 6.0, t.field);
        icons::paint(ui, tile, icon, 22.0, t.icon);
    };
    let r = match &l.content {
        _ if l.frame.is_some() => {
            icon_tile("frame-placeholder");
            tile
        }
        LayerContent::Group(_) => {
            icon_tile("folder");
            tile
        }
        LayerContent::Adjustment(_) => {
            icon_tile("contrast");
            tile
        }
        LayerContent::Fill(f) if crate::gradient_ui::paint_thumbnail(ui, l.id, f, tile) => tile,
        LayerContent::Fill(photocraft_doc::Fill::Solid(color)) => {
            crate::solid_fill_ui::paint_thumbnail(ui, crate::solid_fill_ui::thumbnail_color(app, doc.id, l.id, *color), tile);
            tile
        }
        _ if l.surface().is_some() => {
            let (tex, frame) = app.layer_thumb(ctx, doc, l);
            let (fitted, uv) = fit(area, frame.width(), frame.height());
            widgets::checker(painter, fitted, 6.0);
            painter.image(tex, fitted, uv, egui::Color32::WHITE);
            fitted
        }
        LayerContent::Text(_) => {
            icon_tile("type");
            tile
        }
        _ => {
            icon_tile("image");
            tile
        }
    };
    let stroke = if primary { Stroke::new(1.5, t.text) } else { Stroke::new(1.0, t.field_border) };
    painter.rect_stroke(r, 4.0, stroke, StrokeKind::Outside);
    crate::smart_ui::thumb_badge(ui, l, r);
    r
}

/// Drag a card to reorder: drop it before or after another card, or into a group. ⌥ held on the
/// release copies instead. Dragging onto a document tab copies the layer there.
#[allow(clippy::too_many_arguments)]
fn drag_and_drop(
    app: &mut PhotocraftApp,
    ctx: &egui::Context,
    ui: &egui::Ui,
    doc: &Document,
    l: &Layer,
    rect: Rect,
    resp: &egui::Response,
    taken: bool,
    actions: &mut Vec<(String, Value)>,
) {
    let t = Tokens::get(ctx);
    if resp.drag_started() && !taken {
        ctx.data_mut(|d| d.insert_temp(drag_key(), l.id.0));
        crate::layer_transfer::begin_from_panel(app, ctx, l.id);
    }
    let Some(dragged) = ctx.data(|d| d.get_temp::<u64>(drag_key())) else { return };
    let Some(p) = ctx.input(|i| i.pointer.interact_pos()) else { return };
    let copy = ctx.input(|i| i.modifiers.alt);
    if dragged == l.id.0 {
        crate::layer_transfer::ghost(ctx, p, &l.name);
        if copy {
            ctx.set_cursor_icon(egui::CursorIcon::Copy);
        }
        return;
    }
    if !rect.contains(p) {
        return;
    }
    let position = drop_position((p.x - rect.left()) / rect.width().max(1.0), l.is_group() && l.frame.is_none());
    let selected = app.session.active().map(|st| st.selected_layers()).unwrap_or_default();
    let mut payload = drop_payload(dragged, l.id, position, &selected);
    if background_refused(doc, &payload, l.id, position, copy) {
        return;
    }
    let stroke = Stroke::new(3.0, t.accent);
    let _ = match position {
        "into" => ui.painter().rect_stroke(rect, RADIUS, stroke, StrokeKind::Inside),
        "above" => ui.painter().line_segment([rect.left_top() - vec2(5.0, 0.0), rect.left_bottom() - vec2(5.0, 0.0)], stroke),
        _ => ui.painter().line_segment([rect.right_top() + vec2(5.0, 0.0), rect.right_bottom() + vec2(5.0, 0.0)], stroke),
    };
    if copy && let Some(o) = payload.as_object_mut() {
        o.insert("copy".into(), json!(true));
    }
    if ctx.input(|i| i.pointer.any_released()) {
        app.reorder_preview = None;
        actions.push(("layer.moveTo".into(), payload));
    } else if ctx.input(|i| i.pointer.primary_down()) {
        crate::layers_panel_ui::preview_reorder(app, &payload);
    }
}

/// The right column: new layer, new group, delete (each also takes a dropped card) and the ⋯
/// menu with the rest of the classic panel (its flyout, the kind filter and footer buttons).
fn side_column(app: &mut PhotocraftApp) -> impl FnOnce(&mut egui::Ui) + '_ {
    move |ui| {
        let ctx = ui.ctx().clone();
        let mut actions: Vec<(String, Value)> = Vec::new();
        let Some(st) = app.session.active() else { return };
        let doc = st.doc.clone();
        let (active, selection) = (st.active_layer, st.selected_layers());
        let active_layer = active.and_then(|id| doc.layer(id));
        // A card released over a button this frame (the cards cleared the drag already).
        let dropped = ctx.data(|d| d.get_temp::<Option<u64>>(drag_key().with("released"))).flatten().or_else(|| ctx.data(|d| d.get_temp::<u64>(drag_key())));
        let drop_on = |ui: &egui::Ui, b: &egui::Response, command: &str| -> Option<(String, Value)> {
            let id = dropped?;
            let whole = selection.len() > 1 && selection.iter().any(|s| s.0 == id);
            crate::panels::dropped_on(ui, b).then(|| (command.to_string(), if whole { json!({}) } else { json!({"layer": id}) }))
        };
        ui.spacing_mut().item_spacing.y = 6.0;
        let new_layer = icons::button(ui, "square-plus", 34.0, false, &crate::shortcuts::tip_label(app, "Create a new layer", "layer.new.layer"));
        if new_layer.clicked() {
            // ⌥-click asks first (the New Layer dialog), ⌘/Ctrl-click adds it below.
            let (alt, below) = ui.input(|i| (i.modifiers.alt, i.modifiers.command));
            actions.push(("layer.new.layer".into(), if alt { Value::Null } else if below { json!({"position": "below"}) } else { json!({}) }));
        }
        actions.extend(drop_on(ui, &new_layer, "layer.duplicate"));
        let group = icons::button(ui, "folder-plus", 34.0, false, tl!("Create a new group"));
        if group.clicked() {
            // Several selected layers (or ⇧) go into the new group; ⌥ asks first, ⌘ adds below.
            let m = ui.input(|i| i.modifiers);
            let groups = (m.shift || selection.len() > 1) && active_layer.is_some();
            if m.alt {
                actions.push(((if groups { crate::new_group_ui::FROM_LAYERS } else { crate::new_group_ui::NEW_GROUP }).into(), Value::Null));
            } else if m.command {
                actions.push(("layer.new.group".into(), json!({"position": "below"})));
            } else {
                actions.push(((if groups { "layer.groupLayers" } else { "layer.new.group" }).into(), json!({})));
            }
        }
        actions.extend(drop_on(ui, &group, "layer.groupLayers"));
        let (delete, label) = if app.ui.vector_mask_target && active_layer.is_some_and(|l| l.vector_mask.is_some()) {
            ("layer.vectorMask.delete", tl!("Delete Vector Mask"))
        } else if app.ui.mask_target && active_layer.is_some_and(|l| l.mask.is_some()) {
            ("layer.layerMask.delete", tl!("Delete Layer Mask"))
        } else {
            ("layer.delete", tl!("Delete layer"))
        };
        let trash = icons::button(ui, "trash", 34.0, false, label);
        if trash.clicked() {
            if let Some(picked) = crate::smart_ui::selected(&ctx, &doc, active) {
                // A selected smart filter is what Delete removes (#4751).
                actions.push(crate::smart_ui::trash_action(picked));
                crate::smart_ui::deselect(&ctx, doc.id);
            } else {
                // ⌥/Alt-click deletes without asking.
                let confirm = delete == "layer.delete" && !ui.input(|i| i.modifiers.alt);
                actions.push((delete.into(), if confirm { json!({"__trash": true}) } else { json!({}) }));
            }
        }
        actions.extend(drop_on(ui, &trash, "layer.delete"));
        let more = icons::button(ui, "ellipsis", 34.0, false, tl!("More options"));
        egui::Popup::menu(&more).close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside).show(|ui| more_menu(app, ui, &mut actions));
        run_actions(app, &ctx, actions);
    }
}

/// What the classic Layers panel has beyond the cards: its flyout menu, the kind filter, and the
/// footer's fill/adjustment, mask, style and link buttons.
fn more_menu(app: &mut PhotocraftApp, ui: &mut egui::Ui, actions: &mut Vec<(String, Value)>) {
    ui.set_min_width(220.0);
    ui.menu_button(tl!("Filter"), |ui| crate::layer_filter_ui::bar(app, ui));
    ui.menu_button(tl!("Create new fill or adjustment layer"), |ui| {
        for c in photocraft_engine::command_specs().iter().filter(|c| c.id.starts_with("layer.newAdjustmentLayer.")) {
            if ui.button(tl!(c.label).trim_end_matches('…')).clicked() {
                actions.push((c.id.into(), json!({})));
            }
        }
        ui.separator();
        for (label, id) in [
            (tl!("Solid Color…"), "layer.newFillLayer.solidColor"),
            (tl!("Gradient…"), "layer.newFillLayer.gradient"),
            (tl!("Pattern…"), "layer.newFillLayer.pattern"),
        ] {
            if ui.button(label).clicked() {
                actions.push((id.into(), json!({})));
            }
        }
    });
    ui.menu_button(tl!("Add a layer style"), |ui| {
        if ui.button(tl!("Blending Options…")).clicked() {
            crate::layer_style::open(app, None);
        }
        ui.separator();
        for &(kind, label) in crate::layer_style::KINDS {
            if ui.button(format!("{}…", tl!(label))).clicked() {
                crate::layer_style::open(app, Some(kind));
            }
        }
    });
    let st = app.session.active();
    let active_layer = st.and_then(|s| s.active_layer.and_then(|id| s.doc.layer(id)));
    let has_selection = st.is_some_and(|s| s.doc.selection.is_some());
    let alt = ui.input(|i| i.modifiers.alt);
    let mask_cmd = crate::layer_menu_ui::mask_button_command(active_layer, has_selection, alt);
    let mask_label = if active_layer.is_some_and(|l| l.mask.is_some()) { tl!("Add vector mask") } else { tl!("Add Layer Mask") };
    if ui.add_enabled(mask_cmd.is_some(), egui::Button::new(mask_label)).clicked()
        && let Some(cmd) = mask_cmd
    {
        // A path selected in the Paths panel becomes the vector mask (#4728).
        match crate::vector_ui::selected_document_path(app).filter(|_| cmd.starts_with("layer.vectorMask.")) {
            Some(name) => actions.push(("layer.vectorMask.currentPath".into(), json!({"name": name, "hide": alt}))),
            None => actions.push((cmd.into(), json!({}))),
        }
    }
    if ui.add_enabled(app.session.is_enabled("layer.linkLayers"), egui::Button::new(tl!("Link layers"))).clicked() {
        actions.push(("layer.linkLayers".into(), json!({})));
    }
    ui.separator();
    crate::layer_row_ui::panel_menu(app, ui);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_keeps_the_aspect_ratio_inside_the_area() {
        let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(150.0, 80.0));
        let (r, uv) = fit(area, 2400, 1500);
        assert!((r.width() / r.height() - 1.6).abs() < 0.01);
        assert!(r.height() <= 80.0 + 0.01 && r.width() <= 150.0 + 0.01);
        assert!((uv.width() - 1.0).abs() < 1e-4 && (uv.height() - 0.625).abs() < 1e-4);
        let (tall, _) = fit(area, 0, 0);
        assert!(tall.width() >= 1.0, "a zero-sized document never divides by zero");
    }

    #[test]
    fn drops_land_before_after_or_into_groups() {
        assert_eq!(drop_position(0.1, false), "above");
        assert_eq!(drop_position(0.5, false), "below");
        assert_eq!(drop_position(0.5, true), "into");
        assert_eq!(drop_position(0.9, true), "below");
        let (a, b) = (LayerId(1), LayerId(2));
        assert_eq!(drop_payload(1, b, "above", &[a]), json!({"layer": 1, "target": 2, "position": "above"}));
        assert_eq!(drop_payload(1, b, "below", &[a, LayerId(3)])["layers"], json!([1, 3]));
    }

    /// A group's card is followed by its children's only while it is expanded.
    #[test]
    fn group_children_show_only_when_expanded() {
        let mut session = photocraft_engine::Session::new();
        session.execute("file.new", json!({"width": 8, "height": 8})).unwrap();
        session.execute("layer.new.layer", json!({"name": "inner"})).unwrap();
        session.execute("layer.groupLayers", json!({})).unwrap();
        let mut app = PhotocraftApp::new(session, crate::Services::default());
        let group = app.session.active().unwrap().doc.layers.iter().find(|l| l.is_group()).unwrap().id;
        for expanded in [true, false] {
            app.run("layer.setExpanded", json!({"layer": group.0, "expanded": expanded})).unwrap();
            let doc = app.session.active().unwrap().doc.clone();
            let rows: Vec<(usize, String)> = card_rows(&app, &doc, &[], &[]).into_iter().map(|(d, l)| (d, l.name.clone())).collect();
            assert_eq!(rows.iter().any(|(d, n)| *d == 1 && n == "inner"), expanded, "{rows:?}");
            assert_eq!(rows.last().map(|(d, _)| *d), Some(0), "the Background comes last");
        }
    }

    #[test]
    fn lock_badge_unlocks_or_converts_the_background() {
        let mut session = photocraft_engine::Session::new();
        session.execute("file.new", json!({"width": 8, "height": 8})).unwrap();
        let doc = session.active().unwrap().doc.clone();
        let bg = &doc.layers[0];
        assert_eq!(lock_toggle(&doc, bg).0, "layer.new.layerFromBackground");
        assert!(background_refused(&doc, &json!({"layer": 99}), bg.id, "below", false));
        assert!(!background_refused(&doc, &json!({"layer": 99}), bg.id, "above", false));
    }
}
