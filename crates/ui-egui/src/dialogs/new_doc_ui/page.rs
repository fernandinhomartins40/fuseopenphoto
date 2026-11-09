//! File › New in the studio shell (`docs/redesign/7-novo-documento.png`): the classic dialog's
//! fields, presets and Create (`file.new` with [`super::command_params`]), laid out over the whole
//! window. Formats on the left, their presets, a preview of the page, the settings on the right,
//! Back / Create at the bottom. Window › Classic Layout keeps the dialog ([`super::body`]).

use egui::{Align, Align2, Color32, CornerRadius, Layout, Rect, Response, RichText, Sense, Stroke, StrokeKind, Ui, UiBuilder, WidgetInfo, pos2, vec2};
use serde_json::{Map, Value, json};

use super::{get_f, get_s};
use crate::PhotocraftApp;
use crate::theme::{self, Tokens};
use crate::{icons, widgets};

/// The bottom bar's height.
const BOTTOM_H: f32 = 76.0;
/// The format column: category key (as in [`super::CATEGORIES`], then "Saved") and its icon
/// (empty: the bookmark drawn by [`bookmark`]).
const FORMATS: [(&str, &str); 8] = [
    ("Recent", "clock"),
    ("Photo", "image"),
    ("Print", "file"),
    ("Art & Illustration", "brush"),
    ("Web", "monitor"),
    ("Mobile", "rectangle-vertical"),
    ("Film & Video", "play"),
    ("Saved", ""),
];
/// Translation context of the page's short labels ("Print" is "Impressão" here, not "Imprimir").
const CTX: &str = "newDocument";
const PREFERENCES: &str = "edit.preferences.general";

fn label(s: &str) -> &str {
    crate::i18n::tr_ctx(crate::i18n::current(), CTX, s)
}

/// Draw the page for the dialog `id` and return what it answered (`Some(true)` Create,
/// `Some(false)` Back) with the screen it covers. `tab` is this frame's Tab step for its fields.
pub(crate) fn show(app: &mut PhotocraftApp, ctx: &egui::Context, f: &mut Map<String, Value>, id: egui::Id, interactive: bool, tab: i32) -> (Option<bool>, Rect) {
    let screen = ctx.content_rect();
    let mut outcome = None;
    egui::Area::new(id).order(egui::Order::Foreground).fixed_pos(screen.min).constrain(false).movable(false).show(ctx, |ui| {
        let t = Tokens::get(ctx);
        ui.painter().rect_filled(screen, 0.0, t.canvas);
        // The whole window is the page's: nothing underneath takes the pointer.
        ui.allocate_rect(screen, Sense::hover());
        ui.style_mut().interaction.selectable_labels = false;
        let bar_h = crate::titlebar::bar_height(app, ctx, crate::studio::style::TOP_BAR_H);
        let top = Rect::from_min_size(screen.min, vec2(screen.width(), bar_h));
        let bottom = Rect::from_min_max(pos2(screen.left(), (screen.bottom() - BOTTOM_H).max(top.bottom())), screen.max);
        let body = Rect::from_min_max(pos2(screen.left(), top.bottom()), pos2(screen.right(), bottom.top()));
        let mut back = header(app, ui, top);
        crate::field_tab::begin(ctx);
        let mut create = columns(app, ui, body, f);
        crate::field_tab::end(ctx, tab);
        let (b, c) = footer(ui, bottom, f);
        back |= b;
        create |= c;
        outcome = decide(ui, interactive, back, create);
    });
    (outcome, screen)
}

/// Create or Back from the buttons, else ↩ / Esc on the topmost dialog with no popup open.
fn decide(ui: &mut Ui, interactive: bool, back: bool, create: bool) -> Option<bool> {
    if create {
        return Some(true);
    }
    if back {
        return Some(false);
    }
    if !interactive || egui::Popup::is_any_open(ui.ctx()) {
        return None;
    }
    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        Some(true)
    } else if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        Some(false)
    } else {
        None
    }
}

