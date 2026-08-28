---
title: Configuração
---

<!-- Generated translation; source: guide/configuration.md -->

# Configuração

Vize usa `vize.config.*` para comandos compartilhados de pacotes npm, plugin Vite e configurações de CLI Rust.

## Arquivos de Configuração

O pacote npm comandos e `@vizejs/vite-plugin` carregar esses arquivos da raiz do projeto nesta ordem
prioridade:

- `vize.config.pkl`
- `vize.config.ts`
- `vize.config.js`
- `vize.config.mjs`
- `vize.config.json`

A CLI do Rust lê os mesmos nomes de arquivos de configuração na ordem acima, para configurações nativas de comando, como
`check`, `lint`, `lsp`e `fmt`.

## Configuração do TypeScript

```ts
import { defineConfig } from "vize";

export default defineConfig(({ command, mode, isSsrBuild }) => ({
  compiler: {
    sourceMap: mode !== "production",
    ssr: isSsrBuild,
    vapor: false,
    customRenderer: false,
    templateSyntax: "standard",
  },
  vite: {
    include: [/\.vue$/],
    exclude: [/node_modules/],
    scanPatterns: ["src/**/*.vue"],
    ignorePatterns: ["node_modules/**", "dist/**", ".git/**"],
  },
  linter: {
    enabled: command !== "build",
    preset: "happy-path",
  },
  typeChecker: {
    enabled: true,
    strict: true,
  },
  formatter: {
    printWidth: 100,
    singleQuote: false,
  },
  lsp: {
    lint: true,
    typecheck: false,
    editor: false,
    formatting: false,
  },
  musea: {
    include: ["src/**/*.art.vue"],
    basePath: "/__musea__",
  },
}));
```

## Resolução do Tipo de Vue

O Vize não fixa a superfície de tipos do Vue do pacote de `vize` publicado: `vize check`, a linguagem
servidor e os comandos do pacote resolvem `vue`, `@vue/compiler-sfc`, e tipos ambientais relacionados do projeto
analisado, então as escolhas de patch, minor e pré-release do Vue 3 permanecem sob o controle desse projeto,
em vez da versão usada para construir o Vize. Para resultados previsíveis, declare a versão suportada do Vue
no projeto de usuário (não via internos do Vize), mantenha `vue`, `@vue/compiler-sfc`e
integrações alinhadas como o Nuxt ali, e execute `vize check` da raiz do projeto ou ponto
`typeChecker.tsconfig` no pacote de destino; usar `typeChecker.corsaPath` apenas para escolher o checker
binário, nunca para sobrescrever versões do tipo Vue. Quando um projeto suporta múltiplos intervalos de Vue, teste cada
em sua própria matriz de pacotes para que o Vize siga o grafo de dependência ativa, e não um caminho de tipo codificado fixamente.

## Entradas Experimentais em Flat

Monorepos pode descrever padrões raiz e overrides com escopo de pacote com `entries`. Configurações de objetos simples
são normalizadas para uma entrada internamente, e exportações de array são aceitas por `defineConfig` para
autoria no estilo ESLint-flat-config.

```ts
export default defineConfig({
  formatter: {
    printWidth: 100,
  },
  entries: [
    {
      name: "web app",
      basePath: "apps/web",
      files: ["src/**/*.vue"],
      typeChecker: {
        tsconfig: "tsconfig.app.json",
      },
    },
    {
      name: "ui package",
      basePath: "packages/ui",
      files: ["src/**/*.vue"],
      formatter: {
        singleQuote: true,
      },
    },
  ],
});
```

## Configuração PKL

```pkl
amends "node_modules/vize/pkl/vize.pkl"

compiler {
  sourceMap = true
  vapor = false
  customRenderer = false
  templateSyntax = "standard"
}

vite {
  scanPatterns = new Listing {
    "src/**/*.vue"
  }
}

linter {
  preset = "happy-path"
}

typeChecker {
  enabled = true
  strict = true
}

entries = new Listing {
  new ConfigEntry {
    name = "web app"
    basePath = "apps/web"
    files = new Listing { "src/**/*.vue" }
    typeChecker {
      tsconfig = "tsconfig.app.json"
    }
  }
}

lsp {
  lint = true
  typecheck = false
  editor = false
  formatting = false
}
```

