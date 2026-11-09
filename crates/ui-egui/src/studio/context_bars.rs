//! Floating context bars over the canvas: type (font, style, size, colour, alignment), selection
//! (Adicionar, Subtrair, Inverter, Refinar, Desmarcar), adjustments (Níveis, Curvas, …) and the
//! active tool's options (`options_pill`).
//!
//! The bars only draw: they run the same commands and set the same tool state as the classic
//! options bar (`type_tool::options_bar`, `panels::selection_mode_buttons`) and the Adjustments
//! panel (`layer.newAdjustmentLayer.<kind>`).

use egui::{Align2, Pos2, Rect, Sense, Stroke, Ui, pos2, vec2};
use photocraft_doc::{LayerContent, LayerId};
use photocraft_geom::Point;
use serde_json::{Value, json};

use super::{Mode, style};
use crate::PhotocraftApp;
use crate::state::Tool;
use crate::theme::Tokens;
use crate::type_tool as tt;

#[derive(Default)]
pub struct State {}

/// Distance between a bar and the canvas edge or the type layer it floats over.
const GAP: f32 = 12.0;
/// Height of the bars' buttons.
const ROW_H: f32 = 30.0;

/// Which bars show.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Bars {
    type_bar: bool,
    selection: bool,
    adjustments: bool,
}

/// Cor e luz owns the top of the canvas with its adjustment tabs; elsewhere the type bar shows for
/// the Type tools and for a selected or edited type layer; Selecionar adds the selection bar.
fn bars(mode: Mode, tool: Tool, type_target: bool, has_selection: bool) -> Bars {
    Bars {
        type_bar: mode != Mode::ColorLight && (tool.is_type() || type_target),
        selection: mode == Mode::Select && (has_selection || is_selection_tool(tool)),
        adjustments: mode == Mode::ColorLight,
    }
}

fn is_selection_tool(tool: Tool) -> bool {
    tool.is_type_mask()
        || matches!(
            tool,
            Tool::RectMarquee
                | Tool::EllipseMarquee
                | Tool::SingleRowMarquee
                | Tool::SingleColumnMarquee
                | Tool::Lasso
                | Tool::PolygonLasso
                | Tool::MagneticLasso
                | Tool::MagicWand
                | Tool::QuickSelection
                | Tool::ObjectSelection
        )
}

pub fn show(app: &mut PhotocraftApp, ui: &mut Ui) {
    crate::vector_ui::track_arrow_default_selection(app, ui.ctx());
    let Some(has_selection) = app.session.active().map(|st| st.doc.selection.is_some()) else { return };
    let canvas = crate::rulers::content_rect(app, app.last_canvas_rect);
    let b = bars(app.studio.mode, app.ui.tool, tt::target(app).is_some(), has_selection);
    // Bars at the top centre stack downwards in this order.
    let mut top = canvas.top() + GAP;
    if b.adjustments {
        let r = floating(ui, "studio-adjustment-tabs", pos2(canvas.center().x, top), Align2::CENTER_TOP, canvas, |ui| {
            adjustment_tabs(app, ui);
        });
        top = r.bottom() + GAP / 2.0;
    }
    if b.type_bar {
        let anchor = type_anchor(app);
        let (pos, pivot) = match anchor {
            Some(p) => (p, Align2::CENTER_BOTTOM),
            None => (pos2(canvas.center().x, top), Align2::CENTER_TOP),
        };
        let r = floating(ui, "studio-type-bar", pos, pivot, canvas, |ui| type_bar(app, ui));
        if anchor.is_none() {
            top = r.bottom() + GAP / 2.0;
        }
    }
    if super::options_pill::shows_for(app.ui.tool, app.ui.transform.is_some()) {
        // Beside the Tools button, not under it.
        let left = canvas.left() + super::tools_menu::RESERVED_W;
        let max_w = (canvas.right() - GAP - left - 16.0).max(80.0);
        floating(ui, "studio-tool-options", pos2((left + canvas.right() - GAP) / 2.0, top), Align2::CENTER_TOP, canvas, |ui| {
            super::options_pill::contents(app, ui, max_w);
        });
    }
    if b.selection {
        floating(ui, "studio-selection-bar", pos2(canvas.center().x, canvas.bottom() - GAP), Align2::CENTER_BOTTOM, canvas, |ui| {
            selection_bar(app, ui);
        });
    }
}

