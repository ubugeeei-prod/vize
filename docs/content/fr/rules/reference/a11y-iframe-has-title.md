---
title: "a11y/iframe-has-title"
description: "Exiger un attribut title sur les éléments iframe."
---

# `a11y/iframe-has-title`

Exiger un attribut title sur les éléments iframe.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le cadre de paiement possède une URL source, mais aucun `title` décrivant le contenu intégré.

```vue annotate="remove:2"
<template>
  <iframe src="/checkout"></iframe>
</template>
```

## Bon

`title="Checkout preview"` nomme le contenu de ce cadre.

```vue annotate="add:2"
<template>
  <iframe src="/checkout" title="Checkout preview"></iframe>
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
        "a11y/iframe-has-title": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/iframe_has_title.rs#L15)