## Configuração JSON

```json
{
  "$schema": "./node_modules/vize/schemas/vize.config.schema.json",
  "compiler": {
    "sourceMap": true,
    "vapor": false,
    "customRenderer": false,
    "templateSyntax": "standard"
  },
  "vite": {
    "scanPatterns": ["src/**/*.vue"]
  },
  "linter": {
    "preset": "happy-path"
  },
  "typeChecker": {
    "enabled": true,
    "strict": true
  },
  "musea": {
    "include": ["src/**/*.art.vue"],
    "basePath": "/__musea__"
  }
}
```

## Opções do compilador

Essas opções estão sob `compiler`. Eles são respaldados por esquemas e compartilhados por meio de `defineConfig`; Não
toda integração consome todos os campos ainda.

| Opção               | Valores                               | Uso comum                                                                              |
| ------------------- | ------------------------------------- | -------------------------------------------------------------------------------------- |
| `sourceMap`         | `boolean`                             | Habilitar os mapas de origem no plugin Vite                                            |
| `ssr`               | `boolean`                             | Compilar para SSR quando não estiver dependendo da flag de build SSR do Vite           |
| `vapor`             | `boolean`                             | Ativar compilação em modo vapor                                                        |
| `jsxMode`           | `"vdom"` ou `"vapor"`                 | Backend de saída padrão para componentes `.jsx`/`.tsx`                                 |
| `customRenderer`    | `boolean`                             | Trate tags minúsculas que não sejam HTML como elementos de renderização personalizados |
| `customElements`    | `string[]`                            | Padrões de tag compilados como custom elements (`Tres*` para TresJS)                   |
| `templateSyntax`    | `"standard"`, `"strict"`ou `"quirks"` | Escolha o tratamento de aviso, erro ou peculiaridade do Vue para a sintaxe do modelo   |
| `scriptExt`         | `"ts"` ou `"js"`                      | Preserve a saída do TS ou faça downcompile para JS no comando de build npm             |
| `mode`              | `"module"` ou `"function"`            | Modo de saída de compilador de nível inferior                                          |
| `prefixIdentifiers` | `boolean`                             | Identificadores de prefixos com `_ctx`                                                 |
| `hoistStatic`       | `boolean`                             | Controle o içamento estático do nó                                                     |
| `cacheHandlers`     | `boolean`                             | Cache do gerenciador de eventos de controle                                            |
| `isTs`              | `boolean`                             | Analisar blocos de script como TypeScript                                              |
| `runtimeModuleName` | `string`                              | Módulo de importação em tempo de execução Override                                     |
| `runtimeGlobalName` | `string`                              | Override global em tempo de execução para saída no estilo função/IIFE                  |

Para projetos Vite, opções diretas de plugins sobrepõem a configuração compartilhada:

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [
    vize({
      vapor: true,
      sourceMap: true,
      customRenderer: true,
      templateSyntax: "standard",
    }),
  ],
});
```

## Sintaxe do Template

`compiler.templateSyntax` padrão para `"standard"`.

- `"standard"` aceita sintaxe inválida recuperável, emite avisos e reescreve para saída válida.
- `"strict"` reporta sintaxe inválida como erros de compilação.
- `"quirks"` preserva as peculiaridades de compatibilidade da sintaxe dos modelos sem avisos adicionais.

Os casos conhecidos são:

- `v-for` apelidos com parênteses de borda não combinados. O Vue tira uma `(` dianteira ou `)`
  do alias anterior a ele se divide `value`, `key`e `index`; os modos padrão e estrito relatam
  esses aliases como malformados, enquanto o modo quirk espelha o Vue.
- Elementos HTML não nulos escritos com sintaxe auto-fechante, como `<div />` ou `<span />`.
  modo Standard alerta e reescreve como elementos vazios, erros de modo estrito, e o modo quirk mantém
  como folhas que se fecham sozinhas.

```text
<template>
  <!-- Standard/strict reject this. Quirk mode compiles it as `item in items`. -->
  <div v-for="(item in items">{{ item }}</div>

  <!-- Standard/strict reject this. Quirk mode compiles it as `item in items`. -->
  <div v-for="item) in items">{{ item }}</div>

  <!-- Standard warns and rewrites this as `<div></div>`. Strict errors. Quirk keeps it as a leaf. -->
  <div />
