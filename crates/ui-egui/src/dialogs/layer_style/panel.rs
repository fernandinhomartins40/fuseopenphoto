//! The studio's inline Efeitos editor: Layer Style's model (the dialog's fields, [`super::page`],
//! [`super::apply`]) in a narrow list, one row per effect kind with its on/off box and a chevron
//! that unfolds its parameters, then Preview, Reverter and Aplicar.
//!
//! A colour swatch hands the whole session to the Layer Style dialog, which owns the picker.

use egui::{CornerRadius, Rect, Sense, pos2, vec2};
use serde_json::{Map, Value, json};

use super::{BLENDING, KINDS, effects_of, entry_mut};
use crate::PhotocraftApp;
use crate::theme::Tokens;
use crate::{icons, widgets};

/// One editing session on a layer's style.
pub struct FxEdit {
    layer: u64,
    /// The document revision `original` was read at; an unedited session follows the document.
    revision: u64,
    /// The dialog's fields being edited, and as they were read (Reverter).
    f: Map<String, Value>,
    original: Map<String, Value>,
    /// The unfolded row: an effect kind or [`BLENDING`].
    open: Option<String>,
}

impl FxEdit {
    fn dirty(&self) -> bool {
        super::preview_hash(&self.f) != super::preview_hash(&self.original)
    }
}

/// The fields the canvas previews: an edited session with Preview on.
pub fn preview(fx: Option<&FxEdit>) -> Option<&Map<String, Value>> {
    fx.filter(|fx| fx.dirty() && fx.f.get("preview").and_then(Value::as_bool).unwrap_or(true)).map(|fx| &fx.f)
}

/// The fields of the active layer's style, as the dialog opens them, and the document revision.
fn read(app: &PhotocraftApp) -> Option<(u64, u64, Map<String, Value>)> {
    let st = app.session.active()?;
    let layer = st.doc.layer(st.active_layer?)?;
    let user = app.session.presets.layer_defaults.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let mut f = super::initial_fields_with(layer, Some(BLENDING), st.doc.global_light.angle, user);
    super::set_blending_fields(&mut f, layer, st.doc.mode);
    f.insert("__document".into(), json!(st.doc.id));
    f.insert("patternList".into(), super::pattern_list(app));
    Some((layer.id.0, st.revision, f))
}

pub fn show(app: &mut PhotocraftApp, ui: &mut egui::Ui, fx: &mut Option<FxEdit>) {
    let t = Tokens::get(ui.ctx());
    let Some((layer, revision, fresh)) = read(app) else {
        *fx = None;
        ui.add(egui::Label::new(egui::RichText::new(tl!("Select a layer to edit its effects.")).color(t.text_faint)).wrap());
        return;
    };
    // A new layer, or a document changed under an unedited session (Aplicar, undo), reads again.
    let stale = fx.as_ref().is_none_or(|e| e.layer != layer || (e.revision != revision && !e.dirty()));
    if stale {
        let open = fx.take().filter(|e| e.layer == layer).and_then(|e| e.open);
        *fx = Some(FxEdit { layer, revision, original: fresh.clone(), f: fresh, open });
    }
    let Some(edit) = fx.as_mut() else { return };
    rows(app, ui, edit);
    super::contour::editor(app, ui, &mut edit.f);
    ui.add_space(8.0);
    if footer(app, ui, edit) {
        *fx = None;
        return;
    }
    // A swatch asked for the colour picker: the dialog takes the session over.
    if edit.f.contains_key(super::color_picker::REQUEST) {
        let f = edit.f.clone();
        *fx = None;
        let id = app.ui.open_dialog(crate::state::DialogKind::LayerStyle, f);
        if let Err(e) = super::color_picker::take_request(app, id) {
            app.ui.status = e;
            app.ui.status_error = true;
        }
    }
}

/// Mesclagem, then every effect kind: its box (all its instances on or off; a kind the layer
/// lacks is added), its name and a chevron unfolding the first instance's parameters.
fn rows(app: &mut PhotocraftApp, ui: &mut egui::Ui, edit: &mut FxEdit) {
    let open = edit.open.clone();
    if row(ui, None, tl!("Blending Options"), open.as_deref() == Some(BLENDING)).1 {
        edit.open = toggle(&edit.open, BLENDING);
    }
    if open.as_deref() == Some(BLENDING) {
        params(app, ui, &mut edit.f, BLENDING);
    }
    for &(kind, name) in KINDS {
        let ids: Vec<String> = effects_of(&edit.f)
            .iter()
            .filter(|e| e.get("kind").and_then(Value::as_str) == Some(kind))
            .filter_map(|e| e.get("id").and_then(Value::as_str).map(str::to_string))
            .collect();
        let on = effects_of(&edit.f).iter().any(|e| e.get("kind").and_then(Value::as_str) == Some(kind) && e.get("on").and_then(Value::as_bool) == Some(true));
        let unfolded = open.as_deref() == Some(kind);
        let (checked, chevron) = row(ui, Some(on), tl!(name), unfolded);
        if checked {
            if ids.is_empty() {
                super::add_instance(&mut edit.f, kind);
                edit.open = Some(kind.into());
            }
            for id in &ids {
                if let Some(e) = entry_mut(&mut edit.f, id) {
                    e["on"] = json!(!on);
                }
            }
        }
        if chevron {
            if ids.is_empty() {
                super::add_instance(&mut edit.f, kind);
            }
            edit.open = toggle(&edit.open, kind);
        }
        if let Some(id) = ids.first().filter(|_| unfolded) {
            params(app, ui, &mut edit.f, id);
        }
    }
}

