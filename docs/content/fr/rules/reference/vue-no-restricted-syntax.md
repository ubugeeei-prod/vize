---
title: "vue/no-restricted-syntax"
description: "Interdit la syntaxe configurée dans les expressions du template."
---

# `vue/no-restricted-syntax`

Interdit la syntaxe configurée dans les expressions du template.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | [Options des règles](/rules/options.md) |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le `CallExpression` configuré apparaît dans le template.

```vue annotate="remove:2"
<template>
  <p>{{ format(value) }}</p>
</template>
```

## Bon

Le template lit une liaison préparée sans appeler de fonction.

```vue annotate="add:2"
<template>
  <p>{{ formattedValue }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-syntax": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-syntax": {
        "selectors": [
          "CallExpression"
        ]
      }
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/restricted.rs#L20)
