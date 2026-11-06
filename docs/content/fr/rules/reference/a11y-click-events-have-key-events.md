---
title: "a11y/click-events-have-key-events"
description: "Exiger des gestionnaires de clavier avec les événements de clic."
---

# `a11y/click-events-have-key-events`

Exiger des gestionnaires de clavier avec les événements de clic.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Contrôle les éléments non interactifs sans rôle interactif. Les boutons natifs et les éléments possédant un rôle ARIA interactif sont hors du champ du diagnostic de cette règle.

## Mauvais

Le `div` non interactif possède un gestionnaire de clic, mais aucune gestion des événements de clavier.

```vue annotate="remove:2"
<template>
<div @click="activate">Activate</div>
</template>
```

## Bon

Un `button` natif permet une activation au clavier avec le même gestionnaire `activate`.

```vue annotate="add:2"
<template>
<button @click="activate">Activate</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/click-events-have-key-events": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/click_events_have_key_events.rs#L22)
