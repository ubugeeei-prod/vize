---
title: "script/no-duplicate-attr-inheritance"
description: "Signaler un composant qui applique deux fois ses attributs transmis automatiquement."
---

# `script/no-duplicate-attr-inheritance`

Signaler un composant qui applique deux fois ses attributs transmis automatiquement.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

Les valeurs explicites `inheritAttrs: true` répètent le comportement par défaut de Vue. Cette règle signale ce littéral redondant même si aucun déploiement de `$attrs` sur la racine n’est montré.

```vue annotate="remove:1"
<script setup></script>
<template><div><button v-bind="$attrs">Save</button></div></template>
```

## Bon

`inheritAttrs: false` exprime une véritable désactivation, tandis que l’objet d’options vide laisse l’héritage par défaut implicite. Aucun ne répète la valeur redondante `true`.

```vue annotate="add:1"
<script setup>defineOptions({ inheritAttrs: false })</script>
<template><div><button v-bind="$attrs">Save</button></div></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/no-duplicate-attr-inheritance": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_duplicate_attr_inheritance.rs#L35)
