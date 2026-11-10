---
title: "vue/no-shadow-native-events"
description: "Proíbe eventos de componente que colidem com eventos nativos."
---

# `vue/no-shadow-native-events`

Proíbe eventos de componente que colidem com eventos nativos.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O componente declara `click`, que colide com um evento nativo.

```vue annotate="remove:2"
<script setup>
const emit = defineEmits(['click'])
</script>
<template />
```

## Correto

O evento específico `activate` evita a colisão.

```vue annotate="add:2"
<script setup>
const emit = defineEmits(['activate'])
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
      "vue/no-shadow-native-events": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/events.rs#L27)
