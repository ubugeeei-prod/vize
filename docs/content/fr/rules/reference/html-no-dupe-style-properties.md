---
title: "html/no-dupe-style-properties"
description: "Interdire les propriétés en double dans les attributs de style en ligne."
---

# `html/no-dupe-style-properties`

Interdire les propriétés en double dans les attributs de style en ligne.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Chaque style statique répète une propriété ; `margin` et `MARGIN` sont également considérés comme la même propriété.

```vue annotate="remove:2,3"
<template>
  <div style="color: red; color: blue">text</div>
  <div style="margin: 0; MARGIN: 1px">text</div>
</template>
```

## Bon

Le style statique utilise des propriétés distinctes de couleur et d’arrière-plan. Les liaisons de style dynamiques sont hors du champ de ce contrôle des attributs statiques.

```vue annotate="add:2,3"
<template>
  <div style="color: red; background: blue">text</div>
  <div :style="{ color: a, color: b }">text</div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/no-dupe-style-properties": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles HTML](../html.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_dupe_style_properties.rs#L39)
