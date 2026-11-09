//! File › Create with AI: the user gives an image and Codex rebuilds it as an editable document,
//! layer by layer, while they watch. The app only sets the stage and shows the work: Codex does the
//! rebuilding through the same MCP tools as any chat ([`RECIPE`] tells it how).
//!
//! A run makes a new document with the image as a hidden, locked reference layer, starts a new
//! conversation and locks the editor (only the Codex dock and the window's buttons stay live) until
//! the turn ends or the user stops it. Codex's commands arrive one at a time, even inside a batch,
//! so [`hold`] paces them and each one can be seen; the cursor only shows where the last one
//! landed, Codex doesn't use the mouse.

use egui::{Align2, Color32, Pos2, Rect, RichText, Sense, Stroke, Ui, pos2, vec2};
use serde_json::{Value, json};

use crate::PhotocraftApp;
use crate::codex_panel;
use crate::theme::{self, Tokens};

/// The model a run starts on: the one this recipe was proven with. The dialog can change it.
const DEFAULT_MODEL: &str = "gpt-6.1-sol";
/// Seconds between Codex's edits during a run, so they appear one by one.
const PACE: f64 = 0.12;
/// The icon of the feature, here and on Home.
pub(crate) const ICON: &str = "wand-sparkles";
pub(crate) const COMMAND: &str = "file.createWithAi";

/// The stages Codex announces with a `[[n]]` tag, in order.
const STEPS: [&str; 6] = ["Analyzing the image", "Creating the text", "Generating the images", "Applying the effects", "Organizing the layers", "Checking the result"];

/// How Codex does it, sent with the image. Keep the tags in step with [`STEPS`].
const RECIPE: &str = r#"Rebuild the attached reference image as an editable document in the active document, which is already open at the image's size. Its only layer, "{reference}", is the same image, hidden and locked: leave it hidden and locked at the bottom of the stack. Work on your own until it is finished: don't ask the user anything. Name layers and groups in {language}, and write your messages in {language}.

Progress: begin each message with the tag of the stage you are starting, once per stage and in order: [[1]] analyzing, [[2]] text, [[3]] images, [[4]] effects, [[5]] organizing layers, [[6]] checking. Keep messages to a sentence.

1. Analyze. Find the panels, colours, margins and alignment. Zoom into each region with `render_preview` and its `region` parameter to read the text exactly, and note what passes in front of what and which elements repeat. Never duplicate or close documents to zoom. Flag text you can't read in your last message; never invent it.

2. Text. Every text is a real type layer (`type.create`, `type.setStyle`): titles, prices, descriptions and notes, with the right accents, line breaks, size, weight, colour and spacing. Where the font isn't available use the closest one and say so at the end. Dividers, dotted lines, tables and bands are separate shapes (`shape.create`).

3. Images. Generate each independent element separately with your image generation tool, on a transparent background, guided by the reference: shape, angle, perspective, colours, texture and light. Never cut images out of the reference. A repeated element is generated once and duplicated. A logo that works as one piece may include its lettering. Each generated image arrives in the document by itself as a new layer: move and scale it into place, never place it again.

4. Effects. Keep effects apart from images where possible, so they can be changed without generating again: blur applied in the editor, glows and reflections as their own layers with gradients and blend modes, shadows as adjustable styles or layers, backgrounds and panels as editable shapes and fills.

5. Layers. Group by function and section with clear names, in reading order, while the stacking follows the reference. Each repeated item gets its own subgroup.

6. Check. Compare `render_preview` of the whole document and of the key regions with the reference. Check text and prices again, fix what is hidden, cropped or stacked wrongly, and check the edges of transparent images. Repeat until it matches.

Don't save or close the document: the user saves it when you finish. End with a brief note of what you checked and what remains an approximation."#;

/// Sent, with the same recipe, when a run continues on another account or after a limit.
const RESUME: &str = "This job was interrupted, and you don't have the earlier conversation. The active document already holds part of the work: inspect it with `doc_inspect` and `render_preview`, compare it with the attached reference, keep what is right and finish only what is missing. Start with the tag of the stage you continue from.\n\n";

#[derive(Default)]
pub struct CreateAi {
    dialog: Option<Dialog>,
    run: Option<Run>,
    /// No edit from Codex runs before this time.
    hold_until: f64,
}

#[derive(Default)]
struct Dialog {
    image: Option<Picked>,
    error: Option<String>,
    /// The model chosen here; `None` until the user picks one.
    model: Option<String>,
}

struct Picked {
    path: String,
    size: (u32, u32),
    thumb: Option<egui::TextureHandle>,
}

#[derive(Clone, Debug, PartialEq)]
enum Stage {
    /// The document is ready; Codex isn't signed in or connected yet.
    Waiting,
    Working,
    /// The account ran out: waiting for another one (or another try).
    Paused,
    /// Stop was pressed; Codex is ending its turn.
    Stopping,
    Stopped,
    Done,
    Failed(String),
}

struct Run {
    path: String,
    doc: photocraft_doc::DocId,
    stage: Stage,
    model: Option<String>,
    /// The model to go back to at the end, when the run changed it.
    restore_model: Option<String>,
    /// Sent once: later sends are resumes.
    sent: bool,
    started: f64,
    cursor: Cursor,
}

/// The cursor that follows the edits, in document coordinates.
#[derive(Default)]
struct Cursor {
    at: Option<[f32; 2]>,
    target: Option<[f32; 2]>,
    label: &'static str,
    /// When it last landed, for the ring.
    landed: f64,
}

impl CreateAi {
    /// The editor is locked: a run is going.
    pub(crate) fn locked(&self) -> bool {
        self.run.as_ref().is_some_and(|r| matches!(r.stage, Stage::Waiting | Stage::Working | Stage::Paused | Stage::Stopping))
    }

