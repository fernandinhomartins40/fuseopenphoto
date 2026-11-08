//! Window › Codex: a chat with OpenAI Codex that drives this PhotoCraft.
//!
//! This crate only draws the chat and talks over channels, so it stays free of process I/O (and
//! keeps building for the web). The desktop app supplies [`Services::codex`](crate::Services):
//! it runs `codex app-server`, signed in with ChatGPT, with this window's control channel as an
//! MCP server, and turns the server's messages into [`CodexEvent`]s.
//!
//! The studio shell docks the chat on the right ([`dock`], mockup 1); the classic shell keeps the
//! floating window ([`show`]).

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};

use egui::{Color32, CornerRadius, Frame, Margin, Rect, RichText, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use serde_json::json;

use crate::PhotocraftApp;
use crate::theme::{self, Tokens};

/// What the chat asks the backend to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodexRequest {
    /// Start ChatGPT sign-in in the browser.
    SignIn,
    /// Sign out of ChatGPT.
    SignOut,
    /// Send a message (starts a conversation when there is none).
    Send(String),
    /// Interrupt the running turn.
    Stop,
    /// Forget the conversation; the next message starts a new one.
    NewChat,
}

/// What the backend reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodexEvent {
    /// Who is signed in (`None`: nobody).
    Account(Option<String>),
    /// Streamed text of the assistant message `item`.
    AssistantDelta {
        item: String,
        delta: String,
    },
    /// The finished text of the assistant message `item`.
    AssistantDone {
        item: String,
        text: String,
    },
    /// A tool call or other step, shown dimmed.
    Activity(String),
    /// An image Codex generated, saved at this path.
    Image(PathBuf),
    TurnStarted,
    TurnFinished,
    Error(String),
    /// The Codex process ended; the link is dead.
    Exited(String),
}

/// The two ends of a running backend.
pub struct CodexLink {
    pub tx: Sender<CodexRequest>,
    pub rx: Receiver<CodexEvent>,
}

/// Starts the backend. Called when the window first opens and again after Codex stopped.
pub type CodexStartFn = Box<dyn FnMut() -> Result<CodexLink, String>>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Entry {
    User(String),
    Assistant { item: String, text: String },
    /// Consecutive tool calls and other steps (the backend's raw lines), shown as one checklist.
    Steps(Vec<String>),
    /// A turn ended after this many steps.
    Done(usize),
    Error(String),
}

/// The dock's tabs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Tab {
    #[default]
    Chat,
    Activity,
}

/// What a message is about (the composer's scope menu).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Scope {
    #[default]
    Document,
    Layer,
    Selection,
}

impl Scope {
    const ALL: [Scope; 3] = [Scope::Document, Scope::Layer, Scope::Selection];

    fn label(self) -> &'static str {
        match self {
            Scope::Document => tl!("Document"),
            Scope::Layer => tl!("Layer"),
            Scope::Selection => tl!("Selection"),
        }
    }

    /// The message Codex gets: the user's text, narrowed to the scope.
    fn prompt(self, text: &str) -> String {
        match self {
            Scope::Document => text.to_string(),
            Scope::Layer => format!("Work on the active layer only. {text}"),
            Scope::Selection => format!("Work inside the current selection only. {text}"),
        }
    }
}

/// The chat window's state (not persisted).
#[derive(Default)]
pub struct CodexPanel {
    pub open: bool,
    link: Option<CodexLink>,
    entries: Vec<Entry>,
    input: String,
    busy: bool,
    /// `None` until the backend has said; `Some(None)` when nobody is signed in.
    account: Option<Option<String>>,
    /// Where the running (or last) turn's entries start.
    turn_start: usize,
    tab: Tab,
    scope: Scope,
}

impl CodexPanel {
    /// Apply one backend event; returns an image to place.
    fn apply(&mut self, event: CodexEvent) -> Option<PathBuf> {
        match event {
            CodexEvent::Account(account) => self.account = Some(account),
            CodexEvent::AssistantDelta { item, delta } => match self.entries.last_mut() {
                Some(Entry::Assistant { item: last, text }) if *last == item => text.push_str(&delta),
                _ => self.entries.push(Entry::Assistant { item, text: delta }),
            },
            CodexEvent::AssistantDone { item, text } => {
                if let Some(Entry::Assistant { text: shown, .. }) =
                    self.entries.iter_mut().rev().find(|e| matches!(e, Entry::Assistant { item: i, .. } if *i == item))
                {
                    *shown = text;
                } else if !text.is_empty() {
                    self.entries.push(Entry::Assistant { item, text });
                }
            }
            CodexEvent::Activity(line) => self.step(line),
            CodexEvent::Image(path) => return Some(path),
            CodexEvent::TurnStarted => {
                if !self.busy {
                    self.turn_start = self.entries.len();
                }
                self.busy = true;
            }
            CodexEvent::TurnFinished => self.finish_turn(),
            CodexEvent::Error(e) => self.entries.push(Entry::Error(e)),
            CodexEvent::Exited(why) => {
                self.entries.push(Entry::Error(why));
                self.link = None;
                self.busy = false;
                self.account = None;
            }
        }
        None
    }

