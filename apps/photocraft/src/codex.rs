//! Window › Codex backend: runs `codex app-server` (the JSON-RPC process the Codex IDE extension
//! uses, signed in with ChatGPT, so no API key) and gives it this PhotoCraft as the MCP server
//! `photocraft` (`photocraft-cli mcp --bridge`, over the loopback control channel). The chat UI is
//! `photocraft_ui_egui::codex_panel`.
//!
//! Codex runs with its own home (`CODEX_HOME`, sign-in and history) and without plugins, apps or
//! computer use, so the user's other Codex MCP servers and plugins don't crowd out PhotoCraft's
//! tools: the model sees `photocraft` and image generation. The control token reaches the bridge through the environment only
//! (`PHOTOCRAFT_CONTROL_TOKEN`, passed on with `env_vars`), never a file or a command line.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{ChildStdin, Command, Stdio};
use std::sync::mpsc::{Sender, channel};

use photocraft_ui_egui::codex_panel::{CodexEvent, CodexLink, CodexRequest};
use serde_json::{Value, json};

/// What Codex is told about where it runs.
const INSTRUCTIONS: &str = "You are running inside PhotoCraft, an image editor, as its built-in assistant. \
Act on the user's open documents only with the `photocraft` MCP tools (never the shell or screen control): `command_list` finds the editor's commands, \
`command_run` and `command_batch` run them, `doc_inspect` reads the document and `render_preview` shows it. \
Inspect before you edit, and check the result with a preview. \
When the user wants a new image, create it with your image generation tool: PhotoCraft adds every generated \
image to the active document as a new layer by itself (or opens it when no document is open), so never place \
it again. Answer in the user's language, briefly.";

/// `name` next to this executable (the installer puts `photocraft-cli` and a bundled `codex` there).
fn sibling(name: &str) -> Option<PathBuf> {
    let exe = format!("{name}{}", std::env::consts::EXE_SUFFIX);
    let path = std::env::current_exe().ok()?.parent()?.join(exe);
    path.is_file().then_some(path)
}

/// The Codex CLI: `PHOTOCRAFT_CODEX`, else the copy installed with PhotoCraft, else `PATH`.
pub fn find_codex() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("PHOTOCRAFT_CODEX").filter(|p| !p.is_empty()) {
        return Some(PathBuf::from(path));
    }
    sibling("codex").or_else(|| {
        // npm installs a `codex.cmd` shim on Windows.
        let names: &[&str] = if cfg!(windows) { &["codex.exe", "codex.cmd"] } else { &["codex"] };
        let path = std::env::var_os("PATH")?;
        std::env::split_paths(&path).flat_map(|dir| names.iter().map(move |n| dir.join(n))).find(|p| p.is_file())
    })
}

/// A JSON string, which is also a TOML basic string (for `codex -c key=value`).
fn toml_str(s: &str) -> String {
    Value::String(s.to_owned()).to_string()
}

/// The `codex app-server` arguments that register this PhotoCraft as the `photocraft` MCP server.
fn server_args(cli: &Path, port: u16) -> Vec<String> {
    let args = json!(["mcp", "--bridge", format!("127.0.0.1:{port}")]);
    vec![
        "app-server".into(),
        "-c".into(),
        format!("mcp_servers.photocraft.command={}", toml_str(&cli.to_string_lossy())),
        "-c".into(),
        format!("mcp_servers.photocraft.args={args}"),
        "-c".into(),
        r#"mcp_servers.photocraft.env_vars=["PHOTOCRAFT_CONTROL_TOKEN"]"#.into(),
        "-c".into(),
        "mcp_servers.photocraft.startup_timeout_sec=30".into(),
        "--enable".into(),
        "image_generation".into(),
    ]
    .into_iter()
    .chain(["apps", "plugins", "computer_use", "multi_agent"].into_iter().flat_map(|f| ["--disable".to_string(), f.to_string()]))
    .collect()
}

