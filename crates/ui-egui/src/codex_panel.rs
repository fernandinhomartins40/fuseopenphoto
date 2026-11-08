//! Window › Codex: a chat with OpenAI Codex that drives this PhotoCraft.
//!
//! This crate only draws the chat and talks over channels, so it stays free of process I/O (and
//! keeps building for the web). The desktop app supplies [`Services::codex`](crate::Services):
//! it runs `codex app-server`, signed in with ChatGPT, with this window's control channel as an
//! MCP server, and turns the server's messages into [`CodexEvent`]s.

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};

use egui::{RichText, vec2};
use serde_json::json;

use crate::PhotocraftApp;
use crate::theme::Tokens;

/// What the chat asks the backend to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodexRequest {
    /// Start ChatGPT sign-in in the browser.
    SignIn,
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
    Activity(String),
    Error(String),
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
            CodexEvent::Activity(line) => self.entries.push(Entry::Activity(line)),
            CodexEvent::Image(path) => return Some(path),
            CodexEvent::TurnStarted => self.busy = true,
            CodexEvent::TurnFinished => self.busy = false,
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

    fn request(&mut self, request: CodexRequest) {
        let Some(link) = &self.link else { return };
        if link.tx.send(request).is_err() {
            self.apply(CodexEvent::Exited(tl!("Codex stopped.").into()));
        }
    }
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
    let line = match placed {
        Ok(()) => Entry::Activity(crate::i18n::fmt(tl!("Image {name} added to the document"), &[("name", &name)])),
        Err(e) => Entry::Error(crate::i18n::fmt(tl!("Could not add image {name}: {error}"), &[("name", &name), ("error", &e)])),
    };
    app.codex.entries.push(line);
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

/// Window › Codex.
pub fn toggle(app: &mut PhotocraftApp) {
    app.codex.open = !app.codex.open;
    if app.codex.open && app.codex.link.is_none() && app.codex.entries.is_empty() {
        ensure_started(app);
    }
}

/// The studio shell's docked Codex panel (mockup 1), drawn before the canvas claims the centre.
/// While it is a stub, the studio shell shows the floating window from [`show`] instead.
pub fn dock(_app: &mut PhotocraftApp, _ui: &mut egui::Ui) {}

/// Draw the chat window.
pub fn show(app: &mut PhotocraftApp, ctx: &egui::Context) {
    if !app.codex.open {
        return;
    }
    let t = Tokens::get(ctx);
    let mut close = false;
    let mut send = None;
    let mut actions: Vec<CodexRequest> = Vec::new();
    let mut restart = false;
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
                    close = true;
                }
                if running && ui.small_button(tl!("New Chat")).clicked() {
                    actions.push(CodexRequest::NewChat);
                }
            });
        });
        ui.separator();
        if !running {
            if crate::widgets::secondary_button(ui, tl!("Start Codex"), 0.0).clicked() {
                restart = true;
            }
        } else if account == Some(None) && crate::widgets::primary_button(ui, tl!("Sign in with ChatGPT"), 0.0).clicked() {
            actions.push(CodexRequest::SignIn);
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
                    Entry::Activity(line) => {
                        ui.label(RichText::new(line).small().color(t.text_dim));
                    }
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
                    actions.push(CodexRequest::Stop);
                }
                ui.spinner();
            } else if (crate::widgets::primary_button(ui, tl!("Send"), 0.0).clicked() || enter) && running {
                send = Some(input.trim().to_string()).filter(|s| !s.is_empty());
            }
        });
    });
    app.codex.input = input;
    if let Some(text) = send {
        app.codex.input.clear();
        app.codex.entries.push(Entry::User(text.clone()));
        app.codex.busy = true;
        actions.push(CodexRequest::Send(text));
    }
    if restart {
        ensure_started(app);
    }
    for action in actions {
        if action == CodexRequest::NewChat {
            app.codex.entries.clear();
            app.codex.busy = false;
        }
        app.codex.request(action);
    }
    if close {
        app.codex.open = false;
    }
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
}
