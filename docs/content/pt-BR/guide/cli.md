---
title: CLI
---

<!-- Generated translation; source: guide/cli.md -->

# Referência CLI

> **⚠️ Trabalho em andamento:** Vize está em desenvolvimento ativo e a superfície CLI ainda está evoluindo.

A maioria dos fluxos de trabalho de aplicativos deve instalar o pacote `vize` npm e executá-lo por scripts `package.json`
. Esta página descreve o binário `vize` nativo de Rust de nível inferior para LSP, gerenciamento de IDE,
`check-server`, perfilamento e outros fluxos de trabalho diretos de CLI. O pacote npm expõe ajudantes de configuração
compartilhados, além de comandos `build`, `fmt`, `lint`, `check`, `clean`, `ready`e `upgrade` respaldados por NAPI.

Para uma explicação de nível mais alto do pipeline de análise, veja [Static Analysis](./static-analysis.md).

## Scripts de Pacotes de Aplicação

Para aplicativos, instale a partir do npm e conecte comandos Stable para scripts de projeto:

```bash
vp install -D vize
```

```json
{
  "scripts": {
    "vize:build": "vize build src",
    "vize:fmt": "vize fmt --write src",
    "vize:lint": "vize lint --preset happy-path src",
    "vize:check": "vize check src",
    "vize:ready": "vize ready src"
  }
}
```

```bash
vp run vize:lint
vp run vize:check
vp run vize:ready
```

Use `vp exec vize ...` para depuração local única, mas prefira scripts nomeados para fluxos de trabalho
documentados e CI.

## Instalação Binária de Ferrugem

Para a alpha v1, use os binários pré-construídos de release do GitHub ou o ponto de entrada do Nix. A CLI do Rust ainda não é um canal
crates.io de instalação suportado.

```bash
nix run github:ubugeeei-prod/vize#vize -- --help
```

