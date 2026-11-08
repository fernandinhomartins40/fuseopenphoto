# Window › Codex

A chat with [OpenAI Codex](https://github.com/openai/codex) inside PhotoCraft. Codex signs in with
your ChatGPT account (no API key), edits the open documents through PhotoCraft's own MCP server,
and generates images that land in the active document as new layers.

## How it works

```
PhotoCraft window ── Window › Codex (crates/ui-egui/src/codex_panel.rs)
   │  channels
   ▼
apps/photocraft/src/codex.rs ── stdio JSON-RPC ──► codex app-server   (ChatGPT sign-in, ~/.codex)
   ▲                                                     │ MCP (stdio)
   │ loopback control channel (token)                    ▼
   └──────────────────────────────────────────── photocraft-cli mcp --bridge 127.0.0.1:<port>
```

- `codex app-server` is the same process the Codex IDE extension runs. Sign-in, history and
  settings live in `~/.codex`, shared with the Codex CLI and extension: if you are already signed in
  there, the chat is too. Otherwise **Sign in with ChatGPT** opens the browser.
- The first time the window opens, PhotoCraft starts a loopback control server on a free port with a
  fresh token (or reuses the `--control` one) and registers `photocraft-cli mcp --bridge` with Codex
  as the MCP server `photocraft`. The token goes to the bridge only through the environment
  (`PHOTOCRAFT_CONTROL_TOKEN`, forwarded with `env_vars`).
- Codex runs read-only, without approval prompts: it can't run shell commands or change files. Calls
  to the `photocraft` tools are approved automatically; other MCP servers' calls are declined.
- Generated images (`imageGeneration` items, saved under `~/.codex/generated_images`) are placed
  with File › Place Embedded into the active document, or opened when no document is open.
- File methods of the control channel use `<settings folder>/codex` as their read and write root
  (unless `--automation-read-root`/`--automation-write-root` were given).
- Preferences › Integrations › Allow Agent Control off disables the chat.

## Where Codex comes from

In order: `PHOTOCRAFT_CODEX` (a path), `codex.exe` next to `photocraft.exe` (the installer's copy),
then `codex` / `codex.cmd` on `PATH` (`npm install -g @openai/codex`).

## Building the installer with Codex inside

```powershell
# a folder with codex.exe and its codex-*.exe helpers (from the Codex releases page)
pwsh packaging/windows/package.ps1 -Arch x64 -CodexDir C:\path\to\codex-bin
```

Without `-CodexDir` the MSI is built as before and the chat uses a Codex found on `PATH`.
Codex is Apache-2.0; keep its licence with redistributed copies.
