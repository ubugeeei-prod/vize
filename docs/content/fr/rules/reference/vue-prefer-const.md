---
title: "vue/prefer-const"
description: "Exiger const pour les liaisons que le script et le template ne réassignent jamais."
---

# `vue/prefer-const`

Exiger const pour les liaisons que le script et le template ne réassignent jamais.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le template lit label, et ni le script ni le template ne le réassigne, donc let est inutile.

```vue annotate="remove:2"
<script setup>
let label = "Hello";
</script>
<template><p>{{ label }}</p></template>
```

## Bon

const correspond à une liaison que le script et le template ne font que lire.

```vue annotate="add:2"
<script setup>
const label = "Hello";
</script>
<template><p>{{ label }}</p></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-const": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/script_unused/prefer_const.rs#L16)
