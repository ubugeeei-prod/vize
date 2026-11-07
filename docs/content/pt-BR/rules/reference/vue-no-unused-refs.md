---
title: "vue/no-unused-refs"
description: "Reportar refs de template (ref=\"x\") nunca referenciadas em &lt;script&gt;."
---

# `vue/no-unused-refs`

Reportar refs de template (ref="x") nunca referenciadas em &lt;script&gt;.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O template declara o nome de ref unused sem uma variável de referência correspondente no script.

```vue annotate="remove:1,3"
<template><input ref="unused" /></template>
<script setup>
const x = 1
</script>
```

## Correto

A ref de template inputEl tem uma variável ref de mesmo nome em script setup.

```vue annotate="add:1,3,4"
<template><input ref="inputEl" /></template>
<script setup>
import { ref } from 'vue'
const inputEl = ref(null)
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-unused-refs": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_unused_refs.rs#L66)