/// ☰, app name, breadcrumb, command search, preferences and the window's caption buttons; the
/// empty stretches drag the window. Returns whether ☰ or "Home" asked to go back.
fn header(app: &mut PhotocraftApp, ui: &mut Ui, bar: Rect) -> bool {
    let t = Tokens::get(ui.ctx());
    let ctx = ui.ctx().clone();
    ui.painter().hline(bar.x_range(), bar.bottom() - 0.5, Stroke::new(1.0, t.separator));
    // First, so the controls drawn over it take their own clicks.
    let drag = ui.interact(bar, ui.scope_id().with("new-doc-page-drag"), Sense::click_and_drag());
    if drag.drag_started_by(egui::PointerButton::Primary) {
        crate::titlebar::start_os_move(&ctx);
    }
    if drag.double_clicked() {
        let max = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!max));
    }
    let (cap_l, cap_r) = crate::titlebar::reserved(app, &ctx);
    let mac_inset = if cfg!(target_os = "macos") && app.integrated_titlebar { 78.0 } else { 0.0 };
    let left_edge = bar.left() + if cap_l > 0.0 { cap_l + 4.0 } else { 12.0_f32.max(mac_inset) };
    let right_edge = bar.right() - if cap_r > 0.0 { cap_r + 4.0 } else { 12.0 };
    let row = |l: f32, r: f32| Rect::from_x_y_ranges(l..=r.max(l), bar.y_range());
    let mut left = ui.new_child(UiBuilder::new().max_rect(row(left_edge, right_edge)).layout(Layout::left_to_right(Align::Center)));
    left.spacing_mut().item_spacing.x = 8.0;
    let mut back = menu_button(&mut left).clicked();
    left.label(RichText::new(crate::studio::style::APP_NAME).font(theme::semibold(18.0)).color(t.text));
    left.add_space(6.0);
    vsep(&mut left, 22.0);
    left.add_space(6.0);
    back |= text_link(&mut left, tl!("Home")).clicked();
    left.label(RichText::new("/").font(theme::medium(15.0)).color(t.text_faint));
    left.label(RichText::new(tl!("New Document")).font(theme::medium(15.0)).color(t.text));
    let left_end = left.min_rect().right() + 16.0;
    let mut right = ui.new_child(UiBuilder::new().max_rect(row(left_end, right_edge)).layout(Layout::right_to_left(Align::Center)));
    right.spacing_mut().item_spacing.x = 10.0;
    if cap_r > 0.0 {
        vsep(&mut right, 22.0);
    }
    let prefs = icons::button(&mut right, "settings", 34.0, false, "Preferences").clicked();
    let (lo, hi) = (left_end, right.min_rect().left() - 16.0);
    let w = 430.0_f32.min(hi - lo);
    if w >= 140.0 {
        let x = bar.center().x.clamp(lo + w / 2.0, hi - w / 2.0);
        if search_field(app, ui, Rect::from_center_size(pos2(x, bar.center().y), vec2(w, 36.0))).clicked() {
            app.ui.palette_open = true;
        }
    }
    if app.custom_titlebar {
        crate::titlebar::caption_buttons(app, ui, bar);
    }
    if prefs && let Err(e) = crate::menus::invoke(app, &ctx, PREFERENCES, json!({})) {
        app.ui.status = e;
        app.ui.status_error = true;
    }
    back
}

/// ☰: back to where New was opened from (the studio's menu is in its own top bar).
fn menu_button(ui: &mut Ui) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(34.0, 34.0), Sense::click());
    let resp = resp.on_hover_text(tl!("Back"));
    resp.widget_info(|| WidgetInfo::labeled(egui::Role::Button, true, tl!("Menu")));
    let tint = icons::button_chrome(ui, rect, false, resp.hovered());
    for dy in [-6.0, 0.0, 6.0] {
        ui.painter().hline(rect.center().x - 9.0..=rect.center().x + 9.0, rect.center().y + dy, Stroke::new(1.8, tint));
    }
    resp
}

