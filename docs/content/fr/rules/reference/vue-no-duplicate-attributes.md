---
title: "vue/no-duplicate-attributes"
description: "Interdire les attributs en double sur un même élément."
---

# `vue/no-duplicate-attributes`

Interdire les attributs en double sur un même élément.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le même bouton déclare class deux fois au lieu de réunir les classes dans une seule valeur.

```vue annotate="remove:2"
<template>
  <button class="primary" class="large">Save</button>
</template>
```

## Bon

Les deux noms de classes figurent dans un unique attribut class.

```vue annotate="add:2"
<template>
  <button class="primary large">Save</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-duplicate-attributes": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_duplicate_attributes.rs#L31)
