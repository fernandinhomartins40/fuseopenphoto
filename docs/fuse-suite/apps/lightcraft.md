# LightCraft → Fuse Open Light

Espelha **Adobe Lightroom (Classic)**. Repositório: `storytold/lightcraft`. Nome provisório: **Fuse Open Light**
(`brand.env`: `BRAND_NAME=Fuse Open Light`, `BRAND_FILE=fuse-open-light`).

## Onde está

- Estágio (ROADMAP de 2026-11-09): alpha; beta a ~14 pontos e ~340–600 h. Amplitude 81,6% de 532 itens; pronto para trabalho real ~62%.
- Crates: catalog, codecs, color, denoise-core, denoise, develop, engine, faces, fetch, film, geocode, geom, gpu, heif, inpaint, lensdb, mcp, merge, meta, pipeline, preview, raster, raw, scenes, segment, slideshow, sysmem, testkit, tiff, ui-egui, wincolor.
- Formatos: RAW das câmeras, DNG, HEIF, TIFF, JPEG; catálogo próprio; ajustes não destrutivos.
- MCP: `lightcraft-cli mcp --connect 127.0.0.1:PORTA`; o app com `lightcraft --control PORTA` (padrão 7980).

Releia o `ROADMAP.md` e o `docs/target-app-parity.md` do app antes de começar.

## O que fazer, na ordem do [roteiro](../00-playbook.md)

1. Fork, branch `studio-redesign`, upstream sem push.
2. Workflows de build e checks (copiar deste repositório, trocar nomes de crates e de arquivo).
3. Marca: `brand.env`, `{app}` nos catálogos, ícone original, sem links nem atualizações do fornecedor.
4. Codex: `codex app-server` com o MCP acima e `default_tools_approval_mode="approve"`; painel com login,
   modelo, contas, anexos, conversas salvas (ver [`01-codex.md`](../01-codex.md)).
5. Studio: modos **Biblioteca · Revelar · Máscaras · Retoque · Exportar**; faixa inferior com **Filme de fotos (miniaturas da pasta ou coleção)**; ferramentas rápidas **Recorte, Pincel de máscara, Gradiente linear, Gradiente radial, Remoção de manchas, Olhos vermelhos, Zoom, Mão**; Início e
   Novo documento no padrão (ver [`02-studio-design.md`](../02-studio-design.md)). Mockups do dono antes.
6. Compacto para notebook e acabamento.
7. Função de destaque (abaixo).

## Função de destaque: Revelar como esta foto

O usuário dá uma foto de referência (um estilo que gosta) e escolhe as fotos dele; o Codex analisa a referência (tons, curvas, cor, granulação, vinheta) e revela as fotos com ajustes não destrutivos reais e máscaras (céu, pessoa, fundo, com o crate `segment`), salvando o resultado como predefinição reutilizável.

Receita para o Codex, a adaptar da do Fuse Open Photo (`create_ai.rs`): Ler a referência por regiões; montar os ajustes básicos, curva de tons e HSL; máscaras de céu e assunto; conferir lado a lado com `render_preview`; aplicar ao lote; salvar a predefinição com nome claro.

Mesma mecânica: diálogo com arrastar e botão âmbar com a varinha, documento novo com o material como
referência oculta e bloqueada, editor travado com etapas, cursor animado e Parar, pausa no limite e
retomada em outra conta. O `render_preview` do MCP precisa renderizar o documento (e regiões), não a
janela.

## Riscos

Lotes grandes demoram; mostrar progresso por foto. Nada pode sobrescrever o original: tudo fica nos ajustes.

## Prompt de abertura para a sessão

```
Leia docs/fuse-suite/README.md, 00-playbook.md e apps/lightcraft.md do repositório
fernandinhomartins40/fuseopenphoto (branch studio-redesign). Vamos criar o Fuse Open Light a partir de
storytold/lightcraft, repetindo o que foi feito no Fuse Open Photo. Regras: nunca abrir ou automatizar
navegador nem disparar login;
trabalhar com agentes em paralelo (cada um na sua worktree, com estas mesmas regras); a cada entrega,
testar, compilar e gerar o instalador no GitHub, e só dizer pronto com o instalador publicado do commit
certo; gh sempre com -R; commit e push frequentes. Comece pela
fase 1 (fork e repositório) e me pergunte antes de criar o repositório no GitHub.
```