fn text_link(ui: &mut Ui, text: &str) -> Response {
    let t = Tokens::get(ui.ctx());
    let r = ui.add(egui::Label::new(RichText::new(text).font(theme::medium(15.0)).color(t.text_dim)).sense(Sense::click()));
    r.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// The command search, as on the studio top bar: a click opens the command palette.
fn search_field(app: &PhotocraftApp, ui: &mut Ui, rect: Rect) -> Response {
    let t = Tokens::get(ui.ctx());
    let resp = ui.interact(rect, ui.scope_id().with("new-doc-page-search"), Sense::click()).on_hover_cursor(egui::CursorIcon::Text);
    let text = tl!("Search command");
    resp.widget_info(|| WidgetInfo::labeled(egui::Role::Button, true, text));
    let border = if resp.hovered() { t.accent_border } else { t.field_border };
    let p = ui.painter();
    p.rect(rect, CornerRadius::same(10), t.field, Stroke::new(1.0, border), StrokeKind::Inside);
    icons::paint(ui, Rect::from_center_size(pos2(rect.left() + 20.0, rect.center().y), vec2(16.0, 16.0)), "search", 16.0, t.text_dim);
    let text = match crate::shortcuts::shortcut_label(app, "edit.search") {
        Some(sc) => format!("{text} {sc}"),
        None => text.to_string(),
    };
    let mut job = egui::text::LayoutJob::simple_singleline(text, theme::medium(14.0), t.text);
    job.wrap = egui::text::TextWrapping::truncate_at_width(rect.width() - 48.0);
    let galley = p.layout_job(job);
    p.galley(pos2(rect.left() + 38.0, rect.center().y - galley.size().y / 2.0), galley, t.text);
    resp
}

fn vsep(ui: &mut Ui, h: f32) {
    let t = Tokens::get(ui.ctx());
    let (r, _) = ui.allocate_exact_size(vec2(1.0, h), Sense::hover());
    ui.painter().vline(r.center().x, r.y_range(), Stroke::new(1.0, t.separator));
}

/// The four columns; returns whether a preset's second click asked to create.
fn columns(app: &mut PhotocraftApp, ui: &mut Ui, body: Rect, f: &mut Map<String, Value>) -> bool {
    let t = Tokens::get(ui.ctx());
    let w = body.width();
    let formats_w = (w * 0.15).clamp(180.0, 240.0);
    let list_w = (w * 0.135).clamp(170.0, 215.0);
    let right_w = (w * 0.25).clamp(290.0, 410.0);
    let x1 = body.left() + formats_w;
    let x2 = x1 + list_w;
    let x3 = (body.right() - right_w).max(x2 + 120.0);
    for x in [x1, x2, x3] {
        ui.painter().vline(x, body.y_range(), Stroke::new(1.0, t.separator));
    }
    let col = |l: f32, r: f32| Rect::from_x_y_ranges(l..=r, body.y_range());
    formats(ui, col(body.left(), x1), f);
    let create = presets(app, ui, col(x1, x2), f);
    preview(app, ui, col(x2, x3), f);
    settings(app, ui, col(x3, body.right()), f);
    create
}

/// A column's content area, `margin` inside `r`.
fn column(ui: &mut Ui, r: Rect, margin: egui::Vec2) -> Ui {
    ui.new_child(UiBuilder::new().max_rect(r.shrink2(margin)).layout(Layout::top_down(Align::Min)))
}

fn heading(ui: &mut Ui, text: &str) {
    let t = Tokens::get(ui.ctx());
    ui.add(egui::Label::new(RichText::new(text).font(theme::semibold(19.0)).color(t.text)).truncate());
}

/// Column 1, "Choose a format": the preset categories.
fn formats(ui: &mut Ui, r: Rect, f: &mut Map<String, Value>) {
    let mut c = column(ui, r, vec2(12.0, 22.0));
    c.horizontal(|ui| {
        ui.add_space(8.0);
        heading(ui, tl!("Choose a format"));
    });
    c.add_space(16.0);
    let cat = get_s(f, "__category", "Recent");
    egui::ScrollArea::vertical().id_salt("new-doc-page-formats").show(&mut c, |ui| {
        for (key, icon) in FORMATS {
            if format_row(ui, key, icon, cat == key).clicked() {
                f.insert("__category".into(), json!(key));
            }
            ui.add_space(4.0);
        }
    });
}

fn format_row(ui: &mut Ui, key: &str, icon: &str, on: bool) -> Response {
    let t = Tokens::get(ui.ctx());
    let text = label(key);
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 48.0), Sense::click());
    resp.widget_info(|| WidgetInfo::selected(egui::Role::Button, true, on, text));
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    let (fill, ink) = match (on, resp.hovered()) {
        (true, _) => (t.primary_bg, t.primary_text),
        (false, true) => (t.hover, t.text),
        _ => (Color32::TRANSPARENT, t.text),
    };
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(10), fill);
    let icon_rect = Rect::from_center_size(pos2(rect.left() + 26.0, rect.center().y), vec2(22.0, 22.0));
    if icon.is_empty() {
        bookmark(p, icon_rect, Stroke::new(1.8, ink));
    } else {
        icons::paint(ui, icon_rect, icon, 22.0, ink);
    }
    let font = if on { theme::semibold(16.0) } else { theme::medium(16.0) };
    let mut job = egui::text::LayoutJob::simple_singleline(text.to_owned(), font, ink);
    job.wrap = egui::text::TextWrapping::truncate_at_width(rect.width() - 60.0);
    let galley = p.layout_job(job);
    p.galley(pos2(rect.left() + 52.0, rect.center().y - galley.size().y / 2.0), galley, ink);
    resp
}

