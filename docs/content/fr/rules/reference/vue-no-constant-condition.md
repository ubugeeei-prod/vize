---
title: "vue/no-constant-condition"
description: "Interdit les conditions constantes dans le template."
---

# `vue/no-constant-condition`

Interdit les conditions constantes dans le template.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

La condition ternaire est toujours vraie.

```vue annotate="remove:2"
<template>
  <p>{{ true ? primary : fallback }}</p>
</template>
```

## Bon

La branche dépend de la valeur réactive `ready`.

```vue annotate="add:2"
<template>
  <p>{{ ready ? primary : fallback }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-constant-condition": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_constant_condition.rs#L12)
