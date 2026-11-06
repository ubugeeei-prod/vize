---
title: "vue/v-on-handler-style"
description: "Exigir manipuladores de v-on escritos como referência a método ou função inline."
---

# `vue/v-on-handler-style`

Exigir manipuladores de v-on escritos como referência a método ou função inline.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Os manipuladores colocam mutações e várias instruções diretamente no atributo de evento.

```vue annotate="remove:2,3,4"
<template>
<button @click="count++"></button>
<button @click="doThis(); doThat()"></button>
<button @click="foo = bar"></button>
</template>
```

## Correto

Use uma referência a manipulador ou uma expressão de função, comum ou de seta, quando for necessária lógica inline. O limite da função torna explícita a forma do manipulador.

```vue annotate="add:2,3,4,5"
<template>
<button @click="handler"></button>
<button @click="foo.bar"></button>
<button @click="() => count++"></button>
<button @click="function () { count++ }"></button>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/v-on-handler-style": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_handler_style.rs#L33)
