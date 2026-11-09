---
title: "vue/no-implicit-coercion"
description: "Proíbe coerção implícita em expressões do template."
---

# `vue/no-implicit-coercion`

Proíbe coerção implícita em expressões do template.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

A dupla negação converte o valor implicitamente.

```vue annotate="remove:2"
<template>
  <p>{{ !!value }}</p>
</template>
```

## Correto

`Boolean` torna a conversão explícita.

```vue annotate="add:2"
<template>
  <p>{{ Boolean(value) }}</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-implicit-coercion": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules.rs#L96)
