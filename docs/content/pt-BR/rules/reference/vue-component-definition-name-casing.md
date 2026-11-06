---
title: "vue/component-definition-name-casing"
description: "Exigir PascalCase ou kebab-case nos nomes de definição de componentes."
---

# `vue/component-definition-name-casing`

Exigir PascalCase ou kebab-case nos nomes de definição de componentes.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

O nome de arquivo do componente é verificado. PascalCase e kebab-case são aceitos; o uso misturado de maiúsculas e minúsculas é reportado.

## Incorreto

O nome de arquivo myComponent.vue mistura uma inicial minúscula com uma letra maiúscula interna, em vez de usar PascalCase ou kebab-case.

`myComponent.vue`

```vue
<template><p>Content</p></template>
```

## Correto

Renomear o arquivo para MyComponent.vue aplica PascalCase; o conteúdo do template permanece igual.

`MyComponent.vue`

```vue
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
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L36)
