---
title: "vue/prop-name-casing"
description: "Imposer une casse aux noms des props déclarées."
---

# `vue/prop-name-casing`

Imposer une casse aux noms des props déclarées.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Contrôle les noms des props déclarées, et non la casse des attributs transmis à un enfant.

## Mauvais

Le nom de prop déclaré user_name utilise des mots séparés par des traits de soulignement.

```vue annotate="remove:2,4"
<script setup lang="ts">
defineProps<{ user_name: string }>();
</script>
<template><p>{{ user_name }}</p></template>
```

## Bon

La déclaration et sa référence dans le template utilisent le nom camelCase userName.

```vue annotate="add:2,4"
<script setup lang="ts">
defineProps<{ userName: string }>();
</script>
<template><p>{{ userName }}</p></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prop-name-casing": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prop_name_casing.rs#L52)
