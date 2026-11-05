---
title: "vue/no-preprocessor-lang"
description: "Déconseiller les préprocesseurs CSS au profit du CSS moderne."
---

# `vue/no-preprocessor-lang`

Déconseiller les préprocesseurs CSS au profit du CSS moderne.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le bloc style sélectionne SCSS avec lang. Cela décrit la convention souhaitée sans préprocesseur ; le traitement SFC actuel n’émet pas de diagnostic pour cette règle.

```vue annotate="remove:2"
<template><p>Notice</p></template>
<style lang="scss">
.notice { color: red; }
</style>
```

## Bon

Les mêmes déclarations CSS omettent le lang du préprocesseur. C’est une correction de la convention, et non une différence de diagnostics Mauvais/Bon exécutable actuellement.

```vue annotate="add:2"
<template><p>Notice</p></template>
<style>
.notice { color: red; }
</style>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-preprocessor-lang": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L11)
