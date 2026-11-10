# CADCraft → Fuse Open CAD

Espelha **Autodesk AutoCAD (não é Adobe)**. Repositório: `storytold/cadcraft`. Nome provisório: **Fuse Open CAD**
(`brand.env`: `BRAND_NAME=Fuse Open CAD`, `BRAND_FILE=fuse-open-cad`).

## Onde está

- Estágio (ROADMAP de 2026-11-09): pré-alpha; alpha a ~2 pontos (38% → 40%) e ~125–190 h. Menus com comando 50% (245/491).
- Crates: color, constraints, doc, dwg, dxf, engine, fonts, geom, io, mcp, render, ui-egui.
- Formatos: DWG, DXF.
- MCP: `cadcraft-cli mcp --connect 127.0.0.1:PORTA`; app com `--control PORTA` (padrão 7979).

Releia o `ROADMAP.md` e o `docs/target-app-parity.md` do app antes de começar.

## O que fazer, na ordem do [roteiro](../00-playbook.md)

1. Fork, branch `studio-redesign`, upstream sem push.
2. Workflows de build e checks (copiar deste repositório, trocar nomes de crates e de arquivo).
3. Marca: `brand.env`, `{app}` nos catálogos, ícone original, sem links nem atualizações do fornecedor.
4. Codex: `codex app-server` com o MCP acima e `default_tools_approval_mode="approve"`; painel com login,
   modelo, contas, anexos, conversas salvas (ver [`01-codex.md`](../01-codex.md)).
5. Studio: modos **Desenhar · Modificar · Anotar · Camadas · Plotar**; faixa inferior com **Layouts (Modelo e folhas)**; ferramentas rápidas **Linha, Polilinha, Círculo, Retângulo, Mover, Copiar, Cotar, Pan**; Início e
   Novo documento no padrão (ver [`02-studio-design.md`](../02-studio-design.md)). Mockups do dono antes.
6. Compacto para notebook e acabamento.
7. Função de destaque (abaixo).

## Função de destaque: Croqui ou foto de planta para DXF

Um croqui à mão ou foto de planta vira desenho em escala, com camadas (paredes, portas, cotas), linhas precisas e cotas.

Receita para o Codex, a adaptar da do Fuse Open Photo (`create_ai.rs`): Achar a escala por uma medida conhecida; camadas por tipo; geometria ortogonal; portas e janelas como blocos; cotas; conferir por região.

Mesma mecânica: diálogo com arrastar e botão âmbar com a varinha, documento novo com o material como
referência oculta e bloqueada, editor travado com etapas, cursor animado e Parar, pausa no limite e
retomada em outra conta. O `render_preview` do MCP precisa renderizar o documento (e regiões), não a
janela.

## Riscos

Pré-alpha. Medidas inventadas são perigosas: o Codex pede ao menos uma medida real.

## Prompt de abertura para a sessão

```
Leia docs/fuse-suite/README.md, 00-playbook.md e apps/cadcraft.md do repositório
fernandinhomartins40/fuseopenphoto (branch studio-redesign). Vamos criar o Fuse Open CAD a partir de
storytold/cadcraft, repetindo o que foi feito no Fuse Open Photo. Regras: nunca abrir ou automatizar
navegador nem disparar login; compilar no GitHub; gh sempre com -R; commit e push frequentes. Comece pela
fase 1 (fork e repositório) e me pergunte antes de criar o repositório no GitHub.
```
