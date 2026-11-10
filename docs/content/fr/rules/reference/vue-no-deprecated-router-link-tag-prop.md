---
title: "vue/no-deprecated-router-link-tag-prop"
description: "Interdire la prop `tag` sur &lt;router-link&gt;."
---

# `vue/no-deprecated-router-link-tag-prop`

Interdire la prop `tag` sur &lt;router-link&gt;.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

RouterLink utilise la prop tag supprimée pour demander un élément button.

```vue annotate="remove:2"
<template>
  <router-link to="/home" tag="button">Home</router-link>
</template>
```

## Bon

Le slot fournit navigate à un bouton explicitement écrit dans le template.

```vue annotate="add:2,3,4"
<template>
  <router-link to="/home" v-slot="{ navigate }">
    <button @click="navigate">Home</button>
  </router-link>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-router-link-tag-prop": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_router_link_tag_prop.rs#L38)
