---
title: "vue/attribute-hyphenation"
description: "Imposer un style de nommage des attributs sur les composants personnalisés."
---

# `vue/attribute-hyphenation`

Imposer un style de nommage des attributs sur les composants personnalisés.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Oui | [Options des règles](/rules/options.md) |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

L’attribut du composant utilise la graphie camelCase firstName.

```vue annotate="remove:2"
<template>
<UserCard firstName="Ada" />
</template>
```

## Bon

La graphie first-name respecte la convention configurée qui sépare les mots des attributs de composants par des tirets.

```vue annotate="add:2"
<template>
<UserCard first-name="Ada" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/attribute-hyphenation": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L35)
