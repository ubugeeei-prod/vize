---
title: "vue/no-empty-pattern"
description: "Proíbe padrões de desestruturação vazios no template."
---

# `vue/no-empty-pattern`

Proíbe padrões de desestruturação vazios no template.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O callback desestrutura um objeto vazio.

```vue annotate="remove:2"
<template>
  <p>{{ items.map(({}) => 1) }}</p>
</template>
```

## Correto

O callback nomeia a propriedade consumida.

```vue annotate="add:2"
<template>
  <p>{{ items.map(({ id }) => id) }}</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-empty-pattern": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_empty_pattern.rs#L10)
