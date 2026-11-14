---
title: "ecosystem/nuxt-prefer-nuxt-link"
description: "Preferir NuxtLink para links internos da aplicação."
---

# `ecosystem/nuxt-prefer-nuxt-link`

Preferir NuxtLink para links internos da aplicação.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O destino interno de configurações usa uma âncora comum em uma aplicação Nuxt.

```vue annotate="remove:2"
<template>
  <a href="/settings">Settings</a>
</template>
```

## Correto

NuxtLink trata o mesmo destino interno por meio do roteador do Nuxt.

```vue annotate="add:2"
<template>
  <NuxtLink to="/settings">Settings</NuxtLink>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ecosystem/nuxt-prefer-nuxt-link": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras do ecossistema e Nuxt](../ecosystem.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/nuxt_prefer_nuxt_link.rs#L15)
