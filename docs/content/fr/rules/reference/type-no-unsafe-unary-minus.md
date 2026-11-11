---
title: "type/no-unsafe-unary-minus"
description: "Interdire le moins unaire sur une valeur qui n’est ni number ni bigint."
---

# `type/no-unsafe-unary-minus`

Interdire le moins unaire sur une valeur qui n’est ni number ni bigint.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

label est une chaîne, donc le moins unaire ne nie ni un number ni un bigint.

```vue annotate="remove:2,3"
<script setup lang="ts">
const label = "a";
const negated = -label;
</script>
```

## Bon

count est un nombre, donc le moins unaire nie ce nombre.

```vue annotate="add:2,3"
<script setup lang="ts">
const count = 1;
const negated = -count;
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-unary-minus": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_unary_minus.rs#L5)
