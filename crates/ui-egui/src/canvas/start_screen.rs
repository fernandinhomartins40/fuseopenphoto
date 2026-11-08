use super::tabs::open_on_double_click;
use super::{PhotocraftApp, Rect, Sense};
use crate::studio::style::RADIUS;
use crate::theme::{Tokens, medium, semibold};
use egui::{Stroke, StrokeKind, pos2, vec2};

/// What a click on Home asks for, run once the screen is drawn.
enum Act {
    New,
    Open,
    Preset(crate::new_doc_ui::Preset),
    Clipboard,
    Prefs,
    Back,
}

const CLIPBOARD: &str = "file.newFromClipboard";

pub(super) fn start_screen(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let area = ui.available_rect_before_wrap();
    let t = Tokens::get(ui.ctx());
    ui.painter().rect_filled(area, 0.0, t.chrome);
    open_on_double_click(app, ui, area);

    // File › Open Recent, newest first (on the web there are no paths to reopen).
    let show_recent = app.session.prefs().interface.show_home_recent_files;
    let recent: Vec<String> =
        if cfg!(target_arch = "wasm32") || !show_recent { Vec::new() } else { app.ui.recent_files.iter().take(HOME_RECENT).cloned().collect() };

    let mut act = None;
    if area.width() < 640.0 {
        let col = area.shrink2(vec2(16.0, 20.0));
        ui.scope_builder(egui::UiBuilder::new().max_rect(col), |ui| {
            egui::ScrollArea::vertical().id_salt("home-narrow").auto_shrink([false, false]).show(ui, |ui| {
                render_main(app, ui, &recent, col.width(), col.height(), &mut act);
                ui.add_space(28.0);
                render_formats(app, ui, col.width(), 0.0, &mut act);
            });
        });
    } else {
        wide(app, ui, area, &recent, &mut act);
    }
    if let Some(a) = act {
        run(app, ui.ctx(), a);
    }
}

/// Rail on the left; the hero, drop zone and recents beside the formats; the clipboard strip below.
fn wide(app: &mut PhotocraftApp, ui: &mut egui::Ui, area: Rect, recent: &[String], act: &mut Option<Act>) {
    let t = Tokens::get(ui.ctx());
    let rail = Rect::from_min_max(area.min, pos2(area.left() + if area.width() >= 1200.0 { 240.0 } else { 200.0 }, area.bottom()));
    ui.painter().vline(rail.right(), rail.y_range(), Stroke::new(1.0, t.separator));
    render_rail(app, ui, rail, act);

    let main = Rect::from_min_max(pos2(rail.right(), area.top()), area.max);
    let pad = (main.width() * 0.028).clamp(24.0, 40.0);
    let strip = Rect::from_min_max(pos2(main.left() + pad, main.bottom() - 84.0), pos2(main.right() - pad, main.bottom()));
    ui.painter().hline(strip.x_range(), strip.top(), Stroke::new(1.0, t.separator));
    render_strip(app, ui, strip, act);

    let body = Rect::from_min_max(pos2(main.left() + pad, main.top() + 8.0), pos2(main.right() - pad, strip.top() - 4.0));
    if body.width() < 820.0 {
        // Too narrow for two columns: the formats follow the drop zone.
        ui.scope_builder(egui::UiBuilder::new().max_rect(body), |ui| {
            egui::ScrollArea::vertical().id_salt("home-stack").auto_shrink([false, false]).show(ui, |ui| {
                render_main(app, ui, recent, body.width(), body.height(), act);
                ui.add_space(28.0);
                render_formats(app, ui, body.width(), 0.0, act);
            });
        });
        return;
    }
    let right_w = (body.width() * 0.3).clamp(280.0, 430.0);
    let gap = (body.width() * 0.045).clamp(28.0, 66.0);
    let left = Rect::from_min_max(body.min, pos2(body.right() - right_w - gap, body.bottom()));
    let right = Rect::from_min_max(pos2(body.right() - right_w, body.top()), body.max);
    let divider_x = (left.right() + right.left()) / 2.0;
    ui.painter().vline(divider_x, (body.top() + 52.0)..=(body.bottom() - 36.0).max(body.top() + 52.0), Stroke::new(1.0, t.separator));
    ui.scope_builder(egui::UiBuilder::new().max_rect(left), |ui| {
        egui::ScrollArea::vertical().id_salt("home-main").auto_shrink([false, false]).show(ui, |ui| render_main(app, ui, recent, left.width(), left.height(), act));
    });
    ui.scope_builder(egui::UiBuilder::new().max_rect(right), |ui| {
        egui::ScrollArea::vertical().id_salt("home-formats").auto_shrink([false, false]).show(ui, |ui| render_formats(app, ui, right.width(), right.height(), act));
    });
}

