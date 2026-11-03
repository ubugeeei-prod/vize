---
title: "a11y/no-redundant-roles"
description: "Interdire les rôles ARIA redondants."
---

# `a11y/no-redundant-roles`

Interdire les rôles ARIA redondants.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Oui | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le bouton natif possède déjà le rôle button ; `role="button"` répète donc sa sémantique implicite.

```vue annotate="remove:2"
<template>
  <button role="button">Save</button>
</template>
```

## Bon

Supprimer le rôle répété conserve la sémantique du bouton fournie par HTML.

```vue annotate="add:2"
<template>
  <button>Save</button>
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
        "a11y/no-redundant-roles": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_redundant_roles/report.rs#L31)
