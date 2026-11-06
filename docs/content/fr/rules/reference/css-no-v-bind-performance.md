---
title: "css/no-v-bind-performance"
description: "Signaler le coût de performance de v-bind() en CSS."
---

# `css/no-v-bind-performance`

Signaler le coût de performance de v-bind() en CSS.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Non | Aucune |

Champ d’application : Blocs style en CSS standard (les blocs scss, sass, less et stylus sont ignorés).

## Mauvais

La feuille de style lit la valeur changeante `offset` au moyen du mécanisme CSS `v-bind()` du SFC.

```vue annotate="remove:1,2,3,4,5"
<style scoped>
.card {
  transform: translateX(v-bind(offset));
}
</style>
```

## Bon

L’élément reçoit directement la transformation changeante via sa liaison de style.

```vue annotate="add:1,2,3"
<template>
  <article :style="{ transform: `translateX(${offset}px)` }" class="card" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "css/no-v-bind-performance": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles CSS](../css.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_v_bind_performance.rs#L20)