    pub(crate) fn dialog_open(&self) -> bool {
        self.dialog.is_some()
    }
}

/// File › Create with AI.
pub(crate) fn open(app: &mut PhotocraftApp) {
    if app.create_ai.locked() {
        return;
    }
    app.create_ai.dialog = Some(Dialog::default());
    // Start now, so the models are there by the time the user looks for them.
    codex_panel::ensure_started(app);
}

/// A file the dialog can take: Codex only looks at these formats.
fn is_supported(path: &str) -> bool {
    codex_panel::is_reference_image(path)
}

/// Take `path` as the dialog's image.
fn pick(app: &mut PhotocraftApp, ctx: &egui::Context, path: String) {
    let picked = load(ctx, path);
    if let Some(d) = app.create_ai.dialog.as_mut() {
        match picked {
            Ok(image) => {
                d.image = Some(image);
                d.error = None;
            }
            Err(e) => d.error = Some(e.to_string()),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn load(ctx: &egui::Context, path: String) -> Result<Picked, &'static str> {
    if !is_supported(&path) {
        return Err(tl!("This file isn't a PNG, JPEG, WebP or GIF image."));
    }
    let unreadable = tl!("Could not read this image.");
    let bytes = std::fs::read(&path).map_err(|_| unreadable)?;
    // Decoded, not probed: there is no header-only size for PNG and JPEG.
    let size = photocraft_codecs::decode(&bytes).map_err(|_| unreadable)?.dimensions();
    let thumb = crate::home_recent_ui::load_thumbnail_image_bounded(&path, bytes.len() as u64).map(|img| ctx.load_texture("create-ai-reference", img, egui::TextureOptions::LINEAR));
    Ok(Picked { path, size, thumb })
}

/// Codex doesn't run on the web.
#[cfg(target_arch = "wasm32")]
fn load(_ctx: &egui::Context, _path: String) -> Result<Picked, &'static str> {
    Err(tl!("Could not read this image."))
}

/// Dropped files: the dialog's image while it is open; nothing at all while the editor is locked.
pub(crate) fn take_drop(app: &mut PhotocraftApp, ctx: &egui::Context, files: &[egui::DroppedFileHandle]) -> bool {
    if app.create_ai.locked() {
        return true;
    }
    if !app.create_ai.dialog_open() {
        return false;
    }
    if let Some(file) = files.iter().find(|f| f.path().is_absolute()) {
        pick(app, ctx, file.path().to_string_lossy().into_owned());
    }
    true
}

/// Keep the keyboard from the editor's shortcuts while the dialog or a run has the app. A text field
/// (the chat's) keeps its keys; Esc still closes the dialog.
pub(crate) fn filter_keys(app: &PhotocraftApp, ctx: &egui::Context) {
    let dialog = app.create_ai.dialog_open();
    if (dialog || app.create_ai.locked()) && !ctx.egui_wants_keyboard_input() {
        ctx.input_mut(|i| i.events.retain(|e| !matches!(e, egui::Event::Key { key, .. } if !(dialog && *key == egui::Key::Escape))));
    }
}

/// How long to hold Codex's next request; `None` when it can run now.
pub(crate) fn hold(app: &PhotocraftApp, now: f64) -> Option<f64> {
    (app.create_ai.hold_until > now).then(|| app.create_ai.hold_until - now)
}

/// What the user sees for an edit; `None` for commands that only read.
fn edit_label(command: &str) -> Option<&'static str> {
    let area = command.split('.').next().unwrap_or(command);
    Some(match (area, command) {
        ("type", _) => tl!("Writing the text"),
        ("shape" | "vector", _) => tl!("Drawing a shape"),
        ("filter", _) => tl!("Applying an effect"),
        ("edit", "edit.transform") | ("layer", "layer.moveTo") => tl!("Positioning"),
        ("file", "file.placeEmbedded") => tl!("Placing an image"),
        ("layer", _) => tl!("Adjusting a layer"),
        ("edit" | "image" | "select", _) => tl!("Editing"),
        _ => return None,
    })
}

/// Where in the document a command's own parameters point, if they say.
fn point_in(params: &Value) -> Option<[f32; 2]> {
    let num = |key: &str| params.get(key).and_then(Value::as_f64).map(|v| v as f32);
    if let (Some(x), Some(y)) = (num("x"), num("y")) {
        let (w, h) = (num("width").unwrap_or(0.0), num("height").unwrap_or(0.0));
        return Some([x + w / 2.0, y + h / 2.0]);
    }
    let rect = params.get("rect").or_else(|| params.get("bounds"))?.as_array()?;
    let side = |i: usize| rect.get(i).and_then(Value::as_f64).map(|v| v as f32);
    Some([(side(0)? + side(2)?) / 2.0, (side(1)? + side(3)?) / 2.0])
}

/// The middle of the active layer, in document coordinates.
fn active_layer_center(app: &PhotocraftApp) -> Option<[f32; 2]> {
    let st = app.session.active()?;
    let layer = st.doc.layer(st.active_layer?)?;
    let b = photocraft_compose::layer_bounds(layer, st.doc.bounds());
    (b.width() > 0 && b.height() > 0).then(|| [(b.x0 + b.x1) as f32 / 2.0, (b.y0 + b.y1) as f32 / 2.0])
}

