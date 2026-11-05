---
title: "vue/no-textarea-mustache"
description: "Proibir interpolação com chaves duplas em `<textarea>`."
---

# `vue/no-textarea-mustache`

Proibir interpolação com chaves duplas em `<textarea>`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O textarea coloca message em uma interpolação filha, em vez de vincular seu valor.

```vue annotate="remove:2"
<template>
  <textarea>{{ message }}</textarea>
</template>
```

## Correto

v-model vincula o valor editável do textarea a message.

```vue annotate="add:2"
<template>
  <textarea v-model="message"></textarea>
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
        "vue/no-textarea-mustache": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26)
