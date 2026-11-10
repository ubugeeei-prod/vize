---
title: "vue/no-deprecated-v-bind-sync"
description: "Proibir o modificador obsoleto `.sync` em `v-bind`."
---

# `vue/no-deprecated-v-bind-sync`

Proibir o modificador obsoleto `.sync` em `v-bind`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

As vinculações usam o modificador .sync removido, inclusive em combinação com .camel.

```vue annotate="remove:2,3,4"
<template>
<MyComponent :title.sync="title" />
<MyComponent v-bind:title.sync="title" />
<MyComponent :title.sync.camel="title" />
</template>
```

## Correto

Use uma vinculação comum unidirecional de title ou v-model:title quando for necessário um canal de atualização.

```vue annotate="add:2,3"
<template>
<MyComponent :title="title" />
<MyComponent v-model:title="title" />
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-v-bind-sync": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_bind_sync.rs#L43)
