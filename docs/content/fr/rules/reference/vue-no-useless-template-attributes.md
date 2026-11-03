---
title: "vue/no-useless-template-attributes"
description: "Interdire les attributs inutiles sur les éléments `<template>`."
---

# `vue/no-useless-template-attributes`

Interdire les attributs inutiles sur les éléments `<template>`.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le template conditionnel possède une classe, mais cette enveloppe structurelle ne produit pas d’élément DOM qui puisse la recevoir.

```vue annotate="remove:2"
<template>
<section><template v-if="ready" class="notice"><p>Ready</p></template></section>
</template>
```

## Bon

La classe est déplacée vers le paragraphe qui est effectivement affiché, tandis que v-if reste sur le template structurel.

```vue annotate="add:2"
<template>
<section><template v-if="ready"><p class="notice">Ready</p></template></section>
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
        "vue/no-useless-template-attributes": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_useless_template_attributes.rs#L32)
