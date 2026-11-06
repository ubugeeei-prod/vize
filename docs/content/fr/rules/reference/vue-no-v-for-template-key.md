---
title: "vue/no-v-for-template-key"
description: "Exige de placer la key de `template v-for` sur un enfant réel pour assurer la compatibilité avec Vue 2."
---

# `vue/no-v-for-template-key`

Exige de placer la key de `template v-for` sur un enfant réel pour assurer la compatibilité avec Vue 2.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Cette règle de compatibilité avec Vue 2 doit être activée explicitement. Vue 2 attend la key sur un enfant réel ; Vue 3 l'attend sur `<template v-for>`. Pour la convention Vue 3, utilisez `vue/no-v-for-template-key-on-child` ; n'activez pas les deux conventions ensemble.

## Mauvais

Dans le contrat de Vue 2, la key sur `<template v-for>` est ignorée, car le template ne produit pas d'élément réel.

```vue annotate="remove:1"
<template><ul><template v-for="item in items" :key="item.id"><li>{{ item.label }}</li></template></ul></template>
```

## Bon

Placez la key de l'élément sur l'enfant réel `li` répété, conformément à la convention de Vue 2 vérifiée ici.

```vue annotate="add:1"
<template><ul><template v-for="item in items"><li :key="item.id">{{ item.label }}</li></template></ul></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-v-for-template-key": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_for_template_key.rs#L33)
