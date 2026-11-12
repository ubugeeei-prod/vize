---
title: "type/no-misused-spread"
description: "Interdire d’étaler une valeur dans une position incompatible."
---

# `type/no-misused-spread`

Interdire d’étaler une valeur dans une position incompatible.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

Étaler une chaîne dans un tableau place une valeur dans une position incompatible.

```vue annotate="remove:2,3"
<script setup lang="ts">
const label = "a";
const chars = [...label];
</script>
```

## Bon

Étaler un tableau dans un tableau garde le type des éléments compatible.

```vue annotate="add:2,3"
<script setup lang="ts">
const numbers = [1, 2, 3];
const chars = [...numbers];
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-misused-spread": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_misused_spread.rs#L5)
