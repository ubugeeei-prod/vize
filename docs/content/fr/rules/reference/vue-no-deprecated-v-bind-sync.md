---
title: "vue/no-deprecated-v-bind-sync"
description: "Interdire le modificateur obsolète `.sync` sur `v-bind`."
---

# `vue/no-deprecated-v-bind-sync`

Interdire le modificateur obsolète `.sync` sur `v-bind`.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Les liaisons utilisent le modificateur .sync supprimé, y compris en combinaison avec .camel.

```vue annotate="remove:2,3,4"
<template>
<MyComponent :title.sync="title" />
<MyComponent v-bind:title.sync="title" />
<MyComponent :title.sync.camel="title" />
</template>
```

## Bon

Utilisez une liaison title ordinaire à sens unique, ou v-model:title lorsqu’un canal de mise à jour est nécessaire.

```vue annotate="add:2,3"
<template>
<MyComponent :title="title" />
<MyComponent v-model:title="title" />
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
        "vue/no-deprecated-v-bind-sync": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_bind_sync.rs#L42)
