---
title: "vue/no-restricted-v-on"
description: "Interdit l'argument v-on configuré."
---

# `vue/no-restricted-v-on`

Interdit l'argument v-on configuré.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le bouton écoute l'événement interdit legacy-click.

```vue annotate="remove:2"
<template>
  <button type="button" @legacy-click="save">x</button>
</template>
```

## Bon

Le bouton écoute click.

```vue annotate="add:2"
<template>
  <button type="button" @click="save">x</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-v-on": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-v-on": [
        "legacy-click"
      ]
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_v_on.rs#L13)
