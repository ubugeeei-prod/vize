---
title: "vue/no-define-model-default"
description: "Interdit les valeurs par défaut dans defineModel."
---

# `vue/no-define-model-default`

Interdit les valeurs par défaut dans defineModel.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le model enfant déclare une valeur pouvant désynchroniser le parent.

```vue annotate="remove:1,2"
<script setup>
const model = defineModel({ default: "draft" })
</script>
<template />
```

## Bon

Le parent fournit la valeur initiale du model.

```vue annotate="add:1,2"
<script setup lang="ts">
const model = defineModel<string>()
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
      "vue/no-define-model-default": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc.rs#L43)
