---
title: "type/no-for-in-array"
description: "Interdire for-in sur une valeur de type tableau."
---

# `type/no-for-in-array`

Interdire for-in sur une valeur de type tableau.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

for-in parcourt les clés d’un tableau au lieu des valeurs des éléments.

```vue annotate="remove:4,5"
<script setup lang="ts">
const numbers = [1, 2, 3];
function loopsKeys(): void {
  for (const key in numbers) {
    console.log(key);
  }
}
</script>
```

## Bon

for-of parcourt les éléments numériques du tableau.

```vue annotate="add:4,5"
<script setup lang="ts">
const numbers = [1, 2, 3];
function loopsKeys(): void {
  for (const n of numbers) {
    console.log(n);
  }
}
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-for-in-array": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_for_in_array.rs#L5)
