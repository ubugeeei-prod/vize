---
title: "vue/no-restricted-component-names"
description: "Proíbe o nome de componente configurado."
---

# `vue/no-restricted-component-names`

Proíbe o nome de componente configurado.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O nome do componente é o proibido RestrictedShowcase.

```vue annotate="remove:2"
<script setup lang="ts">
defineOptions({ name: "RestrictedShowcase" });
</script>
<template>
  <p>ok</p>
</template>
```

## Correto

O nome do componente é Widget.

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: "Widget" });
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
      "vue/no-restricted-component-names": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-component-names": [
        "RestrictedShowcase"
      ]
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/component_names.rs#L12)
