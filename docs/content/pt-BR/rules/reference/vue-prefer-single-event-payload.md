---
title: "vue/prefer-single-event-payload"
description: "Exige um único argumento de payload ao emitir eventos personalizados."
---

# `vue/prefer-single-event-payload`

Exige um único argumento de payload ao emitir eventos personalizados.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O evento move declara e emite dois valores, e um listener inline só recebe x.

```vue annotate="remove:2,3"
<script setup lang="ts">
const emit = defineEmits<{ move: [x: number, y: number] }>()
emit("move", 1, 2)
</script>
<template />
```

## Correto

O evento move envia um objeto, e todo listener recebe as duas coordenadas.

```vue annotate="add:2,3"
<script setup lang="ts">
const emit = defineEmits<{ move: [point: { x: number; y: number }] }>()
emit("move", { x: 1, y: 2 })
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
      "vue/prefer-single-event-payload": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/event_payload.rs#L34)