/// A bookmark outline in `r` (Saved, Save Preset…).
fn bookmark(p: &egui::Painter, r: Rect, stroke: Stroke) {
    let r = r.shrink2(vec2(r.width() * 0.2, r.height() * 0.08));
    let pts = vec![r.left_top(), r.right_top(), r.right_bottom(), pos2(r.center().x, r.bottom() - r.height() * 0.28), r.left_bottom(), r.left_top()];
    p.line(pts, stroke);
}

/// Column 2: the category's presets (or the saved ones); returns whether a second click on one
/// asked to create.
fn presets(app: &mut PhotocraftApp, ui: &mut Ui, r: Rect, f: &mut Map<String, Value>) -> bool {
    let mut c = column(ui, r, vec2(12.0, 22.0));
    let cat = get_s(f, "__category", "Recent");
    if cat == "Saved" {
        return saved(app, &mut c, f);
    }
    let units = app.session.prefs().units_and_rulers.clone();
    let cards = super::category_cards(app, f, &cat, &units);
    count_heading(&mut c, label(&cat), cards.len());
    let chosen = get_s(f, "__preset", "");
    let mut create = false;
    egui::ScrollArea::vertical().id_salt("new-doc-page-presets").show(&mut c, |ui| {
        for card in &cards {
            let size = card.size(&units);
            let selected = card.selected(f, &chosen);
            let resp = preset_card(ui, tl!(card.title()), &card.size_line(&units, f), (size.0, size.1), selected);
            if resp.clicked() {
                card.apply(f, &units);
                // Touch-tolerant double-tap (finger jitter > egui's 6 px); see `widgets::second_click`.
                create |= widgets::second_click(ui, &resp);
            }
            ui.add_space(10.0);
        }
    });
    create
}

/// "Recent (5)".
fn count_heading(ui: &mut Ui, name: &str, n: usize) {
    let t = Tokens::get(ui.ctx());
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.label(RichText::new(name).font(theme::semibold(19.0)).color(t.text));
        ui.label(RichText::new(format!("({n})")).font(theme::medium(18.0)).color(t.text_dim));
    });
    ui.add_space(16.0);
}

/// A preset card: page outline at the preset's aspect, bold name, size line; amber when selected.
fn preset_card(ui: &mut Ui, title: &str, size: &str, (w, h): (u32, u32), selected: bool) -> Response {
    let t = Tokens::get(ui.ctx());
    let width = ui.available_width();
    let text = |s: &str, font: egui::FontId, color: Color32| {
        let mut job = egui::text::LayoutJob::simple(s.replace(" x ", " × "), font, color, width - 20.0);
        job.wrap.max_rows = 2;
        job.halign = Align::Center;
        ui.painter().layout_job(job)
    };
    let title_g = text(title, theme::semibold(14.5), t.text);
    let size_g = text(size, egui::FontId::proportional(12.5), t.text_dim);
    let height = 62.0 + title_g.size().y + 6.0 + size_g.size().y + 14.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    resp.widget_info(|| WidgetInfo::selected(egui::Role::Button, true, selected, title));
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    let p = ui.painter();
    let fill = if resp.hovered() { t.hover } else { t.card };
    let stroke = if selected { Stroke::new(2.0, t.primary_bg) } else { Stroke::new(1.0, t.card_border) };
    p.rect(rect, CornerRadius::same(10), fill, stroke, StrokeKind::Inside);
    let s = 32.0 / (w.max(h).max(1) as f32);
    let page = Rect::from_center_size(pos2(rect.center().x, rect.top() + 34.0), vec2((w as f32 * s).max(4.0), (h as f32 * s).max(4.0)));
    p.rect_stroke(page, 2.0, Stroke::new(1.6, t.text), StrokeKind::Inside);
    let title_y = rect.top() + 62.0;
    let size_y = title_y + title_g.size().y + 6.0;
    p.galley(pos2(rect.center().x, title_y), title_g, t.text);
    p.galley(pos2(rect.center().x, size_y), size_g, t.text_dim);
    resp
}

