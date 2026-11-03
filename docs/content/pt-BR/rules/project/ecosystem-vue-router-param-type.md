---
title: "ecosystem/vue-router-param-type"
---

# `ecosystem/vue-router-param-type`

postId não é repetível, portanto um array é inválido.

Severidade padrão: error  
Aplicável a: Declarações alcançáveis do projeto e componentes importados  
Opções: crossFile; severidade da regra (off/warn/error)  
Correção automática: Nenhuma

O roteador completo instalado deve ser alcançável a partir de createApp(...).use(router) da aplicação. Tabelas de rotas desconhecidas ou dinâmicas não comprovam diagnósticos de nomes desconhecidos. Parâmetros ausentes geram avisos porque a navegação pode herdar um valor da rota atual.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-param-type": "warn" },
    },
  },
});
```

```sh
vp run lint
```

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

`postId` é um parâmetro escalar do caminho, mas a navegação fornece a ele o array `["2"]`.

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: ["2"] } });
</script>
<template><p>Post</p></template>
```

## Correto

Passe o escalar `"2"` para o segmento `postId`, que não é repetível.

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Índice de verificações entre arquivos](../cross-file.md)
