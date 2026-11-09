---
title: "vue/next-tick-style"
description: "Exige le style Promise pour nextTick."
---

# `vue/next-tick-style`

Exige le style Promise pour nextTick.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Un callback est passé à `nextTick`.

```vue annotate="remove:3"
<script setup>
import { nextTick } from "vue"
nextTick(() => focusInput())
</script>
<template />
```

## Bon

Le code attend la Promise avant de continuer.

```vue annotate="add:3,4"
<script setup>
import { nextTick } from "vue"
await nextTick()
focusInput()
</script>
<template />
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/next-tick-style": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc.rs#L36)