/// The Saved category: the user's presets, each with a Delete button.
fn saved(app: &mut PhotocraftApp, ui: &mut Ui, f: &mut Map<String, Value>) -> bool {
    let presets = app.session.presets.documents.clone();
    count_heading(ui, label("Saved"), presets.len());
    if presets.is_empty() {
        ui.label(tl!("No saved presets yet."));
        return false;
    }
    let mut create = false;
    egui::ScrollArea::vertical().id_salt("new-doc-page-saved").show(ui, |ui| {
        for preset in &presets {
            ui.push_id(&preset.name, |ui| {
                let s = &preset.settings;
                let line = format!("{} × {} px @ {} ppi", s.width, s.height, widgets::fmt_num(f64::from(s.resolution)));
                let selected = get_s(f, "__savedPreset", "") == preset.name;
                let resp = preset_card(ui, &preset.name, &line, (s.width, s.height), selected);
                let bin = Rect::from_min_size(resp.rect.right_top() + vec2(-34.0, 6.0), vec2(28.0, 28.0));
                let delete = ui.scope_builder(UiBuilder::new().max_rect(bin), |ui| icons::button(ui, "trash", 28.0, false, "Delete")).inner;
                if delete.clicked() {
                    delete_preset(app, f, &preset.name, selected);
                } else if resp.clicked() {
                    super::apply_saved_preset(f, preset);
                    create |= widgets::second_click(ui, &resp);
                }
                ui.add_space(10.0);
            });
        }
    });
    if let Some(error) = f.get("__presetError").and_then(Value::as_str) {
        ui.add(egui::Label::new(error).wrap().selectable(true));
    }
    create
}

fn delete_preset(app: &mut PhotocraftApp, f: &mut Map<String, Value>, name: &str, selected: bool) {
    match app.run("document.presets.delete", json!({ "name": name })) {
        Ok(_) => {
            f.remove("__presetError");
            if selected {
                f.remove("__savedPreset");
            }
        }
        Err(e) => {
            f.insert("__presetError".into(), json!(e.to_string()));
        }
    }
}

/// "1913 × 1029 px · RGB · 8 bits · 72 ppi".
pub(crate) fn summary(f: &Map<String, Value>) -> String {
    let (w, h) = (get_f(f, "width", 1920.0).round() as u32, get_f(f, "height", 1080.0).round() as u32);
    let mode = match get_s(f, "mode", "rgb").as_str() {
        "gray" | "grayscale" => tl!("Grayscale"),
        "cmyk" => "CMYK",
        "lab" => "Lab",
        _ => "RGB",
    };
    let depth = f.get("depth").and_then(Value::as_u64).unwrap_or(8);
    format!("{w} × {h} px · {mode} · {depth} bits · {} ppi", widgets::fmt_num(f64::from(get_f(f, "resolution", 72.0))))
}

/// The centre: title, subtitle and the page preview with its size rulers.
fn preview(app: &PhotocraftApp, ui: &mut Ui, r: Rect, f: &Map<String, Value>) {
    let t = Tokens::get(ui.ctx());
    let mut c = column(ui, r, vec2(28.0, 22.0));
    c.label(RichText::new(tl!("Prepare your next creation.")).font(theme::semibold(36.0)).color(t.text));
    c.add_space(2.0);
    c.label(RichText::new(tl!("Set the format, adjust the settings and create your document.")).font(theme::medium(17.0)).color(t.text_dim));
    c.add_space(18.0);
    widgets::hairline(&mut c);
    c.add_space(18.0);
    c.label(RichText::new(tl!("Document preview")).font(theme::semibold(17.0)).color(t.text));
    let area = c.available_rect_before_wrap();
    // Room for the rulers above and left of the page, and the summary under it.
    let stage = Rect::from_min_max(area.min + vec2(44.0, 56.0), pos2(area.right() - 8.0, area.bottom() - 56.0));
    let p = c.painter();
    p.text(pos2(area.center().x, area.bottom() - 18.0), Align2::CENTER_CENTER, summary(f), theme::medium(15.5), t.text);
    if stage.width() < 24.0 || stage.height() < 24.0 {
        return;
    }
    let (w, h) = (get_f(f, "width", 1920.0).max(1.0), get_f(f, "height", 1080.0).max(1.0));
    let s = (stage.width() / w).min(stage.height() / h);
    let page = Rect::from_center_size(stage.center(), vec2((w * s).max(2.0), (h * s).max(2.0)));
    match super::background_rgb(app, f) {
        Some(rgb) => {
            p.rect_filled(page, 0.0, super::rgb32(rgb));
        }
        None => widgets::checker(p, page, 8.0),
    }
    p.rect_stroke(page, 0.0, Stroke::new(1.0, t.card_border), StrokeKind::Outside);
    let ink = Stroke::new(1.4, t.text_dim);
    ruler(p, page.left()..=page.right(), page.top() - 24.0, &format!("{} px", w.round() as u32), false, ink, t.text);
    ruler(p, page.top()..=page.bottom(), page.left() - 24.0, &format!("{} px", h.round() as u32), true, ink, t.text);
}