    /// Add a step to the current checklist, or start one.
    fn step(&mut self, line: String) {
        match self.entries.last_mut() {
            Some(Entry::Steps(lines)) => lines.push(line),
            _ => self.entries.push(Entry::Steps(vec![line])),
        }
    }

    /// The turn ended: sum it up when it ran steps without errors.
    fn finish_turn(&mut self) {
        if self.busy {
            let turn = self.entries.get(self.turn_start..).unwrap_or(&[]);
            let steps: usize = turn.iter().map(|e| if let Entry::Steps(lines) = e { lines.len() } else { 0 }).sum();
            if steps > 0 && !turn.iter().any(|e| matches!(e, Entry::Error(_))) {
                self.entries.push(Entry::Done(steps));
            }
        }
        self.busy = false;
    }

    fn request(&mut self, request: CodexRequest) {
        let Some(link) = &self.link else { return };
        if link.tx.send(request).is_err() {
            self.apply(CodexEvent::Exited(tl!("Codex stopped.").into()));
        }
    }
}

/// A step's raw line as a checklist label: `server › tool_name` reads "Tool name".
fn step_label(line: &str) -> String {
    let tool = line.rsplit(" › ").next().unwrap_or(line).trim();
    if tool == line.trim() {
        return tool.to_string();
    }
    let words = tool.replace(['_', '.', '-'], " ");
    let mut chars = words.chars();
    chars.next().map_or_else(String::new, |c| c.to_uppercase().chain(chars).collect())
}

/// Drain the backend's events (every frame, so generated images land even while the window is
/// closed).
pub fn poll(app: &mut PhotocraftApp) {
    let mut images = Vec::new();
    while let Some(event) = app.codex.link.as_ref().and_then(|link| link.rx.try_recv().ok()) {
        images.extend(app.codex.apply(event));
    }
    for path in images {
        place_image(app, &path);
    }
}

/// Put a generated image into the active document as a new layer, or open it when no document
/// is open.
fn place_image(app: &mut PhotocraftApp, path: &std::path::Path) {
    let name = path.file_name().map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into_owned());
    let text = path.to_string_lossy().into_owned();
    let placed = if app.session.active().is_some() {
        app.run("file.placeEmbedded", json!({"path": text})).map(|_| ())
    } else if app.open_paths(&[text]) > 0 {
        Ok(())
    } else {
        Err(tl!("could not open the image").to_string())
    };
    match placed {
        Ok(()) => app.codex.step(crate::i18n::fmt(tl!("Image {name} added to the document"), &[("name", &name)])),
        Err(e) => app.codex.entries.push(Entry::Error(crate::i18n::fmt(tl!("Could not add image {name}: {error}"), &[("name", &name), ("error", &e)]))),
    }
}

/// Start the backend if it isn't running.
fn ensure_started(app: &mut PhotocraftApp) {
    if app.codex.link.is_some() {
        return;
    }
    if !app.session.prefs().integrations.allow_agent_control {
        app.codex.entries.push(Entry::Error(tl!("Codex needs Preferences › Integrations › Allow Agent Control.").into()));
        return;
    }
    let Some(start) = app.services.codex.as_mut() else {
        app.codex.entries.push(Entry::Error(tl!("Codex is not available in this version of PhotoCraft.").into()));
        return;
    };
    match start() {
        Ok(link) => app.codex.link = Some(link),
        Err(e) => app.codex.entries.push(Entry::Error(e)),
    }
}

/// Window › Codex: the dock in the studio shell, the floating window in the classic one.
pub fn toggle(app: &mut PhotocraftApp) {
    app.codex.open = !app.codex.open;
    if app.codex.open && app.codex.link.is_none() && app.codex.entries.is_empty() {
        ensure_started(app);
    }
}

/// What the user did this frame, applied after drawing.
enum Act {
    Request(CodexRequest),
    /// Send the composer's text (or a quick action's prompt).
    Send(String),
    Start,
    Restart,
    Close,
    History,
}

