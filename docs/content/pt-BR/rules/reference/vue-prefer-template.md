---
title: "vue/prefer-template"
description: "Prefere template literals à concatenação de strings."
---

# `vue/prefer-template`

Prefere template literals à concatenação de strings.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Um literal é concatenado com `name`.

```vue annotate="remove:2"
<template>
  <p>{{ "Hello " + name }}</p>
</template>
```

## Correto

O template literal mantém a interpolação em uma única string.

```vue annotate="add:2"
<template>
  <p>{{ `Hello ${name}` }}</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-template": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules.rs#L145)
