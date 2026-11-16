---
title: "ecosystem/router-link-require-to"
description: "Exigir um destino `to` nos componentes RouterLink e NuxtLink."
---

# `ecosystem/router-link-require-to`

Exigir um destino `to` nos componentes RouterLink e NuxtLink.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `ecosystem` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

Um link que seja a única raiz de um SFC pode herdar seu target dos atributos do pai. Este exemplo usa um link aninhado, cujo target deve ser explícito.

## Incorreto

O RouterLink aninhado não tem um destino `to`; ele não pode depender da propagação de atributos do elemento raiz.

```vue annotate="remove:2"
<template>
<nav><RouterLink>Settings</RouterLink></nav>
</template>
```

## Correto

`to="/settings"` fornece explicitamente o destino do link aninhado.

```vue annotate="add:2"
<template>
<nav><RouterLink to="/settings">Settings</RouterLink></nav>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ecosystem/router-link-require-to": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras do ecossistema e Nuxt](../ecosystem.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/router_link_require_to.rs#L15)
