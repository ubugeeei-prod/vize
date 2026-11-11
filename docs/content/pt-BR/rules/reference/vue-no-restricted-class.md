---
title: "vue/no-restricted-class"
description: "Proíbe a class configurada."
---

# `vue/no-restricted-class`

Proíbe a class configurada.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O elemento usa a class proibida legacy.

```vue annotate="remove:2"
<template>
  <div class="legacy">x</div>
</template>
```

## Correto

O elemento usa uma class permitida.

```vue annotate="add:2"
<template>
  <div class="current">x</div>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-class": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-class": [
        "legacy"
      ]
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_class.rs#L13)
