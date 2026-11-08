---
title: "html/no-duplicate-class"
description: "Proibir nomes de classe duplicados em um atributo class estático."
---

# `html/no-duplicate-class`

Proibir nomes de classe duplicados em um atributo class estático.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

A lista de classes estática repete o token `btn`.

```vue annotate="remove:2"
<template>
  <div class="btn btn primary">click</div>
</template>
```

## Correto

A lista de classes mantém um token `btn` e o token distinto `primary`.

```vue annotate="add:2"
<template>
  <div class="btn primary">click</div>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/no-duplicate-class": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras HTML](../html.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_duplicate_class.rs#L34)