/// A rounded dark card floating over the canvas, its `pivot` corner at `pos`, kept inside `canvas`;
/// returns its rect.
fn floating(ui: &Ui, id: &str, pos: Pos2, pivot: Align2, canvas: Rect, add: impl FnOnce(&mut Ui)) -> Rect {
    let t = Tokens::get(ui.ctx());
    egui::Area::new(egui::Id::unique(id))
        .pivot(pivot)
        .fixed_pos(pos)
        .constrain_to(canvas)
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(t.card)
                .stroke(Stroke::new(1.0, t.card_border))
                .corner_radius(style::RADIUS)
                .shadow(egui::Shadow { offset: [0, 6], blur: 20, spread: 0, color: t.shadow })
                .inner_margin(egui::Margin::symmetric(8, 5))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        add(ui);
                    });
                });
        })
        .response
        .rect
}

/// An icon (+ label) button; `active` draws the amber outline.
fn pill(ui: &mut Ui, icon: &str, label: Option<&str>, active: bool, tip: &str) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let font = egui::FontId::proportional(13.0);
    let text_w = label.map_or(0.0, |l| ui.painter().layout_no_wrap(l.to_string(), font.clone(), t.text).size().x + 6.0);
    let (r, resp) = ui.allocate_exact_size(vec2(ROW_H + text_w + if label.is_some() { 6.0 } else { 0.0 }, ROW_H), Sense::click());
    let enabled = ui.is_enabled();
    if resp.hovered() && enabled {
        ui.painter().rect_filled(r, t.radius, t.hover);
    }
    if active {
        ui.painter().rect_stroke(r, t.radius, Stroke::new(1.5, t.accent), egui::StrokeKind::Inside);
    }
    let col = if !enabled {
        t.text_faint
    } else if active {
        t.accent
    } else {
        t.text
    };
    let icon_r = Rect::from_center_size(pos2(r.left() + ROW_H / 2.0, r.center().y), vec2(18.0, 18.0));
    crate::icons::paint(ui, icon_r, icon, 16.0, col);
    if let Some(l) = label {
        ui.painter().text(pos2(icon_r.right() + 6.0, r.center().y), Align2::LEFT_CENTER, l, font, col);
    }
    let name = label.unwrap_or(tip).to_string();
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, enabled, active, &name));
    if label == Some(tip) { resp } else { resp.on_hover_text(tip) }
}

fn report(app: &mut PhotocraftApp, r: Result<Value, String>) {
    if let Err(e) = r {
        app.ui.status = e;
        app.ui.status_error = true;
    }
}

fn invoke(app: &mut PhotocraftApp, ctx: &egui::Context, id: &str) {
    let r = crate::menus::invoke(app, ctx, id, json!({}));
    report(app, r);
}

// ---- Type bar ---------------------------------------------------------------------------------

/// Screen point just above the type target's laid-out text, when it is in the active document.
fn type_anchor(app: &mut PhotocraftApp) -> Option<Pos2> {
    // The layout waits for the font scan; until it is done the bar sits at the top centre.
    photocraft_text::try_shared()?;
    let (id, _) = tt::target(app)?;
    let xf = crate::canvas::ViewXform::active(app)?;
    let (l, aff, _) = tt::layout(app, LayerId(id))?;
    let b = l.bounds()?;
    let corners = [(b[0], b[1]), (b[2], b[1]), (b[0], b[3]), (b[2], b[3])].map(|(x, y)| {
        let p = aff.apply(Point::new(f64::from(x), f64::from(y)));
        xf.to_screen(p.x as f32, p.y as f32)
    });
    let r = Rect::from_points(&corners);
    (r.is_finite() && xf.rect.intersects(r)).then(|| pos2(r.center().x, r.top() - GAP))
}

/// The style the type bar shows: the selected characters' (or caret's), else the tool defaults.
struct Shown {
    family: String,
    style: String,
    size: f32,
    rgb: Option<[f32; 3]>,
    align: Option<photocraft_doc::text::TextAlign>,
}

fn shown(app: &PhotocraftApp) -> Shown {
    match tt::styles_at(app) {
        Some((c, p)) => {
            let s = tt::selected_style(&c);
            Shown {
                family: c.font_family.clone(),
                style: if s.is_empty() { "Regular".into() } else { s },
                size: c.size_pt * shown_scale(app),
                rgb: Some(c.color.to_rgb()),
                align: Some(p.align),
            }
        }
        None => {
            let o = &app.ui.tool_options;
            Shown { family: o.type_font.clone(), style: o.type_style.clone(), size: o.type_size, rgb: None, align: None }
        }
    }
}

