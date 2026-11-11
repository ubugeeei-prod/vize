---
title: "vue/valid-v-memo"
description: "Exiger des directives `v-memo` valides."
---

# `vue/valid-v-memo`

Exiger des directives `v-memo` valides.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Un `v-memo` seul ne donne à Vue aucune expression de dépendances pour décider quand réutiliser le sous-arbre.

```vue annotate="remove:2"
<template>
  <div v-memo></div>
</template>
```

## Bon

`v-memo="[valueA, valueB]"` fournit le tableau de dépendances utilisé pour la mémoïsation.

```vue annotate="add:2"
<template>
  <div v-memo="[valueA, valueB]">{{ label }}</div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-memo": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_memo.rs#L32)
