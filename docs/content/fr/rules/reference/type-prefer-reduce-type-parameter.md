---
title: "type/prefer-reduce-type-parameter"
description: "Préférer le paramètre de type de Array#reduce à une assertion de la valeur initiale."
---

# `type/prefer-reduce-type-parameter`

Préférer le paramètre de type de Array#reduce à une assertion de la valeur initiale.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

La valeur initiale est affirmée comme number[] au lieu d’utiliser le paramètre de type de reduce.

```vue annotate="remove:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
const collected = numbers.reduce((all, n) => [...all, n], [] as number[]);
</script>
```

## Bon

reduce<number[]> indique le type de l’accumulateur sans affirmer le tableau initial.

```vue annotate="add:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
const collected = numbers.reduce<number[]>((all, n) => [...all, n], []);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/prefer-reduce-type-parameter": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/prefer_reduce_type_parameter.rs#L5)
