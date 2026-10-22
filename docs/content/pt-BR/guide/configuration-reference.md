---
title: Referência de configuração compartilhada
---

<!-- Reviewed translation; source: guide/configuration-reference.md -->

<span id="standalone-configuration-reference"></span>

# Referência de configuração compartilhada

Use `vue:` na configuração Vite existente e `vueCompilerOptions` no projeto TypeScript. Consulte o [guia de configuração](./configuration.md) para uma configuração mínima.

## Arquivos de configuração

A CLI e o editor descobrem `vite.config.ts`, `.mts`, `.cts`, `.js`, `.mjs` e `.cjs` no projeto mais próximo. `package.json`, `tsconfig.json` e `jsconfig.json` delimitam os projetos. Os comandos podem selecionar uma configuração Vite com `--config`.

Arquivos específicos `vize.config.*` não são mais aceitos. Um arquivo específico antigo ao lado de uma configuração Vite é um erro, o mesmo erro de migração de um projeto que tenha apenas a configuração específica. Mova as opções nativas dos antigos formatos TypeScript, JSON ou Pkl para `vue:` em vez de criar outro arquivo específico.

Caminhos nativos relativos na configuração Vite usam o `root` Vite selecionado, que parte do diretório da configuração. Entradas explícitas da CLI e `--tsconfig` usam o diretório de execução. Opções diretas do plugin e recursos explicitamente definidos no editor prevalecem sobre as opções compartilhadas.

## Configuração TypeScript

```ts
// vite.config.ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  server: { port: 3000 },
}).withVue({
  compiler: {
    sourceMap: true,
    vapor: false,
    templateSyntax: "standard",
  },
  linter: { preset: "happy-path" },
  formatter: { printWidth: 100, singleQuote: false },
  languageServer: { hover: true },
});
```

A seção `vue` aceita `version`, `compiler`, `typeChecker`, `linter`, `formatter`, `languageServer`, `globalTypes`, `files`, `ignores`, `basePath`, `lib`, `vite`, `musea`, `experimentals`, `dialect`, `name` e `entries`. Use os nomes das opções nativas; membros desconhecidos geram um erro de configuração. Veja a [referência do compilador](./compiler-configuration-reference.md) para os modos de compilação e as [opções de regras](/rules/options.md) para opções lint tipadas.

## Resolução dos tipos Vue

`vize check` e o servidor de linguagem resolvem os tipos Vue do projeto analisado. Declare nesse projeto a versão Vue desejada e mantenha `vue`, `@vue/compiler-sfc` e integrações como Nuxt alinhados. As dependências usadas para compilar o próprio Vize não escolhem a versão Vue do consumidor. Projetos que suportam várias versões Vue devem verificar cada intervalo de dependências.

Mantenha declarações de ambiente, arquivos gerados de autoimportação, aliases de caminhos e declarações Vue `ComponentCustomProperties` no `tsconfig.json` do projeto. O `extends` padrão do TypeScript, incluindo caminhos de pacotes e arrays, e as referências de projeto preservam seu significado.

```json
{
  "extends": "./tsconfig.base.json",
  "compilerOptions": { "strict": true, "noEmit": true },
  "vueCompilerOptions": {
    "target": 3.5,
    "strictTemplates": true,
    "checkUnknownComponents": true
  },
  "include": ["src/**/*"]
}
```

As opções Vue aceitas incluem `target`, `strictTemplates`, `checkUnknownProps`, `checkUnknownComponents`, `checkUnknownEvents`, `checkUnknownDirectives`, `strictVModel`, `strictCssModules`, `jsxSlots`, `inferComponentDollarEl`, `inferComponentDollarRefs`, `inferTemplateDollarEl`, `inferTemplateDollarRefs`, `inferTemplateDollarSlots`, `inferTemplateDollarAttrs`, `fallthroughAttributes`, `checkRequiredFallthroughAttributes`, `resolveStyleImports`, `resolveStyleClassNames` e `vapor`. Isso não implica suporte a todas as opções do `vue-tsc`.

`target` aceita Vue 2.x até 2.7, Vue 3.x, `99` e `"auto"`. A seleção automática segue o pacote Vue instalado no projeto; `vue.version` explícito na configuração Vite tem prioridade. Valores não suportados geram um erro de configuração. O rigor do TypeScript permanece em `compilerOptions`; opções explícitas do verificador Vize substituem os padrões correspondentes do projeto.

### Projeto TypeScript e limites de suporte

Essas opções vêm do projeto TypeScript proprietário do arquivo, incluindo JSONC herdado, `extends` padrão por pacote ou array e `references`. `target: "auto"` resolve o pacote Vue instalado nesse projeto; uma versão ausente ou não suportada gera um erro de configuração. Um alvo numérico Vue 2 menor que 2.7 seleciona Vue 2, exatamente 2.7 seleciona Vue 2.7, e Vue 3.x ou `99` seleciona a família Vue 3 moderna. Um `vue.version` explícito no Vite, inclusive `"3"`, tem prioridade sobre `target`.

`strictComponentAttrs` também é suportado, mas exige ativação explícita; `strictTemplates` não o ativa automaticamente. `plugins` reconhece apenas o identificador `vue-router/volar/sfc-typed-router`; plugins de linguagem Vue arbitrários não são carregados.

`optionsWrapper` não é suportado. Nas ferramentas Vue ele descreve um prefixo e sufixo de código, não uma opção para ativar a Options API. A verificação Options API existente no Vize é controlada separadamente por `vue.typeChecker.optionsApi`. Essa lista é um mapeamento limitado, não uma garantia de paridade com todas as opções do `vue-tsc`.

