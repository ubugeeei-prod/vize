---
title: "vue/no-useless-mustaches"
description: "Proibir interpolação com chaves duplas cuja expressão seja uma string literal constante."
---

# `vue/no-useless-mustaches`

Proibir interpolação com chaves duplas cuja expressão seja uma string literal constante.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

A interpolação contém apenas uma string constante e não precisa avaliar uma expressão.

```vue annotate="remove:2,3,4"
<template>
<div>{{ 'x' }}</div>
<div>{{ "x" }}</div>
<div>{{ `x` }}</div>
</template>
```

## Correto

O texto literal é escrito diretamente; expressões com variáveis, strings de template interpoladas e espaços separadores intencionais continuam sendo casos de interpolação.

```vue annotate="add:2,3,4,5"
<template>
<div>x</div>
<div>{{ x }}</div>
<div>{{ `pre-${x}` }}</div>
<span>A</span> {{ " " }} <span>B</span>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-useless-mustaches": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_mustaches.rs#L39)
