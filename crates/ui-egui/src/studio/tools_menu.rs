//! Floating "Ferramentas" button at the canvas' top-left: the current mode's tools (every tool
//! belongs to at least one mode), with their shortcuts, and the foreground/background colour
//! chips (the classic toolbar's, which the studio shell doesn't draw).

use egui::{Align2, Color32, FontId, Rect, Sense, Stroke, StrokeKind, pos2, vec2};
use serde_json::json;

use super::mode_bar::{select_tool, tools};
use super::style::RADIUS;
use crate::PhotocraftApp;
use crate::state::Tool;
use crate::theme::Tokens;

#[derive(Default)]
pub struct State {
    pub open: bool,
    /// The button's top-left offset from the canvas' top-left, once dragged (else the default).
    pub offset: Option<egui::Vec2>,
}

/// Where the button sits until dragged.
const DEFAULT_OFFSET: egui::Vec2 = vec2(14.0, 14.0);
/// More rows than this lay out in two columns.
const ONE_COLUMN_MAX: usize = 8;

fn c32(c: [f32; 4]) -> Color32 {
    let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color32::from_rgb(b(c[0]), b(c[1]), b(c[2]))
}

/// `offset` kept so a `size` button stays inside `canvas`.
fn clamp_offset(offset: egui::Vec2, size: egui::Vec2, canvas: Rect) -> egui::Vec2 {
    let max = (canvas.size() - size).max(vec2(0.0, 0.0));
    vec2(offset.x.clamp(0.0, max.x), offset.y.clamp(0.0, max.y))
}

pub fn show(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let ctx = ui.ctx().clone();
    let canvas = ui.max_rect();
    let open = app.studio.tools.open;
    let font = FontId::proportional(15.0);
    let galley = ui.painter().layout_no_wrap(tl!("Tools").to_string(), font, t.text);
    let size = vec2(galley.size().x + 90.0, 40.0);
    let offset = clamp_offset(app.studio.tools.offset.unwrap_or(DEFAULT_OFFSET), size, canvas);
    let area = egui::Area::new(egui::Id::unique("studio_tools_menu")).order(egui::Order::Foreground).fixed_pos(canvas.min + offset).show(&ctx, |ui| {
        // Drag anywhere on the button to move it; a click (no drag) opens the menu.
        let (r, resp) = ui.allocate_exact_size(size, Sense::click_and_drag());
        if resp.dragged() {
            app.studio.tools.offset = Some(clamp_offset(offset + resp.drag_delta(), size, canvas));
        }
        if resp.hovered() || resp.dragged() {
            ctx.set_cursor_icon(if resp.dragged() { egui::CursorIcon::Grabbing } else { egui::CursorIcon::Grab });
        }
        // Closed: a card with the brush on an amber chip; open: all amber.
        let ink = if open { t.primary_text } else { t.text };
        let fill = if open {
            t.primary_bg
        } else if resp.hovered() {
            t.hover
        } else {
            t.card
        };
        ui.painter().rect(r, RADIUS, fill, Stroke::new(1.0, if open { t.primary_bg } else { t.card_border }), StrokeKind::Inside);
        let chip = Rect::from_center_size(pos2(r.left() + 24.0, r.center().y), vec2(30.0, 30.0));
        if !open {
            ui.painter().rect_filled(chip, RADIUS - 4.0, t.primary_bg);
        }
        crate::icons::paint(ui, chip.shrink(6.0), "brush", 18.0, t.primary_text);
        ui.painter().galley_with_override_text_color(pos2(chip.right() + 12.0, r.center().y - galley.size().y / 2.0), galley, ink);
        crate::icons::paint(ui, Rect::from_center_size(pos2(r.right() - 20.0, r.center().y), vec2(14.0, 14.0)), "chevron-down", 14.0, ink);
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tl!("Tools")));
        if resp.clicked() {
            app.studio.tools.open = !open;
        }
        if open {
            ui.add_space(8.0);
            let below = canvas.bottom() - r.bottom() - 40.0;
            dropdown(app, ui, &t, &ctx, below.max(120.0));
        }
    });
    // A press outside the menu or Escape closes it.
    let outside = ctx.input(|i| i.pointer.any_pressed() && i.pointer.press_origin().is_some_and(|p| !area.response.rect.contains(p)));
    if app.studio.tools.open && (outside || ctx.input(|i| i.key_pressed(egui::Key::Escape))) {
        app.studio.tools.open = false;
    }
}

