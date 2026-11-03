---
title: "ecosystem/nuxt-prefer-nuxt-link"
description: "Préférer NuxtLink pour les liens internes à l’application."
---

# `ecosystem/nuxt-prefer-nuxt-link`

Préférer NuxtLink pour les liens internes à l’application.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

La destination interne des paramètres utilise un lien ordinaire dans une application Nuxt.

```vue annotate="remove:2"
<template>
  <a href="/settings">Settings</a>
</template>
```

## Bon

NuxtLink gère la même destination interne au moyen du routeur Nuxt.

```vue annotate="add:2"
<template>
  <NuxtLink to="/settings">Settings</NuxtLink>
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
        "ecosystem/nuxt-prefer-nuxt-link": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de l’écosystème et de Nuxt](../ecosystem.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/nuxt_prefer_nuxt_link.rs#L14)
