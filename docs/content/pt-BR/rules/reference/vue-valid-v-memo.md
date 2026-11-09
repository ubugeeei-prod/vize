---
title: "vue/valid-v-memo"
description: "Exigir diretivas `v-memo` válidas."
---

# `vue/valid-v-memo`

Exigir diretivas `v-memo` válidas.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

`v-memo` sozinho não fornece ao Vue uma expressão de dependências para decidir quando reutilizar a subárvore.

```vue annotate="remove:2"
<template>
  <div v-memo></div>
</template>
```

## Correto

`v-memo="[valueA, valueB]"` fornece o array de dependências usado para memorização.

```vue annotate="add:2"
<template>
  <div v-memo="[valueA, valueB]">{{ label }}</div>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-memo": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_memo.rs#L28)
