---
title: "vue/camelcase"
description: "Exige camelCase nos identificadores de expressões do template."
---

# `vue/camelcase`

Exige camelCase nos identificadores de expressões do template.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

`user_name` contém um sublinhado interno.

```vue annotate="remove:2"
<template>
  <p>{{ user_name }}</p>
</template>
```

## Correto

`userName` segue camelCase.

```vue annotate="add:2"
<template>
  <p>{{ userName }}</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/camelcase": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules.rs#L54)
