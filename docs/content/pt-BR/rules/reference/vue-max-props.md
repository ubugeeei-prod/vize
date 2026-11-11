---
title: "vue/max-props"
description: "Limita o número de props de um componente Vue."
---

# `vue/max-props`

Limita o número de props de um componente Vue.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O componente declara três props, acima do máximo configurado.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps<{ a: number; b: number; c: number }>();
</script>
<template>
  <p>props</p>
</template>
```

## Correto

O componente declara uma prop.

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ a: number }>();
</script>
<template>
  <p>props</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/max-props": "warn"
    },
    "ruleOptions": {
      "vue/max-props": {
        "maxProps": 2
      }
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/max_props.rs#L16)
