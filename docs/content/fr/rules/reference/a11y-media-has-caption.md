---
title: "a11y/media-has-caption"
description: "Exiger des sous-titres pour les éléments multimédias."
---

# `a11y/media-has-caption`

Exiger des sous-titres pour les éléments multimédias.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

La vidéo possède des contrôles de lecture, mais aucune piste de sous-titres.

```vue annotate="remove:2"
<template>
  <video src="/demo.mp4" controls />
</template>
```

## Bon

Un `track` avec `kind="captions"` fournit les sous-titres anglais de la même vidéo.

```vue annotate="add:2,3,4"
<template>
  <video src="/demo.mp4" controls>
    <track kind="captions" src="/demo.en.vtt" srclang="en" label="English" />
  </video>
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
        "a11y/media-has-caption": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/media_has_caption.rs#L32)
