---
title: "vue/no-loss-of-precision"
description: "Proíbe literais numéricos que perdem precisão."
---

# `vue/no-loss-of-precision`

Proíbe literais numéricos que perdem precisão.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O inteiro não pode ser representado com precisão como number.

```vue annotate="remove:2"
<template>
  <p>{{ 9007199254740993 }}</p>
</template>
```

## Correto

Uma referência precisa evita o literal impreciso no template.

```vue annotate="add:2"
<template>
  <p>{{ preciseValue }}</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-loss-of-precision": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules.rs#L110)