fn run(app: &mut PhotocraftApp, ctx: &egui::Context, act: Act) {
    let r = match act {
        Act::New => {
            let fields = app.new_document_fields();
            app.ui.open_dialog(crate::state::DialogKind::NewDocument, fields);
            Ok(())
        }
        Act::Open => app.open_dialog_file().map(|_| ()),
        Act::Preset(p) => {
            // Made exactly as File › New › Create makes it (remembered settings, New › Recent, fit).
            let units = app.session.prefs().units_and_rulers.clone();
            let mut fields = app.new_document_fields();
            crate::new_doc_ui::apply_preset(&mut fields, &p, &units);
            let id = app.ui.open_dialog(crate::state::DialogKind::NewDocument, fields);
            crate::dialogs::confirm(app, id).map(|_| ())
        }
        Act::Clipboard => crate::menus::invoke(app, ctx, CLIPBOARD, serde_json::json!({})).map(|_| ()),
        Act::Prefs => crate::menus::invoke(app, ctx, "edit.preferences.general", serde_json::json!({})).map(|_| ()),
        Act::Back => {
            app.ui.chrome.home = None;
            Ok(())
        }
    };
    if let Err(e) = r {
        app.ui.status = e;
        app.ui.status_error = true;
    }
}

/// A row on the rail: an icon and a label, the amber pill when `selected`.
fn rail_item(ui: &mut egui::Ui, rect: Rect, icon: &str, label: &str, selected: bool) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let resp = ui.interact(rect, ui.scope_id().with(("home-rail", label)), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
    let ink = if selected { t.primary_text } else { t.text };
    if selected {
        ui.painter().rect_filled(rect, RADIUS, t.primary_bg);
    } else if resp.hovered() {
        ui.painter().rect_filled(rect, RADIUS, t.hover);
    }
    crate::icons::paint(ui, Rect::from_center_size(pos2(rect.left() + 27.0, rect.center().y), vec2(22.0, 22.0)), icon, 22.0, ink);
    let font = if selected { semibold(16.5) } else { medium(16.5) };
    let name = crate::tab_strip::elided(ui, label, font, ink, rect.width() - 70.0);
    ui.painter().galley(pos2(rect.left() + 58.0, rect.center().y - name.size().y / 2.0), name, ink);
    resp
}

/// Home (selected), New document, Open file; Settings at the foot.
fn render_rail(app: &PhotocraftApp, ui: &mut egui::Ui, rail: Rect, act: &mut Option<Act>) {
    let t = Tokens::get(ui.ctx());
    let x = rail.left() + 14.0..=rail.right() - 15.0;
    let row = |y: f32| Rect::from_x_y_ranges(x.clone(), y..=y + 48.0);
    // Home hides the tab strip and options bar, so this entry is the way back to the open
    // documents (with none open there is nothing behind Home).
    if rail_item(ui, row(rail.top() + 38.0), "house", tl!("Home"), true).clicked() && !app.session.documents().is_empty() {
        *act = Some(Act::Back);
    }
    if rail_item(ui, row(rail.top() + 94.0), "plus", tl!("New document"), false).clicked() {
        *act = Some(Act::New);
    }
    if rail_item(ui, row(rail.top() + 149.0), "folder", tl!("Open File"), false).clicked() {
        *act = Some(Act::Open);
    }
    let foot = rail.bottom() - 74.0;
    if foot > rail.top() + 220.0 {
        ui.painter().hline(x.clone(), foot - 14.0, Stroke::new(1.0, t.separator));
        if rail_item(ui, row(foot), "settings", tl!("Settings"), false).clicked() {
            *act = Some(Act::Prefs);
        }
    }
}

