---
title: "a11y/no-access-key"
description: "Interdire l’utilisation de l’attribut accesskey."
---

# `a11y/no-access-key`

Interdire l’utilisation de l’attribut accesskey.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le raccourci `accesskey="s"` peut entrer en conflit avec les raccourcis du navigateur ou des technologies d’assistance.

```vue annotate="remove:2"
<template>
  <button accesskey="s">Save</button>
</template>
```

## Bon

Supprimer `accesskey` conserve le bouton Save ordinaire.

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/no-access-key": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_access_key.rs#L19)
