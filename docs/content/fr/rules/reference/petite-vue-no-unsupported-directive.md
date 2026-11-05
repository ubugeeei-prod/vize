---
title: "petite-vue/no-unsupported-directive"
description: "Interdire les directives non prises en charge par petite-vue."
---

# `petite-vue/no-unsupported-directive`

Interdire les directives non prises en charge par petite-vue.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | Aucune | Non | Aucune |

Champ d’application : Documents HTML détectés comme petite-vue ; les SFC Vue ordinaires sont hors du champ de cette règle.

## Mauvais

`v-memo`, `v-slot:header` et la directive personnalisée `v-my-directive` sont absents de la liste des directives prises en charge par petite-vue. Le script petite-vue désigne ce HTML comme relevant de ce dialecte.

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-memo="[a, b]"></div>
<template v-slot:header></template>
<div v-my-directive></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## Bon

Le remplacement utilise les syntaxes prises en charge `v-scope`, `v-effect`, `v-if`, `v-bind` et `v-on` au lieu de dépendre de directives non prises en charge.

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-scope="{ count: 0 }" v-effect="console.log(count)"></div>
<div v-if="ok" v-bind:title="title" @click="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "petite-vue/no-unsupported-directive": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles petite-vue](../petite-vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/no_unsupported_directive.rs#L43)
