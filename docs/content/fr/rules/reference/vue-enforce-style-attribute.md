---
title: "vue/enforce-style-attribute"
description: "Limite les blocs style aux modes autorisés."
---

# `vue/enforce-style-attribute`

Limite les blocs style aux modes autorisés.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | [Options des règles](/rules/options.md) |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le mode module n'est pas dans la liste autorisée.

```vue annotate="remove:2"
<template><div /></template>
<style module>.card { color: red }</style>
```

## Bon

Le mode scoped est accepté par la politique du projet.

```vue annotate="add:2"
<template><div /></template>
<style scoped>.card { color: red }</style>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/enforce-style-attribute": "warn"
    },
    "ruleOptions": {
      "vue/enforce-style-attribute": {
        "vue/enforce-style-attribute": {
          "allow": [
            "scoped"
          ]
        }
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
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc.rs#L32)