</template>
```

Implementação upstream do Vue:

- [`forAliasRE`](https://github.com/vuejs/core/blob/main/packages/compiler-core/src/utils.ts#L571)
- [`stripParensRE` in `parseForExpression`](https://github.com/vuejs/core/blob/main/packages/compiler-core/src/parser.ts#L493-L530)

Veja [Troubleshooting](./troubleshooting.md) para o comportamento em modo estrito do HTML por trás de tags inválidas
auto-fechadas.

## Modo de Saída JSX & TSX

> Para a API completa de autoria, estilos com escopo, verificação de tipos, suporte a editores e limitações, veja o
> [JSX & TSX guide](./jsx.md). Esta seção cobre apenas as chaves de configuração do modo de saída.

O Vize compila componentes `.jsx`/`.tsx` Vue para saída Virtual DOM ou
[Vapor](https://blog.vuejs.org/posts/vue-vapor). `compiler.jsxMode` seleciona o \*\*global

- - padrão para componentes que não optam explicitamente; Ele é o padrão `"vdom"`.

```ts
// vize.config.ts
import { defineConfig } from "@vizejs/vite-plugin";

export default defineConfig({
  compiler: {
    // Default every .jsx/.tsx component to Vapor output.
    jsxMode: "vapor",
  },
});
```

`jsxMode` é independente do `compiler.vapor`: `vapor` alterna o Vapor para `.vue` SFCs, enquanto `jsxMode`
controla o backend padrão para JSX/TSX. Um projeto pode manter SFCs no VDOM enquanto o JSX é usado por padrão para
Vapor, ou vice-versa. O plugin Vite também aceita `jsxMode` diretamente como opção de plugin, o que
sobrepõe a configuração compartilhada.

### Diretivas por componente

Um componente individual sobrescreve o padrão com um prólogo diretivo, espelhando `"use strict"`:

```tsx
// Compiled to Vapor regardless of the configured default.
const Fast = () => {
  "use vue:vapor";
  return <div class="fast" />;
};

// Compiled to Virtual DOM regardless of the configured default.
const Classic = () => {
  "use vue:vdom";
  return <div class="classic" />;
};
```

Como cada componente é roteado de forma independente, um **único módulo pode misturar ambos os backends**:

```tsx
// vize.config: { compiler: { jsxMode: "vapor" } }

// No directive -> takes the configured default (Vapor here).
export const Dashboard = () => <main>{/* ... */}</main>;

