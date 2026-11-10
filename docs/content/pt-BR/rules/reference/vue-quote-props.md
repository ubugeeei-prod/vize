---
title: "vue/quote-props"
description: "Proíbe aspas desnecessárias em propriedades de objetos."
---

# `vue/quote-props`

Proíbe aspas desnecessárias em propriedades de objetos.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

A chave válida `name` está entre aspas sem necessidade.

```vue annotate="remove:2"
<template>
  <Widget :options='{ "name": value }' />
</template>
```

## Correto

A propriedade usa a forma de identificador sem aspas.

```vue annotate="add:2"
<template>
  <Widget :options="{ name: value }" />
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/quote-props": "warn"
    },
    "ruleOptions": {
      "vue/quote-props": "as-needed"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/quote_props.rs#L10)
