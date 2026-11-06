---
title: "ecosystem/router-link-require-to"
description: "Exiger une destination `to` sur les composants RouterLink et NuxtLink."
---

# `ecosystem/router-link-require-to`

Exiger une destination `to` sur les composants RouterLink et NuxtLink.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `ecosystem` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Un lien constituant l’unique racine d’un SFC peut hériter de sa destination via les attributs du parent. Cet exemple utilise un lien imbriqué, dont la destination doit être explicite.

## Mauvais

Le RouterLink imbriqué n’a pas de destination `to` ; il ne peut pas compter sur la transmission automatique des attributs racines.

```vue annotate="remove:2"
<template>
<nav><RouterLink>Settings</RouterLink></nav>
</template>
```

## Bon

`to="/settings"` fournit explicitement la destination du lien imbriqué.

```vue annotate="add:2"
<template>
<nav><RouterLink to="/settings">Settings</RouterLink></nav>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ecosystem/router-link-require-to": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de l’écosystème et de Nuxt](../ecosystem.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/router_link_require_to.rs#L14)
