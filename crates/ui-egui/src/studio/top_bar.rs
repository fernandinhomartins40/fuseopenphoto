//! Top bar: ☰ (the whole menu tree), app name, document title, undo/redo, command search (⌘K),
//! save status and the Export button; also the window's drag area and caption buttons.

use egui::{CornerRadius, Rect, Sense, Stroke, StrokeKind, pos2, vec2};
use serde_json::json;

use super::style;
use crate::PhotocraftApp;
use crate::menus::{MenuItem, TOP_MENUS};
use crate::theme::{self, Tokens};

/// The bar's view state.
#[derive(Default)]
pub struct State {
    /// The active document (id, saved revision) last frame, to notice a save.
    seen: Option<(u64, u64)>,
    /// When (egui time, seconds) the active document was last saved in this session.
    saved_at: Option<f64>,
    /// The menu tree while ☰ is open: (state hash, items), rebuilt when the state changes.
    menu: Option<(u64, Vec<MenuItem>)>,
}

const EXPORT: &str = "file.export.exportAs";
/// Room kept between the groups and the search field, and the field's widest size.
const GAP: f32 = 16.0;
const SEARCH_W: f32 = 380.0;
const SEARCH_MIN: f32 = 140.0;

pub fn show(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let ctx = ui.ctx().clone();
    let (cap_l, cap_r) = crate::titlebar::reserved(app, &ctx);
    // macOS draws its traffic lights over an integrated title bar's left end.
    let mac_inset = if cfg!(target_os = "macos") && app.integrated_titlebar { 78.0 } else { 0.0 };
    let height = crate::titlebar::bar_height(app, &ctx, style::TOP_BAR_H);
    let mut clicked: Option<String> = None;
    let bar = egui::Panel::top("studio_top_bar").exact_size(height).frame(egui::Frame::NONE.fill(t.chrome)).show(ui, |ui| {
        let full = ui.max_rect();
        let left_edge = full.left() + if cap_l > 0.0 { cap_l + 4.0 } else { 12.0_f32.max(mac_inset) };
        let right_edge = full.right() - if cap_r > 0.0 { cap_r + 4.0 } else { 12.0 };
        let row = |l: f32, r: f32| Rect::from_x_y_ranges(l..=r.max(l), full.y_range());

        // Left: ☰, app name, document name, undo / redo.
        let mut left = ui.new_child(egui::UiBuilder::new().max_rect(row(left_edge, right_edge)).layout(egui::Layout::left_to_right(egui::Align::Center)));
        left.spacing_mut().item_spacing.x = 6.0;
        menu_button(app, &mut left, &mut clicked);
        left.add_space(6.0);
        left.label(egui::RichText::new(style::APP_NAME).font(theme::semibold(18.0)).color(t.text));
        left.add_space(8.0);
        let (sep, _) = left.allocate_exact_size(vec2(1.0, 20.0), Sense::hover());
        left.painter().vline(sep.center().x, sep.y_range(), Stroke::new(1.0, t.separator));
        left.add_space(8.0);
        let name = app.session.active().map_or_else(|| tl!("Untitled").to_string(), |d| d.doc.name.clone());
        left.add_sized(vec2(name_width(&left, &name), 20.0), egui::Label::new(egui::RichText::new(name).font(theme::medium(13.0)).color(t.text_dim)).truncate());
        left.add_space(10.0);
        for (icon, label, id) in [("undo-2", "Undo", "edit.undo"), ("redo-2", "Redo", "edit.redo")] {
            let tip = crate::shortcuts::tip_label(app, label, id);
            if left.add_enabled_ui(crate::menus::is_enabled(app, id), |ui| crate::icons::button(ui, icon, 30.0, false, &tip)).inner.clicked() {
                clicked = Some(id.to_string());
            }
        }
        let left_end = left.min_rect().right();

        // Right: save status, then Export at the far end.
        let mut right = ui.new_child(egui::UiBuilder::new().max_rect(row(left_end + GAP, right_edge)).layout(egui::Layout::right_to_left(egui::Align::Center)));
        right.spacing_mut().item_spacing.x = 14.0;
        if export_button(&mut right, crate::menus::is_enabled(app, EXPORT)).clicked() {
            clicked = Some(EXPORT.to_string());
        }
        save_status(app, &mut right);
        let right_start = right.min_rect().left().min(right_edge);

        // Centre: the command search, centred on the bar when it fits beside the groups.
        let (lo, hi) = (left_end + GAP, right_start - GAP);
        let mut gaps = vec![(lo, hi)];
        if let Some(x) = search_x(full.center().x, lo, hi) {
            let rect = Rect::from_center_size(pos2(x, full.center().y), vec2(search_width(lo, hi), 34.0));
            if search_field(app, ui, rect).clicked() {
                app.ui.palette_open = true;
            }
            gaps = vec![(lo, rect.left()), (rect.right(), hi)];
        }
        // Only the empty stretches drag the window (a press on a button must not move it), and a
        // double-click there maximizes, as on the classic title bar.
        let mut widest = Rect::NOTHING;
        for (i, (l, r)) in gaps.into_iter().enumerate() {
            if r - l < 4.0 {
                continue;
            }
            let gap = row(l, r);
            if gap.width() > widest.width() {
                widest = gap;
            }
            let drag = ui.interact(gap, ui.scope_id().with(("studio-titledrag", i)), Sense::click_and_drag());
            if drag.drag_started_by(egui::PointerButton::Primary) {
                crate::titlebar::start_os_move(&ctx);
            }
            if drag.double_clicked() {
                let max = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!max));
            }
        }
        ctx.data_mut(|d| d.insert_temp(crate::titlebar::modal_drag_area_id(), widest));
    });
    // A click anywhere on the bar keeps a zoom being typed (zoom_field.rs).
    crate::zoom_field::set_title_bar_rect(&ctx, bar.response.rect);
    if app.custom_titlebar {
        crate::titlebar::caption_buttons(app, ui, bar.response.rect);
    }
    if let Some(id) = clicked {
        run(app, &ctx, id);
    }
}

