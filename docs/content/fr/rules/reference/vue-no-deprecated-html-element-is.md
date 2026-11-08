---
title: "vue/no-deprecated-html-element-is"
description: "Interdire l’attribut `is` sur les éléments HTML natifs."
---

# `vue/no-deprecated-html-element-is`

Interdire l’attribut `is` sur les éléments HTML natifs.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Un div natif utilise l’ancien attribut is sans préfixe pour demander un composant Vue.

```vue annotate="remove:2"
<template>
  <div is="MyComponent" />
</template>
```

## Bon

Un composant dynamique utilise :is ; la syntaxe sur un élément natif utilise explicitement le préfixe vue:.

```vue annotate="add:2,3"
<template>
  <component :is="MyComponent" />
  <div is="vue:MyComponent" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-html-element-is": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_html_element_is.rs#L38)
