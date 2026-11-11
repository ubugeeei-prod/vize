---
title: "a11y/role-img-has-name"
description: "Exiger un nom accessible pour les éléments dont le rôle est « img »."
---

# `a11y/role-img-has-name`

Exiger un nom accessible pour les éléments dont le rôle est « img ».

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

L’image définit `role="img"` après un `alt` vide, elle est donc exposée comme une image sans nom accessible.

```vue annotate="remove:2"
<template>
  <img src="/a.png" alt="" role="img" />
</template>
```

## Bon

`alt="Network diagram"` nomme l’image, donc `role="img"` a un nom accessible.

```vue annotate="add:2"
<template>
  <img src="/a.png" alt="Network diagram" role="img" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/role-img-has-name": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_img_has_name.rs#L14)
