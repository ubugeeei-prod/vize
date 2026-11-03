---
title: "vue/no-deprecated-html-element-is"
description: "Proibir o atributo `is` em elementos HTML nativos."
---

# `vue/no-deprecated-html-element-is`

Proibir o atributo `is` em elementos HTML nativos.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Uma div nativa usa o antigo atributo is sem prefixo para solicitar um componente Vue.

```vue annotate="remove:2"
<template>
  <div is="MyComponent" />
</template>
```

## Correto

Um componente dinâmico usa :is; a forma no elemento nativo usa explicitamente o prefixo vue:.

```vue annotate="add:2,3"
<template>
  <component :is="MyComponent" />
  <div is="vue:MyComponent" />
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
        "vue/no-deprecated-html-element-is": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_html_element_is.rs#L39)
