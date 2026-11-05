---
title: "a11y/heading-has-content"
description: "Exigir conteúdo acessível nos elementos de título."
---

# `a11y/heading-has-content`

Exigir conteúdo acessível nos elementos de título.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O `h2` acrescenta um nível de título, mas não tem conteúdo de título.

```vue annotate="remove:2"
<template>
  <h2></h2>
</template>
```

## Correto

`Billing settings` fornece o conteúdo do título de nível dois existente.

```vue annotate="add:2"
<template>
  <h2>Billing settings</h2>
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
        "a11y/heading-has-content": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/heading_has_content.rs#L18)