/// A control request just ran: during a run, pace the next one and send the cursor to this edit.
pub(crate) fn observe(app: &mut PhotocraftApp, ctx: &egui::Context, method: &str, params: &Value) {
    if method != "engine.execute" || !matches!(app.create_ai.run.as_ref().map(|r| &r.stage), Some(Stage::Working)) {
        return;
    }
    let Some(label) = params.get("command").and_then(Value::as_str).and_then(edit_label) else { return };
    let target = params.get("params").and_then(point_in).or_else(|| active_layer_center(app));
    let now = ctx.input(|i| i.time);
    app.create_ai.hold_until = now + PACE;
    if let Some(run) = app.create_ai.run.as_mut() {
        run.cursor.label = label;
        if target.is_some() {
            run.cursor.target = target;
            run.cursor.landed = now;
        }
    }
    ctx.request_repaint();
}

/// The dialog's Create button: makes the document and starts the run.
fn start(app: &mut PhotocraftApp, ctx: &egui::Context) {
    let Some(dialog) = app.create_ai.dialog.as_mut() else { return };
    let model = dialog.model.clone();
    let Some(image) = dialog.image.take() else { return };
    let path = image.path.clone();
    if app.studio.classic {
        // The run shows in the studio shell's dock.
        crate::studio::toggle_classic(app);
    }
    codex_panel::ensure_started(app);
    let made = if app.codex.is_running() { new_document(app, &path, image.size) } else { Err(codex_panel::last_error(app).unwrap_or_else(|| tl!("Codex stopped.").to_string())) };
    match made {
        Ok(doc) => {
            app.create_ai.dialog = None;
            app.codex.open = true;
            codex_panel::new_chat(app);
            let model = model.or_else(|| app.codex.has_model(DEFAULT_MODEL).then(|| DEFAULT_MODEL.to_string()));
            let started = ctx.input(|i| i.time);
            app.create_ai.run = Some(Run { path, doc, stage: Stage::Waiting, model, restore_model: None, sent: false, started, cursor: Cursor::default() });
        }
        Err(e) => {
            if let Some(d) = app.create_ai.dialog.as_mut() {
                d.image = Some(image);
                d.error = Some(e);
            }
        }
    }
}

/// A new transparent document at the image's size with the image as its hidden, locked reference.
fn new_document(app: &mut PhotocraftApp, path: &str, (width, height): (u32, u32)) -> Result<photocraft_doc::DocId, String> {
    let name = std::path::Path::new(path).file_stem().map_or_else(|| tl!("Untitled").to_string(), |n| n.to_string_lossy().into_owned());
    app.run("file.new", json!({"width": width, "height": height, "name": name, "background": "transparent"}))?;
    let doc = app.session.active().map(|st| st.doc.id).ok_or_else(|| tl!("Could not read this image.").to_string())?;
    app.run("file.placeEmbedded", json!({"path": path}))?;
    let layer = app.session.active().and_then(|st| st.active_layer).ok_or_else(|| tl!("Could not read this image.").to_string())?;
    app.run("layer.setProps", json!({"layer": layer.0, "name": tl!("Reference"), "visible": false, "locks": {"all": true}}))?;
    Ok(doc)
}

/// The recipe in the UI's language.
fn recipe(resume: bool) -> String {
    let language = crate::i18n::current().code();
    let text = RECIPE.replace("{reference}", tl!("Reference")).replace("{language}", language);
    if resume { format!("{RESUME}{text}") } else { text }
}

/// Send the recipe, on the run's model.
fn send(app: &mut PhotocraftApp) {
    let Some(run) = app.create_ai.run.as_ref() else { return };
    let (path, resume, wanted) = (run.path.clone(), run.sent, run.model.clone());
    let previous = app.codex.switch_model(wanted);
    let name = std::path::Path::new(&path).file_name().map_or_else(|| path.clone(), |n| n.to_string_lossy().into_owned());
    let shown = crate::i18n::fmt(tl!("Turning {name} into an editable document"), &[("name", &name)]);
    codex_panel::send_as(app, shown, recipe(resume), vec![path]);
    if let Some(run) = app.create_ai.run.as_mut() {
        run.sent = true;
        run.stage = Stage::Working;
        if run.restore_model.is_none() {
            run.restore_model = previous;
        }
    }
}

/// Move the run along what Codex reports. Every frame.
pub(crate) fn tick(app: &mut PhotocraftApp) {
    let Some(stage) = app.create_ai.run.as_ref().map(|r| r.stage.clone()) else { return };
    let (running, signed_in, busy, limit) = (app.codex.is_running(), app.codex.is_signed_in(), app.codex.is_busy(), app.codex.limit_reached());
    let next = match stage {
        Stage::Waiting | Stage::Paused if !running => Some(Stage::Failed(stopped_reason(app))),
        Stage::Waiting if signed_in => {
            send(app);
            None
        }
        // Another account (the switch clears the limit), signed in: carry on from the document.
        Stage::Paused if signed_in && !limit => {
            send(app);
            None
        }
        Stage::Working if !running => Some(Stage::Failed(stopped_reason(app))),
        Stage::Working if !busy && limit => Some(Stage::Paused),
        Stage::Working if !busy => Some(match codex_panel::last_turn_error(app) {
            Some(e) => Stage::Failed(e),
            None => Stage::Done,
        }),
        Stage::Stopping if !busy || !running => Some(Stage::Stopped),
        _ => None,
    };
    if let Some(next) = next {
        let ended = !matches!(next, Stage::Paused);
        if let Some(run) = app.create_ai.run.as_mut() {
            run.stage = next;
        }
        if ended {
            restore_model(app);
        }
    }
}

fn stopped_reason(app: &PhotocraftApp) -> String {
    codex_panel::last_error(app).unwrap_or_else(|| tl!("Codex stopped.").to_string())
}

fn restore_model(app: &mut PhotocraftApp) {
    if let Some(model) = app.create_ai.run.as_mut().and_then(|r| r.restore_model.take()) {
        app.codex.switch_model(Some(model));
    }
}

