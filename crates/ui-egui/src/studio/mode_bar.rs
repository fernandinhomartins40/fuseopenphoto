//! Mode bar: Compor · Selecionar · Retocar · Cor e luz · Texto (`studio::Mode`), centred under
//! the top bar; picking a mode selects its default tool, and picking a tool anywhere else
//! (shortcut, menu, palette) moves the bar to a mode that lists it.
//!
//! Under it, the classic tool options bar is the tool options row, drawn in the house look (see
//! [`options_row`]), so every tool option (brush size, feather, …) stays reachable.

use egui::{CornerRadius, FontId, Rect, Sense, Stroke, pos2, vec2};
use serde_json::json;

use super::Mode;
use super::style::{MODE_BAR_H, RADIUS};
use crate::PhotocraftApp;
use crate::state::Tool;
use crate::theme::Tokens;

#[derive(Default)]
pub struct State {
    /// The tool seen last frame, to notice a tool picked elsewhere.
    last_tool: Option<Tool>,
}

/// The modes in bar order.
pub const MODES: [Mode; 5] = [Mode::Compose, Mode::Select, Mode::Retouch, Mode::ColorLight, Mode::Text];

/// The tools of `mode` (ids without `tools.select.`), as in `docs/redesign.md`; the first is the
/// mode's default.
pub fn tools(mode: Mode) -> &'static [&'static str] {
    match mode {
        Mode::Compose => &[
            "move",
            "artboard",
            "crop",
            "perspectiveCrop",
            "frame",
            "rectangle",
            "ellipseShape",
            "triangle",
            "polygon",
            "star",
            "line",
            "customShape",
            "pen",
            "freeformPen",
            "curvaturePen",
            "addAnchorPoint",
            "deleteAnchorPoint",
            "convertPoint",
            "pathSelection",
            "directSelection",
            "gradient",
            "paintBucket",
            "slice",
            "sliceSelect",
            "hand",
            "zoom",
            "rotateView",
            "ruler",
            "note",
            "count",
        ],
        Mode::Select => &[
            "rectMarquee",
            "ellipseMarquee",
            "singleRowMarquee",
            "singleColumnMarquee",
            "lasso",
            "polygonLasso",
            "magneticLasso",
            "objectSelection",
            "quickSelection",
            "magicWand",
            "horizontalTypeMask",
            "verticalTypeMask",
        ],
        Mode::Retouch => &[
            "brush",
            "pencil",
            "mixerBrush",
            "colorReplacement",
            "eraser",
            "backgroundEraser",
            "magicEraser",
            "spotHealing",
            "remove",
            "healing",
            "patch",
            "contentAwareMove",
            "redEye",
            "cloneStamp",
            "patternStamp",
            "historyBrush",
            "artHistoryBrush",
            "blur",
            "sharpen",
            "smudge",
            "dodge",
            "burn",
            "sponge",
        ],
        Mode::ColorLight => &["eyedropper", "colorSampler", "gradient", "paintBucket", "dodge", "burn", "sponge"],
        Mode::Text => &["type", "verticalType", "horizontalTypeMask", "verticalTypeMask"],
    }
}

/// The mode to show for `tool`: `current` if it lists the tool, else the first mode that does
/// (`current` for a tool no mode lists).
pub fn mode_for_tool(current: Mode, tool: &str) -> Mode {
    if tools(current).contains(&tool) {
        return current;
    }
    MODES.into_iter().find(|m| tools(*m).contains(&tool)).unwrap_or(current)
}

/// `tool`'s id without the `tools.select.` prefix.
pub fn tool_name(tool: Tool) -> String {
    let id = crate::tool_shortcuts::id(tool);
    id.strip_prefix("tools.select.").unwrap_or(&id).to_string()
}

/// Select tool `name` (no prefix) the way the menus and shortcuts do.
pub fn select_tool(app: &mut PhotocraftApp, ctx: &egui::Context, name: &str) {
    if let Err(e) = crate::menus::invoke(app, ctx, &format!("tools.select.{name}"), json!({})) {
        app.ui.status = e;
        app.ui.status_error = true;
    }
}

