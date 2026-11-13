---
title: "vue/v-on-handler-style"
description: "Exigir manipuladores de v-on escritos como referência a método."
---

# `vue/v-on-handler-style`

Exigir manipuladores de v-on escritos como referência a método.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Os manipuladores usam funções inline, mutações ou várias instruções em vez de referências de método reutilizáveis.

```vue annotate="remove:2,3,4"
<template>
<button @click="count++"></button>
<button @click="doThis(); doThat()"></button>
<button @click="foo = bar"></button>
</template>
```

## Correto

Use uma referência a manipulador. Mova a lógica inline para um método nomeado, mantendo no template um manipulador estável e reutilizável.

```vue annotate="add:2,3"
<template>
<button @click="handler"></button>
<button @click="foo.bar"></button>
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
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_handler_style.rs#L38)