fn toggle(open: &Option<String>, key: &str) -> Option<String> {
    (open.as_deref() != Some(key)).then(|| key.to_string())
}

/// One instance's (or Blending Options') parameters, indented under its row.
fn params(app: &mut PhotocraftApp, ui: &mut egui::Ui, f: &mut Map<String, Value>, id: &str) {
    let mut style_click = None;
    f.insert("selected".into(), json!(id));
    let invalid = super::validate_params(f).err();
    ui.indent(("studio-fx", id), |ui| super::page(app, ui, f, id, invalid.as_deref(), &mut style_click, false));
    ui.add_space(4.0);
}

/// A full-width row: the on/off box (none for Mesclagem), the name, the fold chevron. Returns
/// (box clicked, rest of the row clicked).
fn row(ui: &mut egui::Ui, on: Option<bool>, label: &str, unfolded: bool) -> (bool, bool) {
    let t = Tokens::get(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 29.0), Sense::click());
    let cb = Rect::from_center_size(pos2(rect.left() + 12.0, rect.center().y), vec2(14.0, 14.0));
    if resp.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(6), t.hover);
    }
    match on {
        Some(on) => super::fx_checkbox(ui.painter(), cb, on, &t),
        None => icons::paint(ui, cb, if unfolded { "chevron-down" } else { "chevron-right" }, 14.0, t.text_dim),
    }
    // The name keeps clear of the chevron; a long translation is cut short.
    let text_rect = Rect::from_min_max(pos2(rect.left() + 28.0, rect.top()), pos2(rect.right() - 28.0, rect.bottom()));
    let galley = ui.painter().layout(label.to_owned(), egui::FontId::proportional(13.0), t.text, f32::INFINITY);
    ui.painter().with_clip_rect(text_rect).galley(pos2(text_rect.left(), rect.center().y - galley.size().y / 2.0), galley, t.text);
    let chevron = if on.is_some() && unfolded { "chevron-down" } else if on.is_some() { "chevron-right" } else { "" };
    if !chevron.is_empty() {
        icons::paint(ui, Rect::from_center_size(pos2(rect.right() - 12.0, rect.center().y), vec2(14.0, 14.0)), chevron, 14.0, t.text_dim);
    }
    resp.widget_info(|| egui::WidgetInfo::selected(if on.is_some() { egui::Role::CheckBox } else { egui::Role::DisclosureTriangle }, true, on.unwrap_or(unfolded), label));
    let clicked = resp.clicked();
    let on_box = clicked && on.is_some() && resp.interact_pointer_pos().is_some_and(|p| p.x < rect.left() + 26.0);
    (on_box, clicked && !on_box)
}

/// Preview, Reverter and Aplicar. Returns whether the session ended (applied).
fn footer(app: &mut PhotocraftApp, ui: &mut egui::Ui, edit: &mut FxEdit) -> bool {
    widgets::hairline(ui);
    ui.add_space(8.0);
    let dirty = edit.dirty();
    let mut applied = false;
    ui.horizontal_wrapped(|ui| {
        let mut preview = edit.f.get("preview").and_then(Value::as_bool).unwrap_or(true);
        let label = crate::i18n::tr_ctx(crate::i18n::current(), "studio", "Preview");
        if widgets::checkbox(ui, &mut preview, label).changed() {
            edit.f.insert("preview".into(), json!(preview));
        }
        ui.add_enabled_ui(dirty, |ui| {
            if crate::studio::right_panel::outlined_button(ui, None, tl!("Revert"), 0.0).clicked() {
                let open = edit.open.take();
                edit.f = edit.original.clone();
                edit.open = open;
            }
            if widgets::primary_button(ui, tl!("Apply"), 0.0).clicked() {
                match super::confirm(app, &edit.f) {
                    Ok(_) => applied = true,
                    Err(e) => {
                        app.ui.status = e;
                        app.ui.status_error = true;
                    }
                }
            }
        });
    });
    applied
}
