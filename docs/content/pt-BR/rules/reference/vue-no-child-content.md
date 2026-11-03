---
title: "vue/no-child-content"
description: "Proibir conteúdo filho ao usar v-html ou v-text."
---

# `vue/no-child-content`

Proibir conteúdo filho ao usar v-html ou v-text.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

v-text substitui o conteúdo do parágrafo, então o texto alternativo escrito no template não pode ser preservado por essa diretiva.

```vue annotate="remove:2"
<template>
  <p v-text="message">Fallback text</p>
</template>
```

## Correto

Remover o texto filho deixa v-text como a única fonte de conteúdo do parágrafo.

```vue annotate="add:2"
<template>
  <p v-text="message" />
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
        "vue/no-child-content": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L30)
