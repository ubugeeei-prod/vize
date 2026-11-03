---
title: "vue/no-deprecated-slot-scope-attribute"
description: "Interdire l’attribut obsolète `slot-scope`."
---

# `vue/no-deprecated-slot-scope-attribute`

Interdire l’attribut obsolète `slot-scope`.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le template reçoit les props du slot au moyen de l’attribut obsolète slot-scope.

```vue annotate="remove:2"
<template>
<Card><template slot-scope="props">{{ props.name }}</template></Card>
</template>
```

## Bon

La directive #default reçoit ces props sans slot-scope.

```vue annotate="add:2"
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
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
        "vue/no-deprecated-slot-scope-attribute": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_scope_attribute.rs#L33)
