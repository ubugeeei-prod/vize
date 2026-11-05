---
title: "vue/no-deprecated-v-is"
description: "Proibir a diretiva obsoleta `v-is`."
---

# `vue/no-deprecated-v-is`

Proibir a diretiva obsoleta `v-is`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

`v-is` foi descontinuada no Vue 3.1; o exemplo ainda a usa para trocar o componente renderizado.

```vue annotate="remove:2"
<template>
<tr v-is="'MyRow'"></tr>
</template>
```

## Correto

O atributo `is` com o prefixo `vue:`, ou `<component :is>`, seleciona o componente em seu lugar.

```vue annotate="add:2,3"
<template>
<tr is="vue:MyRow"></tr>
<component :is="MyRow" />
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
        "vue/no-deprecated-v-is": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_is.rs#L27)
