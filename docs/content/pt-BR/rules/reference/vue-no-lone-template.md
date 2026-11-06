---
title: "vue/no-lone-template"
description: "Proibir elementos `<template>` desnecessários."
---

# `vue/no-lone-template`

Proibir elementos `<template>` desnecessários.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O template interno não tem uma diretiva nem uma função de slot que lhe dê uma finalidade estrutural.

```vue annotate="remove:2"
<template>
<div><template><p>Details</p></template></div>
</template>
```

## Correto

Remover o invólucro desnecessário deixa o parágrafo diretamente dentro da div.

```vue annotate="add:2"
<template>
<div><p>Details</p></div>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-lone-template": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_lone_template.rs#L32)
