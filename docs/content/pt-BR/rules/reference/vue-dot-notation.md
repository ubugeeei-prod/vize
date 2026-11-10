---
title: "vue/dot-notation"
description: "Prefere notação de ponto para propriedades estáticas."
---

# `vue/dot-notation`

Prefere notação de ponto para propriedades estáticas.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

A propriedade estática `name` usa colchetes.

```vue annotate="remove:2"
<template>
  <p>{{ user["name"] }}</p>
</template>
```

## Correto

A notação de ponto expressa a mesma propriedade diretamente.

```vue annotate="add:2"
<template>
  <p>{{ user.name }}</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/dot-notation": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/dot_notation.rs#L12)
