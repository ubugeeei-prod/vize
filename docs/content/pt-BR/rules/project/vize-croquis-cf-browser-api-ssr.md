---
title: "vize:croquis/cf/browser-api-ssr"
---

# `vize:croquis/cf/browser-api-ssr`

Uma API exclusiva do navegador é usada onde o componente pode ser renderizado no servidor.

| Severidade padrão | Correção automática | Opções |
| --- | --- | --- |
| warning | Nenhuma; revise os arquivos relacionados e aplique a correção | Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade |

Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo

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

`window.innerWidth` é executado durante setup, quando um ambiente SSR não tem o `window` do navegador.

`App.vue`

```vue
<script setup lang="ts">
const width = window.innerWidth;
</script>
<template><p>Content</p></template>
```

## Correto

Inicialize uma ref com um valor seguro para o servidor e leia `window` dentro de `onMounted`, que é executado após a montagem no cliente.

`App.vue`

```vue
<script setup lang="ts">
import { onMounted, ref } from "vue";
const width = ref(0);
onMounted(() => { width.value = window.innerWidth; });
</script>
<template><p>Content</p></template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/browser-api-ssr": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## Páginas relacionadas

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[Índice de verificações entre arquivos](../cross-file.md)
