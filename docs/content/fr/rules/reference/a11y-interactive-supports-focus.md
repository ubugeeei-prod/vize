---
title: "a11y/interactive-supports-focus"
description: "Exiger que les éléments possédant un rôle interactif puissent recevoir le focus."
---

# `a11y/interactive-supports-focus`

Exiger que les éléments possédant un rôle interactif puissent recevoir le focus.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Contrairement à eslint-plugin-vuejs-accessibility, une listbox qui contient une option focalisable (roving tabindex) ou qu’une combobox contrôle par `aria-activedescendant` n’a pas besoin de son propre arrêt de tabulation, un `tabindex` lié dont toutes les branches conditionnelles sont des entiers (`:tabindex="active ? 0 : -1"`) compte comme tabindex, et `:aria-hidden="false"` ne masque pas l’élément.

## Mauvais

Donner à un `span` le rôle button et un gestionnaire de clic ne rend pas l’élément accessible au focus clavier.

```vue annotate="remove:2"
<template>
  <span role="button" @click="open">Open</span>
</template>
```

## Bon

Le bouton natif peut recevoir le focus et conserve la même action `open`.

```vue annotate="add:2"
<template>
  <button type="button" @click="open">Open</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/interactive-supports-focus": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs#L46)