/// The last stage Codex announced, from its messages.
fn stage_of(app: &PhotocraftApp) -> Option<usize> {
    app.codex.assistant_texts().rev().find_map(tag_of)
}

/// The stage of a message starting with its `[[n]]` tag (1-based), as an index.
fn tag_of(text: &str) -> Option<usize> {
    let rest = text.trim_start().strip_prefix("[[")?;
    let (n, _) = rest.split_once("]]")?;
    let n = n.trim().parse::<usize>().ok()?;
    (1..=STEPS.len()).contains(&n).then(|| n - 1)
}

/// A message without its stage tag, for showing.
pub(crate) fn strip_tag(text: &str) -> &str {
    if tag_of(text).is_some()
        && let Some((_, rest)) = text.split_once("]]")
    {
        return rest.trim_start();
    }
    text
}

/// The dialog and, during a run, the lock, the progress bar and the cursor. Every frame.
pub(crate) fn show(app: &mut PhotocraftApp, ctx: &egui::Context) {
    tick(app);
    if app.create_ai.dialog_open() {
        dialog(app, ctx);
    }
    if app.create_ai.run.is_some() {
        if app.create_ai.locked() {
            lock(app, ctx);
            cursor(app, ctx);
        }
        progress(app, ctx);
    }
}

// ---- the dialog ----------------------------------------------------------------------------------

/// What the dialog's controls asked for, applied after drawing.
enum Ask {
    Pick,
    Create,
    Close,
    Model(String),
}

fn dialog(app: &mut PhotocraftApp, ctx: &egui::Context) {
    let t = Tokens::get(ctx);
    let mut asks = Vec::new();
    let modal = egui::Modal::new(egui::Id::unique("create-ai-dialog")).frame(egui::Frame::new().fill(t.card).stroke(Stroke::new(1.0, t.card_border)).corner_radius(14.0).inner_margin(22.0)).show(ctx, |ui| {
        ui.set_width(520.0_f32.min(ctx.content_rect().width() - 80.0));
        ui.horizontal(|ui| {
            ui.add(crate::icons::image(ICON, 22.0, t.accent));
            ui.label(RichText::new(tl!("Create with AI")).font(theme::semibold(20.0)).color(t.text));
            ui.with_layout(crate::direction::trailing(), |ui| {
                if crate::icons::button(ui, "x", 26.0, false, tl!("Close")).clicked() {
                    asks.push(Ask::Close);
                }
            });
        });
        ui.add_space(4.0);
        ui.add(egui::Label::new(RichText::new(tl!("Send an image and Codex rebuilds it as an editable document, with separate layers, real text and new images.")).size(13.5).color(t.text_dim)).wrap());
        ui.add_space(14.0);
        let d = app.create_ai.dialog.as_ref();
        let image = d.and_then(|d| d.image.as_ref());
        if drop_area(ui, &t, image).clicked() {
            asks.push(Ask::Pick);
        }
        if let Some(error) = d.and_then(|d| d.error.as_ref()) {
            ui.add_space(6.0);
            ui.add(egui::Label::new(RichText::new(error).size(13.0).color(t.danger)).wrap());
        }
        ui.add_space(12.0);
        if app.codex.has_models() {
            ui.horizontal(|ui| {
                ui.label(RichText::new(tl!("Model")).size(13.0).color(t.text_dim));
                let chosen = d.and_then(|d| d.model.clone()).or_else(|| app.codex.has_model(DEFAULT_MODEL).then(|| DEFAULT_MODEL.to_string()));
                if let Some(id) = codex_panel::model_picker(ui, &t, &app.codex, chosen.as_deref()) {
                    asks.push(Ask::Model(id));
                }
            });
            ui.add_space(10.0);
        }
        ui.horizontal_top(|ui| {
            ui.add(crate::icons::image("clock", 14.0, t.text_dim));
            ui.add(egui::Label::new(RichText::new(tl!("This can take over an hour and use a lot of your plan. The editor stays locked until it finishes, and you can stop whenever you want.")).size(12.5).color(t.text_dim)).wrap());
        });
        ui.add_space(16.0);
        if create_button(ui, &t, image.is_some()).clicked() {
            asks.push(Ask::Create);
        }
    });
    if modal.should_close() {
        asks.push(Ask::Close);
    }
    for ask in asks {
        match ask {
            Ask::Pick => {
                let c = ctx.clone();
                let picked = app.pick_paths(move |app, paths| {
                    if let Some(path) = paths.into_iter().next() {
                        pick(app, &c, path);
                    }
                });
                if let Err(e) = picked
                    && e != crate::file_dialog::CANCELLED
                    && let Some(d) = app.create_ai.dialog.as_mut()
                {
                    d.error = Some(e);
                }
            }
            Ask::Create => start(app, ctx),
            Ask::Close => app.create_ai.dialog = None,
            Ask::Model(id) => {
                if let Some(d) = app.create_ai.dialog.as_mut() {
                    d.model = Some(id);
                }
            }
        }
    }
}