/// A dimension line with end ticks along `span` at `at` (y, or x when `vertical`), labelled in its
/// middle (the vertical label reads bottom to top).
fn ruler(p: &egui::Painter, span: std::ops::RangeInclusive<f32>, at: f32, text: &str, vertical: bool, ink: Stroke, color: Color32) {
    let (a, b) = (*span.start(), *span.end());
    let galley = p.layout_no_wrap(text.to_owned(), theme::medium(15.0), color);
    let mid = (a + b) / 2.0;
    let half = galley.size().x / 2.0 + 10.0;
    let pt = |along: f32, across: f32| if vertical { pos2(across, along) } else { pos2(along, across) };
    for (from, to) in [(a, (mid - half).max(a)), ((mid + half).min(b), b)] {
        if to > from {
            p.line_segment([pt(from, at), pt(to, at)], ink);
        }
    }
    for end in [a, b] {
        p.line_segment([pt(end, at - 6.0), pt(end, at + 6.0)], ink);
    }
    let gs = galley.size();
    if vertical {
        let pos = pos2(at - gs.y / 2.0, mid + gs.x / 2.0);
        p.add(egui::epaint::TextShape::new(pos, galley, color).with_angle(-std::f32::consts::FRAC_PI_2));
    } else {
        p.galley(pos2(mid - gs.x / 2.0, at - gs.y / 2.0), galley, color);
    }
}

fn field_label(ui: &mut Ui, text: &str) {
    let t = Tokens::get(ui.ctx());
    ui.label(RichText::new(text).size(13.5).color(t.text));
}

fn separator(ui: &mut Ui) {
    ui.add_space(10.0);
    widgets::hairline(ui);
    ui.add_space(10.0);
}

/// The right column, "Configure document": every setting of the classic dialog.
fn settings(app: &mut PhotocraftApp, ui: &mut Ui, r: Rect, f: &mut Map<String, Value>) {
    let mut c = column(ui, r, vec2(24.0, 22.0));
    heading(&mut c, tl!("Configure document"));
    c.add_space(16.0);
    egui::ScrollArea::vertical().id_salt("new-doc-page-settings").show(&mut c, |ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        let w = (ui.available_width() - 6.0).max(120.0);
        field_label(ui, tl!("Name"));
        super::name_field(ui, f, w - 12.0);
        separator(ui);
        ui.label(RichText::new(tl!("Dimensions")).font(theme::semibold(14.5)).color(Tokens::get(ui.ctx()).text));
        dimensions(ui, f, w);
        ui.horizontal(|ui| {
            ui.add_sized(vec2(w * 0.3, 20.0), egui::Label::new(RichText::new(tl!("Units")).size(13.5)));
            super::units_dropdown(ui, f, w * 0.7 - 8.0);
        });
        field_label(ui, crate::i18n::tr_ctx(crate::i18n::current(), "document", "Orientation"));
        orientation(ui, f, w);
        field_label(ui, tl!("Resolution"));
        ui.horizontal(|ui| {
            super::resolution_field(ui, f, w * 0.48);
            super::resolution_unit_dropdown(ui, f, w * 0.52 - 8.0);
        });
        field_label(ui, label("Color"));
        ui.horizontal(|ui| {
            super::mode_dropdown(ui, f, w * 0.5 - 4.0);
            super::depth_dropdown(ui, f, w * 0.5 - 4.0);
        });
        field_label(ui, label("Background"));
        ui.horizontal(|ui| {
            super::background_row(app, ui, f, w * 0.58);
            transparent_box(ui, f);
        });
        separator(ui);
        advanced(app, ui, f, w);
        separator(ui);
        save_preset(app, ui, f);
    });
}

/// Width [link] Height; with the link on, editing one keeps the aspect ratio.
fn dimensions(ui: &mut Ui, f: &mut Map<String, Value>, w: f32) {
    let link_w = 34.0;
    let field_w = ((w - link_w) / 2.0 - 10.0).max(50.0);
    ui.horizontal(|ui| {
        ui.allocate_ui(vec2(field_w + link_w + 12.0, 18.0), |ui| {
            ui.set_min_width(field_w + link_w + 12.0);
            ui.label(RichText::new(tl!("Width")).size(13.5));
        });
        ui.label(RichText::new(tl!("Height")).size(13.5));
    });
    let linked = f.get("__linked").and_then(Value::as_bool).unwrap_or(false);
    let (w0, h0) = (get_f(f, "width", 1920.0), get_f(f, "height", 1080.0));
    ui.horizontal(|ui| {
        // The row's height from the start: otherwise the fields centre on different heights.
        ui.set_min_height(link_w);
        if super::size_field(ui, f, "width", field_w) && linked {
            keep_aspect(f, "height", h0 * get_f(f, "width", w0) / w0.max(1.0));
        }
        let tip = if linked { "Unlink width and height" } else { "Link width and height" };
        if icons::button(ui, if linked { "link" } else { "unlink" }, link_w, linked, tip).clicked() {
            f.insert("__linked".into(), json!(!linked));
        }
        if super::size_field(ui, f, "height", field_w) && linked {
            keep_aspect(f, "width", w0 * get_f(f, "height", h0) / h0.max(1.0));
        }
    });
}

