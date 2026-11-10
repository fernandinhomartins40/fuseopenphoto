# WordCraft → Fuse Open Write

Espelha **Microsoft Word (não é Adobe)**. Repositório: `storytold/wordcraft`. Nome provisório: **Fuse Open Write**
(`brand.env`: `BRAND_NAME=Fuse Open Write`, `BRAND_FILE=fuse-open-write`).

## Onde está

- Estágio (ROADMAP de 2026-11-09): alpha; beta a ~15 pontos (60% → 75%) e ~150–230 h.
- Crates: control-key, doc, docbin, docx, engine, fonts, formats, geom, layout, mcp, metafile, pdf, render, ui-egui, zotero.
- Formatos: DOCX, DOC (binário), PDF, Zotero (citações).
- MCP: `wordcraft-cli mcp --connect 127.0.0.1:PORTA`; app com `--control PORTA` (exemplo 7981).

Releia o `ROADMAP.md` e o `docs/target-app-parity.md` do app antes de começar.

## O que fazer, na ordem do [roteiro](../00-playbook.md)

1. Fork, branch `studio-redesign`, upstream sem push.
2. Workflows de build e checks (copiar deste repositório, trocar nomes de crates e de arquivo).
3. Marca: `brand.env`, `{app}` nos catálogos, ícone original, sem links nem atualizações do fornecedor.
4. Codex: `codex app-server` com o MCP acima e `default_tools_approval_mode="approve"`; painel com login,
   modelo, contas, anexos, conversas salvas (ver [`01-codex.md`](../01-codex.md)).
5. Studio: modos **Escrever · Formatar · Inserir · Revisar · Layout**; faixa inferior com **Páginas (miniaturas) e estrutura de títulos**; ferramentas rápidas **Seleção, Texto, Tabela, Imagem, Comentário, Pincel de formatação, Zoom, Mão**; Início e
   Novo documento no padrão (ver [`02-studio-design.md`](../02-studio-design.md)). Mockups do dono antes.
6. Compacto para notebook e acabamento.
7. Função de destaque (abaixo).

## Função de destaque: Documento a partir de foto ou rascunho

Uma foto de documento, um PDF ou um rascunho solto vira um DOCX com estilos de verdade (títulos, listas, tabelas), sumário, referências e revisão de texto.

Receita para o Codex, a adaptar da do Fuse Open Photo (`create_ai.rs`): Transcrever sem inventar; estilos antes de formatar; tabelas reais; sumário; revisão ortográfica; conferir páginas.

Mesma mecânica: diálogo com arrastar e botão âmbar com a varinha, documento novo com o material como
referência oculta e bloqueada, editor travado com etapas, cursor animado e Parar, pausa no limite e
retomada em outra conta. O `render_preview` do MCP precisa renderizar o documento (e regiões), não a
janela.

## Riscos

O layout Studio precisa de uma área de página contínua; o cursor animado vai ao parágrafo.

## Prompt de abertura para a sessão

```
Leia docs/fuse-suite/README.md, 00-playbook.md e apps/wordcraft.md do repositório
fernandinhomartins40/fuseopenphoto (branch studio-redesign). Vamos criar o Fuse Open Write a partir de
storytold/wordcraft, repetindo o que foi feito no Fuse Open Photo. Regras: nunca abrir ou automatizar
navegador nem disparar login; compilar no GitHub; gh sempre com -R; commit e push frequentes. Comece pela
fase 1 (fork e repositório) e me pergunte antes de criar o repositório no GitHub.
```
