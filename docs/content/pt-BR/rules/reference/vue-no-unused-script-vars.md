---
title: "vue/no-unused-script-vars"
description: "Proibir variáveis, importações e parâmetros de script não usados."
---

# `vue/no-unused-script-vars`

Proibir variáveis, importações e parâmetros de script não usados.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O script setup declara unused, e nem o script nem o template o leem.

```vue annotate="remove:4"
<script setup>
const unused = 1;
</script>
<template><p>Welcome</p></template>
```

## Correto

A interpolação do template lê unused, então a variável declarada é usada.

```vue annotate="add:4"
<script setup>
const unused = 1;
</script>
<template><p>{{ unused }}</p></template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-unused-script-vars": "warn"
    },
    "ruleOptions": {
      "vue/no-unused-script-vars": {
        "varsIgnorePattern": "^_",
        "argsIgnorePattern": "^_"
      }
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/script_unused/no_unused_script_vars.rs#L24)
