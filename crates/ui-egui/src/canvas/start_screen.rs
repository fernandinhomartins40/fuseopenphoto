use super::tabs::{open_on_double_click, paint_dots};
use super::{PhotocraftApp, Rect, Sense};
use crate::studio::style::{APP_NAME, RADIUS};

pub(super) fn start_screen(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let area = ui.available_rect_before_wrap();
    paint_dots(ui, area, app.session.prefs().interface.show_canvas_dots);
    open_on_double_click(app, ui, area);

    // File › Open Recent, newest first (on the web there are no paths to reopen).
    let show_recent = app.session.prefs().interface.show_home_recent_files;
    let recent: Vec<String> =
        if cfg!(target_arch = "wasm32") || !show_recent { Vec::new() } else { app.ui.recent_files.iter().take(HOME_RECENT).cloned().collect() };

    let is_narrow = area.width() < 640.0;
    if is_narrow {
        let card_w = (area.width() - 32.0).clamp(0.0, 520.0);
        let card = Rect::from_center_size(area.center(), egui::vec2(card_w, (area.height() - 40.0).max(0.0)));
        ui.scope_builder(egui::UiBuilder::new().max_rect(card), |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                render_start_content(app, ui, &recent, card.width());
            });
        });
    } else {
        let (pad_x, pad_y, rail_w) = (36.0, 54.0, 180.0);
        let rail_rect = Rect::from_min_size(egui::pos2(area.left() + pad_x, area.top() + pad_y), egui::vec2(rail_w, (area.height() - pad_y * 2.0).max(0.0)));
        let content_x = rail_rect.right() + 40.0;
        let content_rect = Rect::from_min_max(egui::pos2(content_x, area.top() + pad_y), egui::pos2(area.right() - pad_x, area.bottom() - 16.0));

        ui.scope_builder(egui::UiBuilder::new().max_rect(rail_rect), |ui| render_start_rail(app, ui, rail_rect.width()));
        ui.scope_builder(egui::UiBuilder::new().max_rect(content_rect), |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                render_start_content(app, ui, &recent, content_rect.width());
            });
        });
    }
}

/// A rounded icon + label button: amber with dark text when `primary`, a charcoal card otherwise.
fn action_button(ui: &mut egui::Ui, label: &str, icon: &str, primary: bool, width: f32) -> egui::Response {
    let t = crate::theme::Tokens::get(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 38.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    let (fill, stroke, ink) = if primary {
        (if resp.hovered() { t.accent } else { t.primary_bg }, egui::Stroke::NONE, t.primary_text)
    } else {
        (if resp.hovered() { t.hover } else { t.card }, egui::Stroke::new(1.0, t.card_border), t.text)
    };
    ui.painter().rect(rect, RADIUS, fill, stroke, egui::StrokeKind::Inside);
    let galley = ui.painter().layout_no_wrap(label.to_owned(), crate::theme::semibold(13.5), ink);
    let x = rect.center().x - (galley.size().x + 22.0) / 2.0;
    crate::icons::paint(ui, Rect::from_center_size(egui::pos2(x + 8.0, rect.center().y), egui::vec2(16.0, 16.0)), icon, 15.0, ink);
    ui.painter().galley(egui::pos2(x + 22.0, rect.center().y - galley.size().y / 2.0), galley, ink);
    resp
}

/// "New document" (amber) and "Open…".
fn render_start_actions_row(app: &mut PhotocraftApp, ui: &mut egui::Ui, avail_w: f32) {
    let (new_w, open_w) = if avail_w < 400.0 {
        let w = ((avail_w - 12.0) / 2.0).clamp(0.0, 170.0);
        (w, w)
    } else {
        (190.0, 130.0)
    };
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        if action_button(ui, tl!("New document"), "plus", true, new_w).clicked() {
            let fields = app.new_document_fields();
            app.ui.open_dialog(crate::state::DialogKind::NewDocument, fields);
        }
        if action_button(ui, tl!("Open…"), "folder-open", false, open_w).clicked() {
            let _ = app.open_dialog_file();
        }
    });
}

