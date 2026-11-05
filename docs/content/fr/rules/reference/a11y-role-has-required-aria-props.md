---
title: "a11y/role-has-required-aria-props"
description: "Exiger les propriétés obligatoires des rôles ARIA."
---

# `a11y/role-has-required-aria-props`

Exiger les propriétés obligatoires des rôles ARIA.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le rôle checkbox omet `aria-checked`, qui communique l’état de la case à cocher.

```vue annotate="remove:2"
<template>
  <span role="checkbox">Receive updates</span>
</template>
```

## Bon

`aria-checked="false"` fournit l’état exigé par le rôle checkbox.

```vue annotate="add:2"
<template>
  <span role="checkbox" aria-checked="false">Receive updates</span>
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
        "a11y/role-has-required-aria-props": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_has_required_aria_props.rs#L37)
