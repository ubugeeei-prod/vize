---
title: "vue/require-prop-comment"
description: "Exige un commentaire JSDoc sur chaque prop déclarée."
---

# `vue/require-prop-comment`

Exige un commentaire JSDoc sur chaque prop déclarée.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

La prop size n'a pas de commentaire JSDoc.

```vue annotate="remove:2"
<script setup lang="ts">
const { size } = defineProps<{ size: string }>();
</script>
<template>
  <p>{{ size }}</p>
</template>
```

## Bon

La prop size documente son sens avec un commentaire JSDoc.

```vue annotate="add:2,3,4,5"
<script setup lang="ts">
const { size } = defineProps<{
  /** The rendered size. */
  size: string;
}>();
</script>
<template>
  <p>{{ size }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-prop-comment": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_prop_comment.rs#L19)
