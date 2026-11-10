# TextureCraft → Fuse Open Texture

Espelha **Adobe Substance 3D Painter**. Repositório: `storytold/texturecraft`. Nome provisório: **Fuse Open Texture**
(`brand.env`: `BRAND_NAME=Fuse Open Texture`, `BRAND_FILE=fuse-open-texture`).

## Onde está

- Estágio (ROADMAP de 2026-11-09): sem estágio declarado; estimativa honesta por área (2026-11-04): projetos 95%, conjuntos de textura 100%, pilha de camadas 98%.
- Crates: automation, bake, compose, core, engine, formats, geom, paint, render, texel, ui-egui.
- Formatos: glTF/GLB, FBX, COLLADA, USD (usda, usdc, usdz), Alembic, OBJ, PLY, STL; UDIM.
- MCP: `texturecraft-cli mcp --bridge PORTA`; app com `--control 7878`. Pastas: `--read-root`, `--write-root`.

Releia o `ROADMAP.md` e o `docs/target-app-parity.md` do app antes de começar.

## O que fazer, na ordem do [roteiro](../00-playbook.md)

1. Fork, branch `studio-redesign`, upstream sem push.
2. Workflows de build e checks (copiar deste repositório, trocar nomes de crates e de arquivo).
3. Marca: `brand.env`, `{app}` nos catálogos, ícone original, sem links nem atualizações do fornecedor.
4. Codex: `codex app-server` com o MCP acima e `default_tools_approval_mode="approve"`; painel com login,
   modelo, contas, anexos, conversas salvas (ver [`01-codex.md`](../01-codex.md)).
5. Studio: modos **Pintar · Camadas · Máscaras · Assar · Exportar**; faixa inferior com **Conjuntos de textura e pilha de camadas**; ferramentas rápidas **Pincel, Borracha, Projeção, Preenchimento, Clonar, Seletor de material, Girar câmera, Mão**; Início e
   Novo documento no padrão (ver [`02-studio-design.md`](../02-studio-design.md)). Mockups do dono antes.
6. Compacto para notebook e acabamento.
7. Função de destaque (abaixo).

## Função de destaque: Texturizar a partir de fotos de referência

Um modelo 3D e fotos do objeto real (ou uma descrição) viram um material em camadas: base, desgaste, sujeira, detalhes, com máscaras geradoras e mapas assados.

Receita para o Codex, a adaptar da do Fuse Open Photo (`create_ai.rs`): Assar os mapas do modelo; camada base por material; desgaste por máscaras de curvatura e oclusão; detalhes gerados como texturas; conferir em várias vistas.

Mesma mecânica: diálogo com arrastar e botão âmbar com a varinha, documento novo com o material como
referência oculta e bloqueada, editor travado com etapas, cursor animado e Parar, pausa no limite e
retomada em outra conta. O `render_preview` do MCP precisa renderizar o documento (e regiões), não a
janela.

## Riscos

Precisa de GPU real; o snapshot com GPU por software (WARP) é lento.

## Prompt de abertura para a sessão

```
Leia docs/fuse-suite/README.md, 00-playbook.md e apps/texturecraft.md do repositório
fernandinhomartins40/fuseopenphoto (branch studio-redesign). Vamos criar o Fuse Open Texture a partir de
storytold/texturecraft, repetindo o que foi feito no Fuse Open Photo. Regras: nunca abrir ou automatizar
navegador nem disparar login; compilar no GitHub; gh sempre com -R; commit e push frequentes. Comece pela
fase 1 (fork e repositório) e me pergunte antes de criar o repositório no GitHub.
```
