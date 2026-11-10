# MaterialCraft → Fuse Open Material

Espelha **Adobe Substance 3D Designer**. Repositório: `storytold/materialcraft`. Nome provisório: **Fuse Open Material**
(`brand.env`: `BRAND_NAME=Fuse Open Material`, `BRAND_FILE=fuse-open-material`).

## Onde está

- Estágio (ROADMAP de 2026-11-09): sem estágio declarado; ~67% (2026-11-02, materiais portáveis); paridade de renderização e SBS completos ainda não.
- Crates: automation, core, engine, eval, formats, geom, image, nodes, render, ui-egui.
- Formatos: SBS (parcial), imagens, materiais portáveis.
- MCP: `materialcraft-cli mcp --bridge PORTA`; app com `--control 7878`. Pastas: `--read-root`, `--write-root`.

Releia o `ROADMAP.md` e o `docs/target-app-parity.md` do app antes de começar.

## O que fazer, na ordem do [roteiro](../00-playbook.md)

1. Fork, branch `studio-redesign`, upstream sem push.
2. Workflows de build e checks (copiar deste repositório, trocar nomes de crates e de arquivo).
3. Marca: `brand.env`, `{app}` nos catálogos, ícone original, sem links nem atualizações do fornecedor.
4. Codex: `codex app-server` com o MCP acima e `default_tools_approval_mode="approve"`; painel com login,
   modelo, contas, anexos, conversas salvas (ver [`01-codex.md`](../01-codex.md)).
5. Studio: modos **Grafo · Nós · Prévia · Exportar**; faixa inferior com **Biblioteca de nós (o grafo ocupa o centro, não um canvas de pixels)**; ferramentas rápidas **Seleção, Conectar, Comentário, Moldura, Navegar, Zoom, Mão, Prévia 3D**; Início e
   Novo documento no padrão (ver [`02-studio-design.md`](../02-studio-design.md)). Mockups do dono antes.
6. Compacto para notebook e acabamento.
7. Função de destaque (abaixo).

## Função de destaque: Foto para material procedural

Uma foto de superfície (madeira, pedra, tecido) vira um grafo de nós procedural com parâmetros expostos (escala, cor, desgaste) e mapas PBR, em vez de uma textura fixa.

Receita para o Codex, a adaptar da do Fuse Open Photo (`create_ai.rs`): Analisar padrão, escala e variação; montar o grafo em blocos nomeados (forma, cor, altura, rugosidade); expor parâmetros úteis; comparar a prévia com a foto.

Mesma mecânica: diálogo com arrastar e botão âmbar com a varinha, documento novo com o material como
referência oculta e bloqueada, editor travado com etapas, cursor animado e Parar, pausa no limite e
retomada em outra conta. O `render_preview` do MCP precisa renderizar o documento (e regiões), não a
janela.

## Riscos

O layout Studio muda mais aqui: o centro é um editor de grafos. Mockups primeiro.

## Prompt de abertura para a sessão

```
Leia docs/fuse-suite/README.md, 00-playbook.md e apps/materialcraft.md do repositório
fernandinhomartins40/fuseopenphoto (branch studio-redesign). Vamos criar o Fuse Open Material a partir de
storytold/materialcraft, repetindo o que foi feito no Fuse Open Photo. Regras: nunca abrir ou automatizar
navegador nem disparar login; compilar no GitHub; gh sempre com -R; commit e push frequentes. Comece pela
fase 1 (fork e repositório) e me pergunte antes de criar o repositório no GitHub.
```
