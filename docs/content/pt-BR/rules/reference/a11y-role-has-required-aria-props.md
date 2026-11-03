---
title: "a11y/role-has-required-aria-props"
description: "Exigir as propriedades obrigatórias dos papéis ARIA."
---

# `a11y/role-has-required-aria-props`

Exigir as propriedades obrigatórias dos papéis ARIA.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O papel de caixa de seleção omite `aria-checked`, que informa o estado da caixa de seleção.

```vue annotate="remove:2"
<template>
  <span role="checkbox">Receive updates</span>
</template>
```

## Correto

`aria-checked="false"` fornece o estado exigido pelo papel de caixa de seleção.

```vue annotate="add:2"
<template>
  <span role="checkbox" aria-checked="false">Receive updates</span>
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
        "a11y/role-has-required-aria-props": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_has_required_aria_props.rs#L30)