fn run(app: &mut PhotocraftApp, ctx: &egui::Context, acts: Vec<Act>) {
    for act in acts {
        match act {
            Act::Send(text) => {
                let p = &mut app.codex;
                let text = text.trim().to_string();
                if text.is_empty() || p.link.is_none() || p.busy {
                    continue;
                }
                let prompt = p.scope.prompt(&text);
                p.turn_start = p.entries.len();
                p.entries.push(Entry::User(text));
                p.busy = true;
                p.request(CodexRequest::Send(prompt));
            }
            Act::Request(request) => {
                if request == CodexRequest::NewChat {
                    app.codex.entries.clear();
                    app.codex.busy = false;
                    app.codex.turn_start = 0;
                }
                app.codex.request(request);
            }
            Act::Start => ensure_started(app),
            Act::Restart => {
                // Dropping the link stops the old Codex process.
                app.codex.link = None;
                app.codex.busy = false;
                app.codex.account = None;
                ensure_started(app);
            }
            Act::Close => app.codex.open = false,
            Act::History => {
                const HISTORY: &str = "window.panel.history";
                if crate::view_cmds::checked(app, HISTORY) != Some(true)
                    && let Err(e) = crate::menus::invoke(app, ctx, HISTORY, json!({}))
                {
                    app.codex.entries.push(Entry::Error(e));
                }
            }
        }
    }
}

/// The studio shell's docked Codex panel (mockup 1), drawn before the canvas claims the centre.
pub fn dock(app: &mut PhotocraftApp, ui: &mut Ui) {
    if !app.codex.open || app.studio.classic {
        return;
    }
    let t = Tokens::get(ui.ctx());
    let doc = app.session.active().map(|d| (d.doc.name.clone(), d.doc.layer_count()));
    let mut acts = Vec::new();
    let p = &mut app.codex;
    crate::direction::end_panel("codex")
        .resizable(true)
        .default_size(430.0)
        .size_range(320.0..=720.0)
        .frame(Frame::NONE.fill(t.chrome).inner_margin(Margin::symmetric(8, 10)))
        .show(ui, |ui| {
            let card = ui.max_rect();
            ui.painter().rect(card, 12.0, t.card, Stroke::new(1.0, t.card_border), StrokeKind::Inside);
            let mut ui = ui.new_child(egui::UiBuilder::new().max_rect(card.shrink(14.0)).layout(egui::Layout::top_down(egui::Align::Min)));
            ui.set_clip_rect(card);
            egui::Panel::top("codex_head").frame(Frame::NONE).resizable(false).show_separator_line(false).show(&mut ui, |ui| {
                header(ui, &t, p, &mut acts);
                ui.add_space(6.0);
                tabs(ui, &t, &mut p.tab);
                ui.add_space(10.0);
                context_chip(ui, &t, doc.as_ref());
                ui.add_space(6.0);
            });
            egui::Panel::bottom("codex_foot").frame(Frame::NONE).resizable(false).show_separator_line(false).show(&mut ui, |ui| {
                ui.add_space(8.0);
                if p.tab == Tab::Chat {
                    quick_actions(ui, &t, p, &mut acts);
                    ui.add_space(8.0);
                }
                composer(ui, &t, p, &mut acts);
                ui.add_space(8.0);
                footer(ui, &t, p, &mut acts);
            });
            egui::CentralPanel::no_frame().show(&mut ui, |ui| {
                egui::ScrollArea::vertical().stick_to_bottom(true).auto_shrink([false, false]).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 10.0;
                    match p.tab {
                        Tab::Chat => conversation(ui, &t, p, &mut acts),
                        Tab::Activity => activity(ui, &t, p),
                    }
                });
            });
        });
    run(app, &ui.ctx().clone(), acts);
}