/// A rounded icon + label button: amber with dark text when `primary`, outlined otherwise.
fn action_button(ui: &mut egui::Ui, label: &str, icon: &str, primary: bool, size: egui::Vec2) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    let (fill, stroke, ink) = if primary {
        (if resp.hovered() { t.accent } else { t.primary_bg }, Stroke::NONE, t.primary_text)
    } else {
        (if resp.hovered() { t.hover } else { t.chrome }, Stroke::new(1.5, t.text), t.text)
    };
    ui.painter().rect(rect, RADIUS, fill, stroke, StrokeKind::Inside);
    let font = semibold((size.y * 0.34).clamp(14.0, 22.0));
    let icon_s = font.size * 1.15;
    let galley = crate::tab_strip::elided(ui, label, font, ink, (size.x - icon_s - 30.0).max(10.0));
    let x = rect.center().x - (galley.size().x + icon_s + 12.0) / 2.0;
    crate::icons::paint(ui, Rect::from_center_size(pos2(x + icon_s / 2.0, rect.center().y), vec2(icon_s, icon_s)), icon, icon_s, ink);
    ui.painter().galley(pos2(x + icon_s + 12.0, rect.center().y - galley.size().y / 2.0), galley, ink);
    resp
}

/// The hero, its two buttons, the drop zone and the recents (or their placeholder).
fn render_main(app: &mut PhotocraftApp, ui: &mut egui::Ui, recent: &[String], w: f32, h: f32, act: &mut Option<Act>) {
    let t = Tokens::get(ui.ctx());
    let top = ui.cursor().top();
    let title_s = (w / 12.5).clamp(26.0, 62.0);
    ui.add_space((title_s * 0.75).clamp(12.0, 44.0));
    ui.add(egui::Label::new(egui::RichText::new(tl!("What shall we create today?")).font(semibold(title_s)).color(t.text)).wrap());
    ui.add(egui::Label::new(egui::RichText::new(tl!("Start from scratch or open an idea.")).font(medium((title_s * 0.43).clamp(14.0, 26.0))).color(t.text_dim)).wrap());
    ui.add_space((title_s * 0.5).clamp(14.0, 30.0));

    let btn_h = (title_s * 1.1).clamp(42.0, 68.0);
    let new_w = (w * 0.4).clamp(150.0, 318.0).min((w - 12.0) / 2.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 24.0_f32.min(w * 0.03).max(8.0);
        if action_button(ui, tl!("New document"), "plus", true, vec2(new_w, btn_h)).clicked() {
            *act = Some(Act::New);
        }
        if action_button(ui, tl!("Open File"), "folder", false, vec2((new_w * 0.8).max(new_w.min(150.0)), btn_h)).clicked() {
            *act = Some(Act::Open);
        }
    });
    ui.add_space((title_s * 0.65).clamp(18.0, 40.0));

    // Keep the recents in view: the drop zone takes what is left, within reason.
    let used = ui.cursor().top() - top;
    let drop_h = (h - used - if recent.is_empty() { 120.0 } else { 260.0 }).clamp(170.0, 400.0);
    if drop_zone(app, ui, vec2(w, drop_h)).clicked() {
        *act = Some(Act::Open);
    }
    ui.add_space(38.0);
    let (line, _) = ui.allocate_exact_size(vec2(w, 1.0), Sense::hover());
    ui.painter().hline(line.x_range(), line.center().y, Stroke::new(1.0, t.separator));
    ui.add_space(20.0);
    if recent.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            let (r, _) = ui.allocate_exact_size(vec2(24.0, 24.0), Sense::hover());
            crate::icons::paint(ui, r, "file", 22.0, t.text);
            ui.add_space(10.0);
            ui.add(egui::Label::new(egui::RichText::new(tl!("Your recent documents will appear here.")).font(medium(16.0)).color(t.text)).truncate());
        });
    } else {
        crate::home_recent_ui::show_home_recent(app, ui, recent, w);
    }
    ui.add_space(20.0);
}

