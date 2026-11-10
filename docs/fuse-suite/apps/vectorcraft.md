# VectorCraft → Fuse Open Vector

Espelha **Adobe Illustrator**. Repositório: `storytold/vectorcraft`. Nome provisório: **Fuse Open Vector**
(`brand.env`: `BRAND_NAME=Fuse Open Vector`, `BRAND_FILE=fuse-open-vector`).

## Onde está

- Estágio (ROADMAP de 2026-11-09): alpha; beta a ~13 pontos e ~170–270 h (pronto para trabalho real ~62% → 75%, e troca confiável de `.ai`).
- Crates: affinity, brush, cad, color, doc, effects, engine, eps, format, geom, js, mcp, metafile, pathops, pdf, plugins, render, scriptcheck, svg, testkit, text, three-d, tools, trace, ui-egui.
- Formatos: SVG, PDF, EPS, `.ai` (via PDF), Affinity, nativo.
- MCP: `vectorcraft-cli mcp --connect 127.0.0.1:PORTA`; o app com `vectorcraft --control PORTA` (padrão 7979). Pastas permitidas: `--automation-read-root` e `--automation-write-root`.

Releia o `ROADMAP.md` e o `docs/target-app-parity.md` do app antes de começar.

## O que fazer, na ordem do [roteiro](../00-playbook.md)

1. Fork, branch `studio-redesign`, upstream sem push.
2. Workflows de build e checks (copiar deste repositório, trocar nomes de crates e de arquivo).
3. Marca: `brand.env`, `{app}` nos catálogos, ícone original, sem links nem atualizações do fornecedor.
4. Codex: `codex app-server` com o MCP acima e `default_tools_approval_mode="approve"`; painel com login,
   modelo, contas, anexos, conversas salvas (ver [`01-codex.md`](../01-codex.md)).
5. Studio: modos **Desenhar · Selecionar · Texto · Cor · Efeitos**; faixa inferior com **Camadas / Pranchetas / Símbolos**; ferramentas rápidas **Seleção, Seleção direta, Caneta, Texto, Retângulo, Pincel, Conta-gotas, Mão**; Início e
   Novo documento no padrão (ver [`02-studio-design.md`](../02-studio-design.md)). Mockups do dono antes.
6. Compacto para notebook e acabamento.
7. Função de destaque (abaixo).

## Função de destaque: Imagem para vetor editável

Uma imagem (logo, ilustração, cardápio, post) vira um documento vetorial: formas e caminhos reais, textos editáveis, gradientes e efeitos vivos, grupos por função. O crate `trace` dá a vetorização automática como ponto de partida; o Codex refaz o que o traçado erra (letras viram texto real, formas viram primitivas limpas) e gera em separado as partes fotográficas.

Receita para o Codex, a adaptar da do Fuse Open Photo (`create_ai.rs`): Analisar a composição e as cores; texto vira camada de texto com a fonte mais próxima; logos e ícones viram caminhos limpos (`trace` como base, depois simplificar e fechar); fotografias geradas à parte e colocadas; efeitos como aparência viva; grupos por seção; conferir com `render_preview` por região.

Mesma mecânica: diálogo com arrastar e botão âmbar com a varinha, documento novo com o material como
referência oculta e bloqueada, editor travado com etapas, cursor animado e Parar, pausa no limite e
retomada em outra conta. O `render_preview` do MCP precisa renderizar o documento (e regiões), não a
janela.

## Riscos

O `.ai` ainda não abre com fidelidade total; exportar SVG/PDF é o caminho seguro. O traçado de fotos gera milhares de pontos: limitar ao que é gráfico.

## Prompt de abertura para a sessão

```
Leia docs/fuse-suite/README.md, 00-playbook.md e apps/vectorcraft.md do repositório
fernandinhomartins40/fuseopenphoto (branch studio-redesign). Vamos criar o Fuse Open Vector a partir de
storytold/vectorcraft, repetindo o que foi feito no Fuse Open Photo. Regras: nunca abrir ou automatizar
navegador nem disparar login;
trabalhar com agentes em paralelo (cada um na sua worktree, com estas mesmas regras); a cada entrega,
testar, compilar e gerar o instalador no GitHub, e só dizer pronto com o instalador publicado do commit
certo; gh sempre com -R; commit e push frequentes. Comece pela
fase 1 (fork e repositório) e me pergunte antes de criar o repositório no GitHub.
```
