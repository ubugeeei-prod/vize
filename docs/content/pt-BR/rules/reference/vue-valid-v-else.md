---
title: "vue/valid-v-else"
description: "Exigir diretivas `v-else` válidas."
---

# `vue/valid-v-else`

Exigir diretivas `v-else` válidas.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Sim | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Os exemplos atribuem uma expressão a `v-else`, combinam-no com `v-if` ou omitem o ramo condicional adjacente que deve precedê-lo.

```vue annotate="remove:2,3"
<template>
  <div v-else="ready"></div>
  <div v-else v-if="ready"></div>
  <div v-else></div>
</template>
```

## Correto

Coloque `v-else` sozinho imediatamente após o ramo `v-if` correspondente.

```vue annotate="add:2"
<template>
  <div v-if="ready"></div>
  <div v-else></div>
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
        "vue/valid-v-else": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else.rs#L31)
