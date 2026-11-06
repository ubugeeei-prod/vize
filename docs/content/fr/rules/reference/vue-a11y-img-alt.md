---
title: "vue/a11y-img-alt"
description: "Exiger un attribut alt sur les images pour les rendre accessibles."
---

# `vue/a11y-img-alt`

Exiger un attribut alt sur les images pour les rendre accessibles.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Ni l’image statique ni l’image dont la source est dynamique ne fournit d’attribut alt.

```vue annotate="remove:2,3"
<template>
<img src="/photo.jpg" />
<img :src="photo" />
</template>
```

## Bon

Les images informatives reçoivent un texte alt descriptif, les images décoratives un alt vide, et l’image dynamique lie sa description.

```vue annotate="add:2,3,4,5,6,7,8,9"
<template>
<!-- Informative image -->
<img src="/photo.jpg" alt="Team photo from company retreat" />

<!-- Decorative image (empty alt) -->
<img src="/decoration.svg" alt="" />

<!-- Dynamic alt -->
<img :src="photo" :alt="photoDescription" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/a11y-img-alt": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/a11y_img_alt.rs#L33)