/// "Codex [CLI]" … "+ New Chat", ⋯, ✕.
fn header(ui: &mut Ui, t: &Tokens, p: &CodexPanel, acts: &mut Vec<Act>) {
    let running = p.link.is_some();
    ui.horizontal(|ui| {
        ui.label(RichText::new(tl!("Codex")).font(theme::semibold(20.0)).color(t.text));
        Frame::new().stroke(Stroke::new(1.0, t.accent)).corner_radius(5.0).inner_margin(Margin::symmetric(5, 1)).show(ui, |ui| {
            ui.label(RichText::new(tl!("CLI")).font(theme::medium(11.0)).color(t.accent));
        });
        ui.with_layout(crate::direction::trailing(), |ui| {
            if crate::icons::button(ui, "x", 26.0, false, tl!("Close")).clicked() {
                acts.push(Act::Close);
            }
            let more = crate::icons::button(ui, "ellipsis", 26.0, false, tl!("More"));
            egui::Popup::menu(&more).show(|ui| {
                crate::widgets::style_spectrum_popup_menu(ui);
                ui.set_min_width(180.0);
                if let Some(Some(who)) = &p.account {
                    ui.label(RichText::new(who).small().color(t.text_dim));
                    ui.separator();
                }
                let account = (running).then_some(p.account.as_ref()).flatten();
                if let Some(signed_in) = account.map(Option::is_some) {
                    let (label, request) = if signed_in { (tl!("Sign Out"), CodexRequest::SignOut) } else { (tl!("Sign in with ChatGPT"), CodexRequest::SignIn) };
                    if ui.button(label).clicked() {
                        acts.push(Act::Request(request));
                    }
                }
                if ui.button(if running { tl!("Restart Codex") } else { tl!("Start Codex") }).clicked() {
                    acts.push(if running { Act::Restart } else { Act::Start });
                }
            });
            let new_chat = egui::Button::image_and_text(crate::icons::image("plus", 14.0, t.icon), RichText::new(tl!("New Chat")).color(t.text))
                .fill(t.card)
                .stroke(Stroke::new(1.0, t.field_border))
                .corner_radius(8.0);
            if ui.add_enabled(running, new_chat).clicked() {
                acts.push(Act::Request(CodexRequest::NewChat));
            }
        });
    });
}

/// Conversa | Atividade, the active one underlined in amber.
fn tabs(ui: &mut Ui, t: &Tokens, tab: &mut Tab) {
    let top = ui.cursor().top();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for (which, label) in [(Tab::Chat, tl!("Conversation")), (Tab::Activity, tl!("Activity"))] {
            let (rect, resp) = ui.allocate_exact_size(vec2(130.0, 32.0), Sense::click());
            let on = *tab == which;
            let color = if on || resp.hovered() { t.text } else { t.text_dim };
            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, label, theme::medium(15.0), color);
            if on {
                ui.painter().rect_filled(Rect::from_x_y_ranges(rect.x_range(), rect.bottom() - 2.5..=rect.bottom()), 1.0, t.accent);
            }
            resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, on, label));
            if resp.clicked() {
                *tab = which;
            }
        }
    });
    let r = ui.max_rect();
    let y = ui.cursor().top().max(top + 32.0);
    ui.painter().hline(r.x_range(), y, Stroke::new(1.0, t.separator));
}

/// "Current document · name" with the layer count.
fn context_chip(ui: &mut Ui, t: &Tokens, doc: Option<&(String, usize)>) {
    Frame::new().fill(t.field).stroke(Stroke::new(1.0, t.field_border)).corner_radius(10.0).inner_margin(Margin::symmetric(10, 7)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.with_layout(crate::direction::trailing(), |ui| {
            if let Some((_, layers)) = doc {
                let n = u64::try_from(*layers).unwrap_or(u64::MAX);
                let count = crate::i18n::trn(crate::i18n::current(), n, "{n} layer", "{n} layers");
                Frame::new().stroke(Stroke::new(1.0, t.field_border)).corner_radius(8.0).inner_margin(Margin::symmetric(8, 2)).show(ui, |ui| {
                    ui.label(RichText::new(count).small().color(t.text_dim));
                });
            }
            ui.with_layout(crate::direction::row(), |ui| {
                ui.add(crate::icons::image("file", 16.0, t.icon));
                let text = match doc {
                    Some((name, _)) => crate::i18n::fmt(tl!("Current document · {name}"), &[("name", name)]),
                    None => tl!("No document open").to_string(),
                };
                ui.add(egui::Label::new(RichText::new(text).color(t.text)).truncate());
            });
        });
    });
}

fn card() -> Frame {
    Frame::new().corner_radius(12.0).inner_margin(Margin::symmetric(12, 10))
}

