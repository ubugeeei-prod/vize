---
title: "vue/valid-v-html"
description: "Exigir diretivas `v-html` válidas."
---

# `vue/valid-v-html`

Exigir diretivas `v-html` válidas.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

`v-html` não tem sua expressão ou usa um argumento ou modificador que esta diretiva não aceita.

```vue annotate="remove:2,3,4"
<template>
<div v-html></div>
<div v-html:arg="foo"></div>
<div v-html.mod="foo"></div>
</template>
```

## Correto

`v-html="html"` fornece uma expressão válida. A validade sintática não sanitiza HTML nem torna seguro conteúdo não confiável.

```vue annotate="add:2"
<template>
<div v-html="html"></div>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-html": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_html.rs#L28)
