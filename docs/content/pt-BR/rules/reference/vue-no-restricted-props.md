---
title: "vue/no-restricted-props"
description: "Proíbe a prop configurada."
---

# `vue/no-restricted-props`

Proíbe a prop configurada.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O componente declara a prop proibida third.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps<{ third?: number }>();
</script>
<template>
  <p>ok</p>
</template>
```

## Correto

O componente declara a prop first.

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ first?: number }>();
</script>
<template>
  <p>ok</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-props": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-props": [
        "third"
      ]
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_props.rs#L12)
