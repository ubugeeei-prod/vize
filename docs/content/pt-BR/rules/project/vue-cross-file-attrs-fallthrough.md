---
title: "vue/cross-file-attrs-fallthrough"
---

# `vue/cross-file-attrs-fallthrough`

Um pai passa atributos para um filho resolvido cuja raiz não pode herdá-los e que não usa $attrs explicitamente.

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

O pai passa `class="notice"` para um filho resolvido com raiz em fragmento, que não tem um destino automático para atributos e nunca lê `$attrs`.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<template><main>Content</main><aside>Help</aside></template>
```

## Correto

O filho escolhe `<main>` como destino vinculando `$attrs` ali; seu irmão `<aside>` permanece separado.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: { preset: "incremental", crossFile: true,
    rules: { "vue/cross-file-attrs-fallthrough": "warn" },
  },
});
```

```sh
vp run lint
```

## Páginas relacionadas

[Índice de verificações entre arquivos](../cross-file.md)
