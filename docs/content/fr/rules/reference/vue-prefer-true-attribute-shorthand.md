---
title: "vue/prefer-true-attribute-shorthand"
description: "Préférer la syntaxe abrégée pour un attribut booléen lié à `true`."
---

# `vue/prefer-true-attribute-shorthand`

Préférer la syntaxe abrégée pour un attribut booléen lié à `true`.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Un attribut booléen natif disabled est lié à la valeur constante true.

```vue annotate="remove:2"
<template>
<input :disabled="true" />
</template>
```

## Bon

L’attribut natif utilise sa syntaxe booléenne abrégée. Les liaisons à false et les props de composants conservent leurs valeurs explicites.

```vue annotate="add:2,3,4,5"
<template>
<input disabled />
<input :disabled="false" />
<MyComponent :visible="true" />
<MyComponent :visible="isVisible" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-true-attribute-shorthand": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_true_attribute_shorthand.rs#L38)
