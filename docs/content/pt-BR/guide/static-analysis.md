---
title: Análise Estática
---

<!-- Generated translation; source: guide/static-analysis.md -->

# Análise Estática

A pilha de análise do Vize é compartilhada pelo compilador, linter, verificador de tipos, servidor editor e ferramentas Musea
. O objetivo é analisar um SFC do Vue uma vez, manter informações semânticas ricas e reutilizá-las
para diagnósticos e geração de código, em vez de tratar cada comando como uma ferramenta separada.

Os exemplos abaixo assumem que o pacote `vize` npm é instalado e chamado a partir de scripts de projeto, que
é o fluxo de trabalho recomendado para aplicações.

## Pipeline

| Camada   | O que ele faz                                                                              | Usado por                                            |
| -------- | ------------------------------------------------------------------------------------------ | ---------------------------------------------------- |
| Armadura | Tokeniza e analisa templates Vue e estrutura SFC                                           | compilador, linter, formatador                       |
| Croquis  | Constrói escopos, vinculação de metadados, informações macro e gráficos de arquivo cruzado | Compilador, LINT e verificações conscientes de tipos |
| Pátina   | Roda Vue, script, CSS, a11y, SSR, Vapor, Musea e regras de lint conscientes de tipos       | `vize lint`, diagnóstico do editor, ponte Oxlint     |
| Canon    | Gera TypeScript virtual e mapeia diagnósticos de volta para arquivos do Vue                | `vize check`, verificação de tipos de editor         |
| Maestro  | Expõe diagnósticos e recursos do editor por meio do LSP                                    | `vize lsp`, VS Code, Zed                             |

Isso significa que análise estática não é apenas linting. Vinculações de templates, macros do compilador, metadados
componentes, relações de fornecimento e injeção, fluxo de reatividade, TypeScript virtual gerado e metadados
galeria de componentes dependem do mesmo trabalho de análise de nível inferior.

Para os nomes concretos das regras, padrões e códigos diagnósticos cruzados que podem ser emitidos, veja
[Rules](../rules/index.md).

## Linting

Comece com o predefinido padrão:

```json
{
  "scripts": {
    "vize:lint": "vize lint src"
  }
}
```

```bash
vp run vize:lint
```

Use `essential` para CI apenas de correção, `happy-path` para o pacote recomendado padrão,
`opinionated` quando quiser convenções mais fortes, `nuxt` para suposições conscientes do Nuxt e
`incremental` quando você só quer regras explicitamente configuradas para rodar.

```json
{
  "scripts": {
    "vize:lint:ci": "vize lint --preset essential --max-warnings 0 src",
    "vize:lint:opinionated": "vize lint --preset opinionated --help-level short src",
    "vize:lint:fix": "vize lint --fix src",
    "vize:lint:json": "vize lint --format json src"
  }
}
```

```bash
vp run vize:lint:ci
vp run vize:lint:opinionated
vp run vize:lint:fix
vp run vize:lint:json
```

Opte por verificações de arquivo cruzado e conscientes do tipo somente depois que o caminho básico de lint estiver estável:

```json
{
  "scripts": {
    "vize:lint:cross-file": "vize lint --cross-file src",
    "vize:lint:cross-file-tree": "vize lint --cross-file --cross-file-tree src",
    "vize:lint:strict-reactivity": "vize lint --strict-reactivity src"
  }
}
```

```bash
vp run vize:lint:cross-file
vp run vize:lint:cross-file-tree
vp run vize:lint:strict-reactivity
```

O linting entre arquivos analisa relações como fornecer/injetar e fluxo de reatividade entre um conjunto de arquivos
Vue. `--strict-reactivity` ativa a regra nativa de perda de reatividade respaldada por checker, então espere que
seja mais lenta do que as regras normais de template e script lint.

## Sobreposição de Reatividade

O Croquis expõe uma sobreposição de reatividade estável para cada SFC analisado: fontes reativas, requisitos de `.value`
, locais de perda de reatividade e arestas de grafo de efeito com mapeamentos de fonte. O mesmo modelo compacto
JSON alimenta diagnósticos, relatórios, superfícies do editor e a aba **Reatividade** do Playground.

