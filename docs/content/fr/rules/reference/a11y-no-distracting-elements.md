---
title: "a11y/no-distracting-elements"
description: "Interdire les éléments distrayants tels que &lt;marquee&gt; et &lt;blink&gt;."
---

# `a11y/no-distracting-elements`

Interdire les éléments distrayants tels que &lt;marquee&gt; et &lt;blink&gt;.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

L’élément `marquee` introduit un texte en mouvement automatique.

```vue annotate="remove:2"
<template>
  <marquee>Limited offer</marquee>
</template>
```

## Bon

Un paragraphe affiche la même offre sans l’élément marquee distrayant.

```vue annotate="add:2"
<template>
  <p>Limited offer</p>
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
        "a11y/no-distracting-elements": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_distracting_elements.rs#L16)
