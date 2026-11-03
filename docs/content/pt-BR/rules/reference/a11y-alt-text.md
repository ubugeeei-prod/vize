---
title: "a11y/alt-text"
description: "Exigir texto alternativo para elementos de mídia."
---

# `a11y/alt-text`

Exigir texto alternativo para elementos de mídia.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O controle de envio com imagem fornece apenas a URL da imagem; ele não tem texto `alt` que descreva a ação.

```vue annotate="remove:2"
<template>
  <input type="image" src="/submit.png" />
</template>
```

## Correto

`alt="Submit search"` dá ao controle com imagem um nome acessível que descreve o envio da pesquisa.

```vue annotate="add:2"
<template>
  <input type="image" src="/submit.png" alt="Submit search" />
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
        "a11y/alt-text": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/alt_text.rs#L33)
