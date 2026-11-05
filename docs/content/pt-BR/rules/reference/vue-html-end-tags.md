---
title: "vue/html-end-tags"
description: "Exige tags de fechamento explícitas para elementos HTML cujo fechamento pode ser omitido."
---

# `vue/html-end-tags`

Exige tags de fechamento explícitas para elementos HTML cujo fechamento pode ser omitido.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Sim | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

Esta regra exige tags de fechamento explícitas onde o parser HTML permite omiti-las, como em `li`, `p` e `td`. A ausência de tags de fechamento obrigatórias é diagnosticada por `parser/template`; elementos vazios não precisam de tag de fechamento.

## Incorreto

O HTML permite fechar esse `li` implicitamente quando `ul` termina, mas a regra exige a tag de fechamento explícita.

```vue annotate="remove:1"
<template><ul><li>First</ul></template>
```

## Correto

Adicione `</li>` depois do conteúdo do item, antes de fechar o `ul` que o contém.

```vue annotate="add:1"
<template><ul><li>First</li></ul></template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/html-end-tags": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_end_tags.rs#L30)
