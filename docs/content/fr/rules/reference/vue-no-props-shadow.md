---
title: "vue/no-props-shadow"
description: "Interdit aux déclarations de script setup de masquer les props."
---

# `vue/no-props-shadow`

Interdit aux déclarations de script setup de masquer les props.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

La déclaration locale `value` masque la prop homonyme.

```vue annotate="remove:3"
<script setup>
defineProps({ value: String })
const value = "local"
</script>
<template />
```

## Bon

La liaison locale utilise un nom distinct.

```vue annotate="add:3"
<script setup>
defineProps({ value: String })
const localValue = "local"
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
      "vue/no-props-shadow": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/options.rs#L16)
