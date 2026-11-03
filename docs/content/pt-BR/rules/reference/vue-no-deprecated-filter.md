---
title: "vue/no-deprecated-filter"
description: "Proibir a sintaxe obsoleta de filtros do Vue 2 com o operador de barra vertical."
---

# `vue/no-deprecated-filter`

Proibir a sintaxe obsoleta de filtros do Vue 2 com o operador de barra vertical.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

A barra vertical usa a sintaxe de filtros removida do Vue para aplicar capitalize.

```vue annotate="remove:2"
<template>
{{ message | capitalize }}
</template>
```

## Correto

Chamar capitalize(message) aplica a transformação como uma expressão comum.

```vue annotate="add:2"
<template>
{{ capitalize(message) }}
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-filter": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_filter.rs#L56)
