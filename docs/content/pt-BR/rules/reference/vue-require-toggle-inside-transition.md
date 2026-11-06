---
title: "vue/require-toggle-inside-transition"
description: "Exigir uma alternância no elemento envolvido por `<transition>`."
---

# `vue/require-toggle-inside-transition`

Exigir uma alternância no elemento envolvido por `<transition>`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O filho estático dentro de `<Transition>` não tem visibilidade condicional nem seleção dinâmica para provocar uma mudança de entrada ou saída.

```vue annotate="remove:3"
<template>
<transition>
  <div>content</div>
</transition>
</template>
```

## Correto

`v-if="show"` altera a existência do filho, fornecendo um limite de entrada ou saída à transição.

```vue annotate="add:3"
<template>
<transition>
  <div v-if="show">content</div>
</transition>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-toggle-inside-transition": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_toggle_inside_transition.rs#L49)