fn dropdown(app: &mut PhotocraftApp, ui: &mut egui::Ui, t: &Tokens, ctx: &egui::Context, max_h: f32) {
    let font = FontId::proportional(14.0);
    let rows: Vec<(&str, Tool)> = tools(app.studio.mode).iter().filter_map(|n| Some((*n, crate::tool_shortcuts::tool(&format!("tools.select.{n}"))?))).collect();
    let label_w = rows.iter().map(|(n, _)| ui.painter().layout_no_wrap(short(n).to_string(), font.clone(), t.text).size().x).fold(0.0f32, f32::max);
    let w = (label_w + 110.0).max(200.0);
    let per_column = if rows.len() > ONE_COLUMN_MAX { rows.len().div_ceil(2) } else { rows.len().max(1) };
    egui::Frame::NONE.fill(t.card).stroke(Stroke::new(1.0, t.card_border)).corner_radius(RADIUS).inner_margin(6.0).show(ui, |ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 2.0);
        egui::ScrollArea::vertical().id_salt("studio_tools_rows").max_height(max_h).show(ui, |ui| {
            ui.horizontal_top(|ui| {
                for column in rows.chunks(per_column) {
                    ui.vertical(|ui| {
                        for &(name, tool) in column {
                            tool_row(app, ui, t, ctx, name, tool, w);
                        }
                    });
                }
            });
        });
        ui.add_space(4.0);
        ui.separator();
        color_chips(app, ui, t);
    });
}

/// One menu row: icon, short name, shortcut chip; the active tool has the amber outline.
fn tool_row(app: &mut PhotocraftApp, ui: &mut egui::Ui, t: &Tokens, ctx: &egui::Context, name: &str, tool: Tool, w: f32) {
    let active = crate::tool_memory::selected(app);
    let (r, resp) = ui.allocate_exact_size(vec2(w, 36.0), Sense::click());
    if tool == active {
        ui.painter().rect_stroke(r.shrink(1.0), RADIUS - 2.0, Stroke::new(1.5, t.accent), StrokeKind::Inside);
    } else if resp.hovered() {
        ui.painter().rect_filled(r, RADIUS - 2.0, t.hover);
    }
    crate::icons::paint(ui, Rect::from_center_size(pos2(r.left() + 20.0, r.center().y), vec2(18.0, 18.0)), crate::icons::tool_icon(tool), 18.0, t.icon);
    ui.painter().text(pos2(r.left() + 42.0, r.center().y), Align2::LEFT_CENTER, short(name), FontId::proportional(14.0), t.text);
    if let Some(sc) = crate::tool_shortcuts::shortcut(app, tool) {
        let g = ui.painter().layout_no_wrap(crate::shortcuts::pretty(&sc), FontId::proportional(12.0), t.text_dim);
        let chip = Rect::from_center_size(pos2(r.right() - 10.0 - (g.size().x + 10.0).max(20.0) / 2.0, r.center().y), vec2((g.size().x + 10.0).max(20.0), 20.0));
        ui.painter().rect_filled(chip, 5.0, t.chrome);
        ui.painter().galley(chip.center() - g.size() / 2.0, g, t.text_dim);
    }
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, tool == active, tl!(tool.label())));
    if resp.on_hover_text(tl!(tool.label())).clicked() {
        select_tool(app, ctx, name);
        app.studio.tools.open = false;
    }
}

