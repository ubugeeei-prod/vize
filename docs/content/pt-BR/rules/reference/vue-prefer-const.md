---
title: "vue/prefer-const"
description: "Exigir const para vínculos que o script e o template nunca reatribuem."
---

# `vue/prefer-const`

Exigir const para vínculos que o script e o template nunca reatribuem.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O template lê label, e nem o script nem o template o reatribui, então let não é necessário.

```vue annotate="remove:2"
<script setup>
let label = "Hello";
</script>
<template><p>{{ label }}</p></template>
```

## Correto

const corresponde a um vínculo que o script e o template apenas leem.

```vue annotate="add:2"
<script setup>
const label = "Hello";
</script>
<template><p>{{ label }}</p></template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-const": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/script_unused/prefer_const.rs#L20)
