---
title: "a11y/aria-role"
description: "Exigir que elementos com papéis ARIA usem um papel ARIA válido e não abstrato."
---

# `a11y/aria-role`

Exigir que elementos com papéis ARIA usem um papel ARIA válido e não abstrato.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

`datepicker` não é um papel ARIA reconhecido para esta seção.

```vue annotate="remove:2"
<template>
  <section role="datepicker">...</section>
</template>
```

## Correto

A seção usa o papel reconhecido `dialog` e um rótulo que descreve a seleção da data.

```vue annotate="add:2"
<template>
  <section role="dialog" aria-label="Choose a date">...</section>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/aria-role": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de acessibilidade](../accessibility.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_role.rs#L23)