/// Runs a command picked on the bar or in the ☰ menu as the menu bar does: ⌥ + Merge runs the
/// Stamp variant, and ⌥ + an Image › Adjustments item opens it at its last settings.
fn run(app: &mut PhotocraftApp, ctx: &egui::Context, id: String) {
    let alt = ctx.input(|i| i.modifiers.alt);
    let r = if alt && crate::adjust_dialog::has_dialog(&id) {
        let opened = crate::adjust_dialog::open_last(app, &id).map(|_| serde_json::Value::Null).ok_or_else(|| "no document".to_string());
        if opened.is_ok() {
            crate::palette::note_recent(ctx, &id);
        }
        opened
    } else {
        crate::menus::invoke(app, ctx, &crate::menus::alt_click(id, alt), json!({}))
    };
    if let Err(e) = r {
        app.ui.status = e;
        app.ui.status_error = true;
    }
}

/// The document name's width: its own, capped so a long name can't push the search off the bar.
fn name_width(ui: &egui::Ui, name: &str) -> f32 {
    let w = ui.painter().layout_no_wrap(name.to_owned(), theme::medium(13.0), egui::Color32::PLACEHOLDER).size().x;
    w.clamp(20.0, 240.0)
}

/// ☰: the whole menu tree (File … Help) with submenus, checkmarks, enabled state and shortcuts.
fn menu_button(app: &mut PhotocraftApp, ui: &mut egui::Ui, clicked: &mut Option<String>) {
    let (rect, resp) = ui.allocate_exact_size(vec2(34.0, 34.0), Sense::click());
    let open = egui::Popup::is_id_open(ui.ctx(), egui::Popup::default_response_id(&resp));
    let tint = crate::icons::button_chrome(ui, rect, open, resp.hovered());
    for dy in [-6.0, 0.0, 6.0] {
        let y = rect.center().y + dy;
        ui.painter().hline(rect.center().x - 8.0..=rect.center().x + 8.0, y, Stroke::new(1.8, tint));
    }
    let resp = resp.on_hover_text(tl!("Menu"));
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tl!("Menu")));
    let mut kept = app.studio.top.menu.take();
    let app_ref: &PhotocraftApp = app;
    egui::Popup::menu(&resp).show(|ui| {
        // Built only while open (every item's state scales with the document), and reused while
        // what it depends on is unchanged.
        let hash = crate::native_menu::state_hash(app_ref, None);
        let fresh = match kept.take() {
            Some((h, items)) if h == hash && !crate::native_menu::had_input(ui.ctx()) => items,
            _ => crate::menus::menu_items(app_ref),
        };
        let lang = crate::i18n::current();
        ui.set_min_width(200.0);
        crate::widgets::style_spectrum_popup_menu(ui);
        for top in TOP_MENUS {
            let mine: Vec<&MenuItem> = fresh.iter().filter(|i| i.path.first().map(String::as_str) == Some(top)).collect();
            let enabled = mine.iter().any(|c| c.enabled && c.label != "---");
            ui.add_enabled_ui(enabled, |ui| ui.menu_button(crate::i18n::tr(lang, top), |ui| level(ui, &mine, 1, clicked)));
        }
        kept = Some((hash, fresh));
    });
    app.studio.top.menu = kept;
}