fn conversation(ui: &mut Ui, t: &Tokens, p: &CodexPanel, acts: &mut Vec<Act>) {
    if p.entries.is_empty() {
        ui.add_space(8.0);
        ui.label(RichText::new(tl!("Ask Codex to edit the open document or to generate an image.")).color(t.text_faint));
    }
    let last = p.entries.len().saturating_sub(1);
    for (i, entry) in p.entries.iter().enumerate() {
        match entry {
            Entry::User(text) => user_bubble(ui, t, text),
            Entry::Assistant { text, .. } => codex_row(ui, t, true, |ui| {
                ui.add(egui::Label::new(RichText::new(text).color(t.text)).wrap());
            }),
            Entry::Steps(lines) => {
                let running = p.busy && i == last;
                codex_row(ui, t, true, |ui| steps(ui, t, i, lines, running));
            }
            Entry::Done(n) => codex_row(ui, t, false, |ui| {
                let n = u64::try_from(*n).unwrap_or(u64::MAX);
                ui.horizontal(|ui| {
                    check_dot(ui, t);
                    ui.label(RichText::new(crate::i18n::trn(crate::i18n::current(), n, "Finished in {n} step.", "Finished in {n} steps.")).color(t.text));
                });
                ui.add_space(4.0);
                let show = egui::Button::image_and_text(crate::icons::image("clock", 14.0, t.accent), RichText::new(tl!("Show History")).color(t.text))
                    .fill(t.card)
                    .stroke(Stroke::new(1.0, t.accent))
                    .corner_radius(8.0);
                if ui.add(show).clicked() {
                    acts.push(Act::History);
                }
            }),
            Entry::Error(e) => codex_row(ui, t, false, |ui| {
                ui.add(egui::Label::new(RichText::new(e).color(t.danger)).wrap());
            }),
        }
    }
}

/// The user's message: a rounded bubble with a person icon, on the reading end.
fn user_bubble(ui: &mut Ui, t: &Tokens, text: &str) {
    let align = if crate::direction::is_rtl() { egui::Align::Min } else { egui::Align::Max };
    ui.with_layout(egui::Layout::top_down(align), |ui| {
        let w = (ui.available_width() * 0.88).max(120.0);
        Frame::new().fill(t.field).stroke(Stroke::new(1.0, t.field_border)).corner_radius(12.0).inner_margin(Margin::symmetric(12, 10)).show(ui, |ui| {
            ui.set_max_width(w);
            ui.horizontal_top(|ui| {
                person_icon(ui, t.text_dim);
                ui.add(egui::Label::new(RichText::new(text).color(t.text)).wrap());
            });
        });
    });
}

/// A Codex card, with the amber avatar beside it when `avatar` (else indented to match).
fn codex_row(ui: &mut Ui, t: &Tokens, avatar: bool, add: impl FnOnce(&mut Ui)) {
    ui.horizontal_top(|ui| {
        let (r, _) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::hover());
        if avatar {
            ui.painter().rect_filled(r, 7.0, t.primary_bg);
            crate::icons::paint(ui, r, "sparkles", 16.0, t.primary_text);
        }
        card().fill(t.card).stroke(Stroke::new(1.0, t.card_border)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui);
        });
    });
}

/// A turn's steps: ✓ when done, a spinner on the running one, and the raw calls folded below.
fn steps(ui: &mut Ui, t: &Tokens, id: usize, lines: &[String], running: bool) {
    for (k, line) in lines.iter().enumerate() {
        let pending = running && k + 1 == lines.len();
        ui.horizontal(|ui| {
            if pending {
                ui.add(egui::Spinner::new().size(16.0).color(t.accent));
            } else {
                check_dot(ui, t);
            }
            ui.add(egui::Label::new(RichText::new(step_label(line)).color(t.text)).truncate());
        });
        ui.separator();
    }
    egui::CollapsingHeader::new(RichText::new(tl!("Run details")).color(t.text_dim)).id_salt(("codex-details", id)).show(ui, |ui| {
        for line in lines {
            ui.add(egui::Label::new(RichText::new(line).monospace().small().color(t.text_dim)).wrap());
        }
    });
}

/// Every step of the conversation, with its raw call.
fn activity(ui: &mut Ui, t: &Tokens, p: &CodexPanel) {
    let last = p.entries.len().saturating_sub(1);
    let mut any = false;
    for (i, entry) in p.entries.iter().enumerate() {
        match entry {
            Entry::Steps(lines) => {
                any = true;
                card().fill(t.card).stroke(Stroke::new(1.0, t.card_border)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    for (k, line) in lines.iter().enumerate() {
                        ui.horizontal(|ui| {
                            if p.busy && i == last && k + 1 == lines.len() {
                                ui.add(egui::Spinner::new().size(16.0).color(t.accent));
                            } else {
                                check_dot(ui, t);
                            }
                            ui.vertical(|ui| {
                                ui.label(RichText::new(step_label(line)).color(t.text));
                                ui.add(egui::Label::new(RichText::new(line).monospace().small().color(t.text_dim)).wrap());
                            });
                        });
                    }
                });
            }
            Entry::Error(e) => {
                any = true;
                ui.add(egui::Label::new(RichText::new(e).color(t.danger)).wrap());
            }
            _ => {}
        }
    }
    if !any {
        ui.add_space(8.0);
        ui.label(RichText::new(tl!("No activity yet.")).color(t.text_faint));
    }
}

