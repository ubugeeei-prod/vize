---
title: "ecosystem/vue-router-prefer-named-link"
description: "Preferir objetos de rotas nomeadas a strings de caminho estáticas no RouterLink."
---

# `ecosystem/vue-router-prefer-named-link`

Preferir objetos de rotas nomeadas a strings de caminho estáticas no RouterLink.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `ecosystem` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O destino do RouterLink é um caminho literal, em vez de uma rota nomeada.

```vue annotate="remove:2"
<template>
  <RouterLink to="/settings">Settings</RouterLink>
</template>
```

## Correto

O objeto de rota vinculado identifica o destino pelo nome de rota settings.

```vue annotate="add:2"
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
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
        "ecosystem/vue-router-prefer-named-link": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras do ecossistema e Nuxt](../ecosystem.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/vue_router_prefer_named_link.rs#L15)
