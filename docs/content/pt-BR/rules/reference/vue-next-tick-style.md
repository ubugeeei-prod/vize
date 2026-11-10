---
title: "vue/next-tick-style"
description: "Exige o estilo Promise no uso de nextTick."
---

# `vue/next-tick-style`

Exige o estilo Promise no uso de nextTick.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Um callback é passado a `nextTick`.

```vue annotate="remove:3"
<script setup>
import { nextTick } from "vue"
nextTick(() => focusInput())
</script>
<template />
```

## Correto

O código aguarda a Promise antes de continuar.

```vue annotate="add:3,4"
<script setup>
import { nextTick } from "vue"
await nextTick()
focusInput()
</script>
<template />
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/next-tick-style": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/next_tick_style.rs#L12)
