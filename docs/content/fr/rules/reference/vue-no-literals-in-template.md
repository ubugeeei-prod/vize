---
title: "vue/no-literals-in-template"
description: "Interdit les littéraux d'objet, tableau et fonction dans v-bind."
---

# `vue/no-literals-in-template`

Interdit les littéraux d'objet, tableau et fonction dans v-bind.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le littéral d'objet crée une nouvelle valeur au rendu.

```vue annotate="remove:2"
<template>
  <Widget :options="{ eager: true }" />
</template>
```

## Bon

Une liaison stable du script fournit l'objet d'options.

```vue annotate="add:2"
<template>
  <Widget :options="options" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-literals-in-template": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/template_rules.rs#L22)