/// The dashed drop zone: a click opens File › Open (files are dropped anywhere on the window).
fn drop_zone(app: &PhotocraftApp, ui: &mut egui::Ui, size: egui::Vec2) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let (r, resp) = ui.allocate_exact_size(size, Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(start_screen_drop_hint(app));
    let hov = resp.hovered();
    let p = ui.painter();
    if hov {
        p.rect_filled(r, 14.0, t.hover);
    }
    p.extend(egui::Shape::dashed_line(&rounded_outline(r.shrink(1.0), 14.0), Stroke::new(1.5, if hov { t.accent } else { t.text_dim }), 8.0, 6.0));

    let s = (size.y * 0.33).clamp(44.0, 130.0);
    let c = pos2(r.center().x, r.top() + size.y * 0.42 - s * 0.1);
    let ink = if hov { t.accent } else { t.text };
    crate::icons::paint(ui, Rect::from_center_size(c + vec2(s * 0.14, -s * 0.12), vec2(s, s)), "file", s * 0.8, ink);
    p.rect_filled(Rect::from_center_size(c + vec2(-s * 0.14, s * 0.12), vec2(s * 0.52, s * 0.66)), 6.0, if hov { t.hover } else { t.chrome });
    crate::icons::paint(ui, Rect::from_center_size(c + vec2(-s * 0.14, s * 0.12), vec2(s, s)), "file", s * 0.8, ink);

    let head_s = (size.x / 26.0).clamp(15.0, 24.0);
    let y = (r.top() + size.y * 0.42 + s * 0.62).min(r.bottom() - head_s * 2.8);
    p.text(pos2(r.center().x, y), egui::Align2::CENTER_TOP, tl!("Drag an image or PSD to start"), semibold(head_s), t.text);
    // "or choose a file", the second half an amber, underlined link.
    let small = medium(head_s * 0.86);
    let or = p.layout_no_wrap(format!("{} ", tl!("or")), small.clone(), t.text);
    let link = p.layout_no_wrap(tl!("choose a file").to_owned(), small, t.accent);
    let x = r.center().x - (or.size().x + link.size().x) / 2.0;
    let ly = y + head_s * 1.7;
    let link_min = pos2(x + or.size().x, ly);
    p.hline(link_min.x..=link_min.x + link.size().x, ly + link.size().y + 1.0, Stroke::new(1.2, t.accent));
    p.galley(pos2(x, ly), or, t.text);
    p.galley(link_min, link, t.accent);
    resp
}

/// The outline of a rounded rectangle as a closed polyline, for the dashed border.
fn rounded_outline(r: Rect, radius: f32) -> Vec<egui::Pos2> {
    let rad = radius.min(r.width() / 2.0).min(r.height() / 2.0).max(0.0);
    let corners = [(r.right() - rad, r.top() + rad, -90.0_f32), (r.right() - rad, r.bottom() - rad, 0.0), (r.left() + rad, r.bottom() - rad, 90.0), (r.left() + rad, r.top() + rad, 180.0)];
    let mut pts = Vec::with_capacity(4 * 7 + 1);
    for (cx, cy, start) in corners {
        for k in 0..=6 {
            let a = (start + 15.0 * k as f32).to_radians();
            pts.push(pos2(cx + rad * a.cos(), cy + rad * a.sin()));
        }
    }
    if let Some(&first) = pts.first() {
        pts.push(first);
    }
    pts
}

/// Built-in New Document presets offered on Home: (name in `new_doc_ui::CATEGORIES`, short label).
const HOME_PRESETS: &[(&str, &str)] = &[
    ("Default PhotoCraft Size", "Default"),
    ("A4", "A4"),
    ("Letter", "Letter"),
    ("Landscape, 6 x 4", "Landscape"),
    ("Web Most Common", "Web"),
    ("HDTV 1080p", "HDTV 1080p"),
    ("Android 1080p", "Android 1080p"),
];

fn home_presets() -> impl Iterator<Item = (&'static crate::new_doc_ui::Preset, &'static str)> {
    HOME_PRESETS.iter().filter_map(|(name, label)| crate::new_doc_ui::CATEGORIES.iter().flat_map(|c| c.1.iter()).find(|p| p.0 == *name).map(|p| (p, *label)))
}