Você também pode baixar binários específicos da plataforma de
[GitHub Releases](https://github.com/ubugeeei-prod/vize/releases).

Para desenvolvimento local dentro deste repositório, instale a build do workspace:

```bash
cargo install --path crates/vize --force --locked
```

## Scripts de Pacote npm vs CLI Rust

| Need                                                                        | Ponto de entrada recomendado  |
| --------------------------------------------------------------------------- | ----------------------------- |
| Pacotes de scripts para build, formatar, lint, check, ready e upgrade       | `vp run vize:*` do pacote NPM |
| Verificação de tipos apoiada por projetos em `.vue`, `.ts`, `.tsx`e `.d.ts` | Ferrugem `vize check`         |
| LSP, configuração do IDE, `check-server`e artefatos de perfilamento         | Ferrugem `vize` binário       |
| Plugin Shared Vite, comando npm package e configurações da CLI do Rust      | `vize.config.*`               |

## Comandos

```bash
vize [COMMAND]
```

Quando invocado sem um comando, `vize` por padrão é `build`.

| Comando        | Descrição                                                        |
| -------------- | ---------------------------------------------------------------- |
| `build`        | Compilar arquivos SFC do Vue                                     |
| `fmt`          | Formatar arquivos SFC Vue                                        |
| `lint`         | Arquivos SFC Lint Vue                                            |
| `check`        | Entradas de verificação de tipo Vue SFC, TS, TSX e `.d.ts`       |
| `inspector`    | Criar cargas úteis para inspetores de compiladores de playground |
| `clean`        | Remover artefatos de cache gerados pelo Vize                     |
| `ready`        | Execute `fmt`, `lint`, `check`e `build`                          |
| `upgrade`      | Atualize a CLI instalada                                         |
| `check-server` | Inicie o servidor Unix JSON-RPC typecheck                        |
| `musea`        | Subcomandos e andaimes de Musea                                  |
| `lsp`          | Inicie o servidor de idiomas                                     |
| `ide`          | Instalar ou gerenciar integrações de editores                    |

Todos os relatórios `--profile` terminais são feitos pela caixa de `vize_curator` apenas local. Os ganchos de instrumentação
permanecem em `vize_carton`, enquanto o curador possui a forma do relatório CLI ao lado
artefatos voltados para o inspetor e para o agente.

## Build

```bash
vize build src/**/*.vue
vize build --ssr
vize build --profile src
```

Principais opções:

| Opção                 | Descrição                                                                     |
| --------------------- | ----------------------------------------------------------------------------- |
| `-o, --output`        | Saída relativa à fonte abaixo da raiz de entrada comum; rejeita colisões      |
| `-f, --format`        | Formato de saída: `js`, `json`, `stats`                                       |
| `--ssr`               | Habilitar compilação SSR                                                      |
| `--custom-renderer`   | Tratar tags minúsculas não HTML como elementos de renderização personalizados |
| `--custom-elements`   | Padrões de tag compilados como custom elements; repetir para vários padrões   |
| `--script-ext`        | `preserve` ou `downcompile`                                                   |
| `--declaration`       | Emitir `.d.ts` arquivos para os SFCs construídos (alias: `--dts`)             |
| `--declaration-dir`   | Diretório de saída de declaração (padrão: o diretório de saída da compilação) |
| `-j, --threads`       | Substituição da contagem de threads                                           |
| `--profile`           | Perfil de tempo de impressão                                                  |
| `--continue-on-error` | Continue compilando e reporte falhas no final                                 |

## Formato

```bash
vize fmt --check src
vize fmt --write src
```

Principais opções:

| Opção                              | Descrição                                              |
| ---------------------------------- | ------------------------------------------------------ |
| `--check`                          | Arquivos de reporte que mudariam                       |
| `-w, --write`                      | Saída formatada de escrita                             |
| `--single-quote`                   | Estilo de citação com alternância de string            |
| `--print-width`                    | Largura máxima da linha                                |
| `--tab-width`                      | Largura da indentação                                  |
| `--use-tabs`                       | Alternar abas vs espaços                               |
| `--no-semi`                        | Omita pontos e vírgulas                                |
| `--sort-attributes`                | Atributos do modelo de ordenação                       |
| `--single-attribute-per-line`      | Coloque um atributo por linha                          |
| `--max-attributes-per-line`        | Enrolar após uma determinada contagem de atributos     |
| `--normalize-directive-shorthands` | Normalizar `v-bind:` / `v-on:` / `v-slot:` abreviações |
| `--profile`                        | Perfil de tempo de impressão                           |

## Fiapos

```bash
vize lint src
vize lint --preset opinionated src
vize lint --help-level short src
```

Principais opções:

| Opção                 | Descrição                                                                                                               |
| --------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `--fix`               | Aplique correções automáticas seguras de regras que forneçam edições de texto, depois reporte os diagnósticos restantes |
| `-f, --format`        | Formato de saída: `text`, `ansi`, `plain`, `json`, `stylish`, `markdown`, `html`ou `agent`                              |
| `--max-warnings`      | Falha quando os avisos excedem o limite                                                                                 |
| `-q, --quiet`         | Resumo do programa apenas                                                                                               |
| `--help-level`        | `full`, `short`ou `none`                                                                                                |
| `--preset`            | `happy-path`, `opinionated`, `essential`, `incremental`ou `nuxt`                                                        |
| `--cross-file`        | Ative verificações de arquivo entre arquivos opt-in                                                                     |
| `--cross-file-tree`   | Imprima a árvore de fornecer/injeção quando o linting entre arquivos estiver ativado                                    |
| `--strict-reactivity` | Permitir linting nativo de perda de reatividade respaldado por checker                                                  |
| `--profile`           | Perfil de tempo de impressão                                                                                            |
| `--slow-threshold`    | Limiar lento de arquivo para saída de perfil                                                                            |

Presets são destinados à adoção em etapas:

| Preset        | Use quando                                                                          |
| ------------- | ----------------------------------------------------------------------------------- |
| `essential`   | Você quer diagnósticos orientados à correção em CI                                  |
| `happy-path`  | Você quer o pacote recomendado padrão                                               |
| `opinionated` | Você quer convenções mais fortes, regras de roteiro e candidatos com perfil de tipo |
| `incremental` | Você só quer regras explicitamente configuradas                                     |
| `nuxt`        | Você quer regras opinativas com suposições de componentes de Nuxt                   |

Exemplos:

```bash
vize lint --preset essential --max-warnings 0 src
vize lint --preset opinionated --help-level short src
vize lint --cross-file --cross-file-tree src
vize lint --strict-reactivity src
vize lint --format ansi src
vize lint --format plain src
vize lint --format agent src
vize lint --format markdown src
```

## Confere

```bash
vize check
vize check src
vize check --tsconfig tsconfig.app.json
vize check --profile src
```

`vize check` é respaldado por sessões de projetos `vize_canon` e Corsa expostas por meio de [`corsa-bind`](https://github.com/ubugeeei/corsa-bind). O Vize gera TypeScript virtual para SFCs do Vue, executa diagnósticos de projeto em um caminho nativo e mapeia os resultados de volta para as localizações originais.

Quando não há caminhos explícitos, `vize check` usa `tsconfig.json` `files` / `include` /
`exclude` se disponível. Entradas explícitas podem ser arquivos, diretórios ou globos e podem incluir `.vue`,
`.ts`, `.tsx`e `.d.ts`.

Principais opções:

| Opção               | Descrição                                                       |
| ------------------- | --------------------------------------------------------------- |
| `-s, --socket`      | Conecte-se a um `check-server` em funcionamento                 |
| `--tsconfig`        | Sobreposição `tsconfig.json`                                    |
| `-f, --format`      | Formato de saída: `text` ou `json`                              |
| `--show-virtual-ts` | TypeScript virtual gerado por impressão                         |
| `-q, --quiet`       | Resumo do programa apenas                                       |
| `--profile`         | Escreva artefatos de perfil em `node_modules/.vize`             |
| `--corsa-path`      | Sobrescrever o caminho executável Corsa                         |
| `--servers`         | Contagem reservada de servidores Corsa; somente `1` é suportado |
| `--declaration`     | Emita `.d.ts` saída                                             |
| `--declaration-dir` | Diretório de saída para declarações emitidas                    |

Use `--corsa-path` quando quiser fixar um executável Corsa personalizado enquanto desenvolve o Vize ou testa um
`corsa-bind` local de verificação. A chave de configuração compartilhada é `typeChecker.corsaPath`; `typeChecker.tsgoPath`
é mantido apenas como um alias de compatibilidade.

Padrões úteis:

```bash
vize check --tsconfig tsconfig.app.json src
vize check --show-virtual-ts src/components/App.vue
vize check --profile src
vize check --declaration --declaration-dir dist/types
```

Os valores do modelo em todo o projeto e os tipos de ambiente do Vue devem ser visíveis através da configuração
do projeto TypeScript. Inclua arquivos gerados como `auto-imports.d.ts`, `components.d.ts`ou suas próprias declarações
Vue em `tsconfig.json`, e selecione esse projeto com `--tsconfig` quando necessário:

```json
{
  "include": ["src/**/*.ts", "src/**/*.tsx", "src/**/*.vue", "src/**/*.d.ts"]
}
```

```ts
// src/types/vue-app.d.ts
declare module "vue" {
  interface ComponentCustomProperties {
    $t: (key: string) => string;
  }
}
```

## Inspetor

```bash
vize inspector src/App.vue
vize inspector "src/**/*.vue" --target ssr
vize inspector src --format json --output inspector-payload.json
vize inspector src --format agent --output inspector-agent.json
```

`vize inspector` empacota um ou mais arquivos `.vue` no payload consumido pelo playground
inspetor do compilador. O navegador então inspeciona a saída do Vue, a saída do Vize, Virtual TS, VIR e o grafo cruzado de arquivos
, depois produz um permalink mais um link de pull request pré-preenchido.

Use `--format agent` quando outra ferramenta local ou agente de IA precisar da mesma reprodução sem abrir o navegador
. O relatório contém a carga útil exata, URL do playground, métricas resumo e gráfico de importação.
Metadados de carga útil, gráfico e diferencial de linha são construídos pela caixa de `vize_curator` local exclusiva para que a CLI e
inspeção do playground permaneçam alinhadas.

Principais opções:

| Opção               | Descrição                                            |
| ------------------- | ---------------------------------------------------- |
| `-f, --format`      | Formato de saída: `url`, `json`ou `agent`            |
| `--target`          | Destino do compilador: `dom` ou `ssr`                |
| `--playground-url`  | URL base do playground para links gerados            |
| `--max-files`       | Arquivos de limite incluídos em uma carga útil batch |
| `--custom-renderer` | Ativar a comparação de renderizadores personalizados |
| `--template-syntax` | Escolha `standard`, `strict`ou `quirks`              |
| `-o, --output`      | Escreva a URL ou o payload JSON em um arquivo        |

Veja [Compiler Inspector](./compiler-inspector.md) para o fluxo de trabalho dos colaboradores.

## Limpo

```bash
vize clean
vize clean --dry-run
vize clean --scope node-modules
vize clean --scope project
vize clean --force
vize clean path/to/project
```

`vize clean` remove artefatos locais conhecidos pertencentes ao Vize para a raiz do projeto selecionado, depois remove
`.vize` vazio e `node_modules/.vize` pais. A lista de artefatos gerenciados abrange saídas de perfis,
relatórios/snapshots/tokens Musea, sessões Patina, esquemas de configuração, logs LSP, restos de sockets, dumps de
OXC, arquivos de solução alternativa Oxlint e arquivos de projetos Corsa materializados. Entradas desconhecidas sob `.vize`
são preservadas por padrão; Use `--force` apenas quando a raiz do artefato selecionada deve ser removida
por completo. `--dry-run` imprime os caminhos dos artefatos que seriam removidos. Use `--scope node-modules`
ou `--scope project` quando apenas uma raiz de artefato deve ser limpa.

## Pronto

```bash
vize ready src
vize ready --output dist src
```

`vize ready` executa `fmt --write`, `lint`, `check`e `build` em ordem. O comando para no
primeiro passo que falha.

Principais opções:

| Opção          | Descrição                                |
| -------------- | ---------------------------------------- |
| `-o, --output` | Diretório de saída para a etapa de build |
| `--ssr`        | Habilitar compilação SSR para build      |
| `--script-ext` | `preserve` ou `downcompile`              |

## Atualização

```bash
vize upgrade
vize upgrade --dry-run
```

Por padrão, `vize upgrade` atualiza o pacote npm via Vite+:

```bash
vp install -D vize@latest
```

Use `--source cargo` apenas para instalações explícitas de carga locais.

## Musea

```bash
vize musea --help
vize musea serve --port 6006
vize musea new
```

O subcomando `musea` atualmente foca em andaimes e pontos de entrada experimentais.
Para o desenvolvimento diário de galerias, o fluxo de trabalho recomendado hoje é
`@vizejs/vite-plugin-musea`.

O pacote npm também expõe um comando de `vize musea` de conveniência que roda o Vite com o plugin Musea
instalado no seu projeto:

```bash
vp exec vize musea
vp exec vize musea --build
```

## LSP e IDE

```bash
vize lsp
vize lsp --port 9527
vize ide vscode
vize ide zed
```

`vize lsp` inicia o servidor de idiomas diretamente.
`vize ide` adiciona comandos de instalação e gerenciamento específicos do editor para as integrações VS Code e Zed
.

## Opções Globais

```bash
vize --help
vize --version
vize <command> --help
```
