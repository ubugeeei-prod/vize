---
title: "vue/v-if-else-key"
description: "Exige key em componentes condicionais repetidos."
---

# `vue/v-if-else-key`

Exige key em componentes condicionais repetidos.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

As instâncias condicionais repetidas de `Panel` não têm key.

```vue annotate="remove:2"
<template>
  <section><Panel v-if="ready" /><Panel v-else /></section>
</template>
```

## Correto

Cada componente condicional tem uma key estável e única.

```vue annotate="add:2"
<template>
  <section><Panel v-if="ready" key="ready" /><Panel v-else key="fallback" /></section>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/v-if-else-key": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template.rs#L797)
