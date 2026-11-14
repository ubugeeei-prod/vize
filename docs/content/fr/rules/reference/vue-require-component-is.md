---
title: "vue/require-component-is"
description: "Exiger `v-bind:is` sur les éléments `<component>`."
---

# `vue/require-component-is`

Exiger `v-bind:is` sur les éléments `<component>`.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le `<component>` dynamique n’a pas de cible `is` ; Vue ne peut donc pas choisir le composant à afficher.

```vue annotate="remove:2"
<template>
  <component />
</template>
```

## Bon

`:is="currentComponent"` fournit la sélection du composant ; la liaison peut changer à l’exécution.

```vue annotate="add:2"
<template>
  <component :is="currentComponent" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-component-is": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_component_is.rs#L28)
