---
title: "vue/no-preprocessor-lang"
description: "Desencorajar o uso de preprocessadores CSS em favor de CSS moderno."
---

# `vue/no-preprocessor-lang`

Desencorajar o uso de preprocessadores CSS em favor de CSS moderno.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O bloco style seleciona SCSS com lang. Isso descreve a convenção pretendida de não usar preprocessadores; o processamento atual de SFCs não emite esta regra.

```vue annotate="remove:2"
<template><p>Notice</p></template>
<style lang="scss">
.notice { color: red; }
</style>
```

## Correto

As mesmas declarações CSS omitem o lang do preprocessador. Essa é a correção da convenção, e não uma diferença executável de diagnósticos entre Bad/Good hoje.

```vue annotate="add:2"
<template><p>Notice</p></template>
<style>
.notice { color: red; }
</style>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-preprocessor-lang": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L11)
