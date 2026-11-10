# GridCraft → Fuse Open Sheets

Espelha **Microsoft Excel (não é Adobe)**. Repositório: `storytold/gridcraft`. Nome provisório: **Fuse Open Sheets**
(`brand.env`: `BRAND_NAME=Fuse Open Sheets`, `BRAND_FILE=fuse-open-sheets`).

## Onde está

- Estágio (ROADMAP de 2026-11-09): alpha; beta a ~25 pontos e ~160–240 h. Pronto para trabalho real ~50%.
- Crates: calc, chart, core, engine, formula, functions, math, mcp, model, numfmt, pdf, ui-egui, xlsx.
- Formatos: XLSX, PDF; fórmulas e gráficos.
- MCP: `gridcraft-cli mcp --connect PORTA`; app com `--control PORTA` (exemplo 7979); token em `GRIDCRAFT_CONTROL_TOKEN`.

Releia o `ROADMAP.md` e o `docs/target-app-parity.md` do app antes de começar.

## O que fazer, na ordem do [roteiro](../00-playbook.md)

1. Fork, branch `studio-redesign`, upstream sem push.
2. Workflows de build e checks (copiar deste repositório, trocar nomes de crates e de arquivo).
3. Marca: `brand.env`, `{app}` nos catálogos, ícone original, sem links nem atualizações do fornecedor.
4. Codex: `codex app-server` com o MCP acima e `default_tools_approval_mode="approve"`; painel com login,
   modelo, contas, anexos, conversas salvas (ver [`01-codex.md`](../01-codex.md)).
5. Studio: modos **Dados · Fórmulas · Gráficos · Formatar · Revisar**; faixa inferior com **Abas das planilhas**; ferramentas rápidas **Seleção, Preencher, Soma automática, Filtro, Ordenar, Gráfico, Formatar como tabela, Congelar**; Início e
   Novo documento no padrão (ver [`02-studio-design.md`](../02-studio-design.md)). Mockups do dono antes.
6. Compacto para notebook e acabamento.
7. Função de destaque (abaixo).

## Função de destaque: Tabela de foto ou PDF para planilha viva

Uma foto ou PDF de tabela vira planilha com dados tipados (datas, moedas), fórmulas no lugar dos totais, formatação condicional e um gráfico.

Receita para o Codex, a adaptar da do Fuse Open Photo (`create_ai.rs`): Transcrever célula a célula (sem inventar); tipar colunas; trocar totais por fórmulas; validar os totais contra o original; gráfico; congelar cabeçalho.

Mesma mecânica: diálogo com arrastar e botão âmbar com a varinha, documento novo com o material como
referência oculta e bloqueada, editor travado com etapas, cursor animado e Parar, pausa no limite e
retomada em outra conta. O `render_preview` do MCP precisa renderizar o documento (e regiões), não a
janela.

## Riscos

Números errados são graves: a receita confere os totais contra a imagem e marca o que não bate.

## Prompt de abertura para a sessão

```
Leia docs/fuse-suite/README.md, 00-playbook.md e apps/gridcraft.md do repositório
fernandinhomartins40/fuseopenphoto (branch studio-redesign). Vamos criar o Fuse Open Sheets a partir de
storytold/gridcraft, repetindo o que foi feito no Fuse Open Photo. Regras: nunca abrir ou automatizar
navegador nem disparar login; compilar no GitHub; gh sempre com -R; commit e push frequentes. Comece pela
fase 1 (fork e repositório) e me pergunte antes de criar o repositório no GitHub.
```
