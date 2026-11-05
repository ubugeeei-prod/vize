---
title: "vue/v-on-event-hyphenation"
description: "Exigir hífens nos nomes de eventos personalizados em v-on de componentes."
---

# `vue/v-on-event-hyphenation`

Exigir hífens nos nomes de eventos personalizados em v-on de componentes.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Sim | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O listener do componente personalizado usa `@myEvent` em vez de um nome de evento separado por hífen.

```vue annotate="remove:2,3,4"
<template>
<MyComponent @myEvent="handler" />
<MyComponent v-on:myEvent="handler" />
<my-component @update:modelValue="handler" />
</template>
```

## Correto

`@my-event` usa a grafia exigida para eventos personalizados. Os listeners em elementos nativos e os argumentos de evento dinâmicos mostrados abaixo ficam fora desta verificação.

```vue annotate="add:2,3,4"
<template>
<MyComponent @my-event="handler" />
<div @myEvent="handler" />
<MyComponent @[dynamicEvent]="handler" />
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
        "vue/v-on-event-hyphenation": "warn"
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
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_event_hyphenation.rs#L40)
