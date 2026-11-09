---
title: "vue/no-props-shadow"
description: "Proíbe declarações de script setup que ocultam props."
---

# `vue/no-props-shadow`

Proíbe declarações de script setup que ocultam props.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

A declaração local `value` oculta a prop homônima.

```vue annotate="remove:3"
<script setup>
defineProps({ value: String })
const value = "local"
</script>
<template />
```

## Correto

A referência local usa um nome distinto.

```vue annotate="add:3"
<script setup>
defineProps({ value: String })
const localValue = "local"
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
      "vue/no-props-shadow": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc.rs#L48)