/// One menu level: the leaves at `depth` and a submenu for each deeper group, in menu order.
fn level(ui: &mut egui::Ui, items: &[&MenuItem], depth: usize, clicked: &mut Option<String>) {
    let lang = crate::i18n::current();
    // A menu taller than the window scrolls instead of running off it.
    let room = (ui.ctx().content_rect().height() - 60.0).max(120.0);
    egui::ScrollArea::vertical().max_height(room).show(ui, |ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
        ui.spacing_mut().item_spacing.y = 0.0;
        crate::widgets::style_spectrum_popup_menu(ui);
        let mut subs: Vec<&str> = Vec::new();
        let mut last_sep = true;
        for (i, it) in items.iter().enumerate() {
            if it.path.len() == depth {
                if it.label == "---" {
                    if !last_sep && i + 1 < items.len() {
                        ui.add(egui::Separator::default().spacing(9.0));
                        last_sep = true;
                    }
                    continue;
                }
                let text = crate::menu_keys::menu_row_text(ui, it.checked == Some(true), &item_label(lang, it), None);
                let mut b = egui::Button::new(text);
                if let Some(sc) = &it.shortcut {
                    b = b.shortcut_text(crate::shortcuts::pretty(sc));
                }
                if ui.add_enabled(it.enabled, b).clicked() {
                    *clicked = Some(it.id.clone());
                    ui.close();
                }
                last_sep = false;
            } else if let Some(name) = it.path.get(depth).map(String::as_str) {
                if subs.contains(&name) {
                    continue;
                }
                subs.push(name);
                let child: Vec<&MenuItem> = items.iter().copied().filter(|c| c.path.len() > depth && c.path.get(depth).map(String::as_str) == Some(name)).collect();
                let enabled = child.iter().any(|c| c.enabled && c.label != "---");
                let title = crate::menu_keys::menu_row_text(ui, false, crate::i18n::tr(lang, name), None);
                ui.add_enabled_ui(enabled, |ui| ui.menu_button(title, |ui| level(ui, &child, depth + 1, clicked)));
                last_sep = false;
            }
        }
    });
}

/// A row's label in the UI language, as the menu bar shows it.
fn item_label(lang: crate::i18n::Lang, it: &MenuItem) -> String {
    // A document's name (Window › open documents) is shown as it is.
    if it.id.starts_with("window.document.") {
        return it.label.clone();
    }
    if it.id == "file.export.quickExportAsPng"
        && let Some(fmt) = it.label.strip_prefix("Quick Export as ")
    {
        return crate::i18n::tr_id(lang, &it.id, "Quick Export as PNG").replace("PNG", fmt);
    }
    crate::i18n::tr_id(lang, &it.id, &it.label).to_string()
}

