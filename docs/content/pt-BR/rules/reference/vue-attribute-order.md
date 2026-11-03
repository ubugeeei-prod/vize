---
title: "vue/attribute-order"
description: "Aplicar uma ordem consistente aos atributos."
---

# `vue/attribute-order`

Aplicar uma ordem consistente aos atributos.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O manipulador de evento aparece antes da diretiva estrutural v-if e do atributo comum id.

```vue annotate="remove:2"
<template>
  <div @click="onClick" v-if="show" id="main"></div>
</template>
```

## Correto

v-if vem primeiro, seguido de id e do manipulador de evento, conforme a ordem da regra.

```vue annotate="add:2"
<template>
  <div v-if="show" id="main" @click="onClick"></div>
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
        "vue/attribute-order": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_order.rs#L36)
