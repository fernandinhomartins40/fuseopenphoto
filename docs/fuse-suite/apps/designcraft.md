# DesignCraft → Fuse Open Layout

Espelha **Adobe InDesign**. Repositório: `storytold/designcraft`. Nome provisório: **Fuse Open Layout**
(`brand.env`: `BRAND_NAME=Fuse Open Layout`, `BRAND_FILE=fuse-open-layout`).

## Onde está

- Estágio (ROADMAP de 2026-11-09): pré-alpha; alpha a 5–7 h, beta a ~28 pontos e ~420–750 h. Pronto para trabalho real ~47%.
- Crates: color, compose, doc, engine, epub, fonts, format, geom, idml, images, mcp, pdf, pptx, render, textimport, tools, ui-egui.
- Formatos: IDML, PDF, EPUB, PPTX, texto importado; nativo JSON documentado.
- MCP: Crate `mcp` com servidor próprio; app com `--control 7979`. Confirmar a opção de ligação no `--help` da CLI.

Releia o `ROADMAP.md` e o `docs/target-app-parity.md` do app antes de começar.

## O que fazer, na ordem do [roteiro](../00-playbook.md)

1. Fork, branch `studio-redesign`, upstream sem push.
2. Workflows de build e checks (copiar deste repositório, trocar nomes de crates e de arquivo).
3. Marca: `brand.env`, `{app}` nos catálogos, ícone original, sem links nem atualizações do fornecedor.
4. Codex: `codex app-server` com o MCP acima e `default_tools_approval_mode="approve"`; painel com login,
   modelo, contas, anexos, conversas salvas (ver [`01-codex.md`](../01-codex.md)).
5. Studio: modos **Página · Texto · Imagens · Estilos · Exportar**; faixa inferior com **Páginas e espelhadas (miniaturas), com páginas-mestre**; ferramentas rápidas **Seleção, Seleção direta, Texto, Moldura de imagem, Retângulo, Linha, Conta-gotas, Mão**; Início e
   Novo documento no padrão (ver [`02-studio-design.md`](../02-studio-design.md)). Mockups do dono antes.
6. Compacto para notebook e acabamento.
7. Função de destaque (abaixo).

## Função de destaque: Imagem ou PDF para layout editável

Um folheto, cardápio, revista ou anúncio (foto ou PDF achatado) vira um documento com páginas, molduras de texto encadeadas, estilos de parágrafo e caractere, imagens em molduras e grade. É o caso dos cardápios que o dono já fez, no programa certo para isso.

Receita para o Codex, a adaptar da do Fuse Open Photo (`create_ai.rs`): Medir margens, colunas e grade; criar estilos de parágrafo antes do texto; transcrever o texto exato nas molduras (sem inventar); imagens geradas à parte; páginas-mestre para o que se repete; conferir por região.

Mesma mecânica: diálogo com arrastar e botão âmbar com a varinha, documento novo com o material como
referência oculta e bloqueada, editor travado com etapas, cursor animado e Parar, pausa no limite e
retomada em outra conta. O `render_preview` do MCP precisa renderizar o documento (e regiões), não a
janela.

## Riscos

Ainda pré-alpha: conferir que salvar e reabrir preserva tudo antes de prometer a função.

## Prompt de abertura para a sessão

```
Leia docs/fuse-suite/README.md, 00-playbook.md e apps/designcraft.md do repositório
fernandinhomartins40/fuseopenphoto (branch studio-redesign). Vamos criar o Fuse Open Layout a partir de
storytold/designcraft, repetindo o que foi feito no Fuse Open Photo. Regras: nunca abrir ou automatizar
navegador nem disparar login; compilar no GitHub; gh sempre com -R; commit e push frequentes. Comece pela
fase 1 (fork e repositório) e me pergunte antes de criar o repositório no GitHub.
```
