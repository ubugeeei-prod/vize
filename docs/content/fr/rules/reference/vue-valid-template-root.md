---
title: "vue/valid-template-root"
description: "Exiger une racine `<template>` valide selon la sémantique des fragments de Vue 3."
---

# `vue/valid-template-root`

Exiger une racine `<template>` valide selon la sémantique des fragments de Vue 3.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Un `<template>` imbriqué ordinaire occupe la racine du template sans directive lui donnant un rôle de rendu.

```vue annotate="remove:2"
<template>
  <template>content</template>
</template>
```

## Bon

Le `<div>` est un élément racine qui peut être affiché. Cet exemple n’impose pas de restriction universelle à une seule racine pour les fragments de Vue 3.

```vue annotate="add:2"
<template>
  <div>content</div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/valid-template-root": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_template_root.rs#L82)
