---
title: "vue/no-inline-style"
description: "Déconseiller l’utilisation d’attributs de style en ligne."
---

# `vue/no-inline-style`

Déconseiller l’utilisation d’attributs de style en ligne.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

L’attribut statique style place la déclaration de couleur dans l’élément.

```vue annotate="remove:2"
<template>
  <div style="color: red">Text</div>
</template>
```

## Bon

Les classes expriment la couleur fixe ; la largeur dépendant de ratio reste une liaison de style dynamique, hors du contrôle des attributs statiques.

```vue annotate="add:2,3,4"
<template>
  <div class="text-red">Text</div>
  <span :class="{ 'text-red': isRed }">Text</span>
  <div :style="{ width: `${ratio}%` }">Text</div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-inline-style": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_inline_style.rs#L33)
