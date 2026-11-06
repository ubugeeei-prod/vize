---
title: "vue/v-on-event-hyphenation"
description: "Imposer des tirets dans les noms d’événements personnalisés de v-on sur les composants."
---

# `vue/v-on-event-hyphenation`

Imposer des tirets dans les noms d’événements personnalisés de v-on sur les composants.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Oui | [Options des règles](/rules/options.md) |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

L’écouteur du composant personnalisé utilise `@myEvent` au lieu d’un nom d’événement séparé par des tirets.

```vue annotate="remove:2,3,4"
<template>
<MyComponent @myEvent="handler" />
<MyComponent v-on:myEvent="handler" />
<my-component @update:modelValue="handler" />
</template>
```

## Bon

`@my-event` utilise la graphie exigée pour les événements personnalisés. Les écouteurs d’éléments natifs et les arguments d’événements dynamiques montrés ci-dessous sont hors du champ de ce contrôle.

```vue annotate="add:2,3,4"
<template>
<MyComponent @my-event="handler" />
<div @myEvent="handler" />
<MyComponent @[dynamicEvent]="handler" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/v-on-event-hyphenation": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_event_hyphenation.rs#L40)
