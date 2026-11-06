---
title: "script/require-default-prop"
description: "Exiger une valeur par défaut pour chaque prop facultative non booléenne."
---

# `script/require-default-prop`

Exiger une valeur par défaut pour chaque prop facultative non booléenne.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `opinionated`, `nuxt` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`name` et `age` sont des props à l’exécution facultatives, non booléennes et sans valeur par défaut ; leurs valeurs en cas d’omission restent indéterminées.

```vue annotate="remove:4,5,6"
<script lang="ts">
export default {
  props: {
    // optional, non-Boolean, no default
    name: String,
    age: { type: Number },
  }
}
</script>
```

## Bon

`name` reçoit `default: ''`. `enabled` utilise la valeur false implicite de Boolean, et `id`, obligatoire, n’a pas besoin de valeur de repli, ce qui illustre les deux exemptions.

```vue annotate="add:4,5,6"
<script lang="ts">
export default {
  props: {
    name: { type: String, default: '' },
    enabled: Boolean,                 // Boolean defaults to false
    id: { type: Number, required: true },
  }
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
      "script/require-default-prop": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_default_prop.rs#L58)
