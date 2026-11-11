---
title: "vue/no-restricted-custom-event"
description: "Interdit l'événement personnalisé configuré."
---

# `vue/no-restricted-custom-event`

Interdit l'événement personnalisé configuré.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le template émet l'événement interdit legacyEvent.

```vue annotate="remove:2"
<template>
  <button type="button" @click="$emit('legacyEvent')">x</button>
</template>
```

## Bon

Le template émet l'événement save.

```vue annotate="add:2"
<template>
  <button type="button" @click="$emit('save')">x</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-custom-event": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-custom-event": [
        "legacyEvent"
      ]
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_custom_event.rs#L19)
