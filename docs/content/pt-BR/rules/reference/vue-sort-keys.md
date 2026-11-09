---
title: "vue/sort-keys"
description: "Ordena as chaves de opções do componente conforme configurado."
---

# `vue/sort-keys`

Ordena as chaves de opções do componente conforme configurado.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

As chaves de métodos aninhadas não estão em ordem crescente.

```vue annotate="remove:2"
<script>
export default { methods: { zebra() {}, alpha() {} } }
</script>
<template />
```

## Correto

As chaves aninhadas seguem a ordem configurada.

```vue annotate="add:2"
<script>
export default { methods: { alpha() {}, zebra() {} } }
</script>
<template />
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/sort-keys": "warn"
    },
    "ruleOptions": {
      "vue/sort-keys": {
        "vue/sort-keys": {
          "order": "asc",
          "caseSensitive": true,
          "natural": false
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
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/options.rs#L23)
