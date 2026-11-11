---
title: "vue/no-restricted-block"
description: "Interdit le bloc configuré."
---

# `vue/no-restricted-block`

Interdit le bloc configuré.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le SFC contient le bloc i18n interdit.

```vue annotate="remove:4"
<template>
  <p>ok</p>
</template>
<i18n lang="json">{}</i18n>
```

## Bon

Le SFC ne garde que le bloc template.

```vue
<template>
  <p>ok</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-block": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-block": [
        "i18n"
      ]
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/blocks.rs#L14)
