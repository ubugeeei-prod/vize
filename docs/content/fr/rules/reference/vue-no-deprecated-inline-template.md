---
title: "vue/no-deprecated-inline-template"
description: "Interdire l’attribut obsolète `inline-template`."
---

# `vue/no-deprecated-inline-template`

Interdire l’attribut obsolète `inline-template`.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Card utilise l’attribut obsolète inline-template pour le contenu qui lui est fourni.

```vue annotate="remove:2"
<template>
<Card inline-template><p>Details</p></Card>
</template>
```

## Bon

Le même contenu est transmis normalement, sans l’attribut inline-template.

```vue annotate="add:2"
<template>
<Card><p>Details</p></Card>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-inline-template": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_inline_template.rs#L20)