## Modelo da Regra da Pátina

Pátina é a camada de regra de fiapo. Regras são pequenos visitantes sobre a fonte SFC, raiz do template, elementos de template
, diretivas, `v-for`, `v-if`e interpolações. Cada regra carrega metadados
seu nome da regra, categoria, severidade padrão, texto de ajuda e se ela é corrigível. Presets são apenas
registros que decidem quais regras são ativadas juntos.

| Área                        | Regras de exemplo                                                                            | O que eles cobrem                                                 |
| --------------------------- | -------------------------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| Correção do Vue             | `vue/require-v-for-key`, `vue/valid-v-model`, `vue/no-use-v-if-with-v-for`                   | Semântica de template que é local para um componente              |
| Segurança Vue               | `vue/no-v-html`, `vue/no-unsafe-url`                                                         | Sinks de HTML e URL propensos a XSS                               |
| Estrutura da vista          | `vue/sfc-element-order`, `vue/require-scoped-style`, `vue/no-unused-components`              | Formato do SFC, uso de componentes e manutenibilidade             |
| Convenções de escrita       | `script/no-options-api`, `script/no-get-current-instance`, `script/prefer-import-from-vue`   | API de composição do Vue e convenções de macro do compilador      |
| CSS                         | `css/no-important`, `css/no-hardcoded-values`, `css/prefer-logical-properties`               | Blocos de estilo e CSS amigáveis ao sistema de design             |
| Acessibilidade              | `a11y/img-alt`, `a11y/anchor-has-content`, `a11y/label-has-for`                              | Marcação acessível e padrões de interação                         |
| HTML                        | `html/deprecated-element`, `html/id-duplication`, `html/no-empty-palpable-content`           | Validade HTML e marcação semântica                                |
| SSR                         | `ssr/no-browser-globals-in-ssr`, `ssr/no-hydration-mismatch`                                 | Perigos de renderização de servidor/cliente                       |
| Vapor                       | `vapor/no-vue-lifecycle-events`, `vapor/no-inline-template`, `vapor/require-vapor-attribute` | Restrições de template orientadas a vapor                         |
| Musea                       | `musea/require-title`, `musea/valid-variant`, `musea/prefer-design-tokens`                   | Galeria de componentes e autoria de variantes                     |
| Análise consciente de tipos | `type/require-typed-props`, `type/require-typed-emits`, `type/no-reactivity-loss`            | Regras que precisam de contexto semântico ou respaldado por damas |

Os presets embutidos têm como objetivo apoiar a adoção em etapas:

| Preset        | Formato                                                                         |
| ------------- | ------------------------------------------------------------------------------- |
| `essential`   | Correção focada em erros no Vue, segurança e checagens mínimas de HTML          |
| `happy-path`  | Pacote padrão para correção, segurança, a11y, SSR, verificações semânticas      |
| `opinionated` | `happy-path` mais convenções, regras de script e regras de tipo mais fortes     |
| `nuxt`        | Regras opinativas ajustadas para as suposições de importação automática da Nuxt |
| `incremental` | Ponto de partida vazio para adoção governada pelo host, regra por regra         |

## Pragmas de Migração e Regras Personalizadas

O Patina aceita pragmas de desabilitação ESLint existentes para corresponder nomes de regras, incluindo
`eslint-disable`, `eslint-enable`, `eslint-disable-next-line`e `eslint-disable-line`. Isso permite que
projetos migrem regras como `vue/require-v-for-key` sem reescrever todos os comentários de supressão
logo no início.

Módulos de regras JavaScript locais em projetos ainda não são uma API estável em tempo de execução do Vize. Durante a migração, mantenha
essas regras no ESLint ou Oxlint e execute ao lado de `vize lint`, ou use o preset `incremental` para
ativar apenas as regras embutidas do Vize que já correspondem à sua política. O objeto de configuração `rules` controla
severidades das regras embutidas do Vize pelo nome.