/// The dashed drop area; with an image, its preview, name and size.
fn drop_area(ui: &mut Ui, t: &Tokens, image: Option<&Picked>) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 230.0), Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    let label = tl!("Choose an image");
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
    let hov = resp.hovered() || ui.input(|i| !i.raw.hovered_files.is_empty());
    let p = ui.painter();
    p.rect_filled(r, 12.0, if hov { t.hover } else { t.field });
    p.extend(egui::Shape::dashed_line(&crate::canvas::rounded_outline(r.shrink(1.0), 12.0), Stroke::new(1.0, if hov { t.accent } else { t.field_border }), 6.0, 5.0));
    if let Some(image) = image {
        let name = std::path::Path::new(&image.path).file_name().map_or_else(|| image.path.clone(), |n| n.to_string_lossy().into_owned());
        let line = format!("{name} · {} × {} px", image.size.0, image.size.1);
        let text = crate::tab_strip::elided(ui, &line, theme::medium(12.5), t.text_dim, r.width() - 24.0);
        let room = Rect::from_min_max(r.min + vec2(12.0, 12.0), r.max - vec2(12.0, 12.0 + text.size().y + 6.0));
        if let Some(thumb) = &image.thumb {
            let s = (room.width() / thumb.size_vec2().x).min(room.height() / thumb.size_vec2().y);
            let fit = Rect::from_center_size(room.center(), thumb.size_vec2() * s);
            p.image(thumb.id(), fit, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
        }
        p.galley(pos2(r.center().x - text.size().x / 2.0, r.bottom() - 12.0 - text.size().y), text, t.text_dim);
    } else {
        let c = r.center();
        crate::icons::paint(ui, Rect::from_center_size(c - vec2(0.0, 28.0), vec2(30.0, 30.0)), "image", 30.0, if hov { t.accent } else { t.text_dim });
        p.text(c + vec2(0.0, 6.0), Align2::CENTER_CENTER, tl!("Drag your image here"), theme::medium(14.0), t.text);
        p.text(c + vec2(0.0, 28.0), Align2::CENTER_CENTER, tl!("or click to choose a file"), theme::medium(13.0), t.accent);
    }
    resp
}

/// The big amber button with the wand and a soft glow.
fn create_button(ui: &mut Ui, t: &Tokens, enabled: bool) -> egui::Response {
    let label = tl!("Create with AI");
    let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 54.0), if enabled { Sense::click() } else { Sense::hover() });
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, label));
    let p = ui.painter();
    if enabled {
        let glow = if resp.hovered() { 90 } else { 55 };
        p.add(egui::Shadow { offset: [0, 4], blur: 24, spread: 0, color: t.primary_bg.gamma_multiply_u8(glow) }.as_shape(r, 12.0));
    }
    let fill = match (enabled, resp.hovered()) {
        (false, _) => t.field_border,
        (true, true) => t.accent,
        (true, false) => t.primary_bg,
    };
    let ink = if enabled { t.primary_text } else { t.text_faint };
    p.rect_filled(r, 12.0, fill);
    let g = p.layout_no_wrap(label.to_string(), theme::semibold(17.0), ink);
    let x = r.center().x - (g.size().x + 32.0) / 2.0;
    crate::icons::paint(ui, Rect::from_center_size(pos2(x + 11.0, r.center().y), vec2(22.0, 22.0)), ICON, 22.0, ink);
    p.galley(pos2(x + 32.0, r.center().y - g.size().y / 2.0), g, ink);
    if enabled { resp.on_hover_cursor(egui::CursorIcon::PointingHand) } else { resp }
}

// ---- the run -------------------------------------------------------------------------------------

/// Take the pointer over everything but the Codex dock and the caption buttons. The title bar still
/// moves the window: a run takes long enough that the user will want it out of the way.
fn lock(app: &PhotocraftApp, ctx: &egui::Context) {
    let screen = ctx.content_rect();
    let bar = crate::titlebar::bar_height(app, ctx, if crate::studio::style::compact(ctx) { 44.0 } else { crate::studio::style::TOP_BAR_H });
    let (_, cap_r) = crate::titlebar::reserved(app, ctx);
    let dock = app.codex.dock();
    let body = Rect::from_min_max(pos2(screen.left(), screen.top() + bar), screen.max);
    let mut rects = vec![(Rect::from_min_max(screen.min, pos2(screen.right() - cap_r, screen.top() + bar)), true)];
    match dock {
        Some(dock) => {
            rects.push((Rect::from_min_max(body.min, pos2(dock.left(), body.bottom())), false));
            rects.push((Rect::from_min_max(pos2(dock.right(), body.top()), body.max), false));
        }
        None => rects.push((body, false)),
    }
    for (i, (rect, title)) in rects.into_iter().enumerate() {
        egui::Area::new(egui::Id::unique(("create-ai-lock", i))).order(egui::Order::Foreground).fixed_pos(rect.min).constrain(false).show(ctx, |ui| {
            let resp = ui.allocate_rect(Rect::from_min_size(rect.min, rect.size()), Sense::click_and_drag());
            if title && resp.drag_started_by(egui::PointerButton::Primary) {
                crate::titlebar::start_os_move(ctx);
            }
        });
    }
}

/// The pointer over the document, with what it is doing; drawn over everything, takes nothing.
fn cursor(app: &mut PhotocraftApp, ctx: &egui::Context) {
    let Some(xf) = crate::canvas::ViewXform::active(app) else { return };
    let Some(run) = app.create_ai.run.as_mut() else { return };
    let c = &mut run.cursor;
    let Some(target) = c.target else { return };
    let dt = ctx.input(|i| i.stable_dt).min(0.1);
    let k = 1.0 - (-dt * 9.0).exp();
    let at = c.at.unwrap_or(target);
    let at = [at[0] + (target[0] - at[0]) * k, at[1] + (target[1] - at[1]) * k];
    c.at = Some(at);
    let (pos, goal) = (xf.to_screen(at[0], at[1]), xf.to_screen(target[0], target[1]));
    let now = ctx.input(|i| i.time);
    let ring = ((now - c.landed) / 0.45) as f32;
    // Only while something moves: a run can last an hour, on any computer.
    if pos.distance(goal) > 0.5 || ring < 1.0 {
        ctx.request_repaint();
    }
    let t = Tokens::get(ctx);
    let p = ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, egui::Id::unique("create-ai-cursor")));
    if ring < 1.0 && pos.distance(goal) < 12.0 {
        p.circle_stroke(goal, 6.0 + ring * 22.0, Stroke::new(1.5, t.accent.gamma_multiply(1.0 - ring)));
    }
    paint_pointer(&p, pos, t.accent, t.primary_text);
    let text = p.layout_no_wrap(c.label.to_string(), theme::medium(12.0), t.primary_text);
    let chip = Rect::from_min_size(pos + vec2(16.0, 18.0), text.size() + vec2(14.0, 8.0));
    p.rect_filled(chip, 7.0, t.accent);
    p.galley(chip.min + vec2(7.0, 4.0), text, t.primary_text);
}

