---
title: "vue/no-restricted-component-names"
description: "Interdit le nom de composant configuré."
---

# `vue/no-restricted-component-names`

Interdit le nom de composant configuré.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le nom du composant est le nom interdit RestrictedShowcase.

```vue annotate="remove:2"
<script setup lang="ts">
defineOptions({ name: "RestrictedShowcase" });
</script>
<template>
  <p>ok</p>
</template>
```

## Bon

Le nom du composant est Widget.

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: "Widget" });
</script>
<template>
  <p>ok</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-component-names": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-component-names": [
        "RestrictedShowcase"
      ]
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/component_names.rs#L12)
