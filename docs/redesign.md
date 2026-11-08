# Studio shell redesign (spec)

Target look: the mockups in [`docs/redesign/`](redesign/) (1 = Compor + Codex, 2 = Texto,
3 = Cor e luz / Níveis, 4 = Texto / Efeitos, 5 = Selecionar / Matiz-Saturação). Match them as
closely as egui allows: layout, proportions, spacing, colours, corner radii, icon+label buttons.
They are AI mockups: where one contradicts the app or another mockup, keep the feature and follow
the closest mockup.

## Hard rules

1. **No feature is lost.** The studio shell only draws; it runs the same commands
   (`app.run(id, params)`, `crate::menus::invoke(app, ctx, id, params)`) and reuses the existing
   panels' drawing code where possible (layers rows, adjustment editors, type panels, effects
   editor, history, navigator, info). Anything not in a mockup stays reachable through ☰ (the full
   menu tree), the command search (`edit.search` palette), the right rail's Painéis, or the tools
   menu. Window › Classic Layout (`window.toggle.classicLayout`) restores the old shell.
2. **Repo rules from `AGENTS.md` apply**: never panic (no `unwrap`/`expect`/indexing on input), no
   `unsafe`, colours only from `theme::Tokens` (the default Studio theme is the amber house look:
   `t.accent` amber, `t.primary_bg` amber with `t.primary_text` near-black, `t.card` charcoal),
   every user-visible string through `tl!(…)` with a pt-BR row in `i18n/pt-br.tsv`
   (`\t<English>\t<Português>`), and the crate must keep building for wasm (no process/file I/O).
3. **Stay in your files.** Each region is one file under `crates/ui-egui/src/studio/` with its own
   `State`. Shared helpers go in `studio/style.rs` only if you are its owner (see below); otherwise
   keep helpers private in your file. Do not edit `studio.rs`, `app/frame.rs`, `lib.rs`, `theme.rs`
   or another region's file; if you need a hook there, describe it in your final report.
4. Check with `cargo check -p photocraft-ui-egui` and `cargo clippy -p photocraft-ui-egui`
   (warnings in your files are errors). Add small unit tests for any logic (mode/tool mapping,
   layout maths). English code and comments; UI strings in English through `tl!` + pt-BR row.

## Regions (owner files)

| Region | File | Mockup |
|---|---|---|
| Top bar | `studio/top_bar.rs` | all: ☰, **app name**, document name, undo/redo, "Buscar comando ⌘K" field (opens the palette), "Salvo há N s" / unsaved state, amber **Exportar** button; keeps the window drag area and the custom title bar caption buttons (`crate::titlebar`) |
| Mode bar | `studio/mode_bar.rs` | all: centred row Compor · Selecionar · Retocar · Cor e luz · Texto, icon + label, active = amber pill with dark text |
| Tools menu | `studio/tools_menu.rs` | 2–5: floating "Ferramentas ▾" button at the canvas' top-left (amber when open), dropdown of the mode's tools with icon, name and shortcut (5) |
| Context bars | `studio/context_bars.rs` | 1, 2, 4: type bar (font, style, size, colour, align, …); 5: selection bar (Adicionar, Subtrair, Inverter, Refinar, Desmarcar); 3: adjustment tabs (Níveis, Curvas, Exposição, Matiz, Vibração, Mais ▾) |
| Right rail | `studio/right_rail.rs` | 2–5: Histórico, Navegador, Info, Painéis (icon over label, rounded tiles) |
| Right panel | `studio/right_panel.rs` | 2: Texto (Caractere/Parágrafo, Transformar, Avançado); 3: Níveis; 4: Efeitos (layer style list + editor, Reverter/Aplicar); 5: Ajuste/Máscara tabs (Matiz/Saturação). Header with grip, title, pin, collapse |
| Bottom | `studio/bottom.rs` | all: layer properties bar (Modo de mesclagem, Opacidade, Preenchimento, Bloquear icons); layer strip (left tabs Camadas/Canais/Caminhos; layers as cards: eye, kind icon, name, fx, thumbnail, link, mask; selected = amber border; right column +, folder, trash); status line (zoom ▾, −, +, fit, mode · depth · size · layers · file) |
| Codex | `codex_panel.rs` | 1: docked right panel, "Codex CLI" badge, Nova conversa, ⋯, ×; tabs Conversa / Atividade; current-document chip with layer count; user bubble; step checklist with ✓ and "Detalhes da execução"; result card with action button; quick actions; composer with attach, scope ▾ and amber send; footer "Codex CLI · MCP conectado" |

## Modes and tools

Every tool id below is `tools.select.<id>` (run with `app.run`/`menus::invoke`); shortcuts come
from the existing shortcut table. A tool may appear in several modes. The first tool is the mode's
default when the mode is picked.

| Mode | Tools |
|---|---|
| Compor | move, artboard, crop, perspectiveCrop, frame, rectangle, ellipseShape, triangle, polygon, star, line, customShape, pen, freeformPen, curvaturePen, addAnchorPoint, deleteAnchorPoint, convertPoint, pathSelection, directSelection, gradient, paintBucket, slice, sliceSelect, hand, zoom, rotateView, ruler, note, count |
| Selecionar | rectMarquee, ellipseMarquee, singleRowMarquee, singleColumnMarquee, lasso, polygonLasso, magneticLasso, objectSelection, quickSelection, magicWand, horizontalTypeMask, verticalTypeMask |
| Retocar | brush, pencil, mixerBrush, colorReplacement, eraser, backgroundEraser, magicEraser, spotHealing, remove, healing, patch, contentAwareMove, redEye, cloneStamp, patternStamp, historyBrush, artHistoryBrush, blur, sharpen, smudge, dodge, burn, sponge |
| Cor e luz | eyedropper, colorSampler, gradient, paintBucket, dodge, burn, sponge (plus the adjustment context bar and panel) |
| Texto | type, verticalType, horizontalTypeMask, verticalTypeMask |

Picking a tool from anywhere (shortcut, menu, palette) switches the mode bar to the first mode that
lists it, unless the current mode lists it too.

## Identity

The product name is not final: read it from one constant (`studio::style::APP_NAME`) so the
rename is one line.
