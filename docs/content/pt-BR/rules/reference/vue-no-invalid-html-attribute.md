---
title: "vue/no-invalid-html-attribute"
description: "Proibir valores estáticos inválidos para atributos HTML."
---

# `vue/no-invalid-html-attribute`

Proibir valores estáticos inválidos para atributos HTML.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

A âncora usa stylesheet como valor de rel, embora esse valor pertença a elementos link de folhas de estilo.

```vue annotate="remove:2"
<template>
<a href="/guide" rel="stylesheet">Guide</a>
</template>
```

## Correto

A âncora usa help, um valor de rel apropriado para um recurso de ajuda vinculado.

```vue annotate="add:2"
<template>
<a href="/guide" rel="help">Guide</a>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-invalid-html-attribute": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_invalid_html_attribute.rs#L12)
