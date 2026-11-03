---
title: "vue/valid-v-model"
description: "Exiger des directives `v-model` valides."
---

# `vue/valid-v-model`

Exiger des directives `v-model` valides.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Un `<div>` natif ne peut pas utiliser `v-model` comme contrôle de formulaire, et une directive sans valeur sur input n’a pas d’expression cible modifiable.

```vue annotate="remove:2,3"
<template>
  <div v-model="value"></div>
  <input v-model />
</template>
```

## Bon

Liez l’input, le select, le textarea ou le composant personnalisé aux variables modifiables présentées.

```vue annotate="add:2,3,4,5"
<template>
  <input v-model="value" />
  <select v-model="selected"></select>
  <textarea v-model="text"></textarea>
  <MyInput v-model="value" />
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
        "vue/valid-v-model": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_model.rs#L36)
