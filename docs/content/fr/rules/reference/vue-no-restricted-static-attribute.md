---
title: "vue/no-restricted-static-attribute"
description: "Interdit l'attribut statique configuré."
---

# `vue/no-restricted-static-attribute`

Interdit l'attribut statique configuré.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le bouton porte l'attribut interdit data-legacy.

```vue annotate="remove:2"
<template>
  <button type="button" data-legacy="x">x</button>
</template>
```

## Bon

Le bouton ne garde que type.

```vue annotate="add:2"
<template>
  <button type="button">x</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-static-attribute": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-static-attribute": [
        "data-legacy"
      ]
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_static_attribute.rs#L13)
