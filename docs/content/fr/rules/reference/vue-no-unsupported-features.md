---
title: "vue/no-unsupported-features"
description: "Interdit une syntaxe encore non prise en charge sur la version Vue.js configurée."
---

# `vue/no-unsupported-features`

Interdit une syntaxe encore non prise en charge sur la version Vue.js configurée.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

defineModel exige un Vue plus récent que la version configurée.

```vue annotate="remove:2,5"
<script setup lang="ts">
const model = defineModel<string>();
</script>
<template>
  <p>{{ model }}</p>
</template>
```

## Bon

Le composant n'utilise pas de macro limitée par version.

```vue annotate="add:2,5"
<script setup lang="ts">
const label = "ok";
</script>
<template>
  <p>{{ label }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-unsupported-features": "warn"
    },
    "ruleOptions": {
      "vue/no-unsupported-features": {
        "version": "^3.3.0"
      }
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/unsupported_features.rs#L19)