fn type_bar(app: &mut PhotocraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.spacing_mut().item_spacing.x = 6.0;
    let s = shown(app);
    let (r, _) = ui.allocate_exact_size(vec2(22.0, ROW_H), Sense::hover());
    crate::icons::paint(ui, r, "type", 16.0, t.icon);
    font_controls(app, ui, s.family, s.style);
    size_control(app, ui, s.size);
    color_swatch(app, ui, s.rgb);
    crate::widgets::vline(ui, 22.0);
    ui.spacing_mut().item_spacing.x = 2.0;
    align_buttons(app, ui, s.align);
    crate::widgets::vline(ui, 22.0);
    if pill(ui, "ellipsis", None, false, tl!("Character and Paragraph")).clicked() {
        invoke(app, &ui.ctx().clone(), "window.panel.character");
    }
}

/// Font family and style menus, applied as the classic bar applies them.
fn font_controls(app: &mut PhotocraftApp, ui: &mut Ui, mut fam: String, mut style: String) {
    if !tt::fonts_ready(ui, 150.0 + 6.0 + 104.0) {
        return;
    }
    if let Some(picked) = font_picker(app, ui, &mut fam, 150.0) {
        app.ui.tool_options.type_font = fam.clone();
        let st = tt::styles(&fam);
        let picked = picked.filter(|s| st.contains(s));
        style = match &picked {
            Some(s) => s.clone(),
            None if st.contains(&style) => style,
            None => st.first().cloned().unwrap_or_else(|| "Regular".into()),
        };
        app.ui.tool_options.type_style = style.clone();
        // Only a style picked under the family's ">" is set: each layer keeps its face (#3556).
        match picked {
            Some(s) => apply_style(app, ui.ctx(), json!({"font": fam, "fontStyle": s})),
            None => apply_style(app, ui.ctx(), json!({"font": fam})),
        }
    }
    let opts: Vec<(String, String)> = tt::styles(&fam).into_iter().map(|s| (s.clone(), tt::style_label(&s))).collect();
    let opts_ref: Vec<(String, &str)> = opts.iter().map(|(a, b)| (a.clone(), b.as_str())).collect();
    if crate::widgets::dropdown(ui, "studio-type-style", tl!("Style"), &mut style, &opts_ref, 104.0) {
        app.ui.tool_options.type_style = style.clone();
        apply_style(app, ui.ctx(), json!({"fontStyle": style}));
    }
}

/// Size: typed or scrubbed in the field, or a preset from the menu beside it.
fn size_control(app: &mut PhotocraftApp, ui: &mut Ui, size: f32) {
    let (unit, label) = tt::type_unit(app);
    let mut display = size * unit;
    let mut new_size = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        if crate::widgets::value_field(ui, &mut display, unit..=1296.0 * unit, label, 64.0).on_hover_text(tl!("Font size")).changed() {
            new_size = Some(display / unit);
        }
        let chev = crate::icons::button(ui, "chevron-down", 18.0, false, tl!("Font size"));
        crate::press_menu::show(&chev, |ui, menu| {
            for p in SIZE_PRESETS {
                if menu.item(ui, format!("{p} {label}")) {
                    new_size = Some(p / unit);
                }
            }
        });
    });
    if let Some(size) = new_size {
        app.ui.tool_options.type_size = size;
        apply_style(app, ui.ctx(), json!({"size": size, "metricsAsShown": true}));
    }
}

/// Text colour swatch: the shared Color Picker, as the classic bar opens it.
fn color_swatch(app: &mut PhotocraftApp, ui: &mut Ui, rgb: Option<[f32; 3]>) {
    let t = Tokens::get(ui.ctx());
    let [r, g, b] = rgb.unwrap_or_else(|| {
        let [r, g, b, _] = app.session.tools.foreground;
        [r, g, b]
    });
    let (rect, resp) = ui.allocate_exact_size(vec2(26.0, 22.0), Sense::click());
    let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    ui.painter().rect_filled(rect, t.radius_sm, egui::Color32::from_rgb(q(r), q(g), q(b)));
    ui.painter().rect_stroke(rect, t.radius_sm, Stroke::new(1.0, t.field_border), egui::StrokeKind::Outside);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tl!("Set the text color")));
    if resp.on_hover_text(tl!("Set the text color")).clicked() {
        tt::open_color_picker(app, [r, g, b]);
    }
}

