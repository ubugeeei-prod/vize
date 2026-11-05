---
title: "vue/no-deprecated-functional-template"
description: "Interdire l’attribut `functional` sur le `<template>` d’un SFC."
---

# `vue/no-deprecated-functional-template`

Interdire l’attribut `functional` sur le `<template>` d’un SFC.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le template du SFC possède l’attribut functional supprimé et lit l’ancien contexte props.

```vue annotate="remove:1,2"
<template functional>
  <div>{{ props.msg }}</div>
</template>
```

## Bon

Le template ordinaire omet functional et lit directement la liaison msg du composant.

```vue annotate="add:1,2"
<template>
  <div>{{ msg }}</div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-functional-template": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs#L57)