// Opts back into Virtual DOM just for this component.
export const LegacyWidget = () => {
  "use vue:vdom";
  return <aside>{/* ... */}</aside>;
};
```

### Precedência

O modo de saída de um componente resolve nesta ordem:

1. Uma diretiva `"use vue:vapor"` / `"use vue:vdom"` por componente.
2. O `compiler.jsxMode` padrão da configuração (ou da opção `jsxMode` do plugin).
3. O plano B embutido, `"vdom"`.

### Diagnósticos

Uma diretiva que começa com `"use vue:"` mas não nomeia um modo conhecido (um erro de digitação como
`"use vue:vdomx"`) é reportada como erro de compilação em vez de ser ignorada silenciosamente, e duas diretivas de modo
conflitantes em um componente (`"use vue:vapor"` seguidas de `"use vue:vdom"`) também são
diagnosticadas. Prólogos não relacionados, como `"use strict"`, ficam intocados.

## Dialeto Vue

`dialect` seleciona o perfil do dialeto Vue para documentos HTML independentes (`.html`/`.htm`):

```json
{
  "dialect": "petite-vue"
}
```

- `"vue"` trata documentos HTML autônomos como documentos simples do Vue a partir do CDN.
- `"petite-vue"` opta documentos HTML autônomos para o
  [petite-vue](https://github.com/vuejs/petite-vue) dialeto (completações`v-scope`/`v-effect`
  e recursos IDE conscientes da petite-vue).

Quando a chave está ausente, o dialeto é detectado estruturalmente por documento: um `<script src>`
resolvendo para o pacote petite-vue, uma importação ES inline de `petite-vue`ou uma chamada `PetiteVue.createApp`
. Menções a petite-vue em comentários ou prosa nunca mudam o dialeto, e componentes de
em fila única sempre usam o dialeto padrão do Vue.

## Opções de Análise Estática

Use `linter` para o caminho de fiapos npm:

```ts
export default defineConfig({
  linter: {
    enabled: true,
    preset: "opinionated",
    rules: {
      "vue/require-v-for-key": "error",
      "vue/no-v-html": "warn",
    },
  },
});
```

Use `typeChecker` para o caminho da verificação do NPM:

```ts
export default defineConfig({
  typeChecker: {
    enabled: true,
    strict: true,
    checkProps: true,
    checkEmits: true,
    checkTemplateBindings: true,
    // Vue 3 Options API template bindings; default-on (matches vue-tsc).
    optionsApi: true,
  },
});
```

`typeChecker.optionsApi` resolve os bindings de templates da API de Options do Vue 3
(`data`/`computed`/`methods`/`inject`/`setup`/`props` em um `<script> export default { ... }`simples ).
Ele vem na build padrão (não no recurso `legacy`), **está ativado por padrão** (correspondendo `vue-tsc`),
e roda apenas para componentes não`<script setup>`, para que o caminho comum permaneça sem custo; Configure
`optionsApi: false` para optar por não participar. O suporte legado para Vue 2.7 / Nuxt 2 (`typeChecker.legacyVue2`, que adiciona
os globais de templates Nuxt 2) é um opt-in separado para build `legacy`.

`typeChecker.tsconfig` e `typeChecker.corsaPath` fazem parte do esquema compartilhado, mas o caminho Corsa
apoiado por projetos é hoje a superfície Rust CLI. `corsaPath` é compartilhado por `vize check`,
`vize lint`conscientes de tipo , e `vize lsp` (`typeChecker.tsgoPath` é um pseudônimo obsoleto); a pilha de
em tempo de execução é o pacote de plataforma nativa TypeScript 7 (`typescript` / `@typescript/typescript-*`)
com a camada API Corsa/corsa-bind. Deixe `corsaPath` indefinido, exceto se precisar apontar Vize
para um executável `lib/tsc` instalado específico. Mantenha declarações ambientais, arquivos gerados de autoimportação, aliases de caminho e declarações do Vue
`ComponentCustomProperties` no seu projeto `tsconfig.json`, e use um script de pacote
como `vize:check:app` para `--tsconfig` ou `--corsa-path` sobrescrições.

```json
{
  "typeChecker": {
    "servers": 1
  }
}
```

`typeChecker.servers` é reservado para futuros grupos de trabalhadores da Corsa. O executor direto de sessão de projeto
atualmente suporta apenas `1`; valores maiores falham rápido em vez de fingir ajustar a concorrência.

## Opções de Musea

A configuração compartilhada atualmente cobre o conjunto de arquivos da galeria e a rota:

```ts
export default defineConfig({
  musea: {
    include: ["src/**/*.art.vue"],
    exclude: ["node_modules/**", "dist/**"],
    basePath: "/__musea__",
    storybookCompat: false,
    inlineArt: false,
  },
});
```

Passe opções focadas em apresentações, como `previewCss`, `previewSetup`, `tokensPath`, `theme`e
`storybookOutDir` diretamente para `musea()` em `vite.config.ts`.
