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
}

fn c32(c: [f32; 4]) -> Color32 {
    let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color32::from_rgb(b(c[0]), b(c[1]), b(c[2]))
}

pub fn show(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let ctx = ui.ctx().clone();
    let canvas = ui.max_rect();
    let open = app.studio.tools.open;
    let area = egui::Area::new(egui::Id::unique("studio_tools_menu")).order(egui::Order::Foreground).fixed_pos(canvas.min + vec2(14.0, 14.0)).show(&ctx, |ui| {
        // The button: brush icon, "Tools", chevron; amber while open.
        let font = FontId::proportional(15.0);
        let galley = ui.painter().layout_no_wrap(tl!("Tools").to_string(), font, t.text);
        let (r, resp) = ui.allocate_exact_size(vec2(galley.size().x + 82.0, 40.0), Sense::click());
        let ink = if open { t.primary_text } else { t.text };
        let fill = if open {
            t.primary_bg
        } else if resp.hovered() {
            t.hover
        } else {
            t.card
        };
        ui.painter().rect(r, RADIUS, fill, Stroke::new(1.0, if open { t.primary_bg } else { t.card_border }), StrokeKind::Inside);
        crate::icons::paint(ui, Rect::from_center_size(pos2(r.left() + 24.0, r.center().y), vec2(18.0, 18.0)), "brush", 18.0, ink);
        ui.painter().galley_with_override_text_color(pos2(r.left() + 44.0, r.center().y - galley.size().y / 2.0), galley, ink);
        crate::icons::paint(ui, Rect::from_center_size(pos2(r.right() - 20.0, r.center().y), vec2(14.0, 14.0)), "chevron-down", 14.0, ink);
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tl!("Tools")));
        if resp.clicked() {
            app.studio.tools.open = !open;
        }
        if open {
            ui.add_space(8.0);
            dropdown(app, ui, &t, &ctx, (canvas.height() - 120.0).max(120.0));
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
    let label_w = rows.iter().map(|(_, tool)| ui.painter().layout_no_wrap(tl!(tool.label()).to_string(), font.clone(), t.text).size().x).fold(0.0f32, f32::max);
    let w = (label_w + 100.0).max(220.0);
    let active = crate::tool_memory::selected(app);
    egui::Frame::NONE.fill(t.card).stroke(Stroke::new(1.0, t.card_border)).corner_radius(RADIUS).inner_margin(6.0).show(ui, |ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
        egui::ScrollArea::vertical().id_salt("studio_tools_rows").max_height(max_h).show(ui, |ui| {
            for (name, tool) in rows {
                let (r, resp) = ui.allocate_exact_size(vec2(w, 36.0), Sense::click());
                if tool == active {
                    ui.painter().rect_stroke(r.shrink(1.0), RADIUS - 2.0, Stroke::new(1.5, t.accent), StrokeKind::Inside);
                } else if resp.hovered() {
                    ui.painter().rect_filled(r, RADIUS - 2.0, t.hover);
                }
                crate::icons::paint(ui, Rect::from_center_size(pos2(r.left() + 20.0, r.center().y), vec2(18.0, 18.0)), crate::icons::tool_icon(tool), 18.0, t.icon);
                ui.painter().text(pos2(r.left() + 42.0, r.center().y), Align2::LEFT_CENTER, tl!(tool.label()), font.clone(), t.text);
                if let Some(sc) = crate::tool_shortcuts::shortcut(app, tool) {
                    ui.painter().text(pos2(r.right() - 12.0, r.center().y), Align2::RIGHT_CENTER, crate::shortcuts::pretty(&sc), FontId::proportional(12.5), t.text_dim);
                }
                resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, tool == active, tl!(tool.label())));
                if resp.clicked() {
                    select_tool(app, ctx, name);
                    app.studio.tools.open = false;
                }
            }
        });
        ui.add_space(4.0);
        ui.separator();
        color_chips(app, ui, t);
    });
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
