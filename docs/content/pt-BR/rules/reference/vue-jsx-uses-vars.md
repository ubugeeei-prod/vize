---
title: "vue/jsx-uses-vars"
description: "Considera usadas as variáveis referenciadas nos nomes dos elementos JSX."
---

# `vue/jsx-uses-vars`

Considera usadas as variáveis referenciadas nos nomes dos elementos JSX.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

Diagnóstico do exemplo incorreto: `vue/no-unused-components`

Esta regra auxiliar não emite um diagnóstico `vue/jsx-uses-vars`. Ela reconhece os bindings de tags JSX quando `vue/no-unused-components` decide se um componente é usado. Ative as duas regras neste exemplo. `<MyButton.Part />` reconhece o binding raiz `MyButton`; tags com namespace e membros de `this` não fornecem um binding de componente importado.

## Incorreto

`MyButton` é importado, mas nem o template nem o JSX o referenciam, então a regra complementar `vue/no-unused-components` o reporta.

```vue annotate="remove:3"
<script setup lang="jsx">
import MyButton from "./MyButton.vue";
const view = () => <div />;
</script>
<template><div /></template>
```

## Correto

A tag JSX referencia o `MyButton` importado; a regra auxiliar reconhece esse uso e a regra complementar deixa de reportar um componente não utilizado.

```vue annotate="add:3"
<script setup lang="jsx">
import MyButton from "./MyButton.vue";
const view = () => <MyButton />;
</script>
<template><div /></template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/jsx-uses-vars": "error",
        "vue/no-unused-components": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/jsx_uses_vars.rs#L36)