/// "Start with a format": one card per preset; a click creates that document.
fn render_formats(app: &PhotocraftApp, ui: &mut egui::Ui, w: f32, h: f32, act: &mut Option<Act>) {
    let t = Tokens::get(ui.ctx());
    ui.add_space(if h > 0.0 { 34.0 } else { 0.0 });
    ui.label(egui::RichText::new(tl!("Start with a format")).font(semibold((w / 16.5).clamp(18.0, 26.0))).color(t.text));
    ui.add_space(22.0);
    let n = HOME_PRESETS.len() as f32;
    let card_h = if h > 0.0 { ((h - 110.0) / n - 9.0).clamp(54.0, 76.0) } else { 64.0 };
    let units = &app.session.prefs().units_and_rulers;
    ui.spacing_mut().item_spacing.y = 9.0;
    for (p, label) in home_presets() {
        let (_, pw, ph, _) = crate::new_doc_ui::at_pref_resolutions(p, units);
        if format_card(ui, tl!(label), pw, ph, vec2(w - 2.0, card_h)).clicked() {
            *act = Some(Act::Preset(*p));
        }
    }
}

/// One format row: the page's orientation, the name over its size in pixels, a chevron.
fn format_card(ui: &mut egui::Ui, title: &str, pw: u32, ph: u32, size: egui::Vec2) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, title));
    let hov = resp.hovered();
    let p = ui.painter();
    p.rect(rect, RADIUS, if hov { t.hover } else { t.card }, Stroke::new(1.0, if hov { t.accent } else { t.card_border }), StrokeKind::Inside);
    let k = (size.y / 76.0).clamp(0.7, 1.0);
    let page = if pw >= ph { vec2(52.0, 28.0) } else { vec2(28.0, 44.0) } * k;
    let icon_c = pos2(rect.left() + 56.0 * k, rect.center().y);
    p.rect_stroke(Rect::from_center_size(icon_c, page), 4.0, Stroke::new(1.5, if hov { t.accent } else { t.text }), StrokeKind::Inside);
    let sep_x = rect.left() + 112.0 * k;
    p.vline(sep_x, (rect.center().y - 22.0 * k)..=(rect.center().y + 22.0 * k), Stroke::new(1.0, t.separator));
    let text_x = sep_x + 30.0 * k;
    let text_w = (rect.right() - 50.0 - text_x).max(10.0);
    let name = crate::tab_strip::elided(ui, title, semibold(18.0 * k), t.text, text_w);
    let dims = crate::tab_strip::elided(ui, &format!("{pw} × {ph} px"), medium(15.0 * k), t.text_dim, text_w);
    let top = rect.center().y - (name.size().y + dims.size().y + 2.0) / 2.0;
    let dims_y = top + name.size().y + 2.0;
    p.galley(pos2(text_x, top), name, t.text);
    p.galley(pos2(text_x, dims_y), dims, t.text_dim);
    crate::icons::paint(ui, Rect::from_center_size(pos2(rect.right() - 26.0, rect.center().y), vec2(20.0, 20.0)), "chevron-right", 20.0, t.text);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// The foot strip: New from Clipboard, its hint, and "See all formats →" (the New Document dialog).