/// The error when [`find_codex`] finds nothing.
pub const NOT_FOUND: &str = "Codex CLI not found: install it (npm install -g @openai/codex) or set PHOTOCRAFT_CODEX";

/// Start `codex` for a control server on `port` with `token`, working in `cwd`, with its sign-in
/// and history in `home`.
pub fn start(codex: &Path, port: u16, token: &str, cwd: &Path, home: &Path, ctx: egui::Context) -> Result<CodexLink, String> {
    let cli = sibling("photocraft-cli").ok_or("photocraft-cli was not found next to PhotoCraft")?;
    let mut cmd = Command::new(codex);
    cmd.args(server_args(&cli, port))
        .current_dir(cwd)
        .env("PHOTOCRAFT_CONTROL_TOKEN", token)
        .env("CODEX_HOME", home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        // CREATE_NO_WINDOW: the GUI app has no console for Codex to borrow.
        cmd.creation_flags(0x0800_0000);
    }
    let mut child = cmd.spawn().map_err(|e| format!("could not start Codex ({}): {e}", codex.display()))?;
    let (Some(stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
        let _ = child.kill();
        return Err("could not connect to Codex".into());
    };
    let (in_tx, in_rx) = channel::<Input>();
    let (ui_tx, ui_rx) = channel::<CodexRequest>();
    let (ev_tx, ev_rx) = channel::<CodexEvent>();
    let lines = in_tx.clone();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else { break };
            if let Ok(v) = serde_json::from_str::<Value>(&line)
                && lines.send(Input::Message(v)).is_err()
            {
                return;
            }
        }
        let _ = lines.send(Input::Exited);
    });
    std::thread::spawn(move || {
        while let Ok(request) = ui_rx.recv() {
            if in_tx.send(Input::Ui(request)).is_err() {
                return;
            }
        }
        let _ = in_tx.send(Input::Closed);
    });
    let cwd = cwd.to_string_lossy().into_owned();
    std::thread::spawn(move || {
        let mut driver =
            Driver { stdin, next_id: 0, pending: HashMap::new(), thread: None, turn: None, starting: false, queued: Vec::new(), events: ev_tx, ctx, cwd };
        driver.request("initialize", json!({"clientInfo": {"name": "photocraft", "title": photocraft_ui_egui::studio::style::APP_NAME, "version": env!("CARGO_PKG_VERSION")}}));
        for input in in_rx {
            match input {
                Input::Ui(request) => driver.ui(request),
                Input::Message(v) => driver.message(&v),
                Input::Exited => {
                    driver.emit(CodexEvent::Exited("Codex stopped.".into()));
                    break;
                }
                Input::Closed => break,
            }
        }
        let _ = child.kill();
        let _ = child.wait();
    });
    Ok(CodexLink { tx: ui_tx, rx: ev_rx })
}

enum Input {
    Ui(CodexRequest),
    Message(Value),
    Exited,
    /// The window dropped its end.
    Closed,
}

struct Driver {
    stdin: ChildStdin,
    next_id: u64,
    /// Request id → method, to read the matching reply.
    pending: HashMap<u64, &'static str>,
    thread: Option<String>,
    turn: Option<String>,
    /// A `thread/start` is on its way.
    starting: bool,
    /// Messages waiting for the conversation to start.
    queued: Vec<String>,
    events: Sender<CodexEvent>,
    ctx: egui::Context,
    cwd: String,
}

fn text(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(str::to_owned)
}

impl Driver {
    fn write(&mut self, v: &Value) {
        if writeln!(self.stdin, "{v}").and_then(|()| self.stdin.flush()).is_err() {
            self.emit(CodexEvent::Error("could not write to Codex".into()));
        }
    }

    fn request(&mut self, method: &'static str, params: Value) {
        self.next_id += 1;
        self.pending.insert(self.next_id, method);
        self.write(&json!({"id": self.next_id, "method": method, "params": params}));
    }

    fn emit(&self, event: CodexEvent) {
        let _ = self.events.send(event);
        self.ctx.request_repaint();
    }

