---
title: "vue/prefer-props-shorthand"
description: "Recomendar sintaxe abreviada para props (Vue 3.4+)."
---

# `vue/prefer-props-shorthand`

Recomendar sintaxe abreviada para props (Vue 3.4+).

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Sim | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Cada vinculação repete o nome da variável correspondente, inclusive o equivalente camelCase de um argumento separado por hífen.

```vue annotate="remove:2,3,4,5"
<template>
  <MyComponent :foo="foo" />
  <MyComponent :user-name="userName" />
  <span :style="style" />
  <div :aria-label="ariaLabel" />
</template>
```

## Correto

A forma abreviada de vinculação de mesmo nome do Vue 3.4+ remove as expressões repetidas; uma variável de origem diferente, como bar, permanece explícita.

```vue annotate="add:2,3,4,5,6"
<template>
  <MyComponent :foo />
  <MyComponent :user-name />
  <span :style />
  <div :aria-label />
  <MyComponent :foo="bar" />
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-props-shorthand": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_props_shorthand.rs#L39)
