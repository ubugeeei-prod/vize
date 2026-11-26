---
title: "vue/no-undefined-refs"
description: "Interdire les références à des variables non définies dans les templates."
---

# `vue/no-undefined-refs`

Interdire les références à des variables non définies dans les templates.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

`globals` liste les noms qu’une étape de build définit pour tous les templates, comme les imports d’`unplugin-auto-import`. Dans un projet Nuxt, `vize lint` lit les imports automatiques dans le fichier généré `.nuxt/types/imports.d.ts` et les ajoute : ils n’ont donc pas besoin d’être listés ; tant que `nuxi prepare` n’a pas généré ce fichier, la règle est ignorée avec un avis. Les noms sont exacts. Les options n’activent pas la règle. Les couches suivantes remplacent la liste ; une liste vide efface les noms hérités.

## Mauvais

Le template lit missing, alors que le script ne déclare que message.

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template>{{ missing }}</template>
```

## Bon

L’interpolation lit la liaison message existante.

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template>{{ message }}</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-undefined-refs": "warn"
    },
    "ruleOptions": {
      "vue/no-undefined-refs": {
        "globals": [
          "formatCount"
        ]
      }
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_undefined_refs.rs#L13)
