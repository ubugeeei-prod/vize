---
title: Nuxt
description: Adicione o Vize a um projeto Nuxt com um único módulo para compilar, analisar e visualizar componentes Vue.
---

<!-- Generated translation; source: integrations/nuxt.md -->

# Nuxt

`@vizejs/nuxt` adiciona o Vize a um projeto Nuxt como um único módulo. Ele compila os arquivos `.vue` com o compilador Rust do Vize no lugar do compilador Vue padrão do Nuxt, gera uma configuração de lint adequada ao Nuxt e pode servir a galeria de componentes [Musea](../guide/musea.md). Imports automáticos, componentes, middlewares e SSR continuam funcionando pelo Nuxt.

> [!WARNING]
> O Vize está em desenvolvimento ativo e ainda não está pronto para produção. Compare a saída do build e os diagnósticos no seu próprio projeto antes de adotá-lo e consulte o [status do suporte](../stability.md).

## Configuração

### 1. Instale o módulo

Instale o `vp` uma vez pelo [guia de instalação do Vite+](https://viteplus.dev/guide/install) e depois adicione o módulo:

```bash
vp install @vizejs/nuxt
```

### 2. Registre o módulo

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
  },
});
```

`compiler: true` é o padrão, então `modules: ["@vizejs/nuxt"]` já basta. Todas as opções desta página ficam na chave `vize`.

### 3. Inicie o Nuxt

Inicie o servidor de desenvolvimento como de costume:

```bash
vp run dev
```

O módulo adiciona `@vizejs/vite-plugin` à configuração Vite do Nuxt e mantém as transformações próprias do Nuxt. Páginas, layouts, composables e componentes importados automaticamente funcionam como antes.

### 4. Confirme que o Vize compila seus componentes

Ative os logs de depuração do compilador para ver quais arquivos o Vize processa:

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: { debug: true },
  },
});
```

O Nuxt compila componentes sob demanda, então o log mostra `[vize] load: on-demand compiling ...` apenas para os arquivos que uma requisição realmente alcança. Remova `debug` depois de confirmar a configuração.

<span id="options"></span>

## Opções

Todas as opções ficam em `vize` no `nuxt.config.ts`. A maioria dos projetos só precisa das quatro primeiras.

