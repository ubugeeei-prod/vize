---
title: "vue/v-bind-style"
description: "Aplicar um padrão de sintaxe à diretiva `v-bind`."
---

# `vue/v-bind-style`

Aplicar um padrão de sintaxe à diretiva `v-bind`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Sim | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

`v-bind:class` usa a forma longa onde o padrão de vinculação configurado exige a forma abreviada com dois-pontos.

```vue annotate="remove:2"
<template>
  <div v-bind:class="panelClass"></div>
</template>
```

## Correto

`:class` mantém a mesma expressão com a forma abreviada exigida; esta regra trata da grafia, e não do tipo do valor.

```vue annotate="add:2"
<template>
  <div :class="panelClass"></div>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/v-bind-style": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L28)