/// Chips that send a ready-made prompt.
fn quick_actions(ui: &mut Ui, t: &Tokens, p: &CodexPanel, acts: &mut Vec<Act>) {
    let enabled = p.link.is_some() && !p.busy;
    ui.horizontal_wrapped(|ui| {
        let chips = [
            ("sun", tl!("Adjust colors"), tl!("Adjust the colors of the current document so they look balanced.")),
            ("layers", tl!("Organize layers"), tl!("Organize the layers of the current document: clear names and groups.")),
        ];
        for (icon, label, prompt) in chips {
            let chip = egui::Button::image_and_text(crate::icons::image(icon, 15.0, t.icon), RichText::new(label).color(t.text))
                .fill(t.card)
                .stroke(Stroke::new(1.0, t.field_border))
                .corner_radius(10.0)
                .min_size(vec2(0.0, 30.0));
            if ui.add_enabled(enabled, chip).clicked() {
                acts.push(Act::Send(prompt.to_string()));
            }
        }
    });
}

/// The input, the scope menu and the amber Send (Stop while busy).
fn composer(ui: &mut Ui, t: &Tokens, p: &mut CodexPanel, acts: &mut Vec<Act>) {
    let running = p.link.is_some();
    Frame::new().fill(t.field).stroke(Stroke::new(1.0, t.field_border)).corner_radius(12.0).inner_margin(Margin::symmetric(12, 10)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        let edit = ui.add_enabled(
            running,
            egui::TextEdit::multiline(&mut p.input)
                .frame(egui::Frame::NONE)
                .desired_rows(2)
                .desired_width(f32::INFINITY)
                .hint_text(tl!("What do you want to create or edit?")),
        );
        // Enter sends; Shift+Enter is a new line.
        let enter = edit.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift);
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            scope_menu(ui, t, &mut p.scope);
            ui.with_layout(crate::direction::trailing(), |ui| {
                let (icon, tip) = if p.busy { ("square", tl!("Stop")) } else { ("navigation", tl!("Send")) };
                let (rect, resp) = ui.allocate_exact_size(vec2(44.0, 32.0), if running { Sense::click() } else { Sense::hover() });
                let fill = if !running { t.field_border } else if resp.hovered() { t.accent } else { t.primary_bg };
                ui.painter().rect_filled(rect, CornerRadius::same(9), fill);
                crate::icons::paint(ui, rect, icon, 18.0, t.primary_text);
                let resp = resp.on_hover_text(tip);
                resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, running, tip));
                if p.busy {
                    if resp.clicked() {
                        acts.push(Act::Request(CodexRequest::Stop));
                    }
                } else if resp.clicked() || enter {
                    acts.push(Act::Send(std::mem::take(&mut p.input)));
                }
            });
        });
    });
}

fn scope_menu(ui: &mut Ui, t: &Tokens, scope: &mut Scope) {
    let button = egui::Button::image_and_text(crate::icons::image("file", 15.0, t.icon), RichText::new(format!("{}  ▾", scope.label())).color(t.text))
        .fill(t.field)
        .stroke(Stroke::new(1.0, t.field_border))
        .corner_radius(8.0)
        .min_size(vec2(0.0, 30.0));
    let resp = ui.add(button).on_hover_text(tl!("What Codex works on"));
    egui::Popup::menu(&resp).show(|ui| {
        crate::widgets::style_spectrum_popup_menu(ui);
        for s in Scope::ALL {
            if ui.selectable_label(*scope == s, s.label()).clicked() {
                *scope = s;
            }
        }
    });
}