/// The search field's centre x: centred on the bar when it fits, else slid into the
/// free room from `lo` to `hi`; None when the room is too small for the narrowest field.
fn search_x(center: f32, lo: f32, hi: f32) -> Option<f32> {
    let w = search_width(lo, hi);
    if !(center.is_finite() && lo.is_finite() && hi.is_finite()) || w < SEARCH_MIN {
        return None;
    }
    Some(center.clamp(lo + w / 2.0, hi - w / 2.0))
}

fn search_width(lo: f32, hi: f32) -> f32 {
    (hi - lo).min(SEARCH_W)
}

/// The rounded "Search command ⌘K" field; a click opens the command palette.
fn search_field(app: &PhotocraftApp, ui: &mut egui::Ui, rect: Rect) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let resp = ui.interact(rect, ui.scope_id().with("studio-search"), Sense::click()).on_hover_cursor(egui::CursorIcon::Text);
    let label = tl!("Search command");
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
    let border = if resp.hovered() || app.ui.palette_open { t.accent_border } else { t.field_border };
    let p = ui.painter();
    p.rect(rect, CornerRadius::same(10), t.field, Stroke::new(1.0, border), StrokeKind::Inside);
    let icon = Rect::from_center_size(pos2(rect.left() + 20.0, rect.center().y), vec2(16.0, 16.0));
    crate::icons::paint(ui, icon, "search", 16.0, t.text_dim);
    let text = match crate::shortcuts::shortcut_label(app, "edit.search") {
        Some(sc) => format!("{label} {sc}"),
        None => label.to_string(),
    };
    let mut job = egui::text::LayoutJob::simple_singleline(text, theme::medium(13.5), t.text_dim);
    job.wrap = egui::text::TextWrapping::truncate_at_width(rect.width() - 48.0);
    let galley = p.layout_job(job);
    p.galley(pos2(rect.left() + 38.0, rect.center().y - galley.size().y / 2.0), galley, t.text_dim);
    resp
}

