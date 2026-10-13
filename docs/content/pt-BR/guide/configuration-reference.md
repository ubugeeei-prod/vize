---
title: Referência de configuração compartilhada
---

<!-- Reviewed translation; source: guide/configuration-reference.md; scope: introduction and discovery -->

# Referência de configuração compartilhada

Comece com os arquivos existentes `vite.config.*` e `tsconfig.json`; consulte o [guia de configuração](./configuration.md). Esta página descreve opções nativas e formatos específicos opcionais.

## Arquivos de Configuração

A configuração Vite é descoberta quando não há configuração específica no mesmo diretório do projeto mais próximo. A CLI também aceita `--config`; opções diretas do plugin e recursos explicitamente definidos no editor têm prioridade. Caminhos de configurações específicas partem do diretório da configuração. Os caminhos `typeChecker` do Vite e os `basePath` dos escopos usam o `root` Vite selecionado; entradas explícitas da CLI mantêm o diretório de execução como base. Use `vize.entries` e o projeto TypeScript do pacote para definir os escopos.


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
