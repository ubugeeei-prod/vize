---
title: "script/no-deprecated-dollar-listeners-api"
description: "Interdire la propriété d’instance $listeners supprimée dans Vue 3 (fusionnée dans $attrs)."
---

# `script/no-deprecated-dollar-listeners-api`

Interdire la propriété d’instance $listeners supprimée dans Vue 3 (fusionnée dans $attrs).

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

Les accès aux membres et la référence passée directement en argument utilisent tous `$listeners`, que Vue 3 a supprimé après avoir fusionné les écouteurs dans les attributs.

```vue annotate="remove:4,5,9"
<script>
export default {
  mounted() {
    const vm = this
    return vm.$listeners
  }
}
</script>
<template><Child v-on="$listeners" /></template>
```

## Bon

Les accès passent à `this.$attrs` et à `ctx.attrs` dans le contexte setup. Ils remplacent l’API supprimée des écouteurs ; les objets récepteurs illustrés doivent exister dans le contexte environnant du composant.

```vue annotate="add:4,8"
<script>
export default {
  mounted() {
    return this.$attrs
  }
}
</script>
<template><Child v-bind="$attrs" /></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-deprecated-dollar-listeners-api": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_listeners_api.rs#L54)
