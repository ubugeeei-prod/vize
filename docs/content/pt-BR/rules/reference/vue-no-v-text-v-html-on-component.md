---
title: "vue/no-v-text-v-html-on-component"
description: "Proibir v-text / v-html em elementos de componente."
---

# `vue/no-v-text-v-html-on-component`

Proibir v-text / v-html em elementos de componente.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

A tag do componente recebe v-html ou v-text, que substitui o conteúdo do elemento em vez de fornecer slots ao componente.

```vue annotate="remove:2,3"
<template>
  <MyComponent v-html="content" />
  <MyComponent v-text="content" />
</template>
```

## Correto

Elementos HTML nativos podem receber as diretivas; MyComponent recebe seu conteúdo pelo slot padrão.

```vue annotate="add:2,3,4"
<template>
  <div v-html="content"></div>
  <component is="div" v-html="content" />
  <MyComponent>{{ content }}</MyComponent>
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
        "vue/no-v-text-v-html-on-component": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_text_v_html_on_component.rs#L33)
