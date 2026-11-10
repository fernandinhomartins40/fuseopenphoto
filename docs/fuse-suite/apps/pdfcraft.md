# PdfCraft → Fuse Open PDF

Espelha **Adobe Acrobat**. Repositório: `storytold/pdfcraft`. Nome provisório: **Fuse Open PDF**
(`brand.env`: `BRAND_NAME=Fuse Open PDF`, `BRAND_FILE=fuse-open-pdf`).

## Onde está

- Estágio (ROADMAP de 2026-11-09): alpha; beta a ~25 pontos (pronto para trabalho real 50% → 75%) e ~400–700 h.
- Crates: a11y, annot, audit, automation, compare, content, cos, create, crypt, edit, engine, export, filters, fonts, forms, geom, js, measure, model, ocr, optimize, oracle, organize, platform, prc, preflight, print, redact, render, sign, ui-egui, xfa, xfdf.
- Formatos: PDF (edição, formulários, assinatura, redação, OCR), XFA, XFDF, exportação.
- MCP: Servidor no crate `automation`; o `--control` recebe um arquivo `.json` (porta e token). Confirmar a CLI do MCP.

Releia o `ROADMAP.md` e o `docs/target-app-parity.md` do app antes de começar.

## O que fazer, na ordem do [roteiro](../00-playbook.md)

1. Fork, branch `studio-redesign`, upstream sem push.
2. Workflows de build e checks (copiar deste repositório, trocar nomes de crates e de arquivo).
3. Marca: `brand.env`, `{app}` nos catálogos, ícone original, sem links nem atualizações do fornecedor.
4. Codex: `codex app-server` com o MCP acima e `default_tools_approval_mode="approve"`; painel com login,
   modelo, contas, anexos, conversas salvas (ver [`01-codex.md`](../01-codex.md)).
5. Studio: modos **Ler · Editar · Comentar · Formulários · Proteger**; faixa inferior com **Páginas (miniaturas, reordenar arrastando)**; ferramentas rápidas **Seleção, Editar texto, Comentário, Destaque, Assinatura, Redação, Medir, Mão**; Início e
   Novo documento no padrão (ver [`02-studio-design.md`](../02-studio-design.md)). Mockups do dono antes.
6. Compacto para notebook e acabamento.
7. Função de destaque (abaixo).

## Função de destaque: PDF escaneado para PDF editável e acessível

Um PDF escaneado ou uma foto de documento vira PDF com texto real (OCR), campos de formulário onde há lacunas, estrutura acessível (títulos, ordem de leitura), dados sensíveis marcados para redação e um resumo.

Receita para o Codex, a adaptar da do Fuse Open Photo (`create_ai.rs`): OCR por página; conferir o texto por região; detectar campos e criar formulários; marcar tags de acessibilidade; propor (não aplicar) redações de CPF, e-mails, telefones; resumo no fim.

Mesma mecânica: diálogo com arrastar e botão âmbar com a varinha, documento novo com o material como
referência oculta e bloqueada, editor travado com etapas, cursor animado e Parar, pausa no limite e
retomada em outra conta. O `render_preview` do MCP precisa renderizar o documento (e regiões), não a
janela.

## Riscos

Redação é irreversível: o Codex só propõe; o usuário confirma.

## Prompt de abertura para a sessão

```
Leia docs/fuse-suite/README.md, 00-playbook.md e apps/pdfcraft.md do repositório
fernandinhomartins40/fuseopenphoto (branch studio-redesign). Vamos criar o Fuse Open PDF a partir de
storytold/pdfcraft, repetindo o que foi feito no Fuse Open Photo. Regras: nunca abrir ou automatizar
navegador nem disparar login; compilar no GitHub; gh sempre com -R; commit e push frequentes. Comece pela
fase 1 (fork e repositório) e me pergunte antes de criar o repositório no GitHub.
```