/// An arrow pointer with its tip at `tip`.
fn paint_pointer(p: &egui::Painter, tip: Pos2, fill: Color32, line: Color32) {
    let pts = [(0.0, 0.0), (0.0, 17.0), (4.5, 13.0), (8.0, 20.0), (11.0, 18.5), (7.5, 12.0), (13.0, 12.0)].map(|(x, y)| tip + vec2(x, y)).to_vec();
    p.add(egui::Shape::convex_polygon(pts[..3].to_vec(), fill, Stroke::NONE));
    p.add(egui::Shape::convex_polygon(vec![pts[0], pts[2], pts[5], pts[6]], fill, Stroke::NONE));
    p.add(egui::Shape::convex_polygon(vec![pts[2], pts[3], pts[4], pts[5]], fill, Stroke::NONE));
    let mut outline = pts;
    outline.push(tip);
    p.add(egui::Shape::line(outline, Stroke::new(1.0, line)));
}

/// What the bar's buttons asked for, applied after drawing.
enum Bar {
    Stop,
    Retry,
    Save,
    Discard,
    Dismiss,
}

/// The bar at the foot of the canvas: the stage and its way out.
fn progress(app: &mut PhotocraftApp, ctx: &egui::Context) {
    let t = Tokens::get(ctx);
    let Some(run) = app.create_ai.run.as_ref() else { return };
    let stage = run.stage.clone();
    let minutes = ((ctx.input(|i| i.time) - run.started) / 60.0).max(0.0) as u32;
    let step = stage_of(app);
    let canvas = if app.last_canvas_rect.is_positive() { app.last_canvas_rect } else { ctx.content_rect() };
    let mut clicked = None;
    egui::Area::new(egui::Id::unique("create-ai-progress")).order(egui::Order::Tooltip).pivot(Align2::CENTER_BOTTOM).fixed_pos(pos2(canvas.center().x, canvas.bottom() - 16.0)).show(ctx, |ui| {
        egui::Frame::new().fill(t.card).stroke(Stroke::new(1.0, t.card_border)).corner_radius(12.0).inner_margin(egui::Margin::symmetric(14, 9)).shadow(egui::Shadow { offset: [0, 6], blur: 20, spread: 0, color: t.shadow }).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                bar_content(ui, &t, &stage, step, minutes, &mut clicked);
            });
        });
    });
    // The elapsed minutes stay right without keeping frames coming.
    if app.create_ai.locked() {
        ctx.request_repaint_after(std::time::Duration::from_secs(30));
    }
    match clicked {
        Some(Bar::Stop) => {
            if let Some(run) = app.create_ai.run.as_mut() {
                run.stage = Stage::Stopping;
            }
            app.codex.stop_turn();
        }
        Some(Bar::Retry) => {
            app.codex.clear_limit();
            send(app);
        }
        Some(Bar::Save) => {
            let doc = run_doc(app);
            let saved = doc.map(|doc| app.with_document(doc, |app| crate::menus::invoke(app, ctx, "file.saveAs", json!({})).map(|_| ())));
            if let Some(Err(e)) = saved {
                app.ui.status = e;
                app.ui.status_error = true;
            }
            app.create_ai.run = None;
        }
        Some(Bar::Discard) => {
            if let Some(doc) = run_doc(app)
                && let Err(e) = app.with_document(doc, |app| app.run("file.close", json!({})).map(|_| ()))
            {
                app.ui.status = e;
                app.ui.status_error = true;
            }
            app.create_ai.run = None;
        }
        Some(Bar::Dismiss) => app.create_ai.run = None,
        None => {}
    }
}

fn run_doc(app: &PhotocraftApp) -> Option<photocraft_doc::DocId> {
    app.create_ai.run.as_ref().map(|r| r.doc)
}

