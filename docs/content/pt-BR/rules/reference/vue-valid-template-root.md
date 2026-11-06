---
title: "vue/valid-template-root"
description: "Exigir uma raiz `<template>` válida para a semântica de fragmentos do Vue 3."
---

# `vue/valid-template-root`

Exigir uma raiz `<template>` válida para a semântica de fragmentos do Vue 3.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Um `<template>` aninhado comum ocupa a raiz do template sem uma diretiva que lhe dê uma função de renderização.

```vue annotate="remove:2"
<template>
  <template>content</template>
</template>
```

## Correto

A `<div>` é um elemento raiz renderizável. Este exemplo não impõe uma restrição universal de raiz única aos fragmentos do Vue 3.

```vue annotate="add:2"
<template>
  <div>content</div>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-template-root": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_template_root.rs#L82)