/// Left / centre / right; a justified paragraph lights none.
fn align_buttons(app: &mut PhotocraftApp, ui: &mut Ui, align: Option<photocraft_doc::text::TextAlign>) {
    use photocraft_doc::text::TextAlign;
    let shown_align = match align {
        Some(TextAlign::Left) => "left",
        Some(TextAlign::Center) => "center",
        Some(TextAlign::Right) => "right",
        Some(_) => "",
        None => app.ui.tool_options.type_align.as_str(),
    }
    .to_string();
    for (align, icon, tip) in
        [("left", "align-left", tl!("Left align text")), ("center", "align-center", tl!("Center text")), ("right", "align-right", tl!("Right align text"))]
    {
        if pill(ui, icon, None, shown_align == align, tip).clicked() {
            app.ui.tool_options.type_align = align.into();
            apply_style(app, ui.ctx(), json!({"align": align}));
        }
    }
}

/// Photoshop's font size menu presets (in the type unit).
const SIZE_PRESETS: [f32; 15] = [6.0, 8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 18.0, 24.0, 30.0, 36.0, 48.0, 60.0, 72.0, 96.0];

/// Scale of the target layer's transform: sizes show as the layer appears (`type_tool`'s
/// `shown_scale`).
fn shown_scale(app: &PhotocraftApp) -> f32 {
    let Some(t) = tt::target_text(app) else { return 1.0 };
    let m = t.transform.m;
    let k = (m[0] * m[3] - m[1] * m[2]).abs().sqrt() as f32;
    if k.is_finite() && k > 1e-3 { k } else { 1.0 }
}

/// The classic bar's font menu (`type_tool::fonts::font_picker`): recents, favourites, preview.
/// shortcut: copies the private `font_picker`; call it once `type_tool` exports it.
fn font_picker(app: &mut PhotocraftApp, ui: &mut Ui, current: &mut String, width: f32) -> Option<Option<String>> {
    let preview = app.session.prefs().type_.font_preview;
    let mut library = tt::FontLibrary { preview_text: preview_text(app), ..tt::FontLibrary::of(app) };
    let changed = tt::family_picker_in(ui, "studio-type-font", current, width, &tt::families(), preview, &mut library);
    let style = library.picked_style.take();
    library.finish(app);
    if !changed {
        return None;
    }
    app.note_recent_font(current);
    Some(style)
}

/// The font menu's preview: the selected characters, else the target's text, one line, ≤ 60 chars.
fn preview_text(app: &PhotocraftApp) -> Option<String> {
    let (_, range) = tt::target(app)?;
    let text = &tt::target_text(app)?.text;
    let chosen: String = match range {
        Some([a, b]) => text.chars().skip(a).take(b.saturating_sub(a)).collect(),
        None => text.clone(),
    };
    let line = chosen.split(['\n', '\r', '\u{2028}', '\u{2029}']).map(str::trim).filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" ");
    let line: String = line.chars().take(60).collect();
    (!line.is_empty()).then_some(line)
}

/// Apply style properties to the edit or the selected type layers, or to the tool defaults.
/// shortcut: copies the private `type_tool::defaults::apply` (+ `insertion_style::set_font`,
/// `apply_defaults`, `drag_key`); call it once `type_tool` exports it.
fn apply_style(app: &mut PhotocraftApp, ctx: &egui::Context, props: Value) {
    if pending_font(app, &props) {
        return;
    }
    app.ui.type_insertion_style = None;
    let Some(mut p) = tt::formatting_params(app) else {
        apply_defaults(app, &props);
        return;
    };
    if let Some(props) = props.as_object() {
        for (key, value) in props {
            p[key] = value.clone();
        }
    }
    let drag = || {
        let id = ctx.dragged_id()?;
        let t = ctx.input(|i| i.pointer.press_start_time())?;
        Some(format!("type-drag-{}-{}", id.value(), t.to_bits()))
    };
    if let Some(key) = app.ui.text_edit.as_ref().map(|ed| ed.session.clone()).or_else(drag) {
        p["coalesce"] = json!(key);
    }
    let r = app.run("type.setStyle", p);
    report(app, r);
}

