---
title: "max-lines"
description: "Limita o número de linhas de um arquivo."
---

# `max-lines`

Limita o número de linhas de um arquivo.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O arquivo tem mais linhas do que o máximo configurado.

```vue annotate="remove:4,6,7,8,9,10,11,12,13,14,15,16,17"
<script setup lang="ts">
const label = "a";
</script>

<template>
  <ul>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
  </ul>
</template>

```

## Correto

O arquivo permanece dentro do máximo configurado.

```vue annotate="add:5"
<script setup lang="ts">
const label = "a";
</script>
<template>
  <p>{{ label }}</p>
</template>

```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "max-lines": "warn"
    },
    "ruleOptions": {
      "max-lines": {
        "max": 10
      }
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/max_lines.rs#L14)
