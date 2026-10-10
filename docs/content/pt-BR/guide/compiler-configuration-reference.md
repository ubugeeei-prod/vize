---
title: Referência de configuração do compilador
---

<!-- Reviewed translation; source: guide/compiler-configuration-reference.md; scope: reference relocation -->

# Referência de configuração do compilador

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


