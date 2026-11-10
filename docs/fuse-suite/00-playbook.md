# Roteiro: de um app Craft a um app Fuse

A sequência que levou o PhotoCraft ao Fuse Open Photo, em fases. Cada fase termina com um build verde no
GitHub e um instalador que o dono testa. Os detalhes técnicos de cada fase estão nos outros arquivos.

Referência: `fernandinhomartins40/fuseopenphoto`, branch `studio-redesign`. Os commits citados abaixo
mostram como cada passo ficou.

## Fase 0: antes de tudo

- Leia o `AGENTS.md`, o `ROADMAP.md` e o `docs/` do app de origem. Eles mandam no código (sem `unwrap`,
  sem `unsafe`, camadas, "tudo é um comando", testes junto com cada mudança).
- Regras do dono, valem em toda sessão:
  - **Nunca** abrir, automatizar ou emular navegador, nem disparar login do Codex, ChatGPT ou Google. A
    verificação visual é só pelo exemplo `snapshot` offscreen ou por testes kittest. Repita esta regra em
    toda instrução para subagentes.
  - **Trabalhar com agentes.** Divida cada fase em tarefas independentes (por exemplo: barra superior,
    faixa inferior, painel do Codex, Início) e lance subagentes em paralelo, cada um na sua worktree e com
    o seu `CARGO_TARGET_DIR`. Cada subagente faz commit do que fez antes de terminar. A sessão principal
    integra, revisa o diff e faz o push. Toda instrução de subagente repete estas regras. Detalhes em
    [`04-licoes.md`](04-licoes.md#trabalho-com-agentes).
  - **Sempre testar, compilar e gerar o instalador no GitHub.** Toda entrega termina com push, o
    `checks.yml` verde e o `build-windows.yml` publicando o instalador na pré-release `latest-<branch>`.
    Só diga "pronto" depois de conferir que a pré-release saiu do commit certo (`gh release view`, em
    [`03-build-release.md`](03-build-release.md#acompanhar-um-build)); se o build falhar, corrija e repita.
    Compile no GitHub e não na máquina local: ela tem pouco disco, e o Controle Inteligente de Aplicativos
    do Windows bloqueia executáveis novos sem assinatura.
  - Usar sempre `gh ... -R <dono>/<repo>`: sem isso o `gh` resolve para o upstream da storytold.
  - Consultar o GitHub no máximo a cada 5 minutos (limite de API compartilhado).
  - Commit e push frequentes: sessões caem por limite de uso e o trabalho não salvo se perde.

## Fase 1: fork e repositório

1. Criar o repositório no GitHub do dono (ex.: `fuseopenvector`) a partir de um clone do upstream, com
   histórico (`git clone https://github.com/storytold/<app>` e `git remote add origin ...`).
2. Manter o upstream como remoto só de leitura: `git remote set-url --push upstream DISABLED`.
3. Criar o branch de trabalho (`studio-redesign`) e usá-lo como padrão do repositório.
4. Repositório público: os minutos do Actions são gratuitos.

## Fase 2: build no GitHub e instalador

- Trocar os workflows do upstream (que chamam `storytold/craft-common-actions` e segredos que não temos)
  por dois: `build-windows.yml` (instalador MSI e portátil, publicado como pré-release contínua
  `latest-<branch>`) e `checks.yml` (clippy e os testes da interface). Ver
  [`03-build-release.md`](03-build-release.md).
- Empacotar o Codex CLI no instalador (baixado do release oficial `openai/codex` no próprio workflow).

## Fase 3: marca

- Um `brand.env` na raiz (nome, empresa, id reverso, nome de arquivo), lido pelos `build.rs` e pelo script
  de empacotamento. Commits `4c71dd091` e `241af53a0`.
- Trocar no texto da interface o nome do produto por `{app}` nos catálogos de tradução e preencher com
  `APP_NAME`. Mensagens do motor ficam neutras.
- Ícone original: os logos ArtCraft não são abertos (`docs/brand/LICENSE-brand.txt`). No Fuse Open Photo
  o ícone é fundo âmbar, montanhas quase pretas e sol branco.
- Remover links para o fornecedor e desligar a checagem de atualização (`check_updates: None`), com um
  teste que garante isso (commit `ce9a354f1`).
- Preferências e dados num diretório com o nome novo, para não misturar com uma instalação do original.

## Fase 4: Codex dentro do app

O núcleo do produto. Ver [`01-codex.md`](01-codex.md). Em resumo: o app inicia `codex app-server`,
registra o próprio app como servidor MCP (a CLI do app no modo que se conecta ao canal de controle), faz
login com ChatGPT (sem chave de API) e mostra o chat num painel. Depois vêm: aprovação das ferramentas do
app, escolha de modelo, várias contas, anexos, conversas salvas e a função "Criar com IA".

## Fase 5: layout Studio

A interface nova, igual para toda a família. Ver [`02-studio-design.md`](02-studio-design.md). Começa
pelos mockups: o dono gera as telas (uma por modo, mais Início e Novo documento) e elas ficam em
`docs/redesign/`. O layout antigo continua acessível em Janela › Layout clássico.

## Fase 6: responsivo e acabamento

- Modo compacto para notebook (altura < 800 ou largura < 1280): barras mais baixas, trilho só com ícones,
  Codex mais estreito e que reflui o conteúdo.
- Início e Novo documento com a mesma escala delicada do editor (textos de 13–14 px, linhas de 1 px,
  painéis `chrome` sobre o canvas escuro).
- Faixa inferior (camadas, páginas, quadros) com rolagem pela roda do mouse e arraste com retorno visual.

## Fase 7: a função de IA de destaque

Cada app ganha o seu "Criar com IA": o usuário entrega um material (imagem, PDF, rascunho, áudio) e o
Codex reconstrói tudo como um arquivo editável, ao vivo, com cursor animado e etapas. O guia de cada app
diz qual é. A mecânica é a mesma do `create_ai.rs` do Fuse Open Photo ([`01-codex.md`](01-codex.md#criar-com-ia)).

## Antes de entregar cada fase

- Testes da área tocada (`cargo test -p <crate> --lib -- <filtro>`), clippy com `-D warnings` e o build
  do instalador verde no GitHub.
- Telas conferidas pelo `snapshot` offscreen, em 1440×900 e 1100×620.
- Dizer ao dono o que não foi testado de verdade (login, GPU real, Codex real).

## Bibliotecas compartilhadas

- `craft-libs`: componentes Rust comuns, fixados por revisão. Não precisam de fork enquanto não forem
  alterados.
- `craft-i18n`: catálogos TSV. As traduções novas (pt-BR) entram no catálogo do próprio app.
- `craft-fonts`: fontes, fora dos repositórios dos apps (`CRAFT_FONTS_DIR`). Sem ela, uns 18 testes que
  dependem de glifos CJK falham localmente; no CI com fontes passam.
- `craft-ai-integrations`: camada de IA da storytold (conexões, chaves, provedores, widgets egui). É outro
  caminho (chave de API e provedores); a família Fuse usa o Codex CLI com login ChatGPT. Não misturar sem
  decidir com o dono.