/// "● Codex CLI · MCP connected", or what is missing and the button that fixes it.
fn footer(ui: &mut Ui, t: &Tokens, p: &CodexPanel, acts: &mut Vec<Act>) {
    let running = p.link.is_some();
    let (dot, state) = match (&p.account, running) {
        (_, false) => (t.text_faint, tl!("not running")),
        (None, true) => (t.warning, tl!("starting…")),
        (Some(None), true) => (t.warning, tl!("not signed in")),
        (Some(Some(_)), true) => (green(t), tl!("MCP connected")),
    };
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(vec2(10.0, 16.0), Sense::hover());
        ui.painter().circle_filled(r.center(), 4.0, dot);
        ui.label(RichText::new(format!("{} · {state}", tl!("Codex CLI"))).color(t.text_dim));
        ui.with_layout(crate::direction::trailing(), |ui| {
            if !running {
                if crate::widgets::secondary_button(ui, tl!("Start Codex"), 0.0).clicked() {
                    acts.push(Act::Start);
                }
            } else if p.account == Some(None) && crate::widgets::primary_button(ui, tl!("Sign in with ChatGPT"), 0.0).clicked() {
                acts.push(Act::Request(CodexRequest::SignIn));
            }
        });
    });
}

/// The theme's green (the layer label hue): done steps and a live link.
fn green(t: &Tokens) -> Color32 {
    t.layer_label_colors(photocraft_doc::LabelColor::Green).map_or(t.accent, |(swatch, _)| swatch)
}

/// A green disc with a check.
fn check_dot(ui: &mut Ui, t: &Tokens) {
    let (r, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
    ui.painter().circle_filled(r.center(), 8.0, green(t));
    crate::icons::paint(ui, r, "check", 12.0, t.primary_text);
}

/// A head-and-shoulders outline.
fn person_icon(ui: &mut Ui, color: Color32) {
    let (r, _) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::hover());
    let stroke = Stroke::new(1.5, color);
    let c = r.center();
    ui.painter().circle_stroke(pos2(c.x, c.y - 3.5), 3.5, stroke);
    let shoulders: Vec<_> = (0..=12)
        .map(|k| {
            let a = std::f32::consts::PI * (1.0 + k as f32 / 12.0);
            pos2(c.x + 6.5 * a.cos(), c.y + 8.0 + 6.5 * a.sin())
        })
        .collect();
    ui.painter().add(egui::Shape::line(shoulders, stroke));
}

