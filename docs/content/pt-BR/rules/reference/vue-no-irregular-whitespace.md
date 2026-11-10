---
title: "vue/no-irregular-whitespace"
description: "Detecta espaços irregulares em expressões do template."
---

# `vue/no-irregular-whitespace`

Detecta espaços irregulares em expressões do template.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

A expressão contém um espaço sem quebra.

```vue annotate="remove:2"
<template>
  <p>{{ value + other }}</p>
</template>
```

## Correto

A expressão usa espaço comum de código-fonte.

```vue annotate="add:2"
<template>
  <p>{{ value + other }}</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-irregular-whitespace": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_irregular_whitespace.rs#L12)