/// Set the other side (`key`) to `px` for a linked Width/Height edit.
fn keep_aspect(f: &mut Map<String, Value>, key: &str, px: f32) {
    f.insert(key.into(), super::px_value(px));
    f.remove(&super::typed_key(key));
}

/// Portrait / Landscape (amber when the size has it) and the Artboards switch.
fn orientation(ui: &mut Ui, f: &mut Map<String, Value>, w: f32) {
    let t = Tokens::get(ui.ctx());
    let (pw, ph) = (get_f(f, "width", 1920.0), get_f(f, "height", 1080.0));
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (icon, portrait, tip) in [("rectangle-vertical", true, tl!("Portrait")), ("rectangle-horizontal", false, tl!("Landscape"))] {
            let on = (ph > pw) == portrait;
            let (rect, resp) = ui.allocate_exact_size(vec2(52.0, 38.0), Sense::click());
            resp.widget_info(|| WidgetInfo::selected(egui::Role::Button, true, on, tip));
            let resp = resp.on_hover_text(tip).on_hover_cursor(egui::CursorIcon::PointingHand);
            let stroke = if on { Stroke::new(2.0, t.primary_bg) } else { Stroke::new(1.0, t.field_border) };
            ui.painter().rect(rect, CornerRadius::same(8), if resp.hovered() { t.hover } else { t.field }, stroke, StrokeKind::Inside);
            icons::paint(ui, rect, icon, 22.0, if on { t.primary_bg } else { t.text });
            if resp.clicked() {
                super::set_orientation(f, portrait);
            }
        }
        ui.add_space(8.0);
        vsep(ui, 34.0);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.set_max_width(w);
            // Under Orientation, as in Photoshop: the document starts as one artboard (#9086).
            let mut artboard = f.get("artboard").and_then(Value::as_bool).unwrap_or(false);
            let resp = widgets::toggle(ui, &mut artboard, "");
            resp.widget_info(|| WidgetInfo::selected(egui::Role::CheckBox, true, artboard, tl!("Artboards")));
            if resp.changed() {
                f.insert("artboard".into(), json!(artboard));
            }
            ui.label(RichText::new(tl!("Artboards")).size(13.5).color(t.text));
        });
    });
}

/// Transparent: the background menu's Transparent entry as a checkbox (off goes back to white).
fn transparent_box(ui: &mut Ui, f: &mut Map<String, Value>) {
    let mut on = get_s(f, "background", "white") == "transparent";
    if widgets::checkbox(ui, &mut on, tl!("Transparent")).changed() {
        f.insert("background".into(), json!(if on { "transparent" } else { "white" }));
        f.remove("backgroundColor");
        f.remove("__savedPreset");
    }
}

/// "Advanced Options", open by default: the Color Profile.
fn advanced(app: &PhotocraftApp, ui: &mut Ui, f: &mut Map<String, Value>, w: f32) {
    let t = Tokens::get(ui.ctx());
    let open = f.get("__advanced").and_then(Value::as_bool).unwrap_or(true);
    let resp = ui
        .horizontal(|ui| {
            let (chevron, _) = ui.allocate_exact_size(vec2(18.0, 22.0), Sense::hover());
            icons::paint(ui, chevron, if open { "chevron-down" } else { "chevron-right" }, 16.0, t.text);
            ui.label(RichText::new(tl!("Advanced Options")).font(theme::semibold(16.0)).color(t.text));
        })
        .response
        .interact(Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    resp.widget_info(|| WidgetInfo::selected(egui::Role::Button, true, open, tl!("Advanced Options")));
    if resp.clicked() {
        f.insert("__advanced".into(), json!(!open));
    }
    if open {
        field_label(ui, tl!("Color Profile"));
        super::profile_dropdown(app, ui, f, w);
    }
}

/// "Save Preset…" and, once pressed, the classic dialog's naming form.
fn save_preset(app: &mut PhotocraftApp, ui: &mut Ui, f: &mut Map<String, Value>) {
    if f.get("__savingPreset").and_then(Value::as_bool) == Some(true) {
        super::save_preset(app, ui, f);
        return;
    }
    let t = Tokens::get(ui.ctx());
    let text = tl!("Save Preset…");
    let galley = ui.painter().layout_no_wrap(text.to_owned(), theme::medium(16.0), t.text);
    let (rect, resp) = ui.allocate_exact_size(vec2(galley.size().x + 44.0, 36.0), Sense::click());
    resp.widget_info(|| WidgetInfo::labeled(egui::Role::Button, true, text));
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    if resp.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(8), t.hover);
    }
    bookmark(ui.painter(), Rect::from_center_size(pos2(rect.left() + 16.0, rect.center().y), vec2(22.0, 22.0)), Stroke::new(1.8, t.text));
    ui.painter().galley(pos2(rect.left() + 36.0, rect.center().y - galley.size().y / 2.0), galley, t.text);
    if resp.clicked() {
        f.insert("__savingPreset".into(), json!(true));
        f.remove("__presetError");
    }
    if let Some(error) = f.get("__presetError").and_then(Value::as_str) {
        ui.add(egui::Label::new(error).wrap().selectable(true));
    }
}

