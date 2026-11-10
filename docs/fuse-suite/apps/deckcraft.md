# DeckCraft → Fuse Open Slides

Espelha **Microsoft PowerPoint (não é Adobe)**. Repositório: `storytold/deckcraft`. Nome provisório: **Fuse Open Slides**
(`brand.env`: `BRAND_NAME=Fuse Open Slides`, `BRAND_FILE=fuse-open-slides`).

## Onde está

- Estágio (ROADMAP de 2026-11-09): alpha; beta a ~20 pontos e ~120–210 h. Pronto para trabalho real ~55%.
- Crates: anim, av1, bitstream, color, engine, fonts, format, geom, h264, hevc, isobmff, matroska, mcp, media, metafile, model, ogg, opus, pdf, pptx, render, text, ui-egui, vp9.
- Formatos: PPTX, PDF; vídeo e áudio incorporados.
- MCP: `deckcraft-cli mcp --connect PORTA`; app com `deckcraft --control PORTA` (exemplo 7990).

Releia o `ROADMAP.md` e o `docs/target-app-parity.md` do app antes de começar.

## O que fazer, na ordem do [roteiro](../00-playbook.md)

1. Fork, branch `studio-redesign`, upstream sem push.
2. Workflows de build e checks (copiar deste repositório, trocar nomes de crates e de arquivo).
3. Marca: `brand.env`, `{app}` nos catálogos, ícone original, sem links nem atualizações do fornecedor.
4. Codex: `codex app-server` com o MCP acima e `default_tools_approval_mode="approve"`; painel com login,
   modelo, contas, anexos, conversas salvas (ver [`01-codex.md`](../01-codex.md)).
5. Studio: modos **Slides · Texto · Mídia · Transições · Apresentar**; faixa inferior com **Slides (miniaturas)**; ferramentas rápidas **Seleção, Texto, Forma, Imagem, Tabela, Gráfico, Conector, Mão**; Início e
   Novo documento no padrão (ver [`02-studio-design.md`](../02-studio-design.md)). Mockups do dono antes.
6. Compacto para notebook e acabamento.
7. Função de destaque (abaixo).

## Função de destaque: Apresentação a partir de um briefing

Um briefing, um documento ou fotos de slides viram uma apresentação com layouts mestre, textos editáveis, imagens geradas, gráficos e transições.

Receita para o Codex, a adaptar da do Fuse Open Photo (`create_ai.rs`): Roteiro de slides; mestre e layouts primeiro; um slide por ideia; imagens geradas com fundo coerente; gráficos com dados reais; notas do apresentador; conferir cada slide.

Mesma mecânica: diálogo com arrastar e botão âmbar com a varinha, documento novo com o material como
referência oculta e bloqueada, editor travado com etapas, cursor animado e Parar, pausa no limite e
retomada em outra conta. O `render_preview` do MCP precisa renderizar o documento (e regiões), não a
janela.

## Riscos

Briefing vago gera slides genéricos: a receita pede o público e o objetivo antes.

## Prompt de abertura para a sessão

```
Leia docs/fuse-suite/README.md, 00-playbook.md e apps/deckcraft.md do repositório
fernandinhomartins40/fuseopenphoto (branch studio-redesign). Vamos criar o Fuse Open Slides a partir de
storytold/deckcraft, repetindo o que foi feito no Fuse Open Photo. Regras: nunca abrir ou automatizar
navegador nem disparar login; compilar no GitHub; gh sempre com -R; commit e push frequentes. Comece pela
fase 1 (fork e repositório) e me pergunte antes de criar o repositório no GitHub.
```
