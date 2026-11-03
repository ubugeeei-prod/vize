---
title: "vue/valid-v-text"
description: "Exigir diretivas `v-text` válidas."
---

# `vue/valid-v-text`

Exigir diretivas `v-text` válidas.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

`v-text` não tem sua expressão de texto ou usa um argumento ou modificador que não é aceito.

```vue annotate="remove:2,3,4"
<template>
<div v-text></div>
<div v-text:arg="foo"></div>
<div v-text.mod="foo"></div>
</template>
```

## Correto

`v-text="msg"` é sintaticamente válido. A regra de estilo separada `vue/no-v-text` ainda pode preferir interpolação.

```vue annotate="add:2"
<template>
<div v-text="msg"></div>
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
        "vue/valid-v-text": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_text.rs#L27)
