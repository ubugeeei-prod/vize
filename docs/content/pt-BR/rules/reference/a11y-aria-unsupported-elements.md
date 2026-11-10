---
title: "a11y/aria-unsupported-elements"
description: "Proibir atributos ARIA em elementos que não os aceitam."
---

# `a11y/aria-unsupported-elements`

Proibir atributos ARIA em elementos que não os aceitam.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O elemento de metadados tem `aria-hidden`, embora `meta` não aceite atributos ARIA.

```vue annotate="remove:2"
<template>
  <meta charset="utf-8" aria-hidden="true" />
</template>
```

## Correto

Remover o atributo ARIA mantém intacta a declaração do conjunto de caracteres.

```vue annotate="add:2"
<template>
  <meta charset="utf-8" />
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/aria-unsupported-elements": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_unsupported_elements.rs#L21)