fn label(mode: Mode) -> &'static str {
    match mode {
        Mode::Compose => "Compose",
        Mode::Select => "Select",
        Mode::Retouch => "Retouch",
        Mode::ColorLight => "Color and Light",
        Mode::Text => "Text",
    }
}

fn icon(mode: Mode) -> &'static str {
    match mode {
        Mode::Compose => "crop",
        Mode::Select => "square-dashed",
        Mode::Retouch => "bandage",
        Mode::ColorLight => "sun",
        Mode::Text => "type",
    }
}

pub fn show(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    // Follow the active tool (the remembered one, so a held spring-loaded key doesn't flip modes).
    let tool = crate::tool_memory::selected(app);
    if app.studio.modes.last_tool != Some(tool) {
        app.studio.modes.last_tool = Some(tool);
        app.studio.mode = mode_for_tool(app.studio.mode, &tool_name(tool));
    }
    let t = Tokens::get(ui.ctx());
    egui::Panel::top("studio_mode_bar")
        .exact_size(MODE_BAR_H)
        .frame(egui::Frame::NONE.fill(t.chrome).stroke(Stroke::new(1.0, t.separator)))
        .show(ui, |ui| {
            let full = ui.max_rect();
            let font = FontId::proportional(15.0);
            let (icon_w, gap, pad, sep_gap) = (18.0, 10.0, 18.0, 14.0);
            let items: Vec<(Mode, std::sync::Arc<egui::Galley>)> =
                MODES.iter().map(|&m| (m, ui.painter().layout_no_wrap(tl!(label(m)).to_string(), font.clone(), t.text))).collect();
            let widths: Vec<f32> = items.iter().map(|(_, g)| pad * 2.0 + icon_w + gap + g.size().x).collect();
            let total = widths.iter().sum::<f32>() + (items.len().saturating_sub(1)) as f32 * (sep_gap * 2.0 + 1.0);
            let mut x = (full.center().x - total / 2.0).max(full.left() + 8.0);
            let h = (MODE_BAR_H - 16.0).max(20.0);
            let ctx = ui.ctx().clone();
            for (i, ((mode, galley), w)) in items.into_iter().zip(widths).enumerate() {
                if i > 0 {
                    x += sep_gap;
                    let c = full.center().y;
                    ui.painter().line_segment([pos2(x, c - 11.0), pos2(x, c + 11.0)], Stroke::new(1.0, t.separator));
                    x += 1.0 + sep_gap;
                }
                let r = Rect::from_min_size(pos2(x, full.center().y - h / 2.0), vec2(w, h));
                x += w;
                let resp = ui.interact(r, ui.scope_id().with(("mode", i)), Sense::click());
                let active = app.studio.mode == mode;
                let ink = if active { t.primary_text } else { t.text };
                if active {
                    ui.painter().rect_filled(r, RADIUS, t.primary_bg);
                } else if resp.hovered() {
                    ui.painter().rect_filled(r, RADIUS, t.hover);
                }
                let icon_rect = Rect::from_min_size(pos2(r.left() + pad, r.center().y - icon_w / 2.0), vec2(icon_w, icon_w));
                crate::icons::paint(ui, icon_rect, icon(mode), icon_w, ink);
                ui.painter().galley_with_override_text_color(pos2(icon_rect.right() + gap, r.center().y - galley.size().y / 2.0), galley, ink);
                resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, active, tl!(label(mode))));
                if resp.clicked() {
                    app.studio.mode = mode;
                    if let Some(first) = tools(mode).first() {
                        select_tool(app, &ctx, first);
                    }
                }
            }
        });
    // shortcut: the classic options bar as a second row, until context bars hold every option.
    options_row(app, ui);
}

/// Where `theme::apply` publishes the tokens (`Tokens::get` reads them from there).
fn theme_id() -> egui::Id {
    egui::Id::unique("photocraft-theme")
}

