---
title: "vue/no-template-key"
description: "Interdire l’attribut `key` sur `<template>`."
---

# `vue/no-template-key`

Interdire l’attribut `key` sur `<template>`.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Une enveloppe template sans boucle possède une key, alors qu’elle ne constitue pas la limite d’une itération à clé.

```vue annotate="remove:2"
<template>
<template :key="section"><div>Details</div></template>
</template>
```

## Bon

La clé appartient à une itération template v-for, où elle identifie chaque fragment répété.

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><div>{{ item.name }}</div></template>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-template-key": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_key.rs#L29)
