---
title: "vue/sfc-element-order"
description: "Aplicar uma ordem consistente aos elementos de nível superior dos SFCs."
---

# `vue/sfc-element-order`

Aplicar uma ordem consistente aos elementos de nível superior dos SFCs.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O bloco style precede o bloco script, contrariando a ordem configurada dos blocos do SFC.

```vue annotate="remove:2,6,7,8"
<style scoped>
.panel {
  color: red;
}
</style>
<script setup lang="ts">
const label = "Save";
</script>
```

## Correto

Os blocos seguem script → template → style. Projetos podem escolher outra ordem pela opção tipada desta regra.

```vue annotate="add:1,2,3,4,5,6,7,8,10"
<script setup lang="ts">
const label = "Save";
</script>

<template>
  <p>{{ label }}</p>
</template>

<style scoped>
p {
  color: red;
}
</style>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/sfc-element-order": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/sfc_element_order.rs#L50)
