---
title: "ecosystem/vue-router-unknown-route"
---

# `ecosystem/vue-router-unknown-route`

O nome está ausente do roteador completo instalado.

| Severidade padrão | Correção automática | Opções |
| --- | --- | --- |
| error | Nenhuma | crossFile; severidade da regra (off/warn/error) |

Aplicável a: Declarações alcançáveis do projeto e componentes importados

O roteador completo instalado deve ser alcançável a partir de createApp(...).use(router) da aplicação. Tabelas de rotas desconhecidas ou dinâmicas não comprovam diagnósticos de nomes desconhecidos. Parâmetros ausentes geram avisos porque a navegação pode herdar um valor da rota atual.

## Arquivos compartilhados do projeto

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`index.html`

```html
<div id="app"></div>
<script type="module" src="/src/main.ts"></script>
```

`src/main.ts`

```ts
import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
createApp(App).use(router).mount("#app");
```

`src/App.vue`

```vue
<script setup lang="ts">
import { RouterView } from "vue-router";
</script>
<template><RouterView /></template>
```

`src/router.ts`

```ts
import { createRouter, createWebHistory } from "vue-router";
import UserPost from "./UserPost.vue";
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/users/:userId/posts/:postId", name: "user-post", component: UserPost }],
});
```

## Incorreto

O roteador instalado alcançável declara `user-post`, mas a navegação usa o nome incorreto `user-posts`.

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-posts", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

## Correto

Use o nome registrado `user-post`, mantendo os dois parâmetros de caminho declarados.

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-unknown-route": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## Páginas relacionadas

[Índice de verificações entre arquivos](../cross-file.md)
