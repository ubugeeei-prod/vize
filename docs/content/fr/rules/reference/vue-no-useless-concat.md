---
title: "vue/no-useless-concat"
description: "Interdit la concaténation inutile de chaînes littérales."
---

# `vue/no-useless-concat`

Interdit la concaténation inutile de chaînes littérales.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Deux chaînes littérales sont concaténées à l'exécution.

```vue annotate="remove:2"
<template>
  <p>{{ "hello" + " world" }}</p>
</template>
```

## Bon

Une seule chaîne représente le même texte.

```vue annotate="add:2"
<template>
  <p>{{ "hello world" }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-useless-concat": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template.rs#L153)
