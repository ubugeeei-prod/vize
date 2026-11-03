---
title: "vue/require-toggle-inside-transition"
description: "Exiger un mécanisme de bascule sur l’élément enveloppé par `<transition>`."
---

# `vue/require-toggle-inside-transition`

Exiger un mécanisme de bascule sur l’élément enveloppé par `<transition>`.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

L’enfant statique à l’intérieur de `<Transition>` n’a ni visibilité conditionnelle ni sélection dynamique pouvant déclencher une entrée ou une sortie.

```vue annotate="remove:3"
<template>
<transition>
  <div>content</div>
</transition>
</template>
```

## Bon

`v-if="show"` modifie la présence de l’enfant et fournit à la transition une limite d’entrée et de sortie.

```vue annotate="add:3"
<template>
<transition>
  <div v-if="show">content</div>
</transition>
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
        "vue/require-toggle-inside-transition": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_toggle_inside_transition.rs#L49)