/// The menu's short English name of tool `name` (id without `tools.select.`), as in the mockups
/// ("Rectangle", "Lasso"); the full name stays the row's tooltip.
fn short_label(name: &str) -> Option<&'static str> {
    Some(match name {
        "move" => "Move",
        "artboard" => "Artboard",
        "rectMarquee" | "rectangle" => "Rectangle",
        "ellipseMarquee" | "ellipseShape" => "Ellipse",
        "singleRowMarquee" => "Single Row",
        "singleColumnMarquee" => "Single Column",
        "lasso" => "Lasso",
        "polygonLasso" => "Polygonal Lasso",
        "magneticLasso" => "Magnetic Lasso",
        "magicWand" => "Magic Wand",
        "crop" => "Crop",
        "perspectiveCrop" => "Perspective Crop",
        "eyedropper" => "Eyedropper",
        "colorSampler" => "Color Sampler",
        "ruler" => "Ruler",
        "note" => "Note",
        "count" => "Count",
        "brush" => "Brush",
        "pencil" => "Pencil",
        "colorReplacement" => "Color Replacement",
        "mixerBrush" => "Mixer Brush",
        "eraser" => "Eraser",
        "backgroundEraser" => "Background Eraser",
        "magicEraser" => "Magic Eraser",
        "gradient" => "Gradient",
        "paintBucket" => "Paint Bucket",
        "type" => "Horizontal Type",
        "verticalType" => "Vertical Type",
        "horizontalTypeMask" => "Horizontal Type Mask",
        "verticalTypeMask" => "Vertical Type Mask",
        "hand" => "Hand",
        "rotateView" => "Rotate View",
        "zoom" => "Zoom",
        "spotHealing" => "Spot Healing",
        "remove" => "Remove",
        "healing" => "Healing",
        "patch" => "Patch",
        "contentAwareMove" => "Content-Aware Move",
        "redEye" => "Red Eye",
        "cloneStamp" => "Clone Stamp",
        "patternStamp" => "Pattern Stamp",
        "historyBrush" => "History Brush",
        "artHistoryBrush" => "Art History Brush",
        "blur" => "Blur",
        "sharpen" => "Sharpen",
        "smudge" => "Smudge",
        "dodge" => "Dodge",
        "burn" => "Burn",
        "sponge" => "Sponge",
        "quickSelection" => "Quick Selection",
        "objectSelection" => "Object Selection",
        "pen" => "Pen",
        "freeformPen" => "Freeform Pen",
        "curvaturePen" => "Curvature Pen",
        "addAnchorPoint" => "Add Anchor Point",
        "deleteAnchorPoint" => "Delete Anchor Point",
        "convertPoint" => "Convert Point",
        "pathSelection" => "Path Selection",
        "directSelection" => "Direct Selection",
        "triangle" => "Triangle",
        "polygon" => "Polygon",
        "star" => "Star",
        "line" => "Line",
        "customShape" => "Custom Shape",
        "slice" => "Slice",
        "sliceSelect" => "Slice Select",
        "frame" => "Frame",
        _ => return None,
    })
}

/// [`short_label`] in the UI language (context `studio-tool`, else the plain catalog row).
fn short(name: &str) -> &str {
    crate::i18n::tr_ctx(crate::i18n::current(), "studio-tool", short_label(name).unwrap_or(name))
}

