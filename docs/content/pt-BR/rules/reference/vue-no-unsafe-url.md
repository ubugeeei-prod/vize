---
title: "vue/no-unsafe-url"
description: "Alertar sobre vinculações de URL potencialmente inseguras."
---

# `vue/no-unsafe-url`

Alertar sobre vinculações de URL potencialmente inseguras.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O destino da âncora começa com o esquema executável javascript:.

```vue annotate="remove:2"
<template>
<a href="javascript:alert(1)">Continue</a>
</template>
```

## Correto

A âncora usa o destino local comum de navegação /next.

```vue annotate="add:2"
<template>
<a href="/next">Continue</a>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-unsafe-url": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsafe_url.rs#L55)
