---
title: "vue/eqeqeq"
description: "Exige igualdade estrita em expressões do template."
---

# `vue/eqeqeq`

Exige igualdade estrita em expressões do template.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

A igualdade flexível pode converter os operandos.

```vue annotate="remove:2"
<template>
  <p>{{ value == null }}</p>
</template>
```

## Correto

A igualdade estrita compara sem conversão.

```vue annotate="add:2"
<template>
  <p>{{ value === null }}</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/eqeqeq": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules.rs#L68)
