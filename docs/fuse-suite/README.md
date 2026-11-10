# Fuse Suite: a família Fuse a partir dos apps Craft

Guias para repetir, em cada app Craft da storytold, o que foi feito em **Fuse Open Photo** (fork do
PhotoCraft): marca própria, instalador Windows pelo GitHub, Codex CLI integrado e operando o app, e o
layout Studio. Cada sessão nova começa daqui.

Levantamento de 2026-11-09, pela API do GitHub (`storytold/*`).

## Como usar numa sessão nova

1. Escolha o app na tabela abaixo e abra o guia dele em [`apps/`](apps/).
2. Peça à sessão para ler, nesta ordem: este README, [`00-playbook.md`](00-playbook.md), o guia do app,
   e depois [`01-codex.md`](01-codex.md), [`02-studio-design.md`](02-studio-design.md),
   [`03-build-release.md`](03-build-release.md), [`04-licoes.md`](04-licoes.md).
3. Cole o prompt de abertura do guia do app.

O código de referência está neste repositório, `fernandinhomartins40/fuseopenphoto`, branch
`studio-redesign`. Copie as ideias e adapte os nomes; os apps têm a mesma arquitetura, não o mesmo código.

## Os apps

Todos são Rust puro, com egui/eframe sobre wgpu, versão web em WebAssembly, uma CLI e um servidor MCP com
canal de controle JSON. Isso é o que torna o roteiro repetível. Todos têm licença `MIT OR Apache-2.0`.

| App | Espelha | Estágio | Pronto para trabalho real | Guia |
|---|---|---|---|---|
| VectorCraft | Adobe Illustrator | alpha | ~62% | [vectorcraft](apps/vectorcraft.md) |
| LightCraft | Adobe Lightroom | alpha | ~62% | [lightcraft](apps/lightcraft.md) |
| FilmCraft | Adobe Premiere Pro | alpha | ~56% (beta a ~19 pontos) | [filmcraft](apps/filmcraft.md) |
| EffectCraft | Adobe After Effects | alpha | ~45% | [effectcraft](apps/effectcraft.md) |
| FigCraft | Figma | alpha | ~53% | [figcraft](apps/figcraft.md) |
| CADCraft | AutoCAD | pré-alpha | ~38% | [cadcraft](apps/cadcraft.md) |

Os números vêm do `ROADMAP.md` de cada repositório e são estimativas deles. Releia antes de começar:
eles mudam toda semana.

A storytold tem outros apps Craft (DesignCraft, PdfCraft, AniCraft, TextureCraft, MaterialCraft,
SoundCraft, WordCraft, GridCraft, DeckCraft); ficaram fora desta suíte por escolha do dono.
`craft-libs`, `craft-i18n`, `craft-fonts` e `craft-ai-integrations` são bibliotecas compartilhadas que os
apps usam (veja [`00-playbook.md`](00-playbook.md#bibliotecas-compartilhadas)).

## Ordem sugerida

1. **VectorCraft**: o mais próximo do PhotoCraft (camadas, ferramentas, painel de propriedades). Reaproveita
   quase todo o Studio e o "Imagem para vetor" é a continuação natural do "Imagem para PSD".
2. **LightCraft**: mesmo público do Fuse Open Photo; maduro (~62%).
3. **FigCraft**: camadas e páginas como no Vector; o Codex monta telas a partir de um print.
4. **FilmCraft** e **EffectCraft**: o layout muda mais (linha do tempo no lugar da faixa de camadas).
5. **CADCraft**: o menos maduro (~38%) e o que mais foge do Studio (linha de comando, desenho técnico).

Um app por vez. Cada um levou, no Fuse Open Photo, algo como dois dias de sessões com vários agentes.

## Nomes

Provisórios, no padrão do primeiro: **Fuse Open Photo**, **Fuse Open Vector**, **Fuse Open Light**,
**Fuse Open Film** e assim por diante. Cada repositório tem um `brand.env` (nome, empresa, id, nome de
arquivo); trocar o nome depois é trocar esse arquivo.
