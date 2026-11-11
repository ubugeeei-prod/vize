---
title: "type/no-array-delete"
description: "Interdire delete sur un élément d’une valeur de type tableau."
---

# `type/no-array-delete`

Interdire delete sur un élément d’une valeur de type tableau.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

delete numbers[0] retire un élément et laisse un trou au lieu de réindexer le tableau.

```vue annotate="remove:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
delete numbers[0];
</script>
```

## Bon

splice(0, 1) retire le premier élément et garde le tableau dense.

```vue annotate="add:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
numbers.splice(0, 1);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-array-delete": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_array_delete.rs#L5)
