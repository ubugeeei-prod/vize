---
title: "vue/no-restricted-v-bind"
description: "Interdit les arguments v-bind restreints."
---

# `vue/no-restricted-v-bind`

Interdit les arguments v-bind restreints.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

`:v-focus` lie un argument de directive comme attribut.

```vue annotate="remove:2"
<template>
  <input :v-focus="enabled" />
</template>
```

## Bon

`v-focus` utilise directement la syntaxe de directive.

```vue annotate="add:2"
<template>
  <input v-focus="enabled" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-v-bind": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/template_rules.rs#L69)
