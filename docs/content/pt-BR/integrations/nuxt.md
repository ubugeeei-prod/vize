---
title: Nuxt
---

<!-- Generated translation; source: integrations/nuxt.md -->

# Integração Nuxt

> **⚠️ Trabalho em andamento:** O Vize está em desenvolvimento ativo e ainda não está pronto para uso em produção. Teste cuidadosamente antes de adotar em projetos Nuxt.

O Vize oferece integração Nuxt de primeira classe por meio do módulo `@vizejs/nuxt` . Isso substitui o compilador padrão do Vue da Nuxt pelo compilador Rust-native da Vize, proporcionando as mesmas melhorias de velocidade nos projetos Nuxt.

## Começando

### 1. Instalar o Módulo

Instale `vp` uma vez a partir do [Vite+ install guide](https://viteplus.dev/guide/install), depois adicione o módulo:

```bash
vp install @vizejs/nuxt
```

Se quiser usar `pkl` configuração com pnpm, talvez precise instalar o próprio pacote `vize`.
`@vizejs/nuxt` instala `vize` que atende `vize.pkl` com configuração padrão, mas a localização da `vize.pkl` pode variar ao usar o pnpm.

```bash
vp install vize
```

### 2. Registrar o módulo Nuxt

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
  },
});
```

### 3. Iniciar o Nuxt

Comece o servidor de desenvolvimento como de costume:

```bash
vp run dev
```

O módulo injeta `@vizejs/vite-plugin` na configuração do Vite do Nuxt e mantém transformações específicas do Nuxt
no pipeline, então auto-importações, componentes, middleware e comportamento SSR continuam funcionando
Nuxt.
Durante o desenvolvimento, a limpeza de resposta do servidor preserva links válidos de ativos Nuxt codificados por URL,
como `%40fs/` e caminhos codificados `assets/` , enquanto descartam caminhos de byte nulo decodificados ou de travessia.

## Opções de Módulo

`@vizejs/nuxt` mantém o simples `compiler: true | false` switch, mas as opções do módulo também expõem
o compilador Vize e as pontes de compatibilidade Nuxt para projetos que precisam de controle mais rigoroso:

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

| Opção                 | Tipo                                 | Padrão                     | Descrição                                                                                                                                                                                                                                    |
| --------------------- | ------------------------------------ | -------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `compatibility`       | `VizeNuxtCompatibilityOptions`       | Auto-detectado             | As sobrescrituras detectaram versões maiores do Nuxt/Vue para embalagens incomuns. O Nuxt 2 utiliza por padrão a compatibilidade host-compilador do Vue 2; O Nuxt 3/4 é o padrão do Vue 3. O Vue 0.11/1/2 todos usam o modo host-compilador. |
| `compiler`            | `boolean \| VizeNuxtCompilerOptions` | `true`                     | Habilita o Vize como compilador SFC do Vue. Passar um objeto encaminha opções para `@vizejs/vite-plugin` enquanto mantém os padrões Nuxt para `root`, `devUrlBase`, `scanPatterns`sob demanda e gerenciamento de SFC de dependência.         |
| `bridge`              | `boolean \| VizeNuxtBridgeOptions`   | `true`                     | Controla a ponte de transformação Nuxt para autoimportações, importações de componentes, auxiliares i18n e chaves de dados assíncronas estáveis em módulos virtuais do Vize.                                                                 |
| `unocss`              | `boolean \| VizeNuxtUnoCssOptions`   | `true`                     | Controla a ponte UnoCSS para módulos virtuais Vize. `originalSource: false` desativa a leitura de SFCs de fonte; `maxBytes` limita o uso de memória.                                                                                         |
| `dev.stylesheetLinks` | `boolean`                            | `true`                     | Permite a limpeza de links de SSR HTML apenas para desenvolvedores para URLs de assets Nuxt gerados pelo Vize.                                                                                                                               |
| `musea`               | `boolean \| MuseaOptions`            | `false`                    | Opta pela integração com a galeria Musea. Use `true` para os padrões do Musea ou passe um objeto para configurar padrões de inclusão, tokens, CSS de pré-visualização e roteamento.                                                          |
| `nuxtMusea`           | `NuxtMuseaOptions`                   | `{ route: { path: "/" } }` | Documenta a forma simulada Nuxt usada pelos ajudantes de pré-visualização do Musea. O módulo Nuxt não instala a camada mock globalmente porque isso faria sombra para a própria `#imports`do Nuxt.                                           |

