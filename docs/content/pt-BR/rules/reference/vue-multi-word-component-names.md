---
title: "vue/multi-word-component-names"
description: "Exigir nomes de componentes com mais de uma palavra."
---

# `vue/multi-word-component-names`

Exigir nomes de componentes com mais de uma palavra.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

A ocorrência diagnosticada é o nome de arquivo. Renomeie o mesmo componente; alterar uma tag filha não corrige o problema.

## Incorreto

Item.vue dá ao componente um nome de uma única palavra.

`Item.vue`

```vue
<template><p>Item</p></template>
```

## Correto

TodoItem.vue dá ao mesmo template um nome de componente com mais de uma palavra.

`TodoItem.vue`

```vue
<template><p>Item</p></template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/multi-word-component-names": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L34)
