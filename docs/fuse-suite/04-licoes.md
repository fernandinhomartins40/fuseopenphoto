# Lições do Fuse Open Photo

Erros que custaram horas e como evitá-los. Valem para todos os apps (mesmo fork do egui, mesma estrutura).

## egui (fork storytold, 0.37)

- **`ctx.data` dentro de `ctx.data_mut` trava o app** (deadlock). Foi o "trava ao clicar em qualquer
  lugar". Leia antes, escreva depois.
- Um `Area` só oferece o tamanho do frame anterior: conteúdo largo fica cortado no primeiro frame. Use
  `ui.set_max_width`.
- `with_layout(trailing)` solto, fora de uma linha, ocupa toda a altura. Envolva em `ui.horizontal`.
- Conteúdo desenhado num `Ui` filho não informa tamanho ao `Panel`, que encolhe ao mínimo. Use
  `ui.expand_to_include_rect`.
- **Um `Frame` herda o layout do pai.** Dentro de `ui.horizontal_top`, o conteúdo do cartão sai de lado.
  Foi o "card do Codex largo demais". Envolva em `ui.vertical`.
- `ScrollArea::horizontal` não converte a roda vertical: converta à mão (`smooth_scroll_delta().y`).
- O estilo `ScrollStyle::solid()` vem com `dormant_handle_opacity` 0: a barra aparece sem controle.
- `ui.label` numa linha horizontal não quebra linha; use `Label::wrap()` quando o texto pode crescer.
- `MissingGlyphPolicy` só entra em pânico em teste; sem `CRAFT_FONTS_DIR`, testes com CJK falham.
- `Id::new` foi descontinuado no fork: use `Id::unique` (o CI trata aviso como erro).

## Testes

- Teste antes de corrigir: um kittest que falha mostra a causa. A roda que não rolava e os passos de
  lado só apareceram assim.
- Testes do layout clássico precisam de `app.studio.classic = true` ou do tema ProMedium.
- Rótulos repetidos quebram `get_by_label`: filtre pela posição (`get_all_by_label(...).find(...)`).
- Um texto curto como "a" casa com outros rótulos em `get_by_label_contains`.
- Arquivos de teste temporários: `tempfile::tempdir()`.

## Windows e ferramentas

- O Controle Inteligente de Aplicativos bloqueia binários novos (testes, exemplos, o próprio app) com
  "os error 4551". Não contornar: rodar no CI.
- Um `cargo` morto no meio deixa artefatos quebrados (`E0463 can't find crate`, link `LNK2019`):
  `cargo clean -p <crate>`.
- Um script de build no `target` compartilhado guardou o caminho de outra worktree (`env!("CARGO_MANIFEST_DIR")`):
  limpe o crate.
- Heredoc com apóstrofo quebra no Bash da ferramenta: escreva o script num arquivo e execute.
- Nunca deixe `cat` sem entrada num comando: ele espera para sempre.
- Use `timeout` alto (o `cargo test` da interface leva 5–7 min para compilar) ou rode em segundo plano.

## Codex

- "MCP tool call requires approval, but approval policy is never" =
  `default_tools_approval_mode="approve"` no servidor do app (ver [`01-codex.md`](01-codex.md)).
- O `probe_size` não lê PNG nem JPEG sem decodificar: decodifique para saber o tamanho.
- O modo bridge do `render_preview` capturava a janela: o Codex via a interface e duplicava documentos para
  dar zoom. Renderize o documento.
- O Codex agrupa comandos em lotes (`command_batch`), mas o bridge os manda um a um: dá para espaçar no
  `drain_control` sem mudar o protocolo.

## Trabalho com agentes

- Agentes param no limite de 5 horas; o dono troca de conta e diz "continue". Antes de lançar, garanta
  commit dos WIP nas worktrees.
- Cada agente com seu `CARGO_TARGET_DIR`; apague os `target-*` ao terminar (cada um tem ~10 GB).
- O Google bloqueou o login do dono por trocas de conta repetidas, não por automação. Mesmo assim: nenhum
  agente abre navegador, e toda instrução de subagente repete essa regra.
- Muitos observadores consultando o GitHub a cada 3 s esgotaram o limite secundário da API. Uma consulta
  a cada 5 minutos.

## Produto

- O dono testa no PC dele, com GPU Intel e notebook; peça print de tudo que parecer apertado.
- Diga o que não foi testado de verdade (login, GPU real, Codex real, ditado). Ele prefere saber.
- O dono escreve em português; a interface é traduzida pelos catálogos (`i18n/pt-br.tsv`); toda string
  nova ganha linha em pt-BR no mesmo commit.
