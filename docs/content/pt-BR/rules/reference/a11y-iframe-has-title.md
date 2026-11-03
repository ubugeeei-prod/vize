---
title: "a11y/iframe-has-title"
description: "Exigir um atributo title nos elementos iframe."
---

# `a11y/iframe-has-title`

Exigir um atributo title nos elementos iframe.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O quadro de finalização de compra tem uma URL de origem, mas não tem um `title` que descreva o conteúdo incorporado.

```vue annotate="remove:2"
<template>
  <iframe src="/checkout"></iframe>
</template>
```

## Correto

`title="Checkout preview"` dá um nome ao conteúdo desse quadro.

```vue annotate="add:2"
<template>
  <iframe src="/checkout" title="Checkout preview"></iframe>
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
        "a11y/iframe-has-title": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/iframe_has_title.rs#L15)
