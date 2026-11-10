---
title: "vue/no-use-v-if-with-v-for"
description: "Proibir `v-if` no mesmo elemento que `v-for`."
---

# `vue/no-use-v-if-with-v-for`

Proibir `v-if` no mesmo elemento que `v-for`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O mesmo elemento de lista combina v-if e v-for, mas a condição não referencia a variável do laço.

```vue annotate="remove:2"
<template>
  <li v-for="item in items" v-if="showAll" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

## Correto

Com allowUsingIterationVar ativado, a condição referencia item do mesmo v-for.

```vue annotate="add:2"
<template>
  <li v-for="item in items" v-if="item.visible" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-use-v-if-with-v-for": "warn"
    },
    "ruleOptions": {
      "vue/no-use-v-if-with-v-for": {
        "allowUsingIterationVar": true
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
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_if_with_v_for.rs#L35)
