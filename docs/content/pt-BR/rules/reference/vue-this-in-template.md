---
title: "vue/this-in-template"
description: "Proibir `this.` nas expressões do template."
---

# `vue/this-in-template`

Proibir `this.` nas expressões do template.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

As expressões do template acessam explicitamente `this.message`, `this.className` e `this.handleClick`, embora o Vue exponha essas variáveis diretamente.

```vue annotate="remove:2,3,4"
<template>
<div>{{ this.message }}</div>
<div :class="this.className"></div>
<button @click="this.handleClick()"></button>
</template>
```

## Correto

Use `message`, `className` e `handleClick` diretamente. A string literal `'this.is.a.string'` permanece igual porque não é um acesso a membro.

```vue annotate="add:2,3,4,5"
<template>
<div>{{ message }}</div>
<div :class="className"></div>
<button @click="handleClick()"></button>
<div>{{ 'this.is.a.string' }}</div>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/this-in-template": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/this_in_template.rs#L37)
