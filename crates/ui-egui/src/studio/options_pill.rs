//! The active tool's options (the classic options bar's row, `panels::tool_options`, every control
//! kept) drawn inside a floating context bar at the top of the canvas, in the studio look: charcoal
//! rounded fields and dropdowns. Wider than the canvas, the row scrolls horizontally.

use egui::{CornerRadius, Stroke, vec2};

use crate::PhotocraftApp;
use crate::state::Tool;
use crate::theme::Tokens;

/// The Type tools' options are the type bar's (`context_bars`), so they get no pill.
pub fn shows_for(tool: Tool, transforming: bool) -> bool {
    transforming || !tool.is_type()
}

/// Where `theme::apply` publishes the tokens (`Tokens::get` reads them from there).
fn theme_id() -> egui::Id {
    egui::Id::unique("photocraft-theme")
}

/// The tokens the row draws with: fields and dropdowns as charcoal pills on the dark card.
fn row_tokens(t: Tokens) -> Tokens {
    Tokens { field: t.chrome, radius_sm: t.radius_sm.max(8.0), ..t }
}

/// The row, at most `max_w` wide. The look is scoped to this call (tokens and `ui` style are
/// restored after), so the classic shell keeps its own.
pub fn contents(app: &mut PhotocraftApp, ui: &mut egui::Ui, max_w: f32) {
    let ctx = ui.ctx().clone();
    let t = Tokens::get(&ctx);
    let row = row_tokens(t);
    {
        let s = ui.style_mut();
        s.spacing.item_spacing.x = 8.0;
        s.spacing.button_padding = vec2(10.0, 4.0);
        s.spacing.scroll = egui::style::ScrollStyle { bar_width: 4.0, ..egui::style::ScrollStyle::floating() };
        let w = &mut s.visuals.widgets;
        for (v, fill) in [(&mut w.inactive, t.chrome), (&mut w.hovered, t.hover), (&mut w.active, t.pressed), (&mut w.open, t.hover)] {
            v.bg_fill = fill;
            v.weak_bg_fill = fill;
            v.corner_radius = CornerRadius::same(row.radius_sm as u8);
        }
        w.hovered.bg_stroke = Stroke::new(1.0, t.accent_border);
        w.open.bg_stroke = Stroke::new(1.0, t.accent);
    }
    ctx.data_mut(|d| d.insert_temp(theme_id(), row));
    // An Area offers its last size; without this the row stays clipped at the first frame's width.
    ui.set_max_width(max_w);
    egui::ScrollArea::horizontal().id_salt("studio-tool-options").max_width(max_w).show(ui, |ui| {
        crate::panels::tool_options(app, ui, false);
    });
    // Restore, unless something in the row applied another theme meanwhile.
    if Tokens::get(&ctx) == row {
        ctx.data_mut(|d| d.insert_temp(theme_id(), t));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_tokens_reach_tokens_get_and_are_restored() {
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
    fn type_tools_use_the_type_bar_unless_transforming() {
        assert!(!shows_for(Tool::Type, false));
        assert!(!shows_for(Tool::VerticalType, false));
        assert!(shows_for(Tool::Type, true));
        assert!(!shows_for(Tool::HorizontalTypeMask, false));
        assert!(shows_for(Tool::Brush, false));
    }
}
