---
title: "ecosystem/void-link-require-href"
description: "Exigir `href` nos componentes Link do Void Vue."
---

# `ecosystem/void-link-require-href`

Exigir `href` nos componentes Link do Void Vue.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `ecosystem` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O Link importado de @void/vue omite seu destino href.

```vue annotate="remove:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link>Settings</Link>
</template>
```

## Correto

O mesmo Link importado recebe o destino de configurações por meio de href.

```vue annotate="add:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/settings">Settings</Link>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ecosystem/void-link-require-href": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras do ecossistema e Nuxt](../ecosystem.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_require_href.rs#L14)
