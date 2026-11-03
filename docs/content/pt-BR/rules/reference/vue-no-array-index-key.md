---
title: "vue/no-array-index-key"
description: "Proibir o uso direto da variável de índice de v-for como :key."
---

# `vue/no-array-index-key`

Proibir o uso direto da variável de índice de v-for como :key.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

A chave da lista é seu índice atual, então a identidade do item muda quando a lista é reordenada.

```vue annotate="remove:2"
<template>
<li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
</template>
```

## Correto

A chave vem de item.id, preservando a identidade de cada item quando sua posição muda.

```vue annotate="add:2"
<template>
<li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-array-index-key": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_array_index_key.rs#L32)
