---
title: "vue/no-restricted-component-options"
description: "Proíbe a opção de componente configurada."
---

# `vue/no-restricted-component-options`

Proíbe a opção de componente configurada.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O componente define inheritAttrs, que está proibido.

```vue annotate="remove:2"
<script setup lang="ts">
defineOptions({ inheritAttrs: false });
</script>
<template>
  <p>ok</p>
</template>
```

## Correto

O componente define apenas name.

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
      "vue/no-restricted-component-options": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-component-options": [
        "inheritAttrs"
      ]
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/component_options.rs#L12)
