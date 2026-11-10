---
title: "vue/no-console"
description: "Interdit l'accès à console dans les expressions du template."
---

# `vue/no-console`

Interdit l'accès à console dans les expressions du template.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

L'expression d'événement appelle `console.log`.

```vue annotate="remove:2"
<template>
  <button @click="console.log(value)">Save</button>
</template>
```

## Bon

Le template délègue le journal à une liaison du script.

```vue annotate="add:2"
<template>
  <button @click="logValue">Save</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-console": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_console.rs#L10)
