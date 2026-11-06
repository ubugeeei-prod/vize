---
title: "vue/no-invalid-html-attribute"
description: "Interdire les valeurs statiques invalides des attributs HTML."
---

# `vue/no-invalid-html-attribute`

Interdire les valeurs statiques invalides des attributs HTML.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le lien utilise stylesheet comme valeur de rel, alors que cette valeur appartient aux éléments link de feuilles de style.

```vue annotate="remove:2"
<template>
<a href="/guide" rel="stylesheet">Guide</a>
</template>
```

## Bon

Le lien utilise help, une valeur de rel adaptée à une ressource d’aide liée.

```vue annotate="add:2"
<template>
<a href="/guide" rel="help">Guide</a>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-invalid-html-attribute": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_invalid_html_attribute.rs#L12)
