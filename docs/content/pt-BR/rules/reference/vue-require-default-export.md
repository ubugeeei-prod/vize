---
title: "vue/require-default-export"
description: "Exige que o script do componente seja o export padrão."
---

# `vue/require-default-export`

Exige que o script do componente seja o export padrão.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O script do componente não tem export padrão.

```vue
<script lang="ts">
const label = "a";
</script>
<template>
  <p>label</p>
</template>
```

## Correto

O componente é o export padrão do script.

```vue annotate="add:3"
<script lang="ts">
const label = "a";
export default { name: "Label" };
</script>
<template>
  <p>label</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-default-export": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_default_export.rs#L14)
