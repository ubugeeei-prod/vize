---
title: "script/no-deprecated-model-definition"
description: "Interdire l’option model de Vue 2, retirée dans Vue 3."
---

# `script/no-deprecated-model-definition`

Interdire l’option model de Vue 2, retirée dans Vue 3.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

Le composant déclare encore l’option Vue 2 `model` avec `prop` et `event`.

```vue annotate="remove:3,4,5,6"
<script lang="ts">
export default {
  model: {
    prop: 'value',
    event: 'input'
  }
}
</script>
```

## Bon

Le composant déclare la prop `modelValue` et l’événement `update:modelValue`.

```vue annotate="add:3,4"
<script lang="ts">
export default {
  props: ['modelValue'],
  emits: ['update:modelValue']
}
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-deprecated-model-definition": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_model_definition.rs#L32)
