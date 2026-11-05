---
title: "vue/valid-v-is"
description: "Exigir diretivas `v-is` válidas."
---

# `vue/valid-v-is`

Exigir diretivas `v-is` válidas.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

`v-is` tem um argumento ou modificador, não tem valor, ou está em um componente em vez de um elemento HTML nativo.

```vue annotate="remove:2,3,4,5"
<template>
<tr v-is:arg="'MyRow'"></tr>
<tr v-is.mod="'MyRow'"></tr>
<tr v-is></tr>
<MyComponent v-is="'MyRow'" />
</template>
```

## Correto

`v-is` com um valor em um elemento nativo como `<tr>` indica o componente a renderizar em seu lugar.

```vue annotate="add:2"
<template>
<tr v-is="'MyRow'"></tr>
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
        "vue/valid-v-is": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_is.rs#L30)
