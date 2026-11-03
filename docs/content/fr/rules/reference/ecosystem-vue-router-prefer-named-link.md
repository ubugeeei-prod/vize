---
title: "ecosystem/vue-router-prefer-named-link"
description: "Préférer les objets de routes nommées aux chaînes de chemin statiques dans RouterLink."
---

# `ecosystem/vue-router-prefer-named-link`

Préférer les objets de routes nommées aux chaînes de chemin statiques dans RouterLink.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `ecosystem` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

La destination de RouterLink est un chemin littéral plutôt qu’une route nommée.

```vue annotate="remove:2"
<template>
  <RouterLink to="/settings">Settings</RouterLink>
</template>
```

## Bon

L’objet de route lié identifie la destination par son nom de route settings.

```vue annotate="add:2"
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
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
        "ecosystem/vue-router-prefer-named-link": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de l’écosystème et de Nuxt](../ecosystem.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/vue_router_prefer_named_link.rs#L15)
