---
title: "vue/no-negated-condition"
description: "Proíbe ternários iniciados por uma condição negada."
---

# `vue/no-negated-condition`

Proíbe ternários iniciados por uma condição negada.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O ternário começa com uma condição negada.

```vue annotate="remove:2"
<template>
  <p>{{ !ready ? fallback : primary }}</p>
</template>
```

## Correto

A condição positiva e a ordem dos ramos expressam a escolha diretamente.

```vue annotate="add:2"
<template>
  <p>{{ ready ? primary : fallback }}</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-negated-condition": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template.rs#L139)