fn bar_content(ui: &mut Ui, t: &Tokens, stage: &Stage, step: Option<usize>, minutes: u32, clicked: &mut Option<Bar>) {
    let mut button = |ui: &mut Ui, label: &str, primary: bool, then: fn() -> Bar| {
        let b = if primary { crate::widgets::primary_button(ui, label, 0.0) } else { crate::widgets::secondary_button(ui, label, 0.0) };
        if b.clicked() {
            *clicked = Some(then());
        }
    };
    let text = |ui: &mut Ui, s: &str| {
        ui.add(egui::Label::new(RichText::new(s).font(theme::medium(13.0)).color(t.text)).wrap_mode(egui::TextWrapMode::Extend));
    };
    match stage {
        Stage::Waiting => {
            ui.add(egui::Spinner::new().size(16.0).color(t.accent));
            text(ui, tl!("Waiting for Codex: sign in on the panel if it asks"));
            button(ui, tl!("Stop"), false, || Bar::Stop);
        }
        Stage::Working => {
            ui.add(egui::Spinner::new().size(16.0).color(t.accent));
            match step.and_then(|s| STEPS.get(s).map(|name| (s, name))) {
                Some((s, name)) => {
                    let at = crate::i18n::fmt(tl!("Step {n} of {total}"), &[("n", &(s + 1).to_string()), ("total", &STEPS.len().to_string())]);
                    ui.label(RichText::new(at).font(theme::medium(12.5)).color(t.text_dim));
                    text(ui, crate::i18n::t(name));
                }
                None => text(ui, tl!("Getting started")),
            }
            if minutes > 0 {
                ui.label(RichText::new(crate::i18n::fmt(tl!("{n} min"), &[("n", &minutes.to_string())])).font(theme::medium(12.5)).color(t.text_dim));
            }
            button(ui, tl!("Stop"), false, || Bar::Stop);
        }
        Stage::Paused => {
            ui.add(crate::icons::image("info", 16.0, t.accent));
            text(ui, tl!("Usage limit reached: switch accounts on the panel to continue"));
            button(ui, tl!("Try again"), false, || Bar::Retry);
            button(ui, tl!("Stop"), false, || Bar::Stop);
        }
        Stage::Stopping => {
            ui.add(egui::Spinner::new().size(16.0).color(t.accent));
            text(ui, tl!("Stopping…"));
        }
        Stage::Stopped => {
            text(ui, tl!("Stopped. Keep what was done?"));
            button(ui, tl!("Keep"), false, || Bar::Dismiss);
            button(ui, tl!("Discard"), false, || Bar::Discard);
        }
        Stage::Done => {
            ui.add(crate::icons::image(ICON, 16.0, t.accent));
            text(ui, tl!("Your editable document is ready"));
            button(ui, tl!("Close"), false, || Bar::Dismiss);
            button(ui, tl!("Save PSD…"), true, || Bar::Save);
        }
        Stage::Failed(e) => {
            ui.set_max_width(520.0);
            ui.add(egui::Label::new(RichText::new(e).size(13.0).color(t.danger)).wrap());
            button(ui, tl!("Close"), false, || Bar::Dismiss);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::kittest::Queryable;

    #[test]
    fn stage_tags_are_read_and_hidden() {
        assert_eq!(tag_of("[[2]] Creating text"), Some(1));
        assert_eq!(tag_of("  [[6]]"), Some(5));
        assert_eq!(tag_of("[[0]] no"), None);
        assert_eq!(tag_of("[[7]] no"), None);
        assert_eq!(tag_of("[[x]] no"), None);
        assert_eq!(tag_of("plain"), None);
        assert_eq!(strip_tag("[[2]] Creating text"), "Creating text");
        assert_eq!(strip_tag("[[9]] kept"), "[[9]] kept");
        assert_eq!(strip_tag("plain [[2]] text"), "plain [[2]] text");
    }

    #[test]
    fn edits_have_labels_and_reads_do_not() {
        assert!(edit_label("type.create").is_some());
        assert!(edit_label("layer.moveTo").is_some());
        assert!(edit_label("filter.blur.motionBlur").is_some());
        assert!(edit_label("document.inspect").is_none());
        assert!(edit_label("command.list").is_none());
    }

    #[test]
    fn a_command_can_say_where_it_lands() {
        assert_eq!(point_in(&json!({"x": 10, "y": 20})), Some([10.0, 20.0]));
        assert_eq!(point_in(&json!({"x": 10, "y": 20, "width": 100, "height": 40})), Some([60.0, 40.0]));
        assert_eq!(point_in(&json!({"rect": [0, 0, 100, 50]})), Some([50.0, 25.0]));
        assert_eq!(point_in(&json!({"x": 10})), None);
        assert_eq!(point_in(&json!({"rect": "bad"})), None);
        assert_eq!(point_in(&json!({"rect": [1]})), None);
    }

    fn png(dir: &std::path::Path) -> String {
        let path = dir.join("menu.png");
        let image = photocraft_codecs::Image::from_u8(40, 30, photocraft_codecs::ChannelLayout::Rgba, vec![200; 40 * 30 * 4]).unwrap();
        std::fs::write(&path, photocraft_codecs::encode(&image, photocraft_codecs::Format::Png, &photocraft_codecs::EncodeOptions::default()).unwrap()).unwrap();
        path.to_string_lossy().into_owned()
    }

    /// A file that isn't an image is refused, with a reason, and the run can't start.
    #[test]
    fn the_dialog_refuses_what_codex_cannot_see() {
        let mut h = egui_kittest::Harness::builder().with_size(vec2(1200.0, 800.0)).build_eframe(|cc| {
            crate::PhotocraftApp::setup_context(&cc.egui_ctx, Default::default());
            crate::PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default())
        });
        h.run_steps(2);
        h.state_mut().run("file.new", json!({"width": 64, "height": 48})).unwrap();
        crate::menus::invoke(h.state_mut(), &egui::Context::default(), COMMAND, json!({})).unwrap();
        h.run_steps(3);
        assert!(h.state().create_ai.dialog_open());
        let ctx = h.ctx.clone();
        pick(h.state_mut(), &ctx, "C:/nowhere/brief.pdf".into());
        h.run_steps(2);
        assert!(h.query_by_label_contains("isn't a PNG").is_some());
        let docs = h.state().session.documents().len();
        h.get_all_by_label("Create with AI").last().unwrap().click();
        h.run_steps(2);
        assert_eq!(h.state().session.documents().len(), docs, "no image, no run");
        assert!(h.state().create_ai.run.is_none());
    }

    fn running_app(dir: &std::path::Path) -> (crate::PhotocraftApp, std::sync::mpsc::Receiver<codex_panel::CodexRequest>, std::sync::mpsc::Sender<codex_panel::CodexEvent>, String) {
        let (tx, requests) = std::sync::mpsc::channel();
        let (events, rx) = std::sync::mpsc::channel();
        let mut app = crate::PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
        app.codex.connect(codex_panel::CodexLink { tx, rx });
        (app, requests, events, png(dir))
    }

    /// The image becomes a hidden, locked reference in a new document of its size; once signed in
    /// the recipe goes out with the image, on the recipe's model, and the old model comes back.
    #[test]
    fn a_run_goes_from_the_image_to_the_finished_document() {
        use codex_panel::{CodexEvent, CodexModel, CodexRequest};
        let dir = tempfile::tempdir().unwrap();
        let (mut app, requests, events, path) = running_app(dir.path());
        let ctx = egui::Context::default();
        let model = |id: &str| CodexModel { id: id.into(), name: id.into(), description: String::new() };
        events.send(CodexEvent::Models { models: vec![model(DEFAULT_MODEL), model("cheap")], current: Some("cheap".into()) }).unwrap();
        codex_panel::poll(&mut app);
        open(&mut app);
        pick(&mut app, &ctx, path.clone());
        start(&mut app, &ctx);
        assert!(app.create_ai.locked(), "{:?}", app.create_ai.dialog.as_ref().and_then(|d| d.error.clone()));
        let st = app.session.active().unwrap();
        assert_eq!((st.doc.size.width, st.doc.size.height), (40, 30));
        let reference = st.doc.layer(st.active_layer.unwrap()).unwrap();
        assert!(!reference.visible, "the reference stays hidden");
        // Signed out: it waits, without sending anything but the fresh start.
        assert_eq!(requests.try_recv().ok(), Some(CodexRequest::NewChat));
        tick(&mut app);
        assert!(requests.try_recv().is_err());
        events.send(CodexEvent::Account(Some("me@example.com".into()))).unwrap();
        codex_panel::poll(&mut app);
        tick(&mut app);
        assert_eq!(requests.try_recv().ok(), Some(CodexRequest::SetModel(DEFAULT_MODEL.into())));
        let Ok(CodexRequest::Send { text, files }) = requests.try_recv() else { panic!("no recipe") };
        assert_eq!(files, [path]);
        assert!(text.contains("[[1]]") && !text.contains("{language}") && !text.contains("{reference}"));
        // Edits are paced and move the cursor; reads aren't.
        let run_edit = json!({"command": "type.create", "params": {"x": 10, "y": 12}});
        observe(&mut app, &ctx, "engine.execute", &run_edit);
        assert!(hold(&app, 0.0).is_some());
        assert_eq!(app.create_ai.run.as_ref().unwrap().cursor.target, Some([10.0, 12.0]));
        app.create_ai.hold_until = 0.0;
        observe(&mut app, &ctx, "engine.execute", &json!({"command": "document.inspect"}));
        assert!(hold(&app, 0.0).is_none());
        // The stage shows from the message, not its tag.
        events.send(CodexEvent::AssistantDone { item: "a".into(), text: "[[3]] Images".into() }).unwrap();
        codex_panel::poll(&mut app);
        assert_eq!(stage_of(&app), Some(2));
        events.send(CodexEvent::TurnFinished).unwrap();
        codex_panel::poll(&mut app);
        tick(&mut app);
        assert!(!app.create_ai.locked(), "the end unlocks");
        assert_eq!(app.create_ai.run.as_ref().unwrap().stage, Stage::Done);
        assert_eq!(requests.try_recv().ok(), Some(CodexRequest::SetModel("cheap".into())), "the user's model comes back");
    }

    /// Out of usage the run waits; the document carries it to the next account.
    #[test]
    fn a_limit_pauses_and_another_account_resumes() {
        use codex_panel::{CodexEvent, CodexRequest};
        let dir = tempfile::tempdir().unwrap();
        let (mut app, requests, events, path) = running_app(dir.path());
        let ctx = egui::Context::default();
        events.send(CodexEvent::Account(Some("one@example.com".into()))).unwrap();
        codex_panel::poll(&mut app);
        open(&mut app);
        pick(&mut app, &ctx, path);
        start(&mut app, &ctx);
        tick(&mut app);
        while requests.try_recv().is_ok() {}
        events.send(CodexEvent::LimitReached).unwrap();
        events.send(CodexEvent::TurnFinished).unwrap();
        codex_panel::poll(&mut app);
        tick(&mut app);
        assert_eq!(app.create_ai.run.as_ref().unwrap().stage, Stage::Paused);
        assert!(app.create_ai.locked(), "still locked while it waits");
        app.codex.clear_limit();
        tick(&mut app);
        let Ok(CodexRequest::Send { text, .. }) = requests.try_recv() else { panic!("no resume") };
        assert!(text.starts_with("This job was interrupted"), "{text}");
        assert_eq!(app.create_ai.run.as_ref().unwrap().stage, Stage::Working);
    }

    /// Stop ends the turn, and only then the editor unlocks.
    #[test]
    fn stop_waits_for_the_turn_to_end() {
        use codex_panel::{CodexEvent, CodexRequest};
        let dir = tempfile::tempdir().unwrap();
        let (mut app, requests, events, path) = running_app(dir.path());
        let ctx = egui::Context::default();
        events.send(CodexEvent::Account(Some("one@example.com".into()))).unwrap();
        codex_panel::poll(&mut app);
        open(&mut app);
        pick(&mut app, &ctx, path);
        start(&mut app, &ctx);
        tick(&mut app);
        while requests.try_recv().is_ok() {}
        app.create_ai.run.as_mut().unwrap().stage = Stage::Stopping;
        app.codex.stop_turn();
        assert_eq!(requests.try_recv().ok(), Some(CodexRequest::Stop));
        tick(&mut app);
        assert!(app.create_ai.locked());
        events.send(CodexEvent::TurnFinished).unwrap();
        codex_panel::poll(&mut app);
        tick(&mut app);
        assert!(!app.create_ai.locked());
        assert_eq!(app.create_ai.run.as_ref().unwrap().stage, Stage::Stopped);
    }
}
