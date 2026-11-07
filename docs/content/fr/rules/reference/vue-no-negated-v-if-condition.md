---
title: "vue/no-negated-v-if-condition"
description: "Interdire une condition v-if négative lorsque la chaîne comporte un v-else."
---

# `vue/no-negated-v-if-condition`

Interdire une condition v-if négative lorsque la chaîne comporte un v-else.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Les branches associées v-if et v-else commencent par une condition négative.

```vue
<template>
<div v-if="!ok">A</div>
<div v-else>B</div>
</template>
```

## Bon

Une condition ok positive vient en premier ; lors de l’inversion d’une condition, placez d’abord la branche opposée d’origine. Un v-if négatif isolé et les comparaisons !== restent autorisés.

```vue annotate="add:2,3,4,6,7"
<template>
<div v-if="ok">B</div>
<div v-else>A</div>

<div v-if="!ok">A</div>

<div v-if="a === b">A</div>
<div v-else>B</div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-negated-v-if-condition": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_negated_v_if_condition.rs#L41)