fn render_strip(app: &PhotocraftApp, ui: &mut egui::Ui, strip: Rect, act: &mut Option<Act>) {
    let t = Tokens::get(ui.ctx());
    let cy = strip.center().y + 2.0;
    // Right: the amber link with its arrow.
    let link = ui.painter().layout_no_wrap(tl!("See all formats").to_owned(), semibold(17.0), t.accent);
    let link_rect = Rect::from_min_max(pos2(strip.right() - link.size().x - 34.0, cy - 16.0), pos2(strip.right(), cy + 16.0));
    let resp = ui.interact(link_rect, ui.scope_id().with("home-all-formats"), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Link, true, tl!("See all formats")));
    let ink = if resp.hovered() { t.text } else { t.accent };
    let p = ui.painter();
    p.galley(pos2(link_rect.left(), cy - link.size().y / 2.0), link, ink);
    let ax = link_rect.right() - 2.0;
    let arrow = Stroke::new(2.0, ink);
    p.hline((ax - 20.0)..=ax, cy, arrow);
    p.line(vec![pos2(ax - 7.0, cy - 7.0), pos2(ax, cy), pos2(ax - 7.0, cy + 7.0)], arrow);
    if resp.clicked() {
        *act = Some(Act::New);
    }

    // Left: New from Clipboard, then what it does while there is room.
    let enabled = crate::menus::is_enabled(app, CLIPBOARD);
    let label = tl!("Clipboard");
    let g = ui.painter().layout_no_wrap(label.to_owned(), semibold(16.0), t.text);
    let btn = Rect::from_min_size(pos2(strip.left(), cy - 27.0), vec2(g.size().x + 84.0, 54.0));
    let resp = ui.interact(btn, ui.scope_id().with("home-clipboard"), if enabled { Sense::click() } else { Sense::hover() });
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, tl!("New from Clipboard")));
    let ink = if enabled { t.text } else { t.text_faint };
    let p = ui.painter();
    p.rect(btn, RADIUS, if enabled && resp.hovered() { t.hover } else { t.card }, Stroke::new(1.0, t.card_border), StrokeKind::Inside);
    crate::icons::paint(ui, Rect::from_center_size(pos2(btn.left() + 34.0, cy), vec2(22.0, 22.0)), "copy", 22.0, ink);
    p.galley(pos2(btn.left() + 62.0, cy - g.size().y / 2.0), g, ink);
    if enabled && resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
        *act = Some(Act::Clipboard);
    }
    let hint_x = btn.right() + 22.0;
    let room = link_rect.left() - 24.0 - (hint_x + 22.0);
    if room > 120.0 {
        p.vline(hint_x, (cy - 14.0)..=(cy + 14.0), Stroke::new(1.0, t.separator));
        let hint = crate::tab_strip::elided(ui, tl!("Paste an image from the clipboard to create quickly."), medium(15.5), t.text, room);
        p.galley(pos2(hint_x + 22.0, cy - hint.size().y / 2.0), hint, t.text);
    }
}

/// Hint displayed on the start screen for opening files.
pub(crate) fn start_screen_drop_hint(_app: &PhotocraftApp) -> std::borrow::Cow<'static, str> {
    tl!("Drop an image or PSD anywhere to open it.").into()
}

/// Recent files listed on the Home screen.
const HOME_RECENT: usize = 11;

#[cfg(test)]
mod tests {
    use egui_kittest::kittest::Queryable;

    #[test]
    fn every_home_preset_is_a_new_document_preset() {
        assert_eq!(super::home_presets().count(), super::HOME_PRESETS.len());
    }

    #[test]
    fn the_dashed_outline_closes() {
        let pts = super::rounded_outline(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 60.0)), 14.0);
        assert_eq!(pts.first(), pts.last());
        assert!(pts.iter().all(|p| p.x >= -0.01 && p.x <= 100.01 && p.y >= -0.01 && p.y <= 60.01));
    }

    /// A format card on Home makes that document straight away, at the size the card shows.
    #[test]
    fn clicking_a_format_card_creates_that_document() {
        let app = crate::PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1280.0, 720.0)).build_ui_state(
            |ui, (app, ready): &mut (crate::PhotocraftApp, bool)| {
                if *ready {
                    egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| super::super::tabs::document_area(app, ui));
                }
            },
            (app, false),
        );
        crate::PhotocraftApp::setup_context(&h.ctx, crate::theme::ThemeKind::default());
        h.state_mut().1 = true;
        h.run_steps(3);
        let units = h.state().0.session.prefs().units_and_rulers.clone();
        let (p, label) = super::home_presets().find(|(_, l)| *l == "Web").expect("Web preset");
        let (_, w, hgt, _) = crate::new_doc_ui::at_pref_resolutions(p, &units);
        assert_eq!((w, hgt), (1366, 768));
        h.get_by_label(label).click();
        h.run_steps(3);
        let doc = &h.state().0.session.active().expect("a document").doc;
        assert_eq!((doc.size.width, doc.size.height), (w, hgt));
    }
}