| Opção                 | Tipo                                    | Padrão                     | O que faz                                                                                                                                                             |
| --------------------- | --------------------------------------- | -------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `compiler`            | `boolean \| VizeNuxtCompilerOptions`    | `true`                     | Compila SFCs Vue com o Vize. Um objeto repassa as opções para `@vizejs/vite-plugin`.                                                                                  |
| `lint`                | `boolean \| VizeNuxtLintOptions`        | `true`                     | Gera a configuração do Oxlint adequada ao Nuxt. Veja [Lint](#lint).                                                                                                   |
| `checker`             | `boolean \| VizeNuxtLintCheckerOptions` | `false`                    | Executa o lint junto ao servidor de desenvolvimento e mostra os resultados no terminal e no navegador. Veja [Lint durante o desenvolvimento](#lint-while-developing). |
| `musea`               | `boolean \| MuseaOptions`               | `false`                    | Serve a galeria Musea. Veja [Musea](#musea).                                                                                                                          |
| `ui`                  | `boolean \| VizeNuxtUiOptions`          | `false`                    | Registra automaticamente os componentes de `@vizejs/ui`. Veja [imports automáticos](../guide/auto-imports.md#nuxt).                                                   |
| `composables`         | `boolean \| VizeNuxtComposablesOptions` | `false`                    | Importa automaticamente as funções de `@vizejs/composable`. Veja [imports automáticos](../guide/auto-imports.md#nuxt).                                                |
| `bridge`              | `boolean \| VizeNuxtBridgeOptions`      | `true`                     | Reaplica aos módulos compilados pelo Vize as transformações do Nuxt de imports automáticos, componentes, i18n e chaves de dados assíncronos.                          |
| `unocss`              | `boolean \| VizeNuxtUnoCssOptions`      | `true`                     | Permite que o UnoCSS leia o código `.vue` original. `originalSource: false` desativa; `maxBytes` limita o tamanho lido por arquivo.                                   |
| `dev.stylesheetLinks` | `boolean`                               | `true`                     | Remove links de folhas de estilo duplicados ou inseguros do HTML SSR de desenvolvimento.                                                                              |
| `compatibility`       | `VizeNuxtCompatibilityOptions`          | detectado                  | Substitui as versões principais de Nuxt e Vue detectadas. Veja [Nuxt 2 e Vue 2](#nuxt-2-and-vue-2).                                                                   |
| `nuxtMusea`           | `NuxtMuseaOptions`                      | `{ route: { path: "/" } }` | Descreve a rota, a configuração de runtime e os mocks de fetch vistos pelas prévias do Musea.                                                                         |

### Compilador

Qualquer opção de `@vizejs/vite-plugin` pode ser passada em `compiler`. O Nuxt mantém seus próprios padrões para a raiz do projeto, a compilação sob demanda e os arquivos `.vue` de dependências:

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compatibility: {
      // Usually inferred automatically.
      // Nuxt 2 defaults to Vue 2 compatibility mode; Nuxt 3/4 defaults to Vue 3.
      vueVersion: 3,
    },
    compiler: {
      // Any @vizejs/vite-plugin option can be passed here.
      configMode: "auto",
      customRenderer: false,
      debug: false,
      handleNodeModulesVue: true,
      ignorePatterns: ["node_modules/**", ".nuxt/**", ".output/**"],
      precompileBatchSize: 64,
      scanPatterns: [], // Nuxt defaults to on-demand compilation
      sourceMap: true,
      vapor: false,
    },
    bridge: {
      autoImports: true,
      components: true,
      i18n: true,
      stableInjectedKeys: true,
    },
    unocss: {
      originalSource: {
        maxBytes: 2 * 1024 * 1024,
      },
    },
    dev: {
      stylesheetLinks: true,
    },
    musea: false,
  },
});
```

O [guia do plugin Vite](../guide/vite-plugin.md#compiler-options) descreve cada opção do compilador. Com `compiler: false`, o Nuxt mantém o compilador padrão e você usa apenas o lint ou o Musea.

<span id="lint"></span>

### Lint

Com `lint` ativado, o módulo grava `.oxlint.vize.json` na raiz do projeto. É o equivalente para o Oxlint da configuração gerada pelo `@nuxt/eslint`: blocos de regras de acordo com a sua versão do Nuxt e seus diretórios. Adicione esse arquivo ao `.gitignore`.

Se o projeto ainda não tem configuração do Oxlint nem do Vite, o módulo também cria um `oxlint.config.mts` que carrega o arquivo gerado. Uma configuração existente nunca é reescrita; carregue `.oxlint.vize.json` nela você mesmo.

| Opção        | Padrão                | O que faz                                                                                          |
| ------------ | --------------------- | -------------------------------------------------------------------------------------------------- |
| `configFile` | `".oxlint.vize.json"` | Onde a configuração gerada é gravada. Precisa ficar na raiz do lint ou em um diretório acima dela. |
| `autoInit`   | `true`                | Cria `oxlint.config.mts` quando não há configuração do Oxlint ou do Vite.                          |
| `rootDir`    | a raiz do Nuxt        | Diretório usado como base para os globs gerados.                                                   |
| `devtools`   | `{ enabled: "lazy" }` | Mostra o plano de lint resolvido em uma aba do Nuxt DevTools.                                      |

`lint` também aceita as opções de recursos do `@nuxt/eslint-config`: `standalone` (padrão `true`), `import` (padrão `true`), `typescript`, `stylistic`, `tooling`, `formatters` e `nuxt`.

Use `lint.autoInit: false` quando a configuração de lint fica fora da raiz do Nuxt. O [guia do plugin Oxlint](../guide/oxlint.md) explica como executar as regras.

<span id="lint-while-developing"></span>

### Lint durante o desenvolvimento

`checker` executa o Oxlint e as regras de lint do Vize junto ao servidor de desenvolvimento, como o checker do `@nuxt/eslint`:

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    checker: true,
  },
});
```

| Opção         | Padrão                                      | O que faz                                                      |
| ------------- | ------------------------------------------- | -------------------------------------------------------------- |
| `lintOnStart` | `true`                                      | Analisa o projeto inteiro quando o servidor inicia.            |
| `cache`       | `true`                                      | Depois da inicialização, analisa apenas os arquivos alterados. |
| `include`     | `<srcDir>/**/*.{js,jsx,ts,tsx,vue}`         | Arquivos verificados.                                          |
| `exclude`     | `**/node_modules/**` e o diretório de build | Arquivos e diretórios ignorados.                               |
| `emitWarning` | `true`                                      | Mostra avisos no terminal e no navegador.                      |
| `emitError`   | `true`                                      | Mostra erros no terminal e no navegador.                       |
| `fix`         | `false`                                     | Aplica correções seguras antes de relatar.                     |
| `formatter`   | `"stylish"`                                 | Formato de saída do Oxlint.                                    |

<span id="musea"></span>

## Musea

Ative a galeria de componentes com `musea`:

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
    musea: {
      include: ["**/*.art.vue"],
      tokensPath: "assets/tokens.json",
      previewCss: ["assets/styles/main.css", "assets/styles/musea-preview.css"],
      previewSetup: "musea.preview.ts",
    },
    nuxtMusea: {
      route: { path: "/" }, // Musea UI route within __musea__
    },
  },
});
```

Durante o desenvolvimento, a galeria fica em `/__musea__/`. O [guia do Musea](../guide/musea.md) cobre arquivos art, design tokens e testes de regressão visual.

### Onde colocar os arquivos art

O Nuxt percorre todos os arquivos `.vue` dos diretórios de componentes, e os arquivos art do Musea também terminam em `.vue`. Mantenha os arquivos `*.art.vue` fora desses diretórios e indique o local ao Musea:

```txt
app/components/Tag.vue
stories/shared/Tag.art.vue
```

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    musea: {
      include: ["stories/**/*.art.vue"],
    },
  },
});
```