/// At a collapsed caret, a font change is the typing font for the next characters.
fn pending_font(app: &mut PhotocraftApp, props: &Value) -> bool {
    if app.ui.text_edit.as_ref().is_none_or(|ed| ed.caret != ed.anchor) || (props.get("font").is_none() && props.get("fontStyle").is_none()) {
        return false;
    }
    crate::type_transform::finish(app);
    let Some(ed) = app.ui.text_edit.as_ref() else { return false };
    let Some(st) = app.session.active() else { return false };
    let empty = match st.doc.layer(LayerId(ed.layer)).map(|l| &l.content) {
        Some(LayerContent::Text(t)) => t.text.is_empty(),
        _ => false,
    };
    if empty {
        return false;
    }
    let Some((mut style, _)) = tt::styles_at(app) else { return false };
    photocraft_engine::type_cmds::apply_char_props(&mut style, props);
    let (Some(ed), Some(st)) = (app.ui.text_edit.as_ref(), app.session.active()) else { return false };
    app.ui.type_insertion_style = Some(crate::state::TypeInsertionStyle {
        document: st.doc.id.0,
        layer: ed.layer,
        session: ed.session.clone(),
        revision: st.revision,
        caret: ed.caret,
        style,
    });
    true
}

/// Without a type target the bar sets the Type tool's defaults for the next layer.
fn apply_defaults(app: &mut PhotocraftApp, props: &Value) {
    let o = &mut app.ui.tool_options;
    if let Some(v) = props.get("font").and_then(Value::as_str) {
        o.type_font = v.into();
    }
    if let Some(v) = props.get("fontStyle").and_then(Value::as_str) {
        o.type_style = v.into();
        // A named style replaces a weight-axis setting, as apply_char_props does.
        if let Some(axes) = o.type_style_defaults.get_mut("variations").and_then(Value::as_object_mut) {
            axes.remove("wght");
        }
    }
    if let Some(v) = props.get("size").and_then(Value::as_f64) {
        o.type_size = v as f32;
    }
    if let Some(v) = props.get("align").and_then(Value::as_str) {
        o.type_align = v.into();
    }
}

// ---- Selection bar ----------------------------------------------------------------------------

/// `app.ui.selection_mode` values (the classic bar's New / Add / Subtract / Intersect buttons).
const MODE_NEW: u8 = 0;
const MODE_ADD: u8 = 1;
const MODE_SUBTRACT: u8 = 2;
const MODE_INTERSECT: u8 = 3;

/// Clicking the active mode again returns to New selection.
fn toggled_mode(current: u8, clicked: u8) -> u8 {
    if current == clicked { MODE_NEW } else { clicked }
}

fn selection_bar(app: &mut PhotocraftApp, ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    for (mode, icon, label) in
        [(MODE_ADD, "square-plus", tl!("Add")), (MODE_SUBTRACT, "path-op-subtract", tl!("Subtract")), (MODE_INTERSECT, "path-op-intersect", tl!("Intersect"))]
    {
        if pill(ui, icon, Some(label), app.ui.selection_mode == mode, label).clicked() {
            app.ui.selection_mode = toggled_mode(app.ui.selection_mode, mode);
        }
    }
    for (id, icon, label, tip) in [
        ("select.inverse", "square-dashed", tl!("Invert"), tl!("Inverse")),
        ("select.selectAndMask", "sliders-horizontal", tl!("Refine"), tl!("Select and Mask…")),
    ] {
        command_pill(app, ui, &ctx, id, icon, label, tip);
    }
    crate::widgets::vline(ui, 22.0);
    command_pill(app, ui, &ctx, "select.deselect", "x", tl!("Deselect"), tl!("Deselect"));
}

/// A button for a command, greyed out when the command is (as its menu item is).
fn command_pill(app: &mut PhotocraftApp, ui: &mut Ui, ctx: &egui::Context, id: &str, icon: &str, label: &str, tip: &str) {
    let enabled = crate::menus::is_enabled(app, id);
    if ui.add_enabled_ui(enabled, |ui| pill(ui, icon, Some(label), false, tip)).inner.clicked() {
        invoke(app, ctx, id);
    }
}

// ---- Adjustment tabs --------------------------------------------------------------------------

/// The tabs: `layer.newAdjustmentLayer.<kind>`, icon, English label.
const TABS: [(&str, &str, &str); 5] = [
    ("levels", "gauge", "Levels"),
    ("curves", "spline", "Curves"),
    ("exposure", "sun", "Exposure"),
    ("hueSaturation", "blend", "Hue"),
    ("vibrance", "triangle", "Vibrance"),
];

