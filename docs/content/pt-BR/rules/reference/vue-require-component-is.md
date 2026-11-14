---
title: "vue/require-component-is"
description: "Exigir `v-bind:is` em elementos `<component>`."
---

# `vue/require-component-is`

Exigir `v-bind:is` em elementos `<component>`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O `<component>` dinâmico não tem um destino `is`, então o Vue não pode escolher um componente para renderizar.

```vue annotate="remove:2"
<template>
  <component />
</template>
```

## Correto

`:is="currentComponent"` fornece a seleção do componente; a vinculação pode mudar durante a execução.

```vue annotate="add:2"
<template>
  <component :is="currentComponent" />
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-component-is": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_component_is.rs#L28)
