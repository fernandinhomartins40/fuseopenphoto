# FigCraft → Fuse Open Design

Espelha **Figma (não é Adobe)**. Repositório: `storytold/figcraft`. Nome provisório: **Fuse Open Design**
(`brand.env`: `BRAND_NAME=Fuse Open Design`, `BRAND_FILE=fuse-open-design`).

## Onde está

- Estágio (ROADMAP de 2026-11-09): alpha; beta a ~26 pontos e ~162–292 h. Comandos 100% (212/212); amplitude ~85%; pronto para trabalho real ~53%.
- Crates: collab-net, collab, color, doc, engine, format, geom, layout, mcp, pathops, pdf, plugin, render, svg, text, ui-egui, vectordrawable.
- Formatos: formato próprio, SVG, PDF, VectorDrawable; colaboração (crate `collab`, servidor `figcraft-server`).
- MCP: `figcraft-cli mcp --attach PORTA`; app com `--control PORTA` (exemplo 47800).

Releia o `ROADMAP.md` e o `docs/target-app-parity.md` do app antes de começar.

## O que fazer, na ordem do [roteiro](../00-playbook.md)

1. Fork, branch `studio-redesign`, upstream sem push.
2. Workflows de build e checks (copiar deste repositório, trocar nomes de crates e de arquivo).
3. Marca: `brand.env`, `{app}` nos catálogos, ícone original, sem links nem atualizações do fornecedor.
4. Codex: `codex app-server` com o MCP acima e `default_tools_approval_mode="approve"`; painel com login,
   modelo, contas, anexos, conversas salvas (ver [`01-codex.md`](../01-codex.md)).
5. Studio: modos **Desenhar · Componentes · Protótipo · Inspecionar**; faixa inferior com **Páginas e camadas**; ferramentas rápidas **Mover, Moldura, Retângulo, Caneta, Texto, Comentário, Mão, Zoom**; Início e
   Novo documento no padrão (ver [`02-studio-design.md`](../02-studio-design.md)). Mockups do dono antes.
6. Compacto para notebook e acabamento.
7. Função de destaque (abaixo).

## Função de destaque: Print de tela para design editável

Uma captura de um app ou site vira uma tela editável com auto layout, componentes e estilos; com várias capturas, um protótipo navegável.

Receita para o Codex, a adaptar da do Fuse Open Photo (`create_ai.rs`): Grade e espaçamentos; componentes para o que se repete; auto layout; textos reais; ícones como vetores; imagens geradas à parte; interações entre telas; conferir por região.

Mesma mecânica: diálogo com arrastar e botão âmbar com a varinha, documento novo com o material como
referência oculta e bloqueada, editor travado com etapas, cursor animado e Parar, pausa no limite e
retomada em outra conta. O `render_preview` do MCP precisa renderizar o documento (e regiões), não a
janela.

## Riscos

A colaboração tem servidor próprio; manter fora do escopo no início.

## Prompt de abertura para a sessão

```
Leia docs/fuse-suite/README.md, 00-playbook.md e apps/figcraft.md do repositório
fernandinhomartins40/fuseopenphoto (branch studio-redesign). Vamos criar o Fuse Open Design a partir de
storytold/figcraft, repetindo o que foi feito no Fuse Open Photo. Regras: nunca abrir ou automatizar
navegador nem disparar login;
trabalhar com agentes em paralelo (cada um na sua worktree, com estas mesmas regras); a cada entrega,
testar, compilar e gerar o instalador no GitHub, e só dizer pronto com o instalador publicado do commit
certo; gh sempre com -R; commit e push frequentes. Comece pela
fase 1 (fork e repositório) e me pergunte antes de criar o repositório no GitHub.
```