    fn ui(&mut self, request: CodexRequest) {
        match request {
            CodexRequest::SignIn => self.request("account/login/start", json!({"type": "chatgpt"})),
            CodexRequest::SignOut => self.request("account/logout", json!({})),
            CodexRequest::Send(message) => match self.thread.clone() {
                Some(thread) => self.turn_start(&thread, &message),
                None => {
                    self.queued.push(message);
                    if !self.starting {
                        self.starting = true;
                        let params = json!({"cwd": self.cwd, "approvalPolicy": "never", "sandbox": "read-only", "developerInstructions": INSTRUCTIONS});
                        self.request("thread/start", params);
                    }
                }
            },
            CodexRequest::Stop => {
                if let (Some(thread), Some(turn)) = (self.thread.clone(), self.turn.clone()) {
                    self.request("turn/interrupt", json!({"threadId": thread, "turnId": turn}));
                }
            }
            CodexRequest::NewChat => (self.thread, self.turn) = (None, None),
        }
    }

    fn turn_start(&mut self, thread: &str, message: &str) {
        self.request("turn/start", json!({"threadId": thread, "input": [{"type": "text", "text": message}]}));
    }

    fn message(&mut self, v: &Value) {
        let method = v.get("method").and_then(Value::as_str);
        match (v.get("id"), method) {
            (Some(id), Some(method)) => self.server_request(id.clone(), method, v.get("params").unwrap_or(&Value::Null)),
            (Some(id), None) => {
                let Some(method) = id.as_u64().and_then(|id| self.pending.remove(&id)) else { return };
                match v.get("error") {
                    Some(error) => self.failed(method, error),
                    None => self.reply(method, v.get("result").unwrap_or(&Value::Null)),
                }
            }
            (None, Some(method)) => self.notification(method, v.get("params").unwrap_or(&Value::Null)),
            (None, None) => {}
        }
    }

    fn reply(&mut self, method: &str, result: &Value) {
        match method {
            "initialize" => {
                self.write(&json!({"method": "initialized"}));
                self.request("account/read", json!({}));
            }
            "account/read" => {
                let account = result.get("account").filter(|a| !a.is_null());
                let who = account.map(|a| text(a, "email").or_else(|| text(a, "type")).unwrap_or_else(|| "signed in".into()));
                self.emit(CodexEvent::Account(who));
            }
            "account/logout" => self.request("account/read", json!({})),
            "account/login/start" => {
                if let Some(url) = text(result, "authUrl") {
                    self.emit(CodexEvent::Activity("Sign in to ChatGPT in your browser…".into()));
                    if let Err(e) = open::that_detached(&url) {
                        self.emit(CodexEvent::Error(format!("could not open the browser ({e}); sign in at {url}")));
                    }
                }
            }
            "thread/start" => {
                self.starting = false;
                let Some(thread) = result.get("thread").and_then(|t| text(t, "id")) else {
                    self.emit(CodexEvent::Error("Codex did not start a conversation".into()));
                    self.emit(CodexEvent::TurnFinished);
                    return;
                };
                self.thread = Some(thread.clone());
                for message in std::mem::take(&mut self.queued) {
                    self.turn_start(&thread, &message);
                }
            }
            _ => {}
        }
    }

    fn failed(&mut self, method: &str, error: &Value) {
        let message = text(error, "message").unwrap_or_else(|| error.to_string());
        if method == "thread/start" {
            self.starting = false;
            self.queued.clear();
        }
        self.emit(CodexEvent::Error(format!("{method}: {message}")));
        if matches!(method, "thread/start" | "turn/start") {
            self.emit(CodexEvent::TurnFinished);
        }
    }

