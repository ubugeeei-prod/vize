---
title: "vue/enforce-style-attribute"
description: "Restringe blocos style aos modos permitidos."
---

# `vue/enforce-style-attribute`

Restringe blocos style aos modos permitidos.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O modo module não está na lista permitida.

```vue annotate="remove:2"
<template><div /></template>
<style module>.card { color: red }</style>
```

## Correto

O modo scoped é aceito pela política do projeto.

```vue annotate="add:2"
<template><div /></template>
<style scoped>.card { color: red }</style>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/enforce-style-attribute": "warn"
    },
    "ruleOptions": {
      "vue/enforce-style-attribute": {
        "vue/enforce-style-attribute": {
          "allow": [
            "scoped"
          ]
        }
      }
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc.rs#L32)