/// Foreground/background chips with Switch and Default Colors, as on the classic toolbar.
fn color_chips(app: &mut PhotocraftApp, ui: &mut egui::Ui, t: &Tokens) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
        for (target, color, tip) in
            [("foreground", app.session.tools.foreground, tl!("Set foreground color")), ("background", app.session.tools.background, tl!("Set background color"))]
        {
            let (r, resp) = ui.allocate_exact_size(vec2(26.0, 26.0), Sense::click());
            ui.painter().rect(r, 5.0, c32(color), Stroke::new(1.0, t.field_border), StrokeKind::Outside);
            resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tip.to_string()));
            if resp.on_hover_text(tip.to_string()).clicked() {
                crate::color_picker_ui::open(app, target);
            }
        }
        for (icon, id, tip) in [
            ("arrow-left-right", "tools.swapColors", tl!("Switch Foreground and Background Colors (X)")),
            ("contrast", "tools.defaultColors", tl!("Default Foreground and Background Colors (D)")),
        ] {
            // As the classic chips: highlighted type takes the new foreground colour (#5690).
            if crate::icons::button(ui, icon, 26.0, false, tip).clicked() && app.run(id, json!({})).is_ok() {
                crate::type_tool::foreground_changed(app);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use photocraft_engine::shell_commands::TOOLS;
    use egui_kittest::{Harness, kittest::Queryable};
    use serde_json::json;

    /// Clicking every studio control through the whole app frame must not deadlock: the layer
    /// strip once read egui's data inside a `data_mut` and froze the app on any click.
    #[test]
    fn clicking_every_studio_control_does_not_freeze() {
        let mut h = Harness::builder().with_size(vec2(1600.0, 900.0)).with_step_dt(1.0 / 60.0).build_eframe(|cc| {
            crate::PhotocraftApp::setup_context(&cc.egui_ctx, Default::default());
            let mut app = crate::PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
            app.run("file.new", json!({"width": 1200, "height": 800})).unwrap();
            app.run("layer.new.layer", json!({})).unwrap();
            app
        });
        h.run_steps(5);
        for l in ["Tools", "Tools", "Select", "Retouch", "Color and Light", "Text", "Compose", "History", "Navigator", "Info", "Panels", "History", "Menu", "Menu", "Export", "Layer 1", "Codex", "Codex"] {
            h.get_all_by_label(l).next().unwrap_or_else(|| panic!("no control labelled {l:?}")).click();
            h.run_steps(2);
        }
        // Drag the Tools button: it moves (and stays shut); a click then opens it and picks a tool.
        let before = h.get_by_label("Tools").rect();
        let from = before.center();
        h.drag_at(from);
        h.run_steps(1);
        for k in 1..=10 {
            h.hover_at(from + vec2(30.0, 6.0) * k as f32);
            h.run_steps(1);
        }
        h.drop_at(from + vec2(300.0, 60.0));
        h.run_steps(2);
        let after = h.get_by_label("Tools").rect();
        assert!((after.min - before.min).length() > 100.0, "the button did not move: {before:?} -> {after:?}");
        assert!(!h.state().studio.tools.open, "a drag must not open the menu");
        h.get_by_label("Tools").click();
        h.run_steps(2);
        assert!(h.state().studio.tools.open);
        h.get_by_label("Crop Tool").click();
        h.run_steps(2);
        assert_eq!(h.state().ui.tool, crate::state::Tool::Crop);
        assert!(!h.state().studio.tools.open);
    }

    #[test]
    fn button_offset_stays_inside_the_canvas() {
        let canvas = Rect::from_min_size(pos2(100.0, 50.0), vec2(800.0, 600.0));
        let size = vec2(150.0, 40.0);
        assert_eq!(clamp_offset(vec2(-50.0, 5000.0), size, canvas), vec2(0.0, 560.0));
        assert_eq!(clamp_offset(vec2(10.0, 10.0), vec2(2000.0, 40.0), canvas), vec2(0.0, 10.0));
    }


    #[test]
    fn every_tool_has_a_short_label_with_a_pt_br_row() {
        let Some(pt) = crate::i18n::Lang::from_code("pt-br") else { panic!("no pt-br") };
        for (id, _, _) in TOOLS {
            let name = id.strip_prefix("tools.select.").unwrap_or(id);
            let Some(label) = short_label(name) else { panic!("{name} has no short label") };
            let translated = crate::i18n::tr_ctx(pt, "studio-tool", label) != label || crate::i18n::has(pt, label);
            assert!(translated, "{label} has no pt-BR row");
        }
    }
}
