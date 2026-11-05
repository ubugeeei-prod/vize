---
title: "a11y/no-autofocus"
description: "Interdire l’utilisation de l’attribut autofocus."
---

# `a11y/no-autofocus`

Interdire l’utilisation de l’attribut autofocus.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le champ demande à recevoir automatiquement le focus lorsqu’il apparaît.

```vue annotate="remove:2"
<template>
  <input autofocus name="query" />
</template>
```

## Bon

Supprimer `autofocus` évite cette demande de focus automatique tout en conservant le champ de requête.

```vue annotate="add:2"
<template>
  <input name="query" />
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
        "a11y/no-autofocus": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_autofocus.rs#L21)