/// The other adjustments, under Mais ▾.
const MORE: [(&str, &str); 11] = [
    ("brightnessContrast", "Brightness/Contrast"),
    ("colorBalance", "Color Balance"),
    ("blackWhite", "Black & White"),
    ("photoFilter", "Photo Filter"),
    ("channelMixer", "Channel Mixer"),
    ("colorLookup", "Color Lookup"),
    ("invert", "Invert"),
    ("posterize", "Posterize"),
    ("threshold", "Threshold"),
    ("gradientMap", "Gradient Map"),
    ("selectiveColor", "Selective Color"),
];

/// The adjustment kind of the active layer, if it is an adjustment layer.
fn active_adjustment(app: &PhotocraftApp) -> Option<&'static str> {
    let st = app.session.active()?;
    match &st.doc.layer(st.active_layer?)?.content {
        LayerContent::Adjustment(a) => Some(photocraft_engine::commands::adjustment_kind(a)),
        _ => None,
    }
}

fn adjustment_tabs(app: &mut PhotocraftApp, ui: &mut Ui) {
    let active = active_adjustment(app);
    let mut run = None;
    for (kind, icon, label) in TABS {
        if pill(ui, icon, Some(tl!(label)), active == Some(kind), tl!(label)).clicked() {
            run = Some(kind);
        }
    }
    crate::widgets::vline(ui, 22.0);
    let in_more = active.is_some_and(|k| MORE.iter().any(|(m, _)| *m == k));
    let more = pill(ui, "chevron-down", Some(tl!("More")), in_more, tl!("More adjustments"));
    crate::press_menu::show(&more, |ui, menu| {
        ui.set_min_width(190.0);
        for (kind, label) in MORE {
            if menu.item(ui, tl!(label)) {
                run = Some(kind);
            }
        }
    });
    // Like the Adjustments panel: add the layer and show its settings.
    if let Some(kind) = run {
        let r = app.run(&format!("layer.newAdjustmentLayer.{kind}"), json!({}));
        report(app, r);
        app.ui.dock_tabs.properties = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_bar_follows_type_tools_and_type_layers_except_in_colour_mode() {
        assert!(bars(Mode::Text, Tool::Type, false, false).type_bar);
        assert!(bars(Mode::Compose, Tool::Move, true, false).type_bar);
        assert!(!bars(Mode::Compose, Tool::Move, false, false).type_bar);
        assert!(!bars(Mode::ColorLight, Tool::Type, true, false).type_bar);
    }

    #[test]
    fn selection_bar_only_in_select_mode_with_a_selection_or_tool() {
        assert!(bars(Mode::Select, Tool::EllipseMarquee, false, false).selection);
        assert!(bars(Mode::Select, Tool::Move, false, true).selection);
        assert!(bars(Mode::Select, Tool::HorizontalTypeMask, false, false).selection);
        assert!(!bars(Mode::Select, Tool::Move, false, false).selection);
        assert!(!bars(Mode::Compose, Tool::Lasso, false, true).selection);
    }

    #[test]
    fn adjustment_tabs_only_in_colour_mode() {
        assert_eq!(bars(Mode::ColorLight, Tool::Eyedropper, false, true), Bars { adjustments: true, ..Bars::default() });
        assert!(!bars(Mode::Retouch, Tool::Brush, false, false).adjustments);
    }

    #[test]
    fn selection_mode_toggles_back_to_new() {
        assert_eq!(toggled_mode(MODE_NEW, MODE_ADD), MODE_ADD);
        assert_eq!(toggled_mode(MODE_ADD, MODE_ADD), MODE_NEW);
        assert_eq!(toggled_mode(MODE_ADD, MODE_SUBTRACT), MODE_SUBTRACT);
    }

    #[test]
    fn every_adjustment_command_is_offered_once() {
        let mut kinds: Vec<&str> = TABS.iter().map(|t| t.0).chain(MORE.iter().map(|m| m.0)).collect();
        kinds.sort_unstable();
        let mut ids: Vec<String> = photocraft_engine::command_specs()
            .iter()
            .filter_map(|c| c.id.strip_prefix("layer.newAdjustmentLayer.").map(str::to_string))
            .collect();
        ids.sort_unstable();
        assert_eq!(kinds, ids);
        for icon in TABS.iter().map(|t| t.1) {
            assert!(crate::icons::exists(icon), "{icon}");
        }
    }
}
