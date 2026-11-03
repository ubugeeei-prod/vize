---
title: "vue/no-deprecated-inline-template"
description: "Proibir o atributo obsoleto `inline-template`."
---

# `vue/no-deprecated-inline-template`

Proibir o atributo obsoleto `inline-template`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Card usa o atributo obsoleto inline-template para o conteúdo fornecido.

```vue annotate="remove:2"
<template>
<Card inline-template><p>Details</p></Card>
</template>
```

## Correto

O mesmo conteúdo é passado normalmente, sem o atributo inline-template.

```vue annotate="add:2"
<template>
<Card><p>Details</p></Card>
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
        "vue/no-deprecated-inline-template": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_inline_template.rs#L20)
