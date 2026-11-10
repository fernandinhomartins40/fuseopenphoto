# Codex dentro do app

Como o Fuse Open Photo integra o Codex CLI, e o que mudar em cada app. Arquivos de referência:

- `apps/photocraft/src/codex.rs`: processo `codex app-server`, protocolo JSON-RPC, contas, modelos,
  conversas e anexos. Não depende da interface.
- `crates/ui-egui/src/codex_panel.rs`: o painel (chat, etapas, rodapé, menus). Só desenha e troca mensagens
  por canais; compila para a web.
- `crates/ui-egui/src/create_ai.rs`: a função "Criar com IA".
- `apps/photocraft/src/main.rs`, `codex_service`: liga os dois, abre o canal de controle se preciso.

## Como funciona

1. O app inicia `codex app-server` (o mesmo processo que a extensão de IDE do Codex usa), com
   `CODEX_HOME` próprio em `%APPDATA%\<Marca>\codex-home`. Sessões e login ficam separados dos do usuário.
2. Registra o próprio app como servidor MCP `photocraft` por argumentos `-c`:
   - `mcp_servers.<id>.command` = caminho da CLI do app.
   - `mcp_servers.<id>.args` = o modo que se liga ao app aberto (ver tabela abaixo).
   - `mcp_servers.<id>.env_vars = ["<APP>_CONTROL_TOKEN"]`: o token passa só por variável de ambiente,
     nunca por arquivo ou linha de comando.
   - `mcp_servers.<id>.startup_timeout_sec = 30`.
   - **`mcp_servers.<id>.default_tools_approval_mode = "approve"`**: sem isso, com
     `approvalPolicy: "never"`, o Codex recusa toda ferramenta que edita ("MCP tool call requires approval,
     but approval policy is never"). Foi o primeiro erro real do dono.
   - `--enable image_generation` e `--disable apps plugins computer_use multi_agent`: o modelo vê só o app
     e a geração de imagens.
3. Cada conversa (`thread/start` e `thread/resume`) usa `approvalPolicy: "never"`, `sandbox: "read-only"` e
   `developerInstructions` dizendo onde ele está e que deve agir só pelas ferramentas MCP do app.
4. O canal de controle do app abre na hora em que o Codex inicia, numa porta livre com token novo, se o
   app não foi aberto com `--control`.

## O MCP de cada app

O nome do modo varia. Confirme no `--help` da CLI do app antes de escrever os argumentos.

| App | CLI do MCP ligada ao app aberto | Porta padrão do `--control` |
|---|---|---|
| PhotoCraft (feito) | `photocraft-cli mcp --bridge 127.0.0.1:PORTA` | livre |
| VectorCraft | `vectorcraft-cli mcp --connect 127.0.0.1:PORTA` | 7979 |
| LightCraft | `lightcraft-cli mcp --connect 127.0.0.1:PORTA` | 7980 |
| FilmCraft | `filmcraft-cli mcp` (confirmar a opção de ligação) | 9876 |
| EffectCraft | `effectcraft-cli mcp --bridge PORTA` | 9877 |
| DesignCraft | confirmar (tem crate `mcp`) | 7979 |
| PdfCraft | confirmar (o `--control` recebe um arquivo `.json`) | arquivo |
| AniCraft | `anicraft-cli mcp` (confirmar a ligação) | confirmar |
| TextureCraft | `texturecraft-cli mcp --bridge PORTA` | 7878 |
| MaterialCraft | `materialcraft-cli mcp --bridge PORTA` | 7878 |
| SoundCraft | `soundcraft-cli mcp --connect PORTA` | 7801 |
| FigCraft | `figcraft-cli mcp --attach PORTA` | 47800 |
| WordCraft | `wordcraft-cli mcp --connect 127.0.0.1:PORTA` | 7981 |
| GridCraft | `gridcraft-cli mcp --connect PORTA` (token em `GRIDCRAFT_CONTROL_TOKEN`) | 7979 |
| DeckCraft | `deckcraft-cli mcp --connect PORTA` | 7990 |
| CADCraft | `cadcraft-cli mcp --connect 127.0.0.1:PORTA` | 7979 |

Use sempre uma porta livre escolhida pelo app: vários apps Fuse abertos ao mesmo tempo colidiriam nas
portas padrão.

## Recursos do painel, todos já feitos no Fuse Open Photo

| Recurso | Como | Onde |
|---|---|---|
| Login ChatGPT | `account/login/start {type: chatgpt}` abre o navegador do sistema; `account/read` diz quem entrou | `codex.rs` |
| Status honesto | Desconectado, Conectando, Abrindo o navegador, Conectado, MCP conectado (só depois da primeira chamada ao app) | `codex_panel.rs` `status()` |
| Modelo | `model/list` lista; `config/value/write keyPath=model` grava no `config.toml` do `CODEX_HOME`; cada `turn/start` leva `model` | rodapé |
| Várias contas | uma pasta por conta (`codex-home`, `codex-home-2`…), `account-label` com o e-mail, `codex-active-account` com a ativa; a troca reinicia o processo na outra pasta e copia o `config.toml` (o modelo vai junto) | `codex.rs` `account_home` |
| Limite atingido | `codexErrorInfo` = `usageLimitExceeded` ou `rateLimitExceeded` vira um cartão com as outras contas | `is_limit` |
| Anexos | clipe ou arrastar sobre o painel; imagens vão como `{type: localImage, path}`, outros arquivos como linha `Reference file: <caminho>` | `turn_input` |
| Conversas salvas | `thread/list` (50 mais recentes) e `thread/resume`; os itens `userMessage`, `agentMessage` e `mcpToolCall` remontam o chat | `threads`, `history` |
| Imagens geradas | `item/completed` do tipo `imageGeneration` traz `savedPath`; o app coloca a imagem no documento como camada | `place_image` |
| Etapas | cada `mcpToolCall` vira uma linha com ✓; o fim da rodada soma "Concluído em N etapas" | `Entry::Steps` |

Microfone: o Codex não tem ditado. Tem um modo de voz experimental por WebRTC (`thread/realtime/*`), que é
conversa falada, não ditado. Pendente com o dono: testar o ditado do Windows (Win+H) na caixa do chat.

## Criar com IA

A função de destaque. Mecânica, em `create_ai.rs`:

1. Diálogo: arrastar ou escolher o material, escolher o modelo (padrão `gpt-6.1-sol`, o que deu o
   resultado), aviso de tempo e consumo, botão âmbar grande com a varinha.
2. O app cria o documento no tamanho certo, com o material numa camada "Referência" oculta e bloqueada,
   e abre uma conversa nova com a **receita**: instruções completas, no idioma da interface, que o usuário
   não vê (no chat aparece "Transformando X em um documento editável").
3. A receita pede que cada mensagem comece com uma marca de etapa `[[1]]` … `[[6]]`; o app lê a marca,
   mostra "Etapa 3 de 6 · Gerando as imagens" e a esconde do texto.
4. Durante o processo o editor fica travado (só o painel do Codex e a barra da janela respondem), com
   Parar. Os comandos do Codex chegam um a um, mesmo dentro de um lote; o app segura 0,12 s entre
   edições (`hold` em `drain_control`) para cada uma aparecer.
5. Um cursor âmbar desliza até o ponto de cada edição (parâmetros `x`, `y` ou `rect` do comando, senão o
   centro da camada ativa), com uma etiqueta ("Escrevendo o texto").
6. No limite de uso, pausa; outra conta retoma com uma receita de "continuação" que inspeciona o documento.
7. No fim: Salvar no formato nativo do app; parado no meio: Manter ou Descartar.

Para o Codex ampliar detalhes sem mexer na tela, o `render_preview` do MCP em modo bridge passou a
renderizar o documento, e aceita `region: [x, y, largura, altura]` (método de controle `ui.renderPreview`).
Antes ele capturava a janela, então via a interface por cima e duplicava documentos para dar zoom.
Faça o mesmo em cada app.

A receita de cada app sai do processo que o próprio Codex descreveu ao dono (análise, textos reais,
imagens geradas em separado com fundo transparente, efeitos editáveis, grupos por função, conferência).
O guia de cada app traz a adaptação.

Custo real medido: uma sessão de reconstrução de cardápios levou 1h50, 24 imagens e 47,4 milhões de
tokens de entrada (46,5 milhões em cache) com o GPT-6.1-Sol. Avise no diálogo.
