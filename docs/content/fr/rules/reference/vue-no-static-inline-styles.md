---
title: "vue/no-static-inline-styles"
description: "Interdire les attributs de style en ligne statiques."
---

# `vue/no-static-inline-styles`

Interdire les attributs de style en ligne statiques.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le paragraphe porte la déclaration de couleur constante dans son attribut style.

```vue annotate="remove:1,2,3"
<template>
<p style="color: red">Notice</p>
</template>
```

## Bon

Une classe notice et une feuille de style scoped définissent la couleur constante en dehors de l’attribut du template.

```vue annotate="add:1,2"
<template><p class="notice">Notice</p></template>
<style scoped>.notice { color: red; }</style>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-static-inline-styles": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_static_inline_styles.rs#L15)