Com o Musea ativado, o módulo também exclui `**/*.art.vue` da varredura de componentes do Nuxt, então arquivos art ao lado de componentes não são registrados como componentes.

### Disponibilize recursos do Nuxt nas prévias

Componentes costumam depender de `NuxtLink`, `useRoute`, `useNuxtApp`, `useRuntimeConfig`, composables de dados ou componentes nativos do Nuxt. Em uma configuração Vite separada para o Musea, adicione `@vizejs/musea-nuxt` e instale seus mocks em `previewSetup`:

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { musea } from "@vizejs/vite-plugin-musea";
import { nuxtMusea } from "@vizejs/musea-nuxt";

export default defineConfig({
  plugins: [
    nuxtMusea({
      route: { path: "/preview" },
      runtimeConfig: { public: { apiBase: "/api" } },
      fetchMocks: {
        "/api/user": { id: 1, name: "Ada" },
      },
    }),
    musea({
      previewSetup: "musea.preview.ts",
    }),
  ],
});
```

```ts
// musea.preview.ts
import { installNuxtMuseaMocks } from "@vizejs/musea-nuxt";
import { createI18n } from "vue-i18n";
import type { MuseaPreviewSetup } from "@vizejs/vite-plugin-musea";

export default ((app) => {
  installNuxtMuseaMocks(app, {
    route: { path: "/preview" },
    runtimeConfig: { public: { apiBase: "/api" } },
  });

  const i18n = createI18n({
    locale: "ja",
    messages: {
      ja: {/* ... */},
      en: {/* ... */},
    },
  });
  app.use(i18n);
}) satisfies MuseaPreviewSetup;
```

Dentro de uma aplicação Nuxt, o módulo não instala esses mocks globalmente, porque eles substituiriam os `#imports` do Nuxt. Os composables reais do Nuxt são usados nesse caso.

