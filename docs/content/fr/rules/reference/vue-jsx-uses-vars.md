---
title: "vue/jsx-uses-vars"
description: "Considère comme utilisées les variables référencées dans les noms d'éléments JSX."
---

# `vue/jsx-uses-vars`

Considère comme utilisées les variables référencées dans les noms d'éléments JSX.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

Diagnostic du mauvais exemple : `vue/no-unused-components`

Cette règle auxiliaire n'émet pas de diagnostic `vue/jsx-uses-vars`. Elle reconnaît les liaisons des balises JSX lorsque `vue/no-unused-components` détermine si un composant est utilisé. Activez les deux règles pour cet exemple. `<MyButton.Part />` fait reconnaître la liaison racine `MyButton` ; les balises avec espace de noms et les membres de `this` ne fournissent pas de liaison de composant importé.

## Mauvais

`MyButton` est importé, mais ni le template ni le JSX ne le référencent ; la règle complémentaire `vue/no-unused-components` le signale donc.

```vue annotate="remove:3"
<script setup lang="jsx">
import MyButton from "./MyButton.vue";
const view = () => <div />;
</script>
<template><div /></template>
```

## Bon

La balise JSX référence le `MyButton` importé ; la règle auxiliaire reconnaît cet usage et la règle complémentaire ne signale plus de composant inutilisé.

```vue annotate="add:3"
<script setup lang="jsx">
import MyButton from "./MyButton.vue";
const view = () => <MyButton />;
</script>
<template><div /></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/jsx-uses-vars": "error",
        "vue/no-unused-components": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/jsx_uses_vars.rs#L35)