/// The calm left rail: Home only (this build links to no online project or community).
fn render_start_rail(app: &mut PhotocraftApp, ui: &mut egui::Ui, rail_w: f32) {
    let t = crate::theme::Tokens::get(ui.ctx());
    ui.add_space(4.0);
    let (home_rect, home_resp) = ui.allocate_exact_size(egui::vec2(rail_w, 38.0), Sense::click());
    home_resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tl!("Home")));
    // Home hides the tab strip and options bar, so this entry is the way back to the open
    // documents (with none open there is nothing behind Home).
    if !app.session.documents().is_empty() && home_resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
        app.ui.chrome.home = None;
    }
    ui.painter().rect(home_rect, RADIUS, t.card, egui::Stroke::new(1.0, t.card_border), egui::StrokeKind::Inside);
    let house_box = Rect::from_center_size(egui::pos2(home_rect.left() + 20.0, home_rect.center().y), egui::vec2(16.0, 16.0));
    crate::icons::paint(ui, house_box, "house", 16.0, t.accent);
    ui.painter().text(egui::pos2(home_rect.left() + 38.0, home_rect.center().y), egui::Align2::LEFT_CENTER, tl!("Home"), crate::theme::semibold(13.5), t.text);
}

fn section_title(ui: &mut egui::Ui, title: &str) {
    let t = crate::theme::Tokens::get(ui.ctx());
    ui.label(egui::RichText::new(title).font(crate::theme::semibold(15.0)).color(t.text));
    ui.add_space(10.0);
}

fn render_start_content(app: &mut PhotocraftApp, ui: &mut egui::Ui, recent: &[String], content_w: f32) {
    let t = crate::theme::Tokens::get(ui.ctx());
    ui.add_space(4.0);
    ui.label(egui::RichText::new(APP_NAME).font(crate::theme::semibold(28.0)).color(t.text));
    ui.add_space(16.0);
    render_start_actions_row(app, ui, content_w);
    ui.add_space(30.0);

    section_title(ui, tl!("Presets"));
    render_presets(app, ui, content_w);
    ui.add_space(30.0);

    if recent.is_empty() {
        render_first_run_content(app, ui, content_w);
    } else {
        crate::home_recent_ui::show_home_recent(app, ui, recent, content_w);
    }
    ui.add_space(24.0);
}

/// Built-in New Document presets offered on Home, by name (see `new_doc_ui::CATEGORIES`).
const HOME_PRESETS: &[&str] = &["Default PhotoCraft Size", "A4", "Letter", "Landscape, 6 x 4", "Web Most Common", "HDTV 1080p", "Android 1080p"];
const PRESET_CARD: egui::Vec2 = egui::vec2(150.0, 112.0);

fn home_presets() -> impl Iterator<Item = &'static crate::new_doc_ui::Preset> {
    HOME_PRESETS.iter().filter_map(|name| crate::new_doc_ui::CATEGORIES.iter().flat_map(|c| c.1.iter()).find(|p| p.0 == *name))
}

/// One preset card: a page with the preset's proportions, its name and pixel size.
fn preset_card(ui: &mut egui::Ui, title: &str, size: Option<(u32, u32)>, enabled: bool) -> egui::Response {
    let t = crate::theme::Tokens::get(ui.ctx());
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, resp) = ui.allocate_exact_size(PRESET_CARD, sense);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, title));
    let hovered = enabled && resp.hovered();
    let border = if hovered { t.accent } else { t.card_border };
    ui.painter().rect(rect, RADIUS, t.card, egui::Stroke::new(1.0, border), egui::StrokeKind::Inside);
    let ink = if enabled { t.text } else { t.text_faint };
    let icon_area = Rect::from_center_size(egui::pos2(rect.center().x, rect.top() + 34.0), egui::vec2(44.0, 40.0));
    match size {
        Some((w, h)) if w > 0 && h > 0 => {
            let s = (icon_area.width() / w as f32).min(icon_area.height() / h as f32);
            let page = Rect::from_center_size(icon_area.center(), egui::vec2(w as f32 * s, h as f32 * s));
            ui.painter().rect(page, 3.0, t.field, egui::Stroke::new(1.0, if hovered { t.accent } else { t.text_faint }), egui::StrokeKind::Inside);
        }
        _ => crate::icons::paint(ui, icon_area, "copy", 26.0, ink),
    }
    let text_w = rect.width() - 20.0;
    let name = crate::tab_strip::elided(ui, title, crate::theme::semibold(12.5), ink, text_w);
    ui.painter().galley(egui::pos2(rect.left() + 10.0, rect.top() + 66.0), name, ink);
    if let Some((w, h)) = size {
        ui.painter().text(egui::pos2(rect.left() + 10.0, rect.top() + 90.0), egui::Align2::LEFT_TOP, format!("{w} × {h} px"), egui::FontId::proportional(11.0), t.text_dim);
    }
    if enabled { resp.on_hover_cursor(egui::CursorIcon::PointingHand) } else { resp }
}