## Entradas experimentais por escopo

Use o projeto TypeScript do pacote de destino para verificar monorepos. Opções lint Vue por escopo podem usar `vue.entries`:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  root: "apps/web",
  vue: {
    entries: [
      {
        files: ["src/trusted/**/*.vue"],
        linter: { rules: { "vue/no-v-html": "off" } },
      },
    ],
  },
});
```

As entradas aceitam `basePath`, `files`, `ignores` e `linter`; as outras seções continuam sendo opções do projeto inteiro. Não mova referências de projeto TypeScript para essas entradas.

## Reutilizar opções

Não existe um `extends` específico do Vize. Reutilize objetos JavaScript comuns:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
import vpConfig from "./build-options.ts";
import vueConfig from "./vue-options.ts";

export default defineConfig(vpConfig).withVue(vueConfig);
```

Isso não altera o `extends` padrão do TypeScript nem as funções de configuração Vite.

## Opções de análise estática

Coloque as opções lint Vue nativas em `vue.linter` e as opções do servidor de linguagem em `vue.languageServer`. As regras Vue usam `vue.linter.rules`; as regras do Oxlint continuam em `lint.rules`. Veja [execução de lint](/guide/lint-execution.md) (em inglês) para verificações entre arquivos e conscientes de tipos, e [opções de regras](/rules/options.md) para os objetos completos. Campos de opções desconhecidos são rejeitados.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    enabled: true,
    preset: "opinionated",
    rules: {
      "vue/require-v-for-key": "error",
      "vue/no-v-html": "warn",
    },
  },
  typeChecker: {
    enabled: true,
    checkProps: true,
    checkEmits: true,
    checkTemplateBindings: true,
    optionsApi: true,
  },
});
```

`vue.typeChecker.optionsApi` resolve os bindings de templates da Options API do Vue 3, incluindo `data`, `computed`, `methods`, `inject`, `setup` e `props` em um `<script>` convencional. Está ativado por padrão e só executa em componentes sem `<script setup>`; use `false` para desativá-lo. O suporte legado para Vue 2.7 / Nuxt 2 é um recurso separado da build `legacy`.

`vue.typeChecker.tsconfig` seleciona outro projeto TypeScript. Use `vue.typeChecker.corsaPath` somente para escolher um executável do verificador instalado, nunca para escolher os tipos Vue. `vue.typeChecker.servers` limita os processos paralelos do verificador (`1`–`32`); quando omitido, o Vize ajusta a concorrência ao projeto e às CPUs disponíveis.

<span id="opções-de-musea"></span>

## Configuração compartilhada do Musea

Mantenha as opções compartilhadas de descoberta, rota e VRT em `vue.musea`. Opções diretas de `musea()` substituem os valores compartilhados correspondentes.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [musea({ storybookCompat: false })],
  vue: {
    musea: {
      include: ["src/**/*.art.vue"],
      exclude: ["node_modules/**", "dist/**"],
      basePath: "/__musea__",
      inlineArt: false,
    },
  },
});
```

Opções de apresentação como `previewCss`, `previewSetup`, `tokensPath`, `theme` e `storybookOutDir` pertencem às opções do plugin `musea()`.

`musea-vrt` lê as opções compartilhadas `include`, `exclude` e `vrt` da mesma configuração Vite. `vize musea new` cria uma história de exemplo sem criar um arquivo de configuração específico.

## Receitas de migração

As configurações dedicadas JSON/PKL e o `extends` próprio do Vize foram removidos. Reutilize objetos JavaScript importados. Estas receitas preservam as opções compatíveis durante a migração.

### Configuração dinâmica do Vite

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig(({ command, mode, isSsrBuild }) => ({
  vue: {
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
    languageServer: {
      lint: true,
      typecheck: false,
      editor: false,
      formatting: false,
    },
    musea: {
      include: ["src/**/*.art.vue"],
      basePath: "/__musea__",
    },
  },
}));
```

### Migrar configurações JSON e PKL removidas

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  compiler: {
    sourceMap: true,
    vapor: false,
    customRenderer: false,
    templateSyntax: "standard",
  },
  vite: { scanPatterns: ["src/**/*.vue"] },
  linter: { preset: "happy-path" },
  typeChecker: { enabled: true, strict: true },
  languageServer: { lint: true, typecheck: false, editor: false, formatting: false },
  musea: { include: ["src/**/*.art.vue"], basePath: "/__musea__" },
});
```

### Projetos separados por pacote

`entries` define apenas escopos de lint. Mantenha as opções do verificador e do formatador nos projetos Vite e TypeScript de cada pacote. Os dois arquivos abaixo pertencem a projetos separados.

```ts
// apps/web/vite.config.ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  name: "web app",
  files: ["src/**/*.vue"],
  formatter: { printWidth: 100 },
  typeChecker: { tsconfig: "tsconfig.app.json" },
});
```

```ts
// packages/ui/vite.config.ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
export default defineConfig({}).withVue({
  name: "ui package",
  files: ["src/**/*.vue"],
  formatter: { printWidth: 100, singleQuote: true },
});
```

### Análise estática e processos de verificação

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    enabled: true,
    preset: "opinionated",
    rules: {
      "vue/require-v-for-key": "error",
      "vue/no-v-html": "warn",
    },
  },
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

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  typeChecker: { servers: 1 },
});
```

### Configurações de workflows existentes

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  formatter: {
    printWidth: 100,
  },
  linter: {
    preset: "happy-path",
  },
  typeChecker: {
    enabled: true,
    strict: true,
    tsconfig: "tsconfig.json",
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
```
