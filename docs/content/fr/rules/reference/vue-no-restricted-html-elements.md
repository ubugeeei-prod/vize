---
title: "vue/no-restricted-html-elements"
description: "Interdit l'élément configuré."
---

# `vue/no-restricted-html-elements`

Interdit l'élément configuré.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le template utilise le marquee interdit.

```vue annotate="remove:2"
<template>
  <marquee>x</marquee>
</template>
```

## Bon

Le template utilise p.

```vue annotate="add:2"
<template>
  <p>x</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-html-elements": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-html-elements": [
        "marquee"
      ]
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_html.rs#L13)
