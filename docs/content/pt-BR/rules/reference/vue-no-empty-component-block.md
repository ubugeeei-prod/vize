---
title: "vue/no-empty-component-block"
description: "Proibir blocos vazios em SFCs."
---

# `vue/no-empty-component-block`

Proibir blocos vazios em SFCs.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Os blocos template, script e style não contêm conteúdo significativo.

```vue annotate="remove:1,3,5"
<template></template>

<script></script>

<style>
</style>
```

## Correto

Cada bloco mantido contém marcação, declarações de script ou declarações de estilo de fato.

```vue annotate="add:1,2,3,5,6,7,9,10"
<template>
  <div>Hello</div>
</template>

<script setup>
const message = "Hello";
</script>

<style scoped>
.button { color: red; }
</style>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-empty-component-block": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_empty_component_block.rs#L42)
