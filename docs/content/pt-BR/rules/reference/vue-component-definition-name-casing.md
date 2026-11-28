---
title: "vue/component-definition-name-casing"
description: "Exigir PascalCase ou kebab-case nos nomes de definição de componentes."
---

# `vue/component-definition-name-casing`

Exigir PascalCase ou kebab-case nos nomes de definição de componentes.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

Verifica o nome estático de uma definição de componente: a opção name, defineOptions({ name }) e o primeiro argumento de app.component(). O nome de arquivo é verificado por vue/component-file-name-casing.

## Incorreto

O nome my_component declarado com defineOptions separa as palavras com sublinhado, em vez de usar PascalCase.

```vue annotate="remove:2"
<script setup lang="ts">
defineOptions({ name: "my_component" });
</script>
<template><p>Content</p></template>
```

## Correto

O nome declarado MyComponent usa PascalCase, o padrão.

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: "MyComponent" });
</script>
<template><p>Content</p></template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/component-definition-name-casing": "warn"
    },
    "ruleOptions": {
      "vue/component-definition-name-casing": "PascalCase"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L68)