No caso comum de proibir um global de ambiente de execução (regras típicas de ESLint sidecar como
`no-access-process`, `no-access-local-storage`ou `no-restricted-globals` contra `localStorage` /
`sessionStorage`), ative a regra de `script/no-restricted-globals` embutida de opt-in, em vez de manter
ESLint instalado apenas para essas regras. Sua lista padrão de negações é `process`, `localStorage`, e
`sessionStorage`, reportadas em cada referência nua.

Duas regras de script também aceitam configuração local de projeto sob `linter.ruleOptions` (#1891), então equipes
podem impor suas próprias convenções de arquitetura por meio de `vize lint`. `script/no-restricted-globals`
usa uma lista `globals` que **substitui** a lista padrão embutida; `script/no-restricted-members`
desliga até ser configurado e sinaliza `<object>.<property>` acessos de uma lista de `members`. As opções são digitadas
(`name` / `object` / `property` mais uma `message`opcional, com chaves desconhecidas rejeitadas); Um
`message` desaparecido volta a um aviso genérico.

```json
{
  "linter": {
    "rules": {
      "script/no-restricted-globals": "error",
      "script/no-restricted-members": "error"
    },
    "ruleOptions": {
      "script/no-restricted-globals": {
        "globals": [
          { "name": "process", "message": "Read env via a typed helper." },
          { "name": "alert" }
        ]
      },
      "script/no-restricted-members": {
        "members": [
          { "object": "window", "property": "localStorage", "message": "Use authStorage." }
        ]
      }
    }
  }
}
```

## Regras de Arquivo Cruzado

A análise cruzada está em Croquis e é exposta ao linting por meio de diagnósticos de pátina. É
opt-in porque constrói um registro de módulos, grafo de importação, grafo de uso de componentes e índices adicionais de
em todos os arquivos Vue analisados.

Hoje, `vize lint --cross-file` possibilita correspondência de fornecer/injeção, verificações únicas de identificação de elementos, rastreamento de reatividade
e análise assíncrona de condição racial. `--cross-file-tree` imprime a árvore de
fornecer/injetar sobre esses diagnósticos.

```bash
vp run vize:lint:cross-file
vp run vize:lint:cross-file-tree
```

O motor cross-file de nível inferior é mais amplo do que a superfície CLI atual:

| Opção de cruzar limas     | Diagnósticos ou fatos pretendidos                                                                     |
| ------------------------- | ----------------------------------------------------------------------------------------------------- |
| `provide_inject`          | Injeções não combinadas, fornecimentos não utilizados, avisos de string-key, fluxos não reativos      |
| `unique_ids`              | IDs duplicados e IDs não únicos introduzidos dentro dos loops                                         |
| `reactivity_tracking`     | Desestruturação de hélices, aliasing e perda de reatividade entre componentes                         |
| `race_conditions`         | Atualizações de estado assíncrono que podem passar rapidamente pelo estado fornecido ou compartilhado |
| `fallthrough_attrs`       | `$attrs`, `inheritAttrs`, e riscos de queda com múltiplas raízes                                      |
| `component_emits`         | Emitentes não declarados, não utilizados e ouvintes sem produtor                                      |
| `event_bubbling`          | Eventos que atravessa os limites dos componentes sem serem tratados                                   |
| `server_client_boundary`  | Uso da API do navegador e riscos de hidratação ao redor dos limites SSR/cliente                       |
| `error_suspense_boundary` | Componentes assíncronos sem limites úteis de suspense ou erro                                         |
| `circular_dependencies`   | Ciclos de importação e cadeias profundas de importação                                                |
| `component_resolution`    | Uso de componentes não registrados ou não resolvidos                                                  |
| `props_validation`        | Adereços necessários faltando e incompatibilidades no tipo de prop infantil                           |

A direção é manter o linting de arquivo único rápido por padrão, expor grupos de arquivos cruzados explicitamente à medida que amadurecem
eles amadurecem e rotear fatos de projeto de alta confiança para o mesmo fluxo de diagnóstico usado pela CLI
, ponte Oxlint e servidor editor.

## Verificação de Tipos

`vize check` gera TypeScript virtual para SFCs do Vue e solicita aos projetos Corsa
diagnósticos. Ele verifica `.vue`, `.ts`, `.tsx``.d.ts` e mapeia diagnósticos de volta para os arquivos fonte
originais.

```json
{
  "scripts": {
    "vize:check": "vize check",
    "vize:check:src": "vize check src",
    "vize:check:app": "vize check --tsconfig tsconfig.app.json",
    "vize:check:json": "vize check --format json --quiet",
    "vize:check:virtual-ts": "vize check --show-virtual-ts src/components/App.vue",
    "vize:check:profile": "vize check --profile src",
    "vize:check:single-server": "vize check --servers 1 src",
    "vize:check:declarations": "vize check --declaration --declaration-dir dist/types"
  }
}
```

```bash
vp run vize:check
vp run vize:check:src
vp run vize:check:app
vp run vize:check:json
```

Quando não há caminhos fornecidos, `vize check` lê `tsconfig.json` `files`, `include`e `exclude`
campos se houver uma configuração de projeto disponível. Use `--show-virtual-ts` ao depurar código gerado e
`--profile` quando precisar de tempos e artefatos de arquivos virtuais sob `node_modules/.vize`.

```bash
vp run vize:check:virtual-ts
vp run vize:check:profile
vp run vize:check:single-server
```

A saída da declaração está disponível do projeto checker materializado:

```bash
vp run vize:check:declarations
```

Os valores do template em todo o projeto e os arquivos de declaração gerados devem ser visíveis através do TypeScript
configuração do projeto. Coloque declarações ambient sob um caminho incluído pelo seu `tsconfig` e passe
arquivo do projeto para o verificador quando necessário:

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
    $route: { path: string };
  }
}
```

```bash
vp run vize:check:app
```

## Scripts de Pacote npm vs CLI Rust

O pacote npm `vize` é destinado a scripts de pacote e utiliza a vinculação NAPI empacotada:

```json
{
  "scripts": {
    "vize:lint": "vize lint src",
    "vize:check": "vize check src --strict",
    "vize:ready": "vize ready src"
  }
}
```

```bash
vp run vize:lint
vp run vize:check
vp run vize:ready
```

O CLI Rust atualmente possui a superfície de verificação de tipos mais completa, apoiada por projetos:

```bash
nix run github:ubugeeei-prod/vize#vize -- check --tsconfig tsconfig.app.json --profile src
vize check --tsconfig tsconfig.app.json --profile src
vize lsp
```

Use scripts de pacote npm quando quiser fluxos de trabalho instaláveis em uma aplicação. Use a CLI do Rust quando
precisar de `check-server`, LSP, gerenciamento de IDE ou o caminho de diagnóstico de projeto apoiado pela Corsa entre arquivos
Vue e TypeScript.

## Oxlint

Use `oxlint-plugin-vize` quando sua equipe já roda o Oxlint e quiser diagnósticos conscientes do Vue no mesmo comando
:

```bash
vp install -D oxlint oxlint-plugin-vize
vp exec oxlint-vize -c .oxlintrc.json -f stylish src
```

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "settings": {
    "vize": {
      "preset": "essential",
      "helpLevel": "short"
    }
  },
  "rules": {
    "eqeqeq": "error",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn"
  }
}
```

## Caminho da Adoção

1. Adicione um script de `vize:lint:ci` pacote como `vize lint --preset essential src` ao CI.
2. Troque para `happy-path` ou `opinionated` depois que o diagnóstico de correção estiver limpo.
3. Adicione um script de `vize:check` pacote com seu projeto `tsconfig.json`.
4. Ative o linting do editor primeiro, depois a verificação de tipos quando a saída do CI estiver estável.
5. Adicione verificações de reatividade entre arquivos e rigorosas para projetos que se beneficiam de uma análise mais profunda.

Para uma única porta de qualidade, um script de `vize:ready` pacote rodando `vize ready src` executa `fmt

- -write`, `lint`, `check`e `build` em ordem e para na primeira etapa que falha.