/// The tokens the options row draws with: fields and dropdowns as charcoal pills on the dark bar.
fn row_tokens(t: Tokens) -> Tokens {
    Tokens { field: t.card, radius_sm: t.radius_sm.max(8.0), ..t }
}

/// The classic tool options bar (every option of every tool) in the studio look: card-coloured
/// rounded fields and dropdowns, roomier spacing and a bottom separator. The look is scoped to this
/// call (tokens and `ui` style are restored after), so the classic shell keeps its own.
fn options_row(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    let t = Tokens::get(&ctx);
    let row = row_tokens(t);
    let style = ui.style().clone();
    {
        let s = ui.style_mut();
        s.spacing.item_spacing.x = 10.0;
        s.spacing.button_padding = vec2(10.0, 4.0);
        let w = &mut s.visuals.widgets;
        for (v, fill) in [(&mut w.inactive, t.card), (&mut w.hovered, t.hover), (&mut w.active, t.pressed), (&mut w.open, t.hover)] {
            v.bg_fill = fill;
            v.weak_bg_fill = fill;
            v.corner_radius = CornerRadius::same(row.radius_sm as u8);
        }
        w.hovered.bg_stroke = Stroke::new(1.0, t.accent_border);
        w.open.bg_stroke = Stroke::new(1.0, t.accent);
    }
    ctx.data_mut(|d| d.insert_temp(theme_id(), row));
    crate::panels::options_bar(app, ui);
    // Restore, unless something in the bar applied another theme meanwhile.
    if Tokens::get(&ctx) == row {
        ctx.data_mut(|d| d.insert_temp(theme_id(), t));
    }
    ui.set_style(style);
    let y = ui.available_rect_before_wrap().top();
    ui.painter().hline(ui.max_rect().x_range(), y - 0.5, Stroke::new(1.0, t.separator));
}

#[cfg(test)]
mod tests {
    use super::*;
    use photocraft_engine::shell_commands::TOOLS;

    #[test]
    fn every_tool_is_in_a_mode_and_every_mode_tool_exists() {
        for (id, _, _) in TOOLS {
            let name = id.strip_prefix("tools.select.").unwrap_or(id);
            assert!(MODES.iter().any(|m| tools(*m).contains(&name)), "{name} is in no mode");
        }
        for m in MODES {
            assert!(!tools(m).is_empty());
            for name in tools(m) {
                let id = format!("tools.select.{name}");
                assert!(TOOLS.iter().any(|t| t.0 == id), "{name} is not a tool");
                assert!(crate::tool_shortcuts::tool(&id).is_some(), "{name} has no Tool");
            }
        }
    }

    #[test]
    fn options_row_tokens_reach_tokens_get_and_are_restored() {
        let ctx = egui::Context::default();
        crate::theme::apply(&ctx, crate::theme::ThemeKind::Studio);
        let t = Tokens::get(&ctx);
        let row = row_tokens(t);
        assert_ne!(row, t);
        ctx.data_mut(|d| d.insert_temp(theme_id(), row));
        assert_eq!(Tokens::get(&ctx), row, "theme_id() must match theme::apply's key");
        ctx.data_mut(|d| d.insert_temp(theme_id(), t));
        assert_eq!(Tokens::get(&ctx), t);
    }

    #[test]
    fn mode_for_tool_keeps_the_current_mode_when_it_lists_the_tool() {
        assert_eq!(mode_for_tool(Mode::ColorLight, "gradient"), Mode::ColorLight);
        assert_eq!(mode_for_tool(Mode::Select, "gradient"), Mode::Compose);
        assert_eq!(mode_for_tool(Mode::Compose, "brush"), Mode::Retouch);
        assert_eq!(mode_for_tool(Mode::Text, "horizontalTypeMask"), Mode::Text);
        assert_eq!(mode_for_tool(Mode::Retouch, "horizontalTypeMask"), Mode::Select);
        assert_eq!(mode_for_tool(Mode::Retouch, "nope"), Mode::Retouch);
        assert_eq!(tool_name(Tool::Move), "move");
    }
}
