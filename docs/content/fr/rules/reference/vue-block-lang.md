---
title: "vue/block-lang"
description: "Limite le lang d'un bloc SFC à la langue configurée."
---

# `vue/block-lang`

Limite le lang d'un bloc SFC à la langue configurée.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le bloc script omet l'attribut lang configuré.

```vue annotate="remove:1"
<script setup>
const label = "a";
</script>
<template>
  <p>{{ label }}</p>
</template>
```

## Bon

Le bloc script définit lang sur la langue configurée.

```vue annotate="add:1"
<script setup lang="ts">
const label = "a";
</script>
<template>
  <p>{{ label }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/block-lang": "warn"
    },
    "ruleOptions": {
      "vue/block-lang": {
        "script": {
          "lang": "ts",
          "allowNoLang": false
        }
      }
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/block_lang.rs#L12)
