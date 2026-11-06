---
title: "vue/valid-v-else-if"
description: "Exigir diretivas `v-else-if` válidas."
---

# `vue/valid-v-else-if`

Exigir diretivas `v-else-if` válidas.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Os exemplos deixam `v-else-if` sem um ramo condicional anterior ou sem condição, ou lhe dão um argumento ou modificador.

```vue annotate="remove:2,3,4,5"
<template>
<div v-else-if="foo"></div> <!-- without preceding v-if -->
<div v-if="foo"></div><div v-else-if></div>
<div v-if="foo"></div><div v-else-if:arg="bar"></div>
<div v-if="foo"></div><div v-else-if.mod="bar"></div>
</template>
```

## Correto

Coloque `v-else-if` com uma condição imediatamente após o ramo `v-if` ou `v-else-if` que ele continua.

```vue annotate="add:2,3"
<template>
<div v-if="foo"></div>
<div v-else-if="bar"></div>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-else-if": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else_if.rs#L34)
