---
title: "vue/require-prop-comment"
description: "Exige um comentário JSDoc em cada prop declarada."
---

# `vue/require-prop-comment`

Exige um comentário JSDoc em cada prop declarada.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

A prop size não tem comentário JSDoc.

```vue annotate="remove:2"
<script setup lang="ts">
const { size } = defineProps<{ size: string }>();
</script>
<template>
  <p>{{ size }}</p>
</template>
```

## Correto

A prop size documenta seu significado com um comentário JSDoc.

```vue annotate="add:2,3,4,5"
<script setup lang="ts">
const { size } = defineProps<{
  /** The rendered size. */
  size: string;
}>();
</script>
<template>
  <p>{{ size }}</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-prop-comment": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_prop_comment.rs#L18)