    fn notification(&mut self, method: &str, p: &Value) {
        let item = p.get("item").unwrap_or(&Value::Null);
        let kind = item.get("type").and_then(Value::as_str).unwrap_or("");
        match method {
            "account/login/completed" => match p.get("success").and_then(Value::as_bool) {
                Some(true) => self.request("account/read", json!({})),
                _ => self.emit(CodexEvent::Error(format!("sign-in failed: {}", text(p, "error").unwrap_or_default()))),
            },
            "account/updated" => self.request("account/read", json!({})),
            "turn/started" => {
                self.turn = p.get("turn").and_then(|t| text(t, "id"));
                self.emit(CodexEvent::TurnStarted);
            }
            "turn/completed" => {
                self.turn = None;
                let turn = p.get("turn").unwrap_or(&Value::Null);
                if let Some(message) = turn.get("error").and_then(|e| text(e, "message")) {
                    self.emit(CodexEvent::Error(message));
                }
                self.emit(CodexEvent::TurnFinished);
            }
            "error" if p.get("willRetry").and_then(Value::as_bool) != Some(true) => {
                if let Some(message) = p.get("error").and_then(|e| text(e, "message")) {
                    self.emit(CodexEvent::Error(message));
                }
            }
            "item/agentMessage/delta" => {
                if let (Some(item), Some(delta)) = (text(p, "itemId"), text(p, "delta")) {
                    self.emit(CodexEvent::AssistantDelta { item, delta });
                }
            }
            "item/started" => {
                let line = match kind {
                    "mcpToolCall" => Some(format!("{} › {}", text(item, "server").unwrap_or_default(), text(item, "tool").unwrap_or_default())),
                    "imageGeneration" => Some("Generating image…".into()),
                    "webSearch" => Some("Searching the web…".into()),
                    "commandExecution" => text(item, "command").map(|c| format!("$ {c}")),
                    _ => None,
                };
                if let Some(line) = line {
                    self.emit(CodexEvent::Activity(line));
                }
            }
            "item/completed" => match kind {
                "agentMessage" => {
                    if let (Some(id), Some(text)) = (text(item, "id"), text(item, "text")) {
                        self.emit(CodexEvent::AssistantDone { item: id, text });
                    }
                }
                "imageGeneration" => match text(item, "savedPath") {
                    Some(path) => self.emit(CodexEvent::Image(PathBuf::from(path))),
                    None if item.get("failure").is_some_and(|f| !f.is_null()) => {
                        self.emit(CodexEvent::Error("Image generation failed: the usage limit was reached.".into()));
                    }
                    None => self.emit(CodexEvent::Error("Image generation failed.".into())),
                },
                "mcpToolCall" => {
                    if let Some(message) = item.get("error").and_then(|e| text(e, "message")) {
                        self.emit(CodexEvent::Error(message));
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    /// Codex asking PhotoCraft something. Only this PhotoCraft's own tools are approved; Codex
    /// runs read-only and never asks to run commands or change files.
    fn server_request(&mut self, id: Value, method: &str, p: &Value) {
        let result = match method {
            "mcpServer/elicitation/request" if text(p, "serverName").as_deref() == Some("photocraft") => {
                let content = if text(p, "mode").as_deref() == Some("url") { Value::Null } else { json!({}) };
                json!({"action": "accept", "content": content})
            }
            "mcpServer/elicitation/request" => json!({"action": "decline"}),
            "item/commandExecution/requestApproval" | "item/fileChange/requestApproval" => json!({"decision": "decline"}),
            _ => {
                self.write(&json!({"id": id, "error": {"code": -32601, "message": format!("PhotoCraft does not handle {method}")}}));
                return;
            }
        };
        self.write(&json!({"id": id, "result": result}));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_args_are_valid_toml_values() {
        let args = server_args(Path::new(r#"C:\Program Files\Art "Craft"\photocraft-cli.exe"#), 4567);
        assert_eq!(args.first().map(String::as_str), Some("app-server"));
        let command = args.iter().find_map(|a| a.strip_prefix("mcp_servers.photocraft.command=")).unwrap_or_default();
        assert_eq!(serde_json::from_str::<String>(command).unwrap_or_default(), r#"C:\Program Files\Art "Craft"\photocraft-cli.exe"#);
        assert!(args.iter().any(|a| a == r#"mcp_servers.photocraft.args=["mcp","--bridge","127.0.0.1:4567"]"#));
    }
}
