---
title: "vue/no-unsupported-features"
description: "Proíbe sintaxe ainda não suportada na versão configurada do Vue.js."
---

# `vue/no-unsupported-features`

Proíbe sintaxe ainda não suportada na versão configurada do Vue.js.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

defineModel exige um Vue mais novo do que a versão configurada.

```vue annotate="remove:2,5"
<script setup lang="ts">
const model = defineModel<string>();
</script>
<template>
  <p>{{ model }}</p>
</template>
```

## Correto

O componente não usa macros limitadas por versão.

```vue annotate="add:2,5"
<script setup lang="ts">
const label = "ok";
</script>
<template>
  <p>{{ label }}</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-unsupported-features": "warn"
    },
    "ruleOptions": {
      "vue/no-unsupported-features": {
        "version": "^3.3.0"
      }
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/unsupported_features.rs#L19)
