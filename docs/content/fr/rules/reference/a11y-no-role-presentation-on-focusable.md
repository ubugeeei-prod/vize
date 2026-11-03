---
title: "a11y/no-role-presentation-on-focusable"
description: "Interdire role=\"presentation\" ou role=\"none\" sur les éléments pouvant recevoir le focus."
---

# `a11y/no-role-presentation-on-focusable`

Interdire role="presentation" ou role="none" sur les éléments pouvant recevoir le focus.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le lien de facturation pouvant recevoir le focus demande role=presentation, ce qui entre en conflit avec son rôle de lien interactif ; les navigateurs doivent ignorer cette demande de présentation.

```vue annotate="remove:2"
<template>
  <a href="/billing" role="presentation">Billing</a>
</template>
```

## Bon

Supprimez la demande de présentation contradictoire et utilisez le rôle de lien natif ainsi que la destination de facturation.

```vue annotate="add:2"
<template>
  <a href="/billing">Billing</a>
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
        "a11y/no-role-presentation-on-focusable": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_role_presentation_on_focusable.rs#L19)