/// Draw the classic shell's floating chat window (the studio shell docks it instead).
pub fn show(app: &mut PhotocraftApp, ctx: &egui::Context) {
    if !app.codex.open || !app.studio.classic {
        return;
    }
    let t = Tokens::get(ctx);
    let mut acts = Vec::new();
    let running = app.codex.link.is_some();
    let busy = app.codex.busy;
    let account = app.codex.account.clone();
    let entries = app.codex.entries.clone();
    let mut input = std::mem::take(&mut app.codex.input);
    crate::analysis_ui::panel_window(app, ctx, "codex-chat", tl!("Codex"), vec2(-40.0, 40.0), 380.0, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(tl!("Codex")).strong().color(t.text));
            let who = match (&account, running) {
                (_, false) => tl!("not running").to_string(),
                (None, true) => tl!("starting…").to_string(),
                (Some(None), true) => tl!("not signed in").to_string(),
                (Some(Some(who)), true) => who.clone(),
            };
            ui.label(RichText::new(who).small().color(t.text_dim));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if crate::icons::button(ui, "x", 20.0, false, tl!("Close")).clicked() {
                    acts.push(Act::Close);
                }
                if running && ui.small_button(tl!("New Chat")).clicked() {
                    acts.push(Act::Request(CodexRequest::NewChat));
                }
            });
        });
        ui.separator();
        if !running {
            if crate::widgets::secondary_button(ui, tl!("Start Codex"), 0.0).clicked() {
                acts.push(Act::Start);
            }
        } else if account == Some(None) && crate::widgets::primary_button(ui, tl!("Sign in with ChatGPT"), 0.0).clicked() {
            acts.push(Act::Request(CodexRequest::SignIn));
        }
        egui::ScrollArea::vertical().max_height(360.0).stick_to_bottom(true).auto_shrink([false, true]).show(ui, |ui| {
            if entries.is_empty() {
                ui.label(RichText::new(tl!("Ask Codex to edit the open document or to generate an image.")).color(t.text_faint));
            }
            for entry in &entries {
                match entry {
                    Entry::User(text) => {
                        ui.label(RichText::new(text).color(t.accent_text).strong());
                    }
                    Entry::Assistant { text, .. } => {
                        ui.label(RichText::new(text).color(t.text));
                    }
                    Entry::Steps(lines) => {
                        for line in lines {
                            ui.label(RichText::new(line).small().color(t.text_dim));
                        }
                    }
                    Entry::Done(_) => {}
                    Entry::Error(e) => {
                        ui.label(RichText::new(e).color(t.danger));
                    }
                }
                ui.add_space(4.0);
            }
        });
        ui.separator();
        let edit = ui.add_enabled(
            running,
            egui::TextEdit::multiline(&mut input).desired_rows(2).desired_width(f32::INFINITY).hint_text(tl!("Message Codex (Enter to send)")),
        );
        // Enter sends; Shift+Enter is a new line.
        let enter = edit.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift);
        ui.horizontal(|ui| {
            if busy {
                if crate::widgets::secondary_button(ui, tl!("Stop"), 0.0).clicked() {
                    acts.push(Act::Request(CodexRequest::Stop));
                }
                ui.spinner();
            } else if (crate::widgets::primary_button(ui, tl!("Send"), 0.0).clicked() || enter) && running {
                acts.push(Act::Send(std::mem::take(&mut input)));
            }
        });
    });
    app.codex.input = input;
    run(app, ctx, acts);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deltas_stream_into_one_message_and_done_replaces_it() {
        let mut p = CodexPanel::default();
        p.apply(CodexEvent::AssistantDelta { item: "a".into(), delta: "Hel".into() });
        p.apply(CodexEvent::AssistantDelta { item: "a".into(), delta: "lo".into() });
        assert_eq!(p.entries, vec![Entry::Assistant { item: "a".into(), text: "Hello".into() }]);
        p.apply(CodexEvent::Activity("tool".into()));
        p.apply(CodexEvent::AssistantDone { item: "a".into(), text: "Hello!".into() });
        p.apply(CodexEvent::AssistantDelta { item: "b".into(), delta: "Next".into() });
        assert_eq!(p.entries.first(), Some(&Entry::Assistant { item: "a".into(), text: "Hello!".into() }));
        assert_eq!(p.entries.len(), 3);
    }

    #[test]
    fn exit_drops_the_link_and_busy_state() {
        let (tx, _rx) = std::sync::mpsc::channel();
        let (_etx, erx) = std::sync::mpsc::channel();
        let mut p = CodexPanel { link: Some(CodexLink { tx, rx: erx }), busy: true, ..Default::default() };
        assert_eq!(p.apply(CodexEvent::Image("x.png".into())), Some(PathBuf::from("x.png")));
        p.apply(CodexEvent::Exited("gone".into()));
        assert!(p.link.is_none() && !p.busy);
    }

    #[test]
    fn consecutive_steps_group_and_a_clean_turn_is_summed_up() {
        let mut p = CodexPanel::default();
        p.apply(CodexEvent::TurnStarted);
        p.apply(CodexEvent::Activity("photocraft › doc_read".into()));
        p.apply(CodexEvent::Activity("photocraft › layer_add".into()));
        p.apply(CodexEvent::AssistantDelta { item: "a".into(), delta: "ok".into() });
        p.apply(CodexEvent::Activity("Generating image…".into()));
        p.apply(CodexEvent::TurnFinished);
        assert_eq!(
            p.entries,
            vec![
                Entry::Steps(vec!["photocraft › doc_read".into(), "photocraft › layer_add".into()]),
                Entry::Assistant { item: "a".into(), text: "ok".into() },
                Entry::Steps(vec!["Generating image…".into()]),
                Entry::Done(3),
            ]
        );
        // A stray second TurnFinished adds nothing.
        p.apply(CodexEvent::TurnFinished);
        assert_eq!(p.entries.len(), 4);
    }

    #[test]
    fn a_failed_or_empty_turn_has_no_summary() {
        let mut p = CodexPanel::default();
        p.apply(CodexEvent::TurnStarted);
        p.apply(CodexEvent::TurnFinished);
        assert!(p.entries.is_empty());
        p.apply(CodexEvent::TurnStarted);
        p.apply(CodexEvent::Activity("photocraft › x".into()));
        p.apply(CodexEvent::Error("boom".into()));
        p.apply(CodexEvent::TurnFinished);
        assert!(!p.entries.iter().any(|e| matches!(e, Entry::Done(_))));
    }

    #[test]
    fn steps_read_as_labels() {
        assert_eq!(step_label("photocraft › layer_add"), "Layer add");
        assert_eq!(step_label("photocraft › doc.read"), "Doc read");
        assert_eq!(step_label("Generating image…"), "Generating image…");
        assert_eq!(step_label("$ ls -la"), "$ ls -la");
        assert_eq!(step_label("photocraft › "), "");
    }

    #[test]
    fn scope_narrows_the_prompt() {
        assert_eq!(Scope::Document.prompt("hi"), "hi");
        assert!(Scope::Layer.prompt("hi").ends_with(" hi"));
        assert!(Scope::Selection.prompt("hi").contains("selection"));
    }
}