/// Back (outlined), the summary and Create document (amber). Returns (back, create).
fn footer(ui: &mut Ui, r: Rect, f: &Map<String, Value>) -> (bool, bool) {
    let t = Tokens::get(ui.ctx());
    ui.painter().hline(r.x_range(), r.top() + 0.5, Stroke::new(1.0, t.separator));
    let mut row = ui.new_child(UiBuilder::new().max_rect(r.shrink2(vec2(28.0, 0.0))).layout(Layout::left_to_right(Align::Center)));
    let back = bar_button(&mut row, tl!("Back"), false).clicked();
    let mut right = row.new_child(UiBuilder::new().max_rect(row.available_rect_before_wrap()).layout(Layout::right_to_left(Align::Center)));
    right.spacing_mut().item_spacing.x = 16.0;
    let create = bar_button(&mut right, tl!("Create document"), true).clicked();
    vsep(&mut right, 46.0);
    right.add(egui::Label::new(RichText::new(summary(f)).font(theme::medium(15.5)).color(t.text)).truncate());
    (back, create)
}

/// The bottom bar's buttons: "← Back" outlined, or the amber primary "Create document →".
fn bar_button(ui: &mut Ui, text: &str, primary: bool) -> Response {
    let t = Tokens::get(ui.ctx());
    let (font, ink) = if primary { (theme::semibold(17.5), t.primary_text) } else { (theme::medium(16.0), t.text) };
    let galley = ui.painter().layout_no_wrap(text.to_owned(), font, ink);
    let size = vec2(galley.size().x + if primary { 82.0 } else { 74.0 }, if primary { 50.0 } else { 44.0 });
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    resp.widget_info(|| WidgetInfo::labeled(egui::Role::Button, true, text));
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    let p = ui.painter();
    let pressed = resp.is_pointer_button_down_on();
    if primary {
        let fill = match (pressed, resp.hovered()) {
            (true, _) => t.primary_bg.gamma_multiply(0.85),
            (false, true) => t.primary_bg.gamma_multiply(1.08),
            _ => t.primary_bg,
        };
        p.rect_filled(rect, CornerRadius::same(10), fill);
        p.galley(pos2(rect.left() + 34.0, rect.center().y - galley.size().y / 2.0), galley, ink);
        arrow(p, pos2(rect.right() - 30.0, rect.center().y), 1.0, Stroke::new(2.0, ink));
    } else {
        p.rect(rect, CornerRadius::same(8), if resp.hovered() { t.hover } else { Color32::TRANSPARENT }, Stroke::new(1.2, t.field_border), StrokeKind::Inside);
        arrow(p, pos2(rect.left() + 30.0, rect.center().y), -1.0, Stroke::new(1.8, ink));
        p.galley(pos2(rect.left() + 52.0, rect.center().y - galley.size().y / 2.0), galley, ink);
    }
    resp
}

/// An arrow centred on `c`, pointing right (`dir` 1) or left (-1).
fn arrow(p: &egui::Painter, c: egui::Pos2, dir: f32, stroke: Stroke) {
    let tip = c + vec2(8.0 * dir, 0.0);
    p.line_segment([c - vec2(8.0 * dir, 0.0), tip], stroke);
    p.line(vec![tip + vec2(-6.0 * dir, -6.0), tip, tip + vec2(-6.0 * dir, 6.0)], stroke);
}

#[cfg(test)]
mod tests;
