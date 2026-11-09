---
title: "vue/quote-props"
description: "Interdit les guillemets inutiles sur les propriétés d'objet."
---

# `vue/quote-props`

Interdit les guillemets inutiles sur les propriétés d'objet.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | [Options des règles](/rules/options.md) |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

La clé valide `name` est inutilement entre guillemets.

```vue annotate="remove:2"
<template>
  <Widget :options="{ "name": value }" />
</template>
```

## Bon

La propriété utilise sa forme d'identifiant sans guillemets.

```vue annotate="add:2"
<template>
  <Widget :options="{ name: value }" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/quote-props": "warn"
    },
    "ruleOptions": {
      "vue/quote-props": {
        "vue/quote-props": "as-needed"
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
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template.rs#L174)
