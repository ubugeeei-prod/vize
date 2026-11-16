---
title: "vue/valid-v-pre"
description: "Exiger des directives `v-pre` valides."
---

# `vue/valid-v-pre`

Exiger des directives `v-pre` valides.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

`v-pre` possède un argument, un modificateur ou une valeur, alors que cette directive est un marqueur sans valeur qui ignore la compilation.

```vue annotate="remove:2,3,4"
<template>
<div v-pre:arg></div>
<div v-pre.mod></div>
<div v-pre="foo"></div>
</template>
```

## Bon

`v-pre` seul laisse le sous-arbre non compilé ; ses moustaches s’affichent telles qu’elles sont écrites.

```vue annotate="add:2"
<template>
<div v-pre>{{ raw }}</div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-pre": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_pre.rs#L32)
