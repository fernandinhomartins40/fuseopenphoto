# Layout Studio: a identidade da família Fuse

O mesmo desenho em todos os apps, para quem usa um saber usar todos. Referências: as telas em
`docs/redesign/` deste repositório (1 a 5: modos do editor; 6: Início; 7: Novo documento) e o código em
`crates/ui-egui/src/studio/`.

## Princípios

- **Uma tarefa por vez.** Os modos no topo trocam as ferramentas e o painel da direita; o usuário não vê
  as 70 ferramentas do programa original ao mesmo tempo.
- **O Codex é parte do editor**, não uma janela à parte: tem lugar fixo na direita e um botão na barra
  superior.
- **Delicado.** Texto de 13–14 px, títulos de seção de 14 px semibold, linhas de 1 px, raio 10–12,
  painéis `chrome` sobre o canvas mais escuro. Botões de 30–38 px de altura; só o botão principal de uma
  tela pode ser maior.
- **Nada some.** Toda função do layout clássico continua acessível (menu ☰, busca de comando, Janela ›
  Layout clássico). O redesenho só reorganiza.

## Cores (tema Studio)

| Token | RGB | Uso |
|---|---|---|
| `canvas` | 19, 20, 20 | fundo do trabalho, da Início e da área central do Novo documento |
| `dock` | 20, 20, 21 | faixa inferior |
| `chrome` | 29, 29, 30 | barras e painéis |
| `card` | 35, 35, 36 | cartões, menus flutuantes |
| `card_border`, `separator` | 54, 54, 56 | bordas e divisões |
| `field` / `field_border` | 28, 28, 29 / 72, 72, 74 | campos |
| `hover` | 48, 48, 50 | passar o mouse |
| `text` / `text_dim` / `text_faint` | 240 / 170 / 110 | texto |
| `accent` = `primary_bg` | 253, 181, 40 (#FDB528) | âmbar: modo ativo, ferramenta ativa, botão principal |
| `primary_text` | 18, 18, 18 | texto sobre o âmbar |

Cores sempre por `theme::Tokens`, nunca fixas no código. A migração única para o tema Studio fica nas
preferências (versão do arquivo de preferências).

## Regiões do editor

```
┌ barra superior: ☰ · nome · arquivo · desfazer/refazer · busca de comando · status · varinha (Codex) · Exportar ┐
├ barra de modos: Modo 1 · Modo 2 · … (o ativo em âmbar)                                                          ┤
│ ┌ botão da ferramenta ativa ┐  ┌ opções da ferramenta (pílula flutuante) ┐ │ painel da direita │ trilho │
│ │ faixa de ferramentas      │        CANVAS / ÁREA DE TRABALHO             │ (contextual) ou   │ Codex  │
│ └ rápidas (8 ícones)        ┘                                              │ Codex             │ Hist.  │
├ barra de propriedades do item selecionado (mesclagem, opacidade, bloqueios…)                                  ┤
├ faixa inferior: abas (Camadas/Canais/Caminhos, ou o equivalente do app) · cartões com rolagem · ações          ┤
└ linha de status: zoom · modo de cor · tamanho · nº de itens · nome                                              ┘
```

Medidas (normal / compacto): barra superior 52 / 44; barra de modos 54 / 42; status 34 / 28; faixa
inferior 170 / 112; propriedades 42 / 36; trilho 80 / 60 (só ícones no compacto); Codex 430 / 340 de
largura, mínimo 260, máximo 60% da janela. Compacto = altura < 800 ou largura < 1280; "baixo" (< 680 de
altura) esconde linhas opcionais.

Com o Codex aberto, ele desce até a linha de status e a faixa inferior fica só embaixo do canvas.

## Componentes que todo app deve ter

- **Botão de ferramenta** no canto do canvas, mostrando a ferramenta ativa (ícone e nome), arrastável;
  abre a lista do modo em duas colunas. Embaixo, a **faixa de ferramentas rápidas**: as 8 mais usadas, um
  clique em qualquer modo, ativa em âmbar, atalho na dica.
- **Opções da ferramenta** numa pílula flutuante ao lado do botão, com rolagem horizontal se não couber.
- **Barras de contexto** flutuantes: texto (fonte, estilo, tamanho, cor, alinhamento) sobre o texto
  editado; seleção (adicionar, subtrair, inverter, refinar) embaixo.
- **Painel da direita contextual**: Propriedades e Efeitos do item selecionado; os painéis do trilho
  (Histórico, Navegador, Info, Painéis) abrem ali.
- **Faixa inferior** com cartões grandes (miniatura, olho, nome, `fx`, cadeado), rolagem pela roda do
  mouse, barra sempre visível e arraste com retorno: o cartão de origem apagado, uma miniatura com o
  nome seguindo o ponteiro, marcador âmbar onde vai cair.
- **Início**: trilho (Início, Novo documento, Abrir arquivo, Criar com IA, Configurações), título, botões
  principais, área de arrastar, recentes e um cartão "Começar com um formato".
- **Novo documento** como página inteira: formatos, predefinições, prévia com réguas, configurações à
  direita, Voltar e Criar embaixo.

## O que adaptar por app

| App | Modos sugeridos | Faixa inferior |
|---|---|---|
| Vector | Desenhar · Selecionar · Texto · Cor · Efeitos | Camadas / Pranchetas / Símbolos |
| Light | Biblioteca · Revelar · Máscaras · Retoque · Exportar | Filme de fotos |
| Film | Montar · Cortar · Cor · Áudio · Legendas | Linha do tempo |
| Effect | Compor · Animar · Efeitos · Texto · Renderizar | Linha do tempo com camadas |
| Design | Página · Texto · Imagens · Estilos · Exportar | Páginas e espelhadas |
| Pdf | Ler · Editar · Comentar · Formulários · Proteger | Páginas |
| Ani | Desenhar · Animar · Símbolos · Som · Publicar | Linha do tempo de quadros |
| Texture | Pintar · Camadas · Máscaras · Assar · Exportar | Conjuntos de textura |
| Material | Grafo · Nós · Prévia · Exportar | Biblioteca de nós (o grafo é o centro) |
| Sound | Gravar · Editar · Mixar · Efeitos · Exportar | Faixas e mixer |
| Fig | Desenhar · Componentes · Protótipo · Inspecionar | Páginas e camadas |
| Word | Escrever · Formatar · Inserir · Revisar · Layout | Páginas |
| Grid | Dados · Fórmulas · Gráficos · Formatar · Revisar | Abas das planilhas |
| Deck | Slides · Texto · Mídia · Transições · Apresentar | Slides |
| CAD | Desenhar · Modificar · Anotar · Camadas · Plotar | Layouts |

Para cada app: o dono gera os mockups (as mesmas 7 telas, com o conteúdo do app) antes do código, e a
implementação segue as telas.
