---
title: "vapor/prefer-static-class"
description: "Préférer une classe statique à une liaison de classe dynamique pour les chaînes littérales."
---

# `vapor/prefer-static-class`

Préférer une classe statique à une liaison de classe dynamique pour les chaînes littérales.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Oui | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

La liaison de classe évalue une chaîne constante alors que la classe ne change pas.

```vue annotate="remove:2"
<template>
  <section :class="'panel panel-primary'">Profile</section>
</template>
```

## Bon

Un attribut class statique exprime les mêmes classes du panneau sans liaison.

```vue annotate="add:2"
<template>
  <section class="panel panel-primary">Profile</section>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vapor/prefer-static-class": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vapor](../vapor.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/prefer_static_class.rs#L31)
