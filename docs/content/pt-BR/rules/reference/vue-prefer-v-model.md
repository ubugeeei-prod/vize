---
title: "vue/prefer-v-model"
description: "Prefere v-model a um bind e listener update equivalentes."
---

# `vue/prefer-v-model`

Prefere v-model a um bind e listener update equivalentes.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O bind e o listener sincronizam manualmente o mesmo valor.

```vue annotate="remove:2"
<template>
  <TextField :model-value="value" @update:model-value="value = $event" />
</template>
```

## Correto

`v-model` expressa uma vez o mesmo contrato bidirecional.

```vue annotate="add:2"
<template>
  <TextField v-model="value" />
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-v-model": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template.rs#L735)
