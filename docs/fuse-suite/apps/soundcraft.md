# SoundCraft → Fuse Open Sound

Espelha **Avid Pro Tools (e DAWs em geral; não é Adobe)**. Repositório: `storytold/soundcraft`. Nome provisório: **Fuse Open Sound**
(`brand.env`: `BRAND_NAME=Fuse Open Sound`, `BRAND_FILE=fuse-open-sound`).

## Onde está

- Estágio (ROADMAP de 2026-11-09): pré-alpha; alpha a ~5% e ~110–180 h. Menus 482 de 512 do Pro Tools (94,1%).
- Crates: au-host, audio-io, automation, clap-host, dsp, engine, midi, mix, model, playback, time, ui-egui, video, vst3-host.
- Formatos: áudio comum, MIDI, plug-ins VST3, CLAP, AU.
- MCP: `soundcraft-cli mcp --connect PORTA`; app com `--control PORTA` (exemplo 7801). Também `--demo` e pastas de leitura e escrita.

Releia o `ROADMAP.md` e o `docs/target-app-parity.md` do app antes de começar.

## O que fazer, na ordem do [roteiro](../00-playbook.md)

1. Fork, branch `studio-redesign`, upstream sem push.
2. Workflows de build e checks (copiar deste repositório, trocar nomes de crates e de arquivo).
3. Marca: `brand.env`, `{app}` nos catálogos, ícone original, sem links nem atualizações do fornecedor.
4. Codex: `codex app-server` com o MCP acima e `default_tools_approval_mode="approve"`; painel com login,
   modelo, contas, anexos, conversas salvas (ver [`01-codex.md`](../01-codex.md)).
5. Studio: modos **Gravar · Editar · Mixar · Efeitos · Exportar**; faixa inferior com **Faixas e mixer**; ferramentas rápidas **Seleção, Mão (grabber), Corte, Lápis, Fade, Zoom, Scrub, Smart Tool**; Início e
   Novo documento no padrão (ver [`02-studio-design.md`](../02-studio-design.md)). Mockups do dono antes.
6. Compacto para notebook e acabamento.
7. Função de destaque (abaixo).

## Função de destaque: Mixagem assistida

O usuário importa as faixas e diz o resultado desejado (ou dá uma música de referência); o Codex organiza faixas e grupos, ajusta ganho, EQ e compressão com plug-ins reais e automação, e entrega uma mixagem e um master ajustáveis.

Receita para o Codex, a adaptar da do Fuse Open Photo (`create_ai.rs`): Nomear e agrupar faixas; ganho de entrada; EQ e compressão por faixa; barramentos; automação; medir loudness; exportar.

Mesma mecânica: diálogo com arrastar e botão âmbar com a varinha, documento novo com o material como
referência oculta e bloqueada, editor travado com etapas, cursor animado e Parar, pausa no limite e
retomada em outra conta. O `render_preview` do MCP precisa renderizar o documento (e regiões), não a
janela.

## Riscos

Pré-alpha. O cursor animado deve apontar faixas e controles do mixer. Áudio não tem prévia por imagem: usar medidores.

## Prompt de abertura para a sessão

```
Leia docs/fuse-suite/README.md, 00-playbook.md e apps/soundcraft.md do repositório
fernandinhomartins40/fuseopenphoto (branch studio-redesign). Vamos criar o Fuse Open Sound a partir de
storytold/soundcraft, repetindo o que foi feito no Fuse Open Photo. Regras: nunca abrir ou automatizar
navegador nem disparar login; compilar no GitHub; gh sempre com -R; commit e push frequentes. Comece pela
fase 1 (fork e repositório) e me pergunte antes de criar o repositório no GitHub.
```
