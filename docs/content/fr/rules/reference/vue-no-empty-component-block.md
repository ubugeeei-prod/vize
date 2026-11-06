---
title: "vue/no-empty-component-block"
description: "Interdire les blocs SFC vides."
---

# `vue/no-empty-component-block`

Interdire les blocs SFC vides.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Les blocs template, script et style ne contiennent aucun contenu significatif.

```vue annotate="remove:1,3,5"
<template></template>

<script></script>

<style>
</style>
```

## Bon

Chaque bloc conservé contient du balisage, des déclarations de script ou des déclarations de style effectifs.

```vue annotate="add:1,2,3,5,6,7,9,10"
<template>
  <div>Hello</div>
</template>

<script setup>
const message = "Hello";
</script>

<style scoped>
.button { color: red; }
</style>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-empty-component-block": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_empty_component_block.rs#L42)
