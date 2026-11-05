---
title: "a11y/img-alt"
description: "Exiger un attribut alt sur les images pour les rendre accessibles."
---

# `a11y/img-alt`

Exiger un attribut alt sur les images pour les rendre accessibles.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

L’image de l’avatar ne possède pas d’attribut `alt`.

```vue annotate="remove:2"
<template>
  <img src="/avatar.png" />
</template>
```

## Bon

`alt="User avatar"` fournit une alternative textuelle à l’avatar.

```vue annotate="add:2"
<template>
  <img src="/avatar.png" alt="User avatar" />
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
        "a11y/img-alt": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/img_alt.rs#L17)
