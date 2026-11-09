//! The Home screen's top bar: ☰ (the whole menu tree), the app name, "Home", the command search
//! in the middle, Preferences, and the window's drag area and caption buttons.

use egui::{CornerRadius, Rect, Sense, Stroke, StrokeKind, pos2, vec2};

use crate::PhotocraftApp;
use crate::menus::{MenuItem, TOP_MENUS};
use crate::studio::style::APP_NAME;
use crate::theme::{self, Tokens};

const BAR_H: f32 = 60.0;
const SEARCH_W: f32 = 430.0;
const SEARCH_MIN: f32 = 140.0;
const PREFS: &str = "edit.preferences.general";

pub(crate) fn home_bar(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let ctx = ui.ctx().clone();
    let (cap_l, cap_r) = crate::titlebar::reserved(app, &ctx);
    let mac_inset = if cfg!(target_os = "macos") && app.integrated_titlebar { 78.0 } else { 0.0 };
    let height = crate::titlebar::bar_height(app, &ctx, BAR_H);
    let mut clicked: Option<String> = None;
    let bar = egui::Panel::top("home_top_bar").exact_size(height).frame(egui::Frame::NONE.fill(t.canvas)).show(ui, |ui| {
        let full = ui.max_rect();
        ui.painter().hline(full.x_range(), full.bottom() - 0.5, Stroke::new(1.0, t.separator));
        let left_edge = full.left() + if cap_l > 0.0 { cap_l + 4.0 } else { 16.0_f32.max(mac_inset) };
        let right_edge = full.right() - if cap_r > 0.0 { cap_r + 8.0 } else { 16.0 };
        let row = |l: f32, r: f32| Rect::from_x_y_ranges(l..=r.max(l), full.y_range());

        // Left: ☰, app name, a divider and where we are ("Home", painted: the rail has the button).
        let mut left = ui.new_child(egui::UiBuilder::new().max_rect(row(left_edge, right_edge)).layout(egui::Layout::left_to_right(egui::Align::Center)));
        left.spacing_mut().item_spacing.x = 6.0;
        menu_button(app, &mut left, &mut clicked);
        left.add_space(10.0);
        left.label(egui::RichText::new(APP_NAME).font(theme::semibold(21.0)).color(t.text));
        left.add_space(14.0);
        let (sep, _) = left.allocate_exact_size(vec2(1.0, 28.0), Sense::hover());
        left.painter().vline(sep.center().x, sep.y_range(), Stroke::new(1.0, t.separator));
        left.add_space(14.0);
        let here = left.painter().layout_no_wrap(tl!("Home").to_owned(), theme::medium(15.0), t.text_dim);
        let (r, _) = left.allocate_exact_size(here.size(), Sense::hover());
        left.painter().galley(r.min, here, t.text_dim);
        let left_end = left.min_rect().right();

        // Right: Preferences.
        let mut right = ui.new_child(egui::UiBuilder::new().max_rect(row(left_end + 16.0, right_edge)).layout(egui::Layout::right_to_left(egui::Align::Center)));
        let tip = crate::shortcuts::tip_label(app, "Preferences", PREFS);
        if crate::icons::button_with_icon_size(&mut right, "settings", 38.0, 22.0, false, &tip).clicked() {
            clicked = Some(PREFS.to_string());
        }
        if cap_r > 0.0 {
            right.add_space(8.0);
            let (sep, _) = right.allocate_exact_size(vec2(1.0, 28.0), Sense::hover());
            right.painter().vline(sep.center().x, sep.y_range(), Stroke::new(1.0, t.separator));
        }
        let right_start = right.min_rect().left().min(right_edge);

        // Centre: the command search, centred on the bar when it fits.
        let (lo, hi) = (left_end + 16.0, right_start - 16.0);
        let w = (hi - lo).min(SEARCH_W);
        let mut gaps = vec![(lo, hi)];
        if w >= SEARCH_MIN && full.center().x.is_finite() {
            let x = full.center().x.clamp(lo + w / 2.0, hi - w / 2.0);
            let rect = Rect::from_center_size(pos2(x, full.center().y), vec2(w, 40.0));
            if search_field(app, ui, rect).clicked() {
                app.ui.palette_open = true;
            }
            gaps = vec![(lo, rect.left()), (rect.right(), hi)];
        }
        drag_gaps(ui, &ctx, gaps, row);
    });
    crate::zoom_field::set_title_bar_rect(&ctx, bar.response.rect);
    if app.custom_titlebar {
        crate::titlebar::caption_buttons(app, ui, bar.response.rect);
    }
    if let Some(id) = clicked
        && let Err(e) = crate::menus::invoke(app, &ctx, &id, serde_json::json!({}))
    {
        app.ui.status = e;
        app.ui.status_error = true;
    }
}

