---
title: "vue/no-restricted-class"
description: "Interdit la class configurée."
---

# `vue/no-restricted-class`

Interdit la class configurée.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

L'élément utilise la class interdite legacy.

```vue annotate="remove:2"
<template>
  <div class="legacy">x</div>
</template>
```

## Bon

L'élément utilise une class autorisée.

```vue annotate="add:2"
<template>
  <div class="current">x</div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-class": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-class": [
        "legacy"
      ]
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_class.rs#L13)
