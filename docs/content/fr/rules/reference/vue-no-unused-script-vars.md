---
title: "vue/no-unused-script-vars"
description: "Interdire les variables, importations et paramètres de script inutilisés."
---

# `vue/no-unused-script-vars`

Interdire les variables, importations et paramètres de script inutilisés.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

script setup déclare unused, et ni le script ni le template ne le lit.

```vue annotate="remove:4"
<script setup>
const unused = 1;
</script>
<template><p>Welcome</p></template>
```

## Bon

L’interpolation du template lit unused, donc la liaison déclarée est utilisée.

```vue annotate="add:4"
<script setup>
const unused = 1;
</script>
<template><p>{{ unused }}</p></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-unused-script-vars": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/script_unused/no_unused_script_vars.rs#L16)
