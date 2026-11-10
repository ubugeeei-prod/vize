---
title: "vue/no-define-model-default"
description: "Proíbe valores padrão em defineModel."
---

# `vue/no-define-model-default`

Proíbe valores padrão em defineModel.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O model filho declara um padrão que pode dessincronizar o pai.

```vue annotate="remove:1,2"
<script setup>
const model = defineModel({ default: "draft" })
</script>
<template />
```

## Correto

O pai fornece o valor inicial do model.

```vue annotate="add:1,2"
<script setup lang="ts">
const model = defineModel<string>()
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
      "vue/no-define-model-default": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/no_define_model_default.rs#L11)
