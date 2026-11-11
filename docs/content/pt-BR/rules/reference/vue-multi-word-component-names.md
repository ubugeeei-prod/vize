---
title: "vue/multi-word-component-names"
description: "Exigir nomes de componentes com mais de uma palavra."
---

# `vue/multi-word-component-names`

Exigir nomes de componentes com mais de uma palavra.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `nuxt`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

Neste exemplo só com template, a ocorrência diagnosticada é o nome de arquivo. Um componente que declara um nome estático é verificado por esse nome em vez do nome de arquivo. A exceção exata Item configurada não permite outro nome de uma palavra.

## Incorreto

Thing.vue é um nome de uma palavra que não está na lista de exceções configurada.

`Thing.vue`

```vue
<template><p>Item</p></template>
```

## Correto

Item.vue é permitido pela exceção exata Item configurada.

`Item.vue`

```vue
<template><p>Item</p></template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/multi-word-component-names": "error"
    },
    "ruleOptions": {
      "vue/multi-word-component-names": {
        "ignores": [
          "Item"
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
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L41)