## Configuração Avançada

### Nuxt 2 e Legacy Vue

Projetos Nuxt 2 usam saída do compilador Vue 2. O compilador SFC nativo do Vize mira o Vue 3, então o módulo Nuxt
evita automaticamente substituir o compilador host quando detecta o Nuxt 2. Para Nuxt 2 Bridge
ou outras configurações baseadas em Vite com Vue, o plugin Vite recebe `vueVersion: 2`, que mantém
`@vitejs/plugin-vue2`, `vue-loader`, ou o próprio compilador da Nuxt encarregado de `.vue` arquivos.

O mesmo modo host-compilador está disponível para projetos antigos do Vue via `vueVersion: 0.11`,
`vueVersion: 1`ou `vueVersion: "legacy"`.

Se seu projeto envolve o Nuxt de uma forma que oculta a versão do Nuxt Kit, defina explicitamente a compatibilidade
override:

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

### Usando o plugin Vite Diretamente

Alternativamente, você pode usar o plugin Vite diretamente. Como a Nuxt usa o Vite por baixo, isso funciona, mas carece de algumas otimizações específicas para a Nuxt:

```ts
// nuxt.config.ts
import vize from "@vizejs/vite-plugin";

export default defineNuxtConfig({
  vite: {
    plugins: [vize()],
  },
});
```

## Integração de Musea

O módulo Nuxt também suporta integração com Musea (galeria de componentes):

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

Quando configurada, a galeria Musea está disponível em `/__musea__/` durante o desenvolvimento.

### Posicionamento de Arquivos de Arte

A descoberta automática de componentes Nuxt escaneia arquivos `.vue` dentro de diretórios de componentes configurados. Como
arquivos de arte de Musea também terminam em `.vue`, mantenha `*.art.vue` arquivos fora desses diretórios nos projetos Nuxt
e aponte Musea para esse local:

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

Quando o Musea é ativado por meio do `@vizejs/nuxt`, o módulo também exclui `**/*.art.vue` do scanner de componentes
da Nuxt, para que arquivos legados colocalizados não cheguem ao webpack ou pipeline de componentes Vite da Nuxt.

### Configuração de Prévia para Nuxt

Projetos Nuxt frequentemente usam recursos que precisam estar disponíveis no ambiente de visualização do Musea
(`NuxtLink`, `useRoute`, `useNuxtApp`, `useRuntimeConfig`, composáveis de dados e componentes
Nuxt integrados). Use `@vizejs/musea-nuxt` na configuração independente do Musea Vite e instale sua camada de visualização
mock a partir de `previewSetup`:

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
      ja: {
        /* ... */
      },
      en: {
        /* ... */
      },
    },
  });
  app.use(i18n);
}) satisfies MuseaPreviewSetup;
```

## Como Funciona

Quando o módulo Nuxt é instalado:

1. **Injeção de plugin Vite** — O módulo `@vizejs/vite-plugin` registra como um plugin Vite, interceptando `.vue` compilação de arquivos.
2. **Calço de compatibilidade** — O plugin expõe uma API de compatibilidade `@vitejs/plugin-vue` , então as verificações internas do Nuxt (que sondam o plugin Vue) funcionam corretamente.
3. **Suporte SSR** — O `vize_atelier_ssr` da Vize cuida da compilação do lado do servidor. O plugin isola variáveis do ambiente cliente e servidor para evitar contaminação cruzada.
4. **Recursos Nuxt preservados** — Autoimportações, composáveis, middleware e outros recursos Nuxt funcionam através da própria camada de transformação do Nuxt, que roda após a compilação do Vize.

## Exemplo do Mundo Real

O site da conferência [Vue Fes Japan 2026](https://vuefes.jp/2026) usa Vize com Nuxt 4:

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

Essa configuração usa o Musea para desenvolvimento de componentes e documentação, mantendo o compilador padrão da Nuxt para compilações de produção.

## Notas

- O Vize está em desenvolvimento ativo — teste minuciosamente antes de usar em projetos Nuxt em produção
- A compilação SSR é suportada via `vize_atelier_ssr`
- Recursos específicos do Nuxt (autoimportações, composáveis, middleware) funcionam pela própria camada de transformação do Nuxt
- O módulo Nuxt suporta Nuxt 2, Nuxt 3 e Nuxt 4. O Nuxt 2 usa o modo de compatibilidade host-compilador porque o compilador SFC nativo do Vize mira a saída do Vue 3.
