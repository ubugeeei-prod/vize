---
title: "vue/no-lone-template"
description: "Interdire les éléments `<template>` inutiles."
---

# `vue/no-lone-template`

Interdire les éléments `<template>` inutiles.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le template interne n’a ni directive ni rôle de slot lui donnant une fonction structurelle.

```vue annotate="remove:2"
<template>
<div><template><p>Details</p></template></div>
</template>
```

## Bon

Supprimer l’enveloppe inutile laisse le paragraphe directement à l’intérieur de div.

```vue annotate="add:2"
<template>
<div><p>Details</p></div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-lone-template": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_lone_template.rs#L32)
