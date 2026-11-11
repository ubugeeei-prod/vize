---
title: "a11y/no-static-element-interactions"
description: "Interdire les gestionnaires d’événements sur les éléments statiques."
---

# `a11y/no-static-element-interactions`

Interdire les gestionnaires d’événements sur les éléments statiques.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Contrairement à eslint-plugin-vuejs-accessibility, un `role` lié dont une branche conditionnelle est interactive, comme `:role="clickable ? 'button' : undefined"`, n’est pas statique, et `:aria-hidden="false"` ne masque pas l’élément.

## Mauvais

Une section statique reçoit une action liée à la touche Entrée sans posséder de rôle interactif.

```vue annotate="remove:2"
<template>
  <section @keydown.enter="select">Select</section>
</template>
```

## Bon

Un bouton natif porte la même action dans un élément interactif approprié.

```vue annotate="add:2"
<template>
  <button type="button" @keydown.enter="select">Select</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/no-static-element-interactions": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_static_element_interactions.rs#L39)
