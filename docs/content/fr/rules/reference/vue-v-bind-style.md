---
title: "vue/v-bind-style"
description: "Imposer un style de directive `v-bind`."
---

# `vue/v-bind-style`

Imposer un style de directive `v-bind`.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Oui | [Options des règles](/rules/options.md) |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

`v-bind:class` utilise la forme longue alors que le style de liaison configuré exige la syntaxe abrégée avec deux-points.

```vue annotate="remove:2"
<template>
  <div v-bind:class="panelClass"></div>
</template>
```

## Bon

`:class` conserve la même expression avec la syntaxe abrégée exigée ; cette règle concerne la graphie, pas le type de la valeur.

```vue annotate="add:2"
<template>
  <div :class="panelClass"></div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/v-bind-style": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L28)
