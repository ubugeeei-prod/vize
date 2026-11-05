---
title: "vue/prefer-true-attribute-shorthand"
description: "Preferir a forma abreviada para um atributo booleano vinculado a `true`."
---

# `vue/prefer-true-attribute-shorthand`

Preferir a forma abreviada para um atributo booleano vinculado a `true`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Um atributo booleano nativo disabled vincula o valor constante true.

```vue annotate="remove:2"
<template>
<input :disabled="true" />
</template>
```

## Correto

O atributo nativo usa sua forma booleana abreviada. Vinculações false e props de componentes mantêm seus valores explícitos.

```vue annotate="add:2,3,4,5"
<template>
<input disabled />
<input :disabled="false" />
<MyComponent :visible="true" />
<MyComponent :visible="isVisible" />
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
        "vue/prefer-true-attribute-shorthand": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_true_attribute_shorthand.rs#L38)
