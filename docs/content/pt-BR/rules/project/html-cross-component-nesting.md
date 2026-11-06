---
title: "html/cross-component-nesting"
---

# `html/cross-component-nesting`

Verificar o aninhamento HTML real após compor os componentes importados.

| Severidade padrão | Correção automática | Opções |
| --- | --- | --- |
| warning | Nenhuma | crossFile; severidade da regra (off/warn/error) |

Aplicável a: Declarações alcançáveis do projeto e componentes importados

## Arquivos compartilhados do projeto

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

## Incorreto

O `<p>` do pai contém um filho resolvido cuja raiz é `<div>`, produzindo um aninhamento inválido de bloco em parágrafo após a composição.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><p><Child /></p></template>
```

`Child.vue`

```vue
<template><div>Block content</div></template>
```

## Correto

Use um contêiner `<section>` que possa conter o elemento de bloco do filho; o filho permanece inalterado.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><section><Child /></section></template>
```

`Child.vue`

```vue
<template><div>Block content</div></template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: { preset: "incremental", crossFile: true,
    rules: { "html/cross-component-nesting": "warn" },
  },
});
```

```sh
vp run lint
```

## Páginas relacionadas

[Índice de verificações entre arquivos](../cross-file.md)
