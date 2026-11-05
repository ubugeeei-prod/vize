---
title: "script/prefer-use-template-ref"
description: "Recommander useTemplateRef plutôt que ref pour les références de template (Vue 3.5+)."
---

# `script/prefer-use-template-ref`

Recommander useTemplateRef plutôt que ref pour les références de template (Vue 3.5+).

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

La ref nullable `input` est associée au littéral `ref="input"` du template, ce qui l’identifie comme une référence d’élément plutôt que comme une donnée nullable ordinaire.

```vue annotate="remove:2,3"
<script setup lang="ts">
import { ref } from 'vue'
const input = ref<HTMLInputElement | null>(null)
</script>
<template>
  <input ref="input" />
</template>
```

## Bon

`useTemplateRef<HTMLInputElement>('input')` de Vue 3.5+ rend cette référence de template explicite. `error = ref(null)`, sans association, reste une donnée ordinaire et est volontairement hors du champ de cette règle.

```vue annotate="add:2,3,4,5,6,10"
<script setup lang="ts">
import { ref, useTemplateRef } from 'vue'
// Paired with the template ref below.
const input = useTemplateRef<HTMLInputElement>('input')
// A nullable data ref the template never binds as a ref.
const error = ref(null)
</script>
<template>
  <input ref="input" />
  <p>{{ error }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-template-ref": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_template_ref.rs#L75)
