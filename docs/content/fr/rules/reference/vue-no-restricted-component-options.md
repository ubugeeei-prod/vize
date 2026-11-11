---
title: "vue/no-restricted-component-options"
description: "Interdit l'option de composant configurée."
---

# `vue/no-restricted-component-options`

Interdit l'option de composant configurée.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le composant définit inheritAttrs, qui est interdit.

```vue annotate="remove:2"
<script setup lang="ts">
defineOptions({ inheritAttrs: false });
</script>
<template>
  <p>ok</p>
</template>
```

## Bon

Le composant ne définit que name.

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
      "vue/no-restricted-component-options": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-component-options": [
        "inheritAttrs"
      ]
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/component_options.rs#L12)
