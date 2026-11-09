---
title: "vue/require-v-for-key"
description: "Exigir `v-bind:key` nas diretivas `v-for`."
---

# `vue/require-v-for-key`

Exigir `v-bind:key` nas diretivas `v-for`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Cada `<li>` repetido não tem uma chave que identifique seu item correspondente durante atualizações da lista.

```vue annotate="remove:2"
<template>
  <li v-for="item in items">{{ item.name }}</li>
</template>
```

## Correto

`:key="item.id"` dá a cada nó repetido a identidade do item, em vez de sua posição atual.

```vue annotate="add:2"
<template>
  <li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-v-for-key": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_v_for_key.rs#L47)