/// Only the empty stretches drag the window; a double-click there maximizes.
fn drag_gaps(ui: &mut egui::Ui, ctx: &egui::Context, gaps: Vec<(f32, f32)>, row: impl Fn(f32, f32) -> Rect) {
    let mut widest = Rect::NOTHING;
    for (i, (l, r)) in gaps.into_iter().enumerate() {
        if r - l < 4.0 {
            continue;
        }
        let gap = row(l, r);
        if gap.width() > widest.width() {
            widest = gap;
        }
        let drag = ui.interact(gap, ui.scope_id().with(("home-titledrag", i)), Sense::click_and_drag());
        if drag.drag_started_by(egui::PointerButton::Primary) {
            crate::titlebar::start_os_move(ctx);
        }
        if drag.double_clicked() {
            let max = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!max));
        }
    }
    ctx.data_mut(|d| d.insert_temp(crate::titlebar::modal_drag_area_id(), widest));
}

/// The rounded "Search command Ctrl+K" field; a click opens the command palette.
fn search_field(app: &PhotocraftApp, ui: &mut egui::Ui, rect: Rect) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let resp = ui.interact(rect, ui.scope_id().with("home-search"), Sense::click()).on_hover_cursor(egui::CursorIcon::Text);
    let label = tl!("Search command");
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
    let border = if resp.hovered() || app.ui.palette_open { t.accent_border } else { t.field_border };
    let p = ui.painter();
    p.rect(rect, CornerRadius::same(10), t.canvas, Stroke::new(1.0, border), StrokeKind::Inside);
    crate::icons::paint(ui, Rect::from_center_size(pos2(rect.left() + 24.0, rect.center().y), vec2(18.0, 18.0)), "search", 18.0, t.text);
    let text = match crate::shortcuts::shortcut_label(app, "edit.search") {
        Some(sc) => format!("{label} {sc}"),
        None => label.to_string(),
    };
    let mut job = egui::text::LayoutJob::simple_singleline(text, theme::medium(15.0), t.text);
    job.wrap = egui::text::TextWrapping::truncate_at_width(rect.width() - 56.0);
    let galley = p.layout_job(job);
    p.galley(pos2(rect.left() + 44.0, rect.center().y - galley.size().y / 2.0), galley, t.text);
    resp
}

/// ☰: the whole menu tree (File … Help), as the menu bar has it.
fn menu_button(app: &PhotocraftApp, ui: &mut egui::Ui, clicked: &mut Option<String>) {
    let (rect, resp) = ui.allocate_exact_size(vec2(38.0, 38.0), Sense::click());
    let open = egui::Popup::is_id_open(ui.ctx(), egui::Popup::default_response_id(&resp));
    let tint = crate::icons::button_chrome(ui, rect, open, resp.hovered());
    for dy in [-7.0, 0.0, 7.0] {
        ui.painter().hline(rect.center().x - 10.0..=rect.center().x + 10.0, rect.center().y + dy, Stroke::new(1.8, tint));
    }
    let resp = resp.on_hover_text(tl!("Menu"));
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tl!("Menu")));
    egui::Popup::menu(&resp).show(|ui| {
        // Built only while open; with no document open the tree is cheap to rebuild.
        let items = crate::menus::menu_items(app);
        let lang = crate::i18n::current();
        ui.set_min_width(200.0);
        crate::widgets::style_spectrum_popup_menu(ui);
        for top in TOP_MENUS {
            let mine: Vec<&MenuItem> = items.iter().filter(|i| i.path.first().map(String::as_str) == Some(top)).collect();
            let enabled = mine.iter().any(|c| c.enabled && c.label != "---");
            ui.add_enabled_ui(enabled, |ui| ui.menu_button(crate::i18n::tr(lang, top), |ui| level(ui, &mine, 1, clicked)));
        }
    });
}

/// One menu level: the leaves at `depth` and a submenu for each deeper group, in menu order.
fn level(ui: &mut egui::Ui, items: &[&MenuItem], depth: usize, clicked: &mut Option<String>) {
    let lang = crate::i18n::current();
    let room = (ui.ctx().content_rect().height() - 60.0).max(120.0);
    egui::ScrollArea::vertical().max_height(room).show(ui, |ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
        crate::widgets::style_spectrum_popup_menu(ui);
        let mut subs: Vec<&str> = Vec::new();
        for it in items {
            if it.path.len() == depth {
                if it.label == "---" {
                    ui.separator();
                    continue;
                }
                let mut b = egui::Button::new(crate::i18n::tr_id(lang, &it.id, &it.label));
                if let Some(sc) = &it.shortcut {
                    b = b.shortcut_text(crate::shortcuts::pretty(sc));
                }
                if ui.add_enabled(it.enabled, b).clicked() {
                    *clicked = Some(it.id.clone());
                    ui.close();
                }
            } else if let Some(name) = it.path.get(depth).map(String::as_str)
                && !subs.contains(&name)
            {
                subs.push(name);
                let child: Vec<&MenuItem> = items.iter().copied().filter(|c| c.path.len() > depth && c.path.get(depth).map(String::as_str) == Some(name)).collect();
                let enabled = child.iter().any(|c| c.enabled && c.label != "---");
                ui.add_enabled_ui(enabled, |ui| ui.menu_button(crate::i18n::tr(lang, name), |ui| level(ui, &child, depth + 1, clicked)));
            }
        }
    });
}
