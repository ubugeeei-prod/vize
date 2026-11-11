---
title: "vue/require-default-export"
description: "Exige que le script du composant soit l'export par défaut."
---

# `vue/require-default-export`

Exige que le script du composant soit l'export par défaut.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le script du composant n'a pas d'export par défaut.

```vue
<script lang="ts">
const label = "a";
</script>
<template>
  <p>label</p>
</template>
```

## Bon

Le composant est l'export par défaut du script.

```vue annotate="add:3"
<script lang="ts">
const label = "a";
export default { name: "Label" };
</script>
<template>
  <p>label</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-default-export": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_default_export.rs#L14)
