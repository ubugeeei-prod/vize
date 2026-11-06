---
title: "script/no-deprecated-dollar-scopedslots-api"
description: "Interdire la propriété d’instance $scopedSlots supprimée dans Vue 3 (utiliser $slots)."
---

# `script/no-deprecated-dollar-scopedslots-api`

Interdire la propriété d’instance $scopedSlots supprimée dans Vue 3 (utiliser $slots).

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`this.$scopedSlots`, `ctx.$scopedSlots` et la référence directe `$scopedSlots` utilisent l’API des slots à portée de Vue 2, supprimée dans Vue 3.

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const header = this.$scopedSlots.header
const footer = ctx.$scopedSlots.footer
render($scopedSlots.default)
</script>
```

## Bon

Remplacer `$scopedSlots` par `$slots` utilise l’API unifiée des slots. L’exemple supprime l’écriture dépréciée sans établir un contexte setup pour les objets récepteurs.

```vue annotate="add:2,3,4"
<script setup lang="ts">
const header = this.$slots.header
const footer = ctx.$slots.footer
render($slots.default)
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-deprecated-dollar-scopedslots-api": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_scopedslots_api.rs#L45)