fn render_presets(app: &mut PhotocraftApp, ui: &mut egui::Ui, content_w: f32) {
    let units = app.session.prefs().units_and_rulers.clone();
    let mut picked = None;
    let mut from_clipboard = false;
    ui.horizontal_wrapped(|ui| {
        ui.set_max_width(content_w);
        ui.spacing_mut().item_spacing = egui::vec2(12.0, 12.0);
        for p in home_presets() {
            let (_, w, h, _) = crate::new_doc_ui::at_pref_resolutions(p, &units);
            if preset_card(ui, tl!(p.0), Some((w, h)), true).clicked() {
                picked = Some(*p);
            }
        }
        let clip = "file.newFromClipboard";
        if preset_card(ui, tl!("New from Clipboard"), None, crate::menus::is_enabled(app, clip)).clicked() {
            from_clipboard = true;
        }
    });

    let r = if let Some(p) = picked {
        // Made exactly as File › New › Create makes it (remembered settings, New › Recent, fit).
        let mut fields = app.new_document_fields();
        crate::new_doc_ui::apply_preset(&mut fields, &p, &units);
        let id = app.ui.open_dialog(crate::state::DialogKind::NewDocument, fields);
        crate::dialogs::confirm(app, id).map(|_| ())
    } else if from_clipboard {
        crate::menus::invoke(app, ui.ctx(), "file.newFromClipboard", serde_json::json!({})).map(|_| ())
    } else {
        Ok(())
    };
    if let Err(e) = r {
        app.ui.status = e;
        app.ui.status_error = true;
    }
}

fn render_first_run_content(app: &mut PhotocraftApp, ui: &mut egui::Ui, content_w: f32) {
    let t = crate::theme::Tokens::get(ui.ctx());
    let (drop_r, drop_resp) = ui.allocate_exact_size(egui::vec2(content_w.clamp(0.0, 640.0), 170.0), Sense::click());
    let drop_resp = drop_resp.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(tl!("Open…"));
    let is_hov = drop_resp.hovered();

    let border_color = if is_hov { t.accent } else { t.card_border };
    ui.painter().rect(drop_r, RADIUS, if is_hov { t.hover } else { t.card }, egui::Stroke::new(1.0, border_color), egui::StrokeKind::Inside);

    let icon_box = Rect::from_center_size(egui::pos2(drop_r.center().x, drop_r.center().y - 18.0), egui::vec2(40.0, 40.0));
    crate::icons::paint(ui, icon_box, "image", 34.0, if is_hov { t.accent } else { t.text_dim });
    ui.painter().text(
        egui::pos2(drop_r.center().x, drop_r.center().y + 22.0),
        egui::Align2::CENTER_CENTER,
        start_screen_drop_hint(app),
        crate::theme::medium(14.0),
        if is_hov { t.text } else { t.text_dim },
    );
    if drop_resp.clicked() {
        let _ = app.open_dialog_file();
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
    #[test]
    fn every_home_preset_is_a_new_document_preset() {
        assert_eq!(super::home_presets().count(), super::HOME_PRESETS.len());
    }
}
