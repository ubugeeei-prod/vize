---
title: "vue/no-undefined-refs"
description: "Interdire les références à des variables non définies dans les templates."
---

# `vue/no-undefined-refs`

Interdire les références à des variables non définies dans les templates.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le template lit missing, alors que le script ne déclare que message.

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template>{{ missing }}</template>
```

## Bon

L’interpolation lit la liaison message existante.

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template>{{ message }}</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-undefined-refs": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_undefined_refs.rs#L14)
