# Build, instalador e publicação

Tudo compila no GitHub Actions; o dono baixa o instalador da página de Releases.

## Workflows (copiar de `.github/workflows/` deste repositório e renomear)

**`build-windows.yml`** (em todo push do branch de trabalho):

1. `runs-on: windows-latest`; `concurrency` com `cancel-in-progress: true` (um push novo cancela o build
   anterior; não empurre no meio de um build que você quer).
2. Rust estável, cache do `target`.
3. WiX 5 (`dotnet tool install --global wix --version 5.0.2` e as extensões UI e Util).
4. Codex CLI: `npm install -g @openai/codex`, copiar `codex*.exe` para uma pasta e passar ao empacotador,
   que os põe ao lado do executável do app.
5. `cargo build --release` do app e da CLI.
6. Empacotar (`packaging/windows/package.ps1`): MSI e zip portátil, com nome e ícone vindos do `brand.env`.
7. Publicar como pré-release contínua `latest-<branch>`: apaga a anterior e cria de novo com os dois
   arquivos.

**`checks.yml`** (no branch de trabalho e em `worktree-agent-*`): clippy com `-D warnings` dos crates
mexidos e os testes da interface nova filtrados por nome. A suíte inteira da interface (~5.800 testes)
não roda no CI; rode a área tocada localmente.

O upstream usa `storytold/craft-common-actions` e segredos próprios: não funciona no fork. Substitua.

## `brand.env`

```
BRAND_NAME=Fuse Open Vector
BRAND_COMPANY=Fuse
BRAND_ID=com.fuse.openvector
BRAND_FILE=fuse-open-vector
```

Os `build.rs` exportam como variáveis de compilação (`<APP>_BRAND_NAME` …) e `studio::style` lê com
`env!`. O empacotador lê o mesmo arquivo. Ícones em `assets/app-icon/` (PNG, SVG, ICO e ICNS
regenerados).

## Assinatura

Os executáveis saem sem assinatura. O Controle Inteligente de Aplicativos do Windows, quando ligado,
bloqueia a execução, sem opção de "executar assim mesmo". Para uso próprio, o dono desligou o recurso.
Para distribuir, é preciso um certificado de assinatura de código (pago; decisão do dono) e um passo de
assinatura no workflow (`packaging/windows/sign.ps1` já existe no upstream).

## Acompanhar um build

```sh
gh run list -R <dono>/<repo> --commit $(git rev-parse HEAD) --json name,status,conclusion
gh release view latest-<branch> -R <dono>/<repo> --json publishedAt,targetCommitish
```

Consultar no máximo a cada 5 minutos. Um build que falha em menos de um minuto sem passos ("workflow
file issue") foi falha do GitHub: `gh run rerun <id> -R <dono>/<repo>` resolveu.
