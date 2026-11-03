---
title: "vue/no-unsandboxed-iframe"
description: "Exigir um atributo sandbox nos elementos iframe."
---

# `vue/no-unsandboxed-iframe`

Exigir um atributo sandbox nos elementos iframe.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O frame incorporado não tem um atributo sandbox que limite suas capacidades.

```vue annotate="remove:2"
<template>
<iframe src="/embed"></iframe>
</template>
```

## Correto

sandbox aplica restrições; allow-scripts habilita explicitamente essa única capacidade quando necessário.

```vue annotate="add:2,3"
<template>
<iframe src="/embed" sandbox></iframe>
<iframe src="/embed" sandbox="allow-scripts"></iframe>
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
        "vue/no-unsandboxed-iframe": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsandboxed_iframe.rs#L32)
