---
title: "a11y/no-refer-to-non-existent-id"
description: "Interdire les références à des identifiants inexistants."
---

# `a11y/no-refer-to-non-existent-id`

Interdire les références à des identifiants inexistants.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

`aria-labelledby` pointe vers `save-label`, mais aucun élément ne déclare cet identifiant.

```vue
<template>
  <button aria-labelledby="save-label">Save</button>
</template>
```

## Bon

Ajouter le span correspondant résout la référence et fournit le libellé du bouton.

```vue annotate="add:2"
<template>
  <span id="save-label">Save changes</span>
  <button aria-labelledby="save-label">Save</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/no-refer-to-non-existent-id": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_refer_to_non_existent_id.rs#L38)
