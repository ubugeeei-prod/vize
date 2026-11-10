---
title: "vue/no-sparse-arrays"
description: "Proíbe arrays esparsos em expressões do template."
---

# `vue/no-sparse-arrays`

Proíbe arrays esparsos em expressões do template.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

A ausência do segundo item cria um array esparso.

```vue annotate="remove:2"
<template>
  <p>{{ [1,, 3] }}</p>
</template>
```

## Correto

Cada posição do array tem um valor explícito.

```vue annotate="add:2"
<template>
  <p>{{ [1, 2, 3] }}</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-sparse-arrays": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_sparse_arrays.rs#L12)
