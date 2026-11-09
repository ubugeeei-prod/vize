---
title: "vue/require-macro-variable-name"
description: "Exige les noms conventionnels des variables de macro."
---

# `vue/require-macro-variable-name`

Exige les noms conventionnels des variables de macro.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le résultat de `defineProps` est lié à `p`.

```vue annotate="remove:2"
<script setup>
const p = defineProps({ value: String })
</script>
<template />
```

## Bon

Le nom conventionnel `props` identifie le résultat.

```vue annotate="add:2"
<script setup>
const props = defineProps({ value: String })
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
      "vue/require-macro-variable-name": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc.rs#L56)
