# EffectCraft → Fuse Open Motion

Espelha **Adobe After Effects**. Repositório: `storytold/effectcraft`. Nome provisório: **Fuse Open Motion**
(`brand.env`: `BRAND_NAME=Fuse Open Motion`, `BRAND_FILE=fuse-open-motion`).

## Onde está

- Estágio (ROADMAP de 2026-11-09): alpha; beta a ~30 pontos e ~350–620 h. Amplitude ~91%; pronto para trabalho real ~45%.
- Crates: aep, automation, effects, engine, expr, export, keyframe, lottie, media, model, path, project, psd, render, sam2, script, segment, svg, text, time, track, ui-egui e codificadores (av1, hevc, vp9, opus).
- Formatos: `.aep` (leitura), Lottie, PSD como camadas, SVG, MP4/WebM.
- MCP: `effectcraft-cli mcp --bridge PORTA`; app com `--control 9877`. É o mesmo modo `--bridge` do PhotoCraft.

Releia o `ROADMAP.md` e o `docs/target-app-parity.md` do app antes de começar.

## O que fazer, na ordem do [roteiro](../00-playbook.md)

1. Fork, branch `studio-redesign`, upstream sem push.
2. Workflows de build e checks (copiar deste repositório, trocar nomes de crates e de arquivo).
3. Marca: `brand.env`, `{app}` nos catálogos, ícone original, sem links nem atualizações do fornecedor.
4. Codex: `codex app-server` com o MCP acima e `default_tools_approval_mode="approve"`; painel com login,
   modelo, contas, anexos, conversas salvas (ver [`01-codex.md`](../01-codex.md)).
5. Studio: modos **Compor · Animar · Efeitos · Texto · Renderizar**; faixa inferior com **Linha do tempo com as camadas da composição e os keyframes**; ferramentas rápidas **Seleção, Mão, Zoom, Rotação, Âncora, Forma, Caneta, Texto**; Início e
   Novo documento no padrão (ver [`02-studio-design.md`](../02-studio-design.md)). Mockups do dono antes.
6. Compacto para notebook e acabamento.
7. Função de destaque (abaixo).

## Função de destaque: Animar a partir de uma imagem ou storyboard

O usuário dá um layout estático (ou um storyboard de quadros) e descreve o movimento. O Codex recria a peça em camadas separadas (como o Imagem para PSD), depois anima com keyframes e expressões reais, transições entre quadros e exporta Lottie ou vídeo.

Receita para o Codex, a adaptar da do Fuse Open Photo (`create_ai.rs`): Reconstruir em camadas (mesma receita do Fuse Open Photo); definir a duração; animar por camada com keyframes nomeados; expressões só quando simplificam; conferir quadros em vários tempos com `render_preview`.

Mesma mecânica: diálogo com arrastar e botão âmbar com a varinha, documento novo com o material como
referência oculta e bloqueada, editor travado com etapas, cursor animado e Parar, pausa no limite e
retomada em outra conta. O `render_preview` do MCP precisa renderizar o documento (e regiões), não a
janela.

## Riscos

Renderizar vídeo para conferir é lento: conferir quadros soltos. `sam2` e `segment` ajudam a recortar.

## Prompt de abertura para a sessão

```
Leia docs/fuse-suite/README.md, 00-playbook.md e apps/effectcraft.md do repositório
fernandinhomartins40/fuseopenphoto (branch studio-redesign). Vamos criar o Fuse Open Motion a partir de
storytold/effectcraft, repetindo o que foi feito no Fuse Open Photo. Regras: nunca abrir ou automatizar
navegador nem disparar login;
trabalhar com agentes em paralelo (cada um na sua worktree, com estas mesmas regras); a cada entrega,
testar, compilar e gerar o instalador no GitHub, e só dizer pronto com o instalador publicado do commit
certo; gh sempre com -R; commit e push frequentes. Comece pela
fase 1 (fork e repositório) e me pergunte antes de criar o repositório no GitHub.
```