/// "✓ Saved 7 s ago" / "• Not saved", from the active document's dirty state and the last save
/// this session saw.
fn save_status(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let Some(d) = app.session.active() else {
        app.studio.top.seen = None;
        return;
    };
    let now = ui.input(|i| i.time);
    let cur = (d.doc.id.0, d.saved_revision);
    let dirty = d.is_dirty();
    let st = &mut app.studio.top;
    st.saved_at = track(st.seen, cur, st.saved_at, now);
    st.seen = Some(cur);
    let since = st.saved_at.map(|at| now - at);
    let (template, n) = save_text(dirty, since);
    let text = match n {
        Some(n) => crate::i18n::fmt(tl!(template), &[("n", n.to_string().as_str())]),
        None => tl!(template).to_string(),
    };
    // Keep the count current: every second for the first minute, then every minute.
    if let Some(s) = since.filter(|_| !dirty) {
        ui.ctx().request_repaint_after(std::time::Duration::from_secs(if s < 60.0 { 1 } else { 30 }));
    }
    ui.spacing_mut().item_spacing.x = 6.0;
    ui.label(egui::RichText::new(text).font(theme::medium(13.0)).color(t.text_dim));
    if dirty {
        let (dot, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
        ui.painter().circle_filled(dot.center(), 3.5, t.accent);
    } else {
        let (r, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
        crate::icons::paint(ui, r, "check", 16.0, t.text_dim);
    }
}

/// When the active document was last saved: now when its saved revision moved since last frame
/// (a save), unknown when another document became active.
fn track(prev: Option<(u64, u64)>, cur: (u64, u64), at: Option<f64>, now: f64) -> Option<f64> {
    match prev {
        Some((id, rev)) if id == cur.0 => if rev != cur.1 { Some(now) } else { at },
        _ => None,
    }
}

/// The status line's English template and its count, for `since` seconds since the last save.
fn save_text(dirty: bool, since: Option<f64>) -> (&'static str, Option<u64>) {
    match since {
        _ if dirty => ("Not saved", None),
        Some(s) if s.is_finite() && s >= 0.0 => {
            // Truncating whole seconds of a finite, non-negative count is the intent.
            let s = s as u64;
            match s {
                0..60 => ("Saved {n} s ago", Some(s)),
                60..3600 => ("Saved {n} min ago", Some(s / 60)),
                _ => ("Saved {n} h ago", Some(s / 3600)),
            }
        }
        _ => ("Saved", None),
    }
}

/// The amber primary "Export" button with an export glyph (a tray with an arrow out of it).
fn export_button(ui: &mut egui::Ui, enabled: bool) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let label = tl!("Export");
    let galley = ui.painter().layout_no_wrap(label.to_owned(), theme::semibold(14.5), t.primary_text);
    let size = vec2(galley.size().x + 56.0, 36.0);
    let (rect, resp) = ui.allocate_exact_size(size, if enabled { Sense::click() } else { Sense::hover() });
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, label));
    let fill = match (enabled, resp.is_pointer_button_down_on(), resp.hovered()) {
        (false, ..) => t.primary_bg.gamma_multiply(0.45),
        (true, true, _) => t.primary_bg.gamma_multiply(0.85),
        (true, false, true) => t.primary_bg.gamma_multiply(1.08),
        _ => t.primary_bg,
    };
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(10), fill);
    let ink = Stroke::new(1.7, t.primary_text);
    let c = pos2(rect.left() + 24.0, rect.center().y);
    // Tray.
    p.line(vec![pos2(c.x - 7.0, c.y - 1.0), pos2(c.x - 7.0, c.y + 7.0), pos2(c.x + 7.0, c.y + 7.0), pos2(c.x + 7.0, c.y - 1.0)], ink);
    // Arrow up out of it.
    p.line_segment([pos2(c.x, c.y + 3.0), pos2(c.x, c.y - 8.0)], ink);
    p.line(vec![pos2(c.x - 4.0, c.y - 4.0), pos2(c.x, c.y - 8.0), pos2(c.x + 4.0, c.y - 4.0)], ink);
    p.galley(pos2(rect.left() + 40.0, rect.center().y - galley.size().y / 2.0), galley, t.primary_text);
    resp.on_hover_text(tl!("Export As…"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_text_counts_up_and_flags_unsaved() {
        assert_eq!(save_text(true, Some(3.0)), ("Not saved", None));
        assert_eq!(save_text(false, None), ("Saved", None));
        assert_eq!(save_text(false, Some(7.9)), ("Saved {n} s ago", Some(7)));
        assert_eq!(save_text(false, Some(125.0)), ("Saved {n} min ago", Some(2)));
        assert_eq!(save_text(false, Some(7300.0)), ("Saved {n} h ago", Some(2)));
        assert_eq!(save_text(false, Some(f64::NAN)), ("Saved", None));
        assert_eq!(save_text(false, Some(-1.0)), ("Saved", None));
    }

    #[test]
    fn track_notes_a_save_and_forgets_on_switch() {
        assert_eq!(track(None, (1, 0), None, 5.0), None);
        assert_eq!(track(Some((1, 0)), (1, 0), None, 5.0), None);
        assert_eq!(track(Some((1, 0)), (1, 4), None, 5.0), Some(5.0));
        assert_eq!(track(Some((1, 4)), (1, 4), Some(5.0), 9.0), Some(5.0));
        assert_eq!(track(Some((1, 4)), (2, 4), Some(5.0), 9.0), None);
    }

    #[test]
    fn search_centres_then_slides_then_hides() {
        assert_eq!(search_x(500.0, 100.0, 900.0), Some(500.0));
        assert_eq!(search_x(500.0, 400.0, 900.0), Some(590.0));
        assert_eq!(search_x(500.0, 400.0, 500.0), None);
        assert_eq!(search_x(f32::NAN, 0.0, 900.0), None);
    }
}
