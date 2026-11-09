---
title: "vue/no-restricted-syntax"
description: "Proíbe sintaxe configurada nas expressões do template."
---

# `vue/no-restricted-syntax`

Proíbe sintaxe configurada nas expressões do template.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O `CallExpression` configurado aparece no template.

```vue annotate="remove:2"
<template>
  <p>{{ format(value) }}</p>
</template>
```

## Correto

O template lê uma referência preparada sem chamar função.

```vue annotate="add:2"
<template>
  <p>{{ formattedValue }}</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-syntax": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-syntax": {
        "selectors": [
          "CallExpression"
        ]
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
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/restricted.rs#L16)
