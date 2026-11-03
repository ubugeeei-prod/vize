---
title: "vue/valid-v-else-if"
description: "Exiger des directives `v-else-if` valides."
---

# `vue/valid-v-else-if`

Exiger des directives `v-else-if` valides.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Les exemples laissent `v-else-if` sans branche conditionnelle précédente ou sans condition, ou lui donnent un argument ou un modificateur.

```vue annotate="remove:2,3,4,5"
<template>
<div v-else-if="foo"></div> <!-- without preceding v-if -->
<div v-if="foo"></div><div v-else-if></div>
<div v-if="foo"></div><div v-else-if:arg="bar"></div>
<div v-if="foo"></div><div v-else-if.mod="bar"></div>
</template>
```

## Bon

Placez `v-else-if` avec une condition juste après la branche `v-if` ou `v-else-if` qu’il prolonge.

```vue annotate="add:2,3"
<template>
<div v-if="foo"></div>
<div v-else-if="bar"></div>
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
        "vue/valid-v-else-if": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else_if.rs#L34)
