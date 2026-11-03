---
title: "a11y/aria-props"
description: "Proibir atributos ARIA inválidos."
---

# `a11y/aria-props`

Proibir atributos ARIA inválidos.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

`aria-lable` está escrito incorretamente e não é um atributo ARIA aceito.

```vue annotate="remove:2"
<template>
  <button aria-lable="Save changes">Save</button>
</template>
```

## Correto

O atributo aceito `aria-label` fornece o nome do botão.

```vue annotate="add:2"
<template>
  <button aria-label="Save changes">Save</button>
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
        "a11y/aria-props": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_props.rs#L18)
