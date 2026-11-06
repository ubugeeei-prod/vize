---
title: "css/no-v-bind-performance"
description: "Alertar sobre o custo de desempenho do v-bind() no CSS."
---

# `css/no-v-bind-performance`

Alertar sobre o custo de desempenho do v-bind() no CSS.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Não | Nenhuma |

Aplicável a: Blocos style em CSS puro (blocos scss, sass, less e stylus são ignorados).

## Incorreto

A folha de estilos lê o valor variável de `offset` por meio do mecanismo `v-bind()` do CSS de SFC.

```vue annotate="remove:1,2,3,4,5"
<style scoped>
.card {
  transform: translateX(v-bind(offset));
}
</style>
```

## Correto

O elemento recebe a transformação variável diretamente por meio da vinculação de estilo.

```vue annotate="add:1,2,3"
<template>
  <article :style="{ transform: `translateX(${offset}px)` }" class="card" />
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "css/no-v-bind-performance": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras CSS](../css.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_v_bind_performance.rs#L20)