<span id="nuxt-2-and-vue-2"></span>

## Nuxt 2 e Vue 2

O módulo oferece suporte a Nuxt 2, 3 e 4. O compilador do Vize gera código Vue 3, então no Nuxt 2 o módulo deixa a compilação de `.vue` com o compilador existente (`@vitejs/plugin-vue2`, `vue-loader` ou o do próprio Nuxt) e continua fornecendo lint e Musea. O mesmo vale para projetos Vue mais antigos com `vueVersion: 0.11`, `vueVersion: 1` ou `vueVersion: "legacy"`.

As versões são detectadas automaticamente. Se um wrapper as esconde do Nuxt Kit, defina-as você mesmo:

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compatibility: {
      nuxtVersion: 2,
      vueVersion: 2,
    },
  },
});
```

## Use o plugin Vite diretamente

O Nuxt faz o build com o Vite, então o plugin Vite também funciona sem o módulo. Nesse caso, você perde as pontes específicas do Nuxt e a geração de lint descritas acima:

```ts
// nuxt.config.ts
import vize from "@vizejs/vite-plugin";

export default defineNuxtConfig({
  vite: {
    plugins: [vize()],
  },
});
```

## Como funciona

1. **Plugin Vite:** o módulo registra `@vizejs/vite-plugin`, que compila os arquivos `.vue`.
2. **Compatibilidade com o plugin Vue:** o plugin expõe a mesma API de `@vitejs/plugin-vue`, então as verificações internas do Nuxt continuam passando.
3. **SSR:** o Vize também compila o build do servidor e mantém separadas as variáveis de ambiente do cliente e do servidor.
4. **Transformações do Nuxt:** imports automáticos, composables e middlewares passam pela camada de transformação do Nuxt, que roda depois que o Vize compila o componente.

As versões publicadas de `@vizejs/nuxt` fixam `vize`, `@vizejs/vite-plugin`, `@vizejs/native` e os demais pacotes do Vize na mesma versão.

## Exemplo: Vue Fes Japan 2026

O site do [Vue Fes Japan 2026](https://vuefes.jp/2026) usa o módulo com Nuxt 4:

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: false, // compiler disabled (using Nuxt's default)
    musea: {
      include: ["**/*.art.vue"],
      inlineArt: false,
      tokensPath: "assets/tokens.json",
      previewCss: ["assets/styles/main.css", "assets/styles/musea-preview.css"],
      previewSetup: "musea.preview.ts",
    },
  },
});
```

Ele usa o Musea para desenvolver e documentar componentes e mantém o compilador padrão do Nuxt no build de produção.

## Solução de problemas

- **Configuração Pkl com pnpm:** `@vizejs/nuxt` depende de `vize`, que traz o `vize.pkl` padrão. Com pnpm, esse arquivo pode não estar em `node_modules/vize`. Instale `vize` diretamente se usar uma configuração Pkl:

  ```bash
  vp install vize
  ```

- **Um `lint.configFile` personalizado é ignorado:** o arquivo precisa estar em `lint.rootDir` ou em um diretório acima dele. Uma configuração dentro de `.nuxt/` não consegue aplicar as exclusões de projeto do Oxlint.
- **Links de recursos no HTML SSR de desenvolvimento:** `dev.stylesheetLinks` mantém links válidos de recursos do Nuxt codificados em URL, como `%40fs/` e caminhos `assets/` codificados, e remove links que contêm bytes nulos ou travessia de caminho depois de decodificados.

Para outros problemas, consulte a [solução de problemas](../guide/troubleshooting.md) ou [relate um problema](../guide/testing.md).
