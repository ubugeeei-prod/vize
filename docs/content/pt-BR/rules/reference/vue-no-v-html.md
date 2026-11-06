---
title: "vue/no-v-html"
description: "Alertar sobre v-html para prevenir vulnerabilidades XSS."
---

# `vue/no-v-html`

Alertar sobre v-html para prevenir vulnerabilidades XSS.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

v-html interpreta content como HTML, em vez de texto comum.

```vue annotate="remove:2"
<template>
  <article v-html="content" />
</template>
```

## Correto

A interpolação com chaves duplas exibe content como texto escapado, em vez de injetar HTML.

```vue annotate="add:2"
<template>
  <article>{{ content }}</article>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-v-html": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_html.rs#L51)
