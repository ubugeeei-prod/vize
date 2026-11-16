---
title: "vue/v-on-style"
description: "Aplicar um padrão de sintaxe à diretiva `v-on`."
---

# `vue/v-on-style`

Aplicar um padrão de sintaxe à diretiva `v-on`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Sim | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

`v-on:click` usa a forma longa de listener de evento onde a regra exige a forma abreviada.

```vue annotate="remove:2"
<template>
  <div v-on:click="handleClick"></div>
</template>
```

## Correto

`@click` mantém o mesmo manipulador e usa a forma abreviada configurada.

```vue annotate="add:2"
<template>
  <div @click="handleClick"></div>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/v-on-style": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_on_style.rs#L29)
