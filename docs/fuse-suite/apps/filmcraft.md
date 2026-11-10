# FilmCraft → Fuse Open Film

Espelha **Adobe Premiere Pro**. Repositório: `storytold/filmcraft`. Nome provisório: **Fuse Open Film**
(`brand.env`: `BRAND_NAME=Fuse Open Film`, `BRAND_FILE=fuse-open-film`).

## Onde está

- Estágio (ROADMAP de 2026-11-09): alpha; beta a ~19 pontos de pronto para trabalho real (~56% hoje) e ~400–700 h, mais abrir `.prproj`. Amplitude ~86%.
- Crates: 51 crates: codecs próprios (h264, hevc, av1, prores, dnx, mpeg2v, aac, ac3, opus, flac, mp3enc), contêineres (isobmff, matroska, mxf, mpegts, avi, ogg), edit, project, render, scopes, captions, speech, interchange, automation.
- Formatos: MP4/MOV, MKV, MXF, ProRes, DNxHD; legendas; intercâmbio (XML/EDL/AAF conforme o crate `interchange`).
- MCP: crate `automation`; app com `filmcraft --control PORTA` e `FILMCRAFT_CONTROL_TOKEN` no ambiente; o Codex liga com `filmcraft-cli mcp --bridge 127.0.0.1:PORTA` (conferido no `.mcp.json` e no `docs/control-protocol.md` do upstream).

Releia o `ROADMAP.md` e o `docs/target-app-parity.md` do app antes de começar.

## O que fazer, na ordem do [roteiro](../00-playbook.md)

1. Fork, branch `studio-redesign`, upstream sem push.
2. Workflows de build e checks (copiar deste repositório, trocar nomes de crates e de arquivo).
3. Marca: `brand.env`, `{app}` nos catálogos, ícone original, sem links nem atualizações do fornecedor.
4. Codex: `codex app-server` com o MCP acima e `default_tools_approval_mode="approve"`; painel com login,
   modelo, contas, anexos, conversas salvas (ver [`01-codex.md`](../01-codex.md)).
5. Studio: modos **Montar · Cortar · Cor · Áudio · Legendas**; faixa inferior com **Linha do tempo (trilhas de vídeo e áudio) no lugar da faixa de camadas**; ferramentas rápidas **Seleção, Navalha, Ondulação, Deslizar, Caneta (keyframes), Texto, Zoom, Mão**; Início e
   Novo documento no padrão (ver [`02-studio-design.md`](../02-studio-design.md)). Mockups do dono antes.
6. Compacto para notebook e acabamento.
7. Função de destaque (abaixo).

## Função de destaque: Montagem automática a partir de um roteiro

O usuário entrega uma pasta de clipes e um briefing (ou um vídeo de referência). O Codex assiste por quadros-chave, transcreve as falas (crate `speech`), monta o primeiro corte na linha do tempo seguindo o roteiro, coloca legendas, títulos e trilha, e ajusta a cor de forma consistente.

Receita para o Codex, a adaptar da do Fuse Open Photo (`create_ai.rs`): Inventariar os clipes (duração, conteúdo por quadros); transcrever; montar o corte na ordem do briefing; legendas a partir da transcrição; títulos como texto editável; cor com ajustes, não renderizada; conferir quadros-chave com `render_preview`.

Mesma mecânica: diálogo com arrastar e botão âmbar com a varinha, documento novo com o material como
referência oculta e bloqueada, editor travado com etapas, cursor animado e Parar, pausa no limite e
retomada em outra conta. O `render_preview` do MCP precisa renderizar o documento (e regiões), não a
janela.

## Riscos

Vídeo pesa: limitar análise a quadros-chave. O cursor animado deve ir à posição na linha do tempo, não ao canvas.

## Prompt de abertura para a sessão

```
Leia docs/fuse-suite/README.md, 00-playbook.md e apps/filmcraft.md do repositório
fernandinhomartins40/fuseopenphoto (branch studio-redesign). Vamos criar o Fuse Open Film a partir de
storytold/filmcraft, repetindo o que foi feito no Fuse Open Photo. Regras: nunca abrir ou automatizar
navegador nem disparar login;
trabalhar com agentes em paralelo (cada um na sua worktree, com estas mesmas regras); a cada entrega,
testar, compilar e gerar o instalador no GitHub, e só dizer pronto com o instalador publicado do commit
certo; gh sempre com -R; commit e push frequentes. Comece pela
fase 1 (fork e repositório) e me pergunte antes de criar o repositório no GitHub.
```
