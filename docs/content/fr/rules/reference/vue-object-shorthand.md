---
title: "vue/object-shorthand"
description: "Exige la syntaxe courte des propriétés d'objet dans le template."
---

# `vue/object-shorthand`

Exige la syntaxe courte des propriétés d'objet dans le template.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

L'objet répète `value` comme clé et valeur.

```vue annotate="remove:2"
<template>
  <Widget :options="{ value: value }" />
</template>
```

## Bon

La syntaxe courte conserve la liaison sans répétition.

```vue annotate="add:2"
<template>
  <Widget :options="{ value }" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/object-shorthand": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/object_shorthand.rs#L10)
